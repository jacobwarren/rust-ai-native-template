//! Durable lifecycle content and deterministic HTML, not approval evidence.
use crate::{
    config::{self, Result, valid_id},
    discovery, discovery_feedback, discovery_view, history, repo, task_view, tracking,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Before {
    pub outcome: String,
    pub unknowns: String,
    pub assumptions: String,
    pub alternatives: String,
    pub verification: String,
    pub references: Vec<String>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub observation: String,
    pub decision: String,
    pub effect: String,
    pub references: Vec<String>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub question: String,
    pub options: Vec<String>,
    pub answer: usize,
    pub explanation: String,
    pub references: Vec<String>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct After {
    pub change: String,
    pub demonstration: String,
    pub verification: String,
    pub risks: String,
    pub references: Vec<String>,
    pub quiz: Vec<Question>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    #[serde(default)]
    pub discovery_sha256: Option<String>,
    #[serde(default)]
    pub feedback_sha256: Option<String>,
    pub notes_count: usize,
    pub content: After,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discoveries: Vec<discovery::Discovery>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub feedback: Vec<discovery_feedback::Feedback>,
    pub schema_version: u32,
    pub task: String,
    pub source_revision: Value,
    pub source_sha256: String,
    pub before: Before,
    pub notes: Vec<Note>,
    pub publications: Vec<Publication>,
}
pub fn publication_current(a: &Artifact) -> bool {
    a.publications.last().is_some_and(|p| {
        p.notes_count == a.notes.len()
            && (a.discoveries.is_empty()
                || (p.discovery_sha256 == discovery_feedback::discovery_hash(a).ok().flatten()
                    && p.feedback_sha256 == history::json_digest(&a.feedback).ok()))
    })
}
pub fn directory(id: &str) -> String {
    format!("harness/task-artifacts/{id}")
}
fn checked_dir(root: &Path, id: &str) -> Result<PathBuf> {
    if !valid_id(id) {
        return Err("invalid task ID".into());
    }
    let dir = root.join(directory(id));
    for p in [
        &root.join("harness"),
        &root.join("harness/task-artifacts"),
        &dir,
    ] {
        if p.symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return Err("artifact paths must not be symlinks".into());
        }
    }
    for name in [
        "record.json",
        "index.html",
        "unknowns.html",
        "implementation.html",
        "buy-in.html",
        "quiz.html",
        "blindspots.html",
        "interventions.html",
        "interview.html",
        "reference.html",
        "plan.html",
        ".lock",
        ".record.tmp",
        ".html.tmp",
    ] {
        if dir
            .join(name)
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return Err("artifact files must not be symlinks".into());
        }
    }
    Ok(dir)
}
fn text_fields(fields: &[&str]) -> Result<()> {
    if fields.iter().any(|s| !tracking::nonempty(s)) {
        return Err("artifact fields must be nonempty".into());
    }
    Ok(())
}
fn references(root: &Path, refs: &[String]) -> Result<()> {
    if refs.is_empty()
        || !tracking::unique(refs)
        || refs.iter().any(|r| !tracking::reference(root, r))
    {
        return Err("artifact references need existing source files or HTTPS URLs".into());
    }
    Ok(())
}
fn validate(root: &Path, id: &str, a: &Artifact) -> Result<()> {
    if ![1, 2].contains(&a.schema_version)
        || a.task != id
        || a.source_sha256.len() != 64
        || !a.source_sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("invalid artifact identity".into());
    }
    if a.schema_version == 1 && (!a.discoveries.is_empty() || !a.feedback.is_empty()) {
        return Err("discovery content requires artifact schema version 2".into());
    }
    for d in &a.discoveries {
        discovery::validate(root, d)?;
    }
    discovery_feedback::validate(a)?;
    let b = &a.before;
    text_fields(&[
        &b.outcome,
        &b.unknowns,
        &b.assumptions,
        &b.alternatives,
        &b.verification,
    ])?;
    references(root, &b.references)?;
    for n in &a.notes {
        text_fields(&[&n.observation, &n.decision, &n.effect])?;
        references(root, &n.references)?;
    }
    let mut last = 0;
    for p in &a.publications {
        if p.notes_count == 0 || p.notes_count < last || p.notes_count > a.notes.len() {
            return Err("invalid publication checkpoint".into());
        }
        last = p.notes_count;
        let c = &p.content;
        text_fields(&[&c.change, &c.demonstration, &c.verification, &c.risks])?;
        references(root, &c.references)?;
        if c.quiz.is_empty() {
            return Err("publish at least one scenario question".into());
        }
        for q in &c.quiz {
            text_fields(&[&q.question, &q.explanation])?;
            if q.options.len() < 2 || !tracking::unique(&q.options) || q.answer >= q.options.len() {
                return Err("quiz needs distinct options and an in-range answer".into());
            }
            references(root, &q.references)?;
        }
    }
    Ok(())
}
pub fn read(root: &Path, id: &str) -> Result<Artifact> {
    let a = config::read(&checked_dir(root, id)?.join("record.json"))?;
    validate(root, id, &a)?;
    Ok(a)
}
pub fn check(root: &Path, id: &str, final_stage: bool, base: Option<&str>) -> Result<()> {
    if crate::authored::exists(root, id) {
        return crate::authored::check(root, id, final_stage, base);
    }
    if let Some(base) = base
        && history::read(root, base, &format!("{}/authored.json", directory(id)))?.is_some()
    {
        return Err("authored history was removed".into());
    }
    let a = read(root, id)?;
    for page in task_view::PAGES {
        if fs::read_to_string(checked_dir(root, id)?.join(page))? != task_view::render(&a, page) {
            return Err(format!(
                "HTML artifact {page} is stale or modified; run task-artifact render --task ID"
            )
            .into());
        }
    }
    let active = crate::task_record::read(root, id)?.schema_version >= 4;
    if active && a.discoveries.is_empty() {
        return Err("five discovery tools are mandatory; run task-artifact discover".into());
    }
    if !a.discoveries.is_empty() {
        for page in discovery_view::PAGES {
            if fs::read_to_string(checked_dir(root, id)?.join(page))?
                != discovery_view::render(&a, page)?
            {
                return Err(
                    format!("discovery HTML {page} is missing or stale; render again").into(),
                );
            }
        }
        if final_stage {
            discovery_feedback::completion(&a)?;
        }
    }
    if final_stage
        && (a.notes.is_empty()
            || !a
                .publications
                .last()
                .is_some_and(|p| p.notes_count == a.notes.len()))
    {
        return Err(
            "complete task requires implementation notes, current buy-in brief and quiz".into(),
        );
    }
    if let Some(base) = base
        && let Some(bytes) = history::read(root, base, &format!("{}/record.json", directory(id)))?
    {
        let old: Artifact = serde_json::from_slice(&bytes)?;
        if old.before != a.before
            || old.source_revision != a.source_revision
            || old.source_sha256 != a.source_sha256
            || old.task != a.task
            || !a.notes.starts_with(&old.notes)
            || !a.publications.starts_with(&old.publications)
            || !a.discoveries.starts_with(&old.discoveries)
            || !a.feedback.starts_with(&old.feedback)
        {
            return Err("artifact history was rewritten; preserve preflight and append notes/brief revisions".into());
        }
    }
    Ok(())
}
struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn replace(dir: &Path, name: &str, temp: &str, bytes: &[u8]) -> Result<()> {
    let mut file = fs::File::create(dir.join(temp))?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(dir.join(temp), dir.join(name))?;
    Ok(())
}
pub fn command(root: &Path, action: &str, id: &str, input: Option<&Path>) -> Result<Value> {
    if crate::authored::exists(root, id) {
        if action == "decide" {
            return crate::authored_command::command(
                root,
                action,
                id,
                input.ok_or("feedback input required")?,
            );
        }
        return Err("task-authored HTML is never overwritten by templates; edit files and register a new revision".into());
    }
    // A malformed/nonexistent task must not create orphan artifacts.
    let task = crate::task_record::read(root, id)?;
    let initial_source = if action == "start" {
        Some(repo::fingerprint(root)?)
    } else {
        None
    };
    let dir = checked_dir(root, id)?;
    fs::create_dir_all(&dir)?;
    let lock = dir.join(".lock");
    let _handle = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .map_err(|e| {
            format!("artifact is locked; inspect interrupted writers before removing .lock: {e}")
        })?;
    let _lock = Lock(lock);
    let mut a = if action == "start" {
        if dir.join("record.json").exists() {
            return Err("preflight already exists; append notes instead".into());
        }
        Artifact {
            discoveries: vec![],
            feedback: vec![],
            schema_version: 1,
            task: id.into(),
            source_revision: repo::git(root)["head"].clone(),
            source_sha256: initial_source.ok_or("missing initial source identity")?,
            before: config::read(input.ok_or("start needs input")?)?,
            notes: vec![],
            publications: vec![],
        }
    } else {
        read(root, id)?
    };
    if task.schema_version >= 4 && ["note", "publish"].contains(&action) && a.discoveries.is_empty()
    {
        return Err(
            "create the five discovery artifacts before implementation notes or publication".into(),
        );
    }
    match action {
        "start" | "render" => (),
        "note" => a
            .notes
            .push(config::read(input.ok_or("note needs input")?)?),
        "discover" => {
            let d = config::read(input.ok_or("discover needs input")?)?;
            discovery::validate(root, &d)?;
            if a.discoveries.last() == Some(&d) {
                return Err("discovery is unchanged".into());
            }
            a.schema_version = 2;
            a.discoveries.push(d);
        }
        "decide" => {
            let f = config::read(input.ok_or("decide needs input")?)?;
            discovery_feedback::import(&mut a, f)?;
        }
        "publish" => a.publications.push(Publication {
            discovery_sha256: discovery_feedback::discovery_hash(&a)?,
            feedback_sha256: Some(history::json_digest(&a.feedback)?),
            notes_count: a.notes.len(),
            content: config::read(input.ok_or("publish needs input")?)?,
        }),
        _ => return Err("unknown artifact action".into()),
    }
    validate(root, id, &a)?;
    // JSON commits first; interruption leaves repairable stale HTML, which admission rejects.
    if action != "render" {
        replace(
            &dir,
            "record.json",
            ".record.tmp",
            &serde_json::to_vec_pretty(&a)?,
        )?;
    }
    for page in task_view::PAGES {
        replace(
            &dir,
            page,
            ".html.tmp",
            task_view::render(&a, page).as_bytes(),
        )?;
    }
    if !a.discoveries.is_empty() {
        for page in discovery_view::PAGES {
            replace(
                &dir,
                page,
                ".html.tmp",
                discovery_view::render(&a, page)?.as_bytes(),
            )?;
        }
    }
    Ok(
        json!({"ok":true,"task":id,"artifact":format!("{}/index.html", directory(id)),
        "pages":task_view::PAGES.iter().chain(if a.discoveries.is_empty() { [].iter() } else { discovery_view::PAGES.iter() }).map(|p| format!("{}/{p}", directory(id))).collect::<Vec<_>>(),
        "discovery_sha256": discovery_feedback::discovery_hash(&a)?,
        "feedback_base_sha256":history::json_digest(&a.feedback)?,
        "stage":if publication_current(&a) {"review"} else if a.notes.is_empty() {"before"} else {"during"},
        "scope":"Human explanation; not approval, CI evidence or proof of lifecycle timing"}),
    )
}
