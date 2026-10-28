//! Shared presentation, copied into a task and then into every registered bundle.
use crate::{
    authored::{self, File, Input},
    config::Result,
    history,
};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub const CSS: &str = include_str!("artifact_foundation.css");
pub const FILE: &str = "foundation.css";
pub const VERSION: &str = "1.0.0";

pub fn link(page: &str) -> String {
    let depth = Path::new(page).components().count().saturating_sub(1);
    format!(
        "<link rel=\"stylesheet\" href=\"{}foundation.css\" data-artifact-foundation=\"1\">",
        "../".repeat(depth)
    )
}

// The caller holds the task's authored.lock. Only the working copy is updated.
pub fn install(root: &Path, id: &str) -> Result<Value> {
    let dest = authored::path(root, id, FILE)?;
    let temp = authored::path(root, id, "authored-style.tmp")?;
    fs::write(&temp, CSS)?;
    fs::rename(temp, dest)?;
    Ok(
        json!({"ok":true,"task":id,"version":VERSION,"sha256":history::digest(CSS.as_bytes()),
        "path":format!("harness/task-artifacts/{id}/{FILE}"),"link":link("page.html"),
        "next":"Place the link in each HTML head before task-specific CSS, then register. Nested pages need ../ per directory. Historical snapshots are untouched."}),
    )
}

pub fn prepare(root: &Path, id: &str, input: &mut Input) -> Result<()> {
    if fs::read(authored::path(root, id, FILE)?)? != CSS.as_bytes() {
        return Err("foundation differs from the current shared style; run task-artifact style --task ID, then review and register".into());
    }
    for f in &input.files {
        if f.path.ends_with(".html") {
            let text = fs::read_to_string(authored::path(root, id, &f.path)?)?;
            if !text.contains(&link(&f.path)) {
                return Err(format!(
                    "{} needs the local foundation link in its head: {}",
                    f.path,
                    link(&f.path)
                )
                .into());
            }
        }
    }
    // Always include the canonical asset. Reject caller-supplied conflicting metadata.
    if input.files.iter().any(|f| f.path == FILE) {
        return Err(
            "foundation.css is registered automatically; omit it from the input manifest".into(),
        );
    }
    let evidence = input
        .files
        .first()
        .ok_or("empty artifact bundle")?
        .evidence
        .clone();
    input.files.push(File {
        purpose: "supporting".into(), path: FILE.into(),
        description: format!("Shared artifact foundation {VERSION}; frozen with this revision."),
        evidence, feedback: "Propose changes to the shared foundation in the task conversation.".into(),
        verification: "Matches the bundled foundation at registration; hashes preserve the revision, not visual correctness.".into(),
    });
    Ok(())
}
