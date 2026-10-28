//! Explicit registration and response recording. Never renders or executes authored files.
use crate::{
    artifact_style,
    authored::{self, Feedback, Input, Ledger, Revision},
    config::{self, Result},
    history, repo, task_record,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
pub fn context(root: &Path, id: &str) -> Result<Value> {
    let l = authored::read(root, id)?;
    let r = l.revisions.last().ok_or("missing revision")?;
    Ok(
        json!({"ok":true,"task":id,"mode":"authored","revision_sha256":history::json_digest(r)?,
        "feedback_base_sha256":history::json_digest(&l.feedback)?,"revision":l.revisions.len(),
        "files":r.files.iter().map(|s|&s.file).collect::<Vec<_>>(),
        "scope":"Registered bytes and provenance; not a template, content-quality check or JavaScript sandbox"}),
    )
}
pub fn command(root: &Path, action: &str, id: &str, input: &Path) -> Result<Value> {
    task_record::read(root, id)?;
    let source = repo::fingerprint(root)?;
    let dir = authored::path(root, id, "authored.json")?
        .parent()
        .ok_or("missing artifact directory")?
        .to_path_buf();
    fs::create_dir_all(&dir)?;
    let lock = authored::path(root, id, "authored.lock")?;
    let _handle = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)?;
    let _lock = Lock(lock);
    let mut ledger = if authored::exists(root, id) {
        authored::read(root, id)?
    } else {
        Ledger {
            schema_version: 1,
            task: id.into(),
            revisions: vec![],
            feedback: vec![],
        }
    };
    match action {
        "style" => return artifact_style::install(root, id),
        "register" => {
            let mut input: Input = config::read(input)?;
            artifact_style::prepare(root, id, &mut input)?;
            let files = authored::inspect(root, id, &input)?;
            let number = ledger.revisions.len() + 1;
            let snapshot = authored::path(root, id, &format!("authored-revisions/{number}"))?;
            if let Some(parent) = snapshot.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::create_dir(&snapshot).map_err(|e|format!("snapshot already exists or cannot be created; inspect interrupted registration before retry: {e}"))?;
            for s in &files {
                let dest = snapshot.join(&s.file.path);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                let bytes = fs::read(authored::path(root, id, &s.file.path)?)?;
                if history::digest(&bytes) != s.sha256 {
                    return Err(
                        "artifact changed during registration; inspect orphan snapshot and retry"
                            .into(),
                    );
                }
                fs::write(dest, bytes)?;
            }
            ledger.revisions.push(Revision {
                summary: input.summary,
                source_sha256: source,
                feedback_sha256: history::json_digest(&ledger.feedback)?,
                files,
            });
        }
        "decide" => {
            let f: Feedback = config::read(input)?;
            authored::validate_feedback(&f, id)?;
            let r = ledger
                .revisions
                .last()
                .ok_or("register authored HTML before importing feedback")?;
            if f.revision_sha256 != history::json_digest(r)?
                || f.feedback_base_sha256 != history::json_digest(&ledger.feedback)?
            {
                return Err(
                    "authored feedback is stale; inspect current context before importing".into(),
                );
            }
            // Even a correct envelope must not refer to unregistered edits.
            authored::check(root, id, false, None)?;
            ledger.feedback.push(f);
        }
        _ => return Err("unknown authored action".into()),
    }
    let temp = authored::path(root, id, "authored.tmp")?;
    let mut file = fs::File::create(&temp)?;
    file.write_all(&serde_json::to_vec_pretty(&ledger)?)?;
    file.sync_all()?;
    fs::rename(temp, authored::path(root, id, "authored.json")?)?;
    context(root, id)
}
