//! Contract and integrity checks for task-authored HTML. No renderer owns these bytes.
use crate::{
    artifact_workflow,
    config::{self, Result},
    history, task_artifact, tracking,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

pub const BEFORE: [&str; 5] = [
    "blindspots",
    "interventions",
    "interview",
    "reference",
    "plan",
];
pub const AFTER: [&str; 3] = ["implementation", "buy-in", "quiz"];
#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub purpose: String,
    pub path: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub feedback: String,
    pub verification: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    #[serde(default)]
    pub plan_evidence: Option<String>,
    pub summary: String,
    pub files: Vec<File>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub file: File,
    pub sha256: String,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<artifact_workflow::Checkpoint>,
    pub summary: String,
    pub source_sha256: String,
    pub feedback_sha256: String,
    pub files: Vec<Snapshot>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Feedback {
    pub schema_version: u32,
    pub task: String,
    pub revision_sha256: String,
    pub feedback_base_sha256: String,
    pub actor: String,
    pub payload: serde_json::Value,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    pub schema_version: u32,
    pub task: String,
    pub revisions: Vec<Revision>,
    pub feedback: Vec<Feedback>,
}
pub fn exists(root: &Path, id: &str) -> bool {
    root.join(task_artifact::directory(id))
        .join("authored.json")
        .exists()
}
pub fn safe(root: &Path, relative: &str) -> Result<PathBuf> {
    if !tracking::source_path(relative) {
        return Err("invalid authored source path".into());
    }
    let mut p = root.to_path_buf();
    for c in Path::new(relative).components() {
        p.push(c);
        if p.symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return Err("authored paths must not be symlinks".into());
        }
    }
    Ok(p)
}
pub fn path(root: &Path, id: &str, name: &str) -> Result<PathBuf> {
    if !config::valid_id(id) {
        return Err("invalid task ID".into());
    }
    safe(root, &format!("{}/{name}", task_artifact::directory(id)))
}
fn metadata(root: &Path, files: impl Iterator<Item = File>) -> Result<()> {
    let mut paths = BTreeSet::new();
    let mut purposes = BTreeSet::new();
    for f in files {
        if !tracking::source_path(&f.path)
            || f.path.starts_with("authored")
            || f.path == "record.json"
            || !paths.insert(f.path.clone())
            || !tracking::nonempty(&f.description)
            || !tracking::nonempty(&f.feedback)
            || !tracking::nonempty(&f.verification)
            || f.evidence.is_empty()
            || !tracking::unique(&f.evidence)
            || f.evidence.iter().any(|p| !tracking::reference(root, p))
        {
            return Err("invalid authored file metadata, duplicate path, or missing evidence/feedback/verification description".into());
        }
        if f.purpose != "supporting"
            && (!BEFORE.contains(&f.purpose.as_str()) && !AFTER.contains(&f.purpose.as_str())
                || !purposes.insert(f.purpose.clone()))
        {
            return Err("unknown or duplicate artifact purpose".into());
        }
        if f.purpose != "supporting" && !f.path.ends_with(".html") {
            return Err("required purposes need standalone HTML files".into());
        }
    }
    for p in BEFORE {
        if !purposes.contains(p) {
            return Err(format!("missing mandatory authored purpose: {p}").into());
        }
    }
    Ok(())
}
pub fn inspect(root: &Path, id: &str, input: &Input) -> Result<Vec<Snapshot>> {
    if !tracking::nonempty(&input.summary) {
        return Err("registration needs a revision summary".into());
    }
    metadata(root, input.files.iter().cloned())?;
    input
        .files
        .iter()
        .map(|f| {
            let bytes = fs::read(path(root, id, &f.path)?)?;
            if bytes.is_empty() {
                return Err("authored files must not be empty".into());
            }
            if f.purpose != "supporting" {
                let text = std::str::from_utf8(&bytes)?.to_ascii_lowercase();
                if !text.contains("<html") && !text.contains("<!doctype html") {
                    return Err("required artifact is not an HTML document".into());
                }
            }
            Ok(Snapshot {
                file: f.clone(),
                sha256: history::digest(&bytes),
            })
        })
        .collect()
}
pub fn read(root: &Path, id: &str) -> Result<Ledger> {
    let l: Ledger = config::read(&path(root, id, "authored.json")?)?;
    if ![1, 2].contains(&l.schema_version) || l.task != id || l.revisions.is_empty() {
        return Err("invalid authored ledger identity".into());
    }
    for (i, r) in l.revisions.iter().enumerate() {
        if !tracking::nonempty(&r.summary) {
            return Err("missing authored revision summary".into());
        }
        metadata(root, r.files.iter().map(|s| s.file.clone()))?;
        for s in &r.files {
            let bytes = fs::read(path(
                root,
                id,
                &format!("authored-revisions/{}/{path}", i + 1, path = s.file.path),
            )?)?;
            if history::digest(&bytes) != s.sha256 {
                return Err("authored historical snapshot is missing or changed".into());
            }
        }
        if !(0..=l.feedback.len()).any(|n| {
            history::json_digest(&l.feedback[..n]).ok().as_ref() == Some(&r.feedback_sha256)
        }) {
            return Err("authored revision has invalid feedback provenance".into());
        }
    }
    for (i, f) in l.feedback.iter().enumerate() {
        validate_feedback(f, id)?;
        if !l
            .revisions
            .iter()
            .any(|r| history::json_digest(r).ok().as_ref() == Some(&f.revision_sha256))
            || f.feedback_base_sha256 != history::json_digest(&l.feedback[..i])?
        {
            return Err("authored feedback history was rewritten".into());
        }
    }
    artifact_workflow::history_check(root, id, &l)?;
    Ok(l)
}
pub fn validate_feedback(f: &Feedback, id: &str) -> Result<()> {
    if f.schema_version != 1
        || f.task != id
        || !tracking::nonempty(&f.actor)
        || f.payload.is_null()
        || f.payload.as_str().is_some_and(|s| !tracking::nonempty(s))
    {
        return Err("feedback needs matching task, actor and a non-null payload".into());
    }
    Ok(())
}
pub fn check(root: &Path, id: &str, complete: bool, base: Option<&str>) -> Result<()> {
    let l = read(root, id)?;
    let r = l.revisions.last().ok_or("missing authored revision")?;
    for s in &r.files {
        if history::digest(&fs::read(path(root, id, &s.file.path)?)?) != s.sha256 {
            return Err(format!(
                "authored file changed since registration: {}; register a new revision",
                s.file.path
            )
            .into());
        }
    }
    if complete {
        artifact_workflow::final_check(root, &l)?;
        for p in AFTER {
            if !r.files.iter().any(|s| s.file.purpose == p) {
                return Err(format!("missing final authored purpose: {p}").into());
            }
        }
        if r.feedback_sha256 != history::json_digest(&l.feedback)? {
            return Err("new feedback needs a reviewed artifact revision before completion".into());
        }
    }
    if let Some(base) = base
        && let Some(bytes) = history::read(
            root,
            base,
            &format!("{}/authored.json", task_artifact::directory(id)),
        )?
    {
        let old: Ledger = serde_json::from_slice(&bytes)?;
        if old.task != l.task
            || !l.revisions.starts_with(&old.revisions)
            || !l.feedback.starts_with(&old.feedback)
        {
            return Err("authored ledger history was rewritten".into());
        }
    }
    // Switching modes does not license erasing the preceding template history.
    if let Some(base) = base {
        for p in history::paths(root, base)?.iter().filter(|p| {
            p.starts_with(&format!("{}/", task_artifact::directory(id)))
                && p.ends_with("record.json")
        }) {
            if history::read(root, base, p)?.as_deref()
                != Some(fs::read(safe(root, p)?)?.as_slice())
            {
                return Err("preserve the legacy record when switching to authored HTML".into());
            }
        }
    }
    Ok(())
}
