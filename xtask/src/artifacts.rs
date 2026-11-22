//! The task brief: one self-contained HTML page a standard task commits before its
//! code. Checks establish presence, a floor of text and commit order; a person
//! judges whether it is useful.
use crate::{
    config::{Result, valid_id},
    history,
    task_record::{Task, Tier},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

/// The one task page. The record's four review answers, not a second page,
/// report on the finished work.
pub const BRIEF: &str = "brief.html";
/// Fewest characters, outside tags and styles, the brief may hold. Script text
/// counts: an interactive page builds its content from data in a script.
/// A floor against placeholders, not a measure of usefulness.
pub const MIN_TEXT: usize = 1200;
/// Largest file the brief should be, stylesheet included. Advice from `next`,
/// never a gate.
pub const BRIEF_BYTES: u64 = 20_000;

/// Whether this task owes a brief.
pub fn owed(task: &Task) -> bool {
    task.tier() == Tier::Standard
}
/// Opening tag of the shared stylesheet, carried inline so a page renders on its own.
pub const MARKER: &str = "<style data-artifact-foundation=\"1\">";
const CSS: &str = include_str!("artifact_foundation.css");
const BLOCK: &str = "target/artifact-style.html";

pub fn directory(id: &str) -> String {
    format!("harness/task-artifacts/{id}")
}

/// Edits that record or review work rather than change the product.
pub fn administrative(path: &str) -> bool {
    [
        "harness/task-artifacts/",
        "harness/tasks/",
        "harness/transitions/",
        "docs/plans/",
    ]
    .iter()
    .any(|p| path.starts_with(p))
        || ["harness/docs.json", "harness/gaps.json"].contains(&path)
}

/// A repository-relative path from a tool event, rejecting traversal and outside paths.
pub fn relative(root: &Path, raw: &str) -> Result<String> {
    let path = Path::new(raw);
    let rel = if path.is_absolute() {
        path.strip_prefix(root)
            .map_err(|_| format!("{raw} is outside the repository"))?
    } else {
        path
    };
    if rel.as_os_str().is_empty() || rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(format!("unsafe path {raw}").into());
    }
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn dir(root: &Path, id: &str) -> Result<PathBuf> {
    if !valid_id(id) {
        return Err("invalid task ID".into());
    }
    let dir = root.join(directory(id));
    for p in [
        root.join("harness"),
        root.join("harness/task-artifacts"),
        dir.clone(),
    ] {
        if p.symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return Err("artifact paths must not be symlinks".into());
        }
    }
    Ok(dir)
}

/// Create the task's folder and write the current stylesheet as a block to paste into the
/// brief's head. Inline styles keep a page whole in any viewer and unchanged by later edits.
pub fn install(root: &Path, id: &str) -> Result<Value> {
    fs::create_dir_all(dir(root, id)?)?;
    fs::create_dir_all(root.join("target"))?;
    fs::write(root.join(BLOCK), format!("{MARKER}\n{CSS}</style>\n"))?;
    Ok(
        json!({"ok":true,"task":id,"folder":directory(id),"style":BLOCK,
        "next":"Paste the style block first in the brief's head, then write the brief described in docs/ARTIFACTS.md."}),
    )
}

/// A task that owes a brief must have one, with the stylesheet and more than a
/// placeholder's text. With a base revision the branch's commits are read too:
/// the brief comes before the first product-code commit. `head` names the branch
/// tip when HEAD is a merge commit built by a code host; `complete` also requires
/// the work to be committed.
pub fn check(
    root: &Path,
    id: &str,
    owed: bool,
    complete: bool,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<()> {
    let dir = dir(root, id)?;
    let present = verify(&dir)?;
    if !owed {
        return Ok(());
    }
    if !present {
        return Err(format!("missing {BRIEF}; see docs/ARTIFACTS.md").into());
    }
    let text = visible_text(&fs::read_to_string(dir.join(BRIEF))?);
    if text < MIN_TEXT {
        return Err(format!(
            "{BRIEF} holds {text} characters outside its tags and styles; it needs at least {MIN_TEXT} to be more than a placeholder"
        )
        .into());
    }
    if let Some(base) = base {
        order(root, id, base, head, complete)?;
    }
    Ok(())
}

/// The page's own content: its characters outside tags, style blocks and spaces.
fn visible_text(html: &str) -> usize {
    // ASCII lowering keeps every byte offset, so both strings index alike.
    let lower = html.to_ascii_lowercase();
    let seen = |text: &str| text.chars().filter(|c| !c.is_whitespace()).count();
    let mut count = 0;
    let mut at = 0;
    while let Some(open) = lower[at..].find('<').map(|i| i + at) {
        count += seen(&html[at..open]);
        let tag = &lower[open + 1..];
        let close = tag.starts_with("style").then_some("</style");
        // A style block runs to its closing tag; any other tag to its own end.
        let from = match close {
            Some(close) => match lower[open..].find(close) {
                Some(i) => open + i,
                None => return count,
            },
            None => open,
        };
        match lower[from..].find('>') {
            Some(i) => at = from + i + 1,
            None => return count,
        }
    }
    count + seen(&html[at..])
}

/// Whether the brief exists and carries the inline stylesheet.
pub fn exists(root: &Path, id: &str) -> Result<bool> {
    verify(&dir(root, id)?)
}

/// Whether the brief exists, after rejecting a symlink or one without the inline stylesheet.
fn verify(dir: &Path) -> Result<bool> {
    let path = dir.join(BRIEF);
    let Ok(meta) = path.symlink_metadata() else {
        return Ok(false);
    };
    if meta.file_type().is_symlink() {
        return Err(format!("{BRIEF} must not be a symlink").into());
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("cannot read {BRIEF}: {e}"))?;
    if !text.contains(MARKER) {
        return Err(format!(
            "{BRIEF} must carry the shared stylesheet inline; paste the block from cargo xtask task-artifact style --task ID"
        )
        .into());
    }
    Ok(true)
}

/// Completion reads commits, so product changes and the task's brief must be committed.
pub fn committed(root: &Path, id: &str) -> Result<()> {
    let prefix = format!("{}/", directory(id));
    let dirty: Vec<String> = history::uncommitted(root)?
        .into_iter()
        .filter(|p| p.starts_with(&prefix) || !administrative(p))
        .collect();
    if !dirty.is_empty() {
        return Err(format!(
            "commit product changes, and the task's brief, before completion; completion reads commits: {}",
            dirty.join(", ")
        )
        .into());
    }
    Ok(())
}

fn order(root: &Path, id: &str, base: &str, head: Option<&str>, complete: bool) -> Result<()> {
    if complete {
        committed(root, id)?;
    }
    let path = format!("{}/{BRIEF}", directory(id));
    let commits = history::commits(root, base, head)?;
    let Some(first) = commits
        .iter()
        .position(|c| c.iter().any(|p| !administrative(p)))
    else {
        return Ok(());
    };
    let before_code = commits[..first].iter().any(|c| c.contains(&path));
    if history::read(root, base, &path)?.is_none() && !before_code {
        // Read from a host's merge commit without the branch tip, the target's
        // own commits can look like this branch's code.
        let hint = if head.is_none() && history::head_is_merge(root)? {
            "; HEAD is a merge commit, so if it is a pull request's checkout pass the branch tip as handoff --head"
        } else {
            ""
        };
        return Err(format!(
            "{BRIEF} must be committed before the first product-code commit{hint}"
        )
        .into());
    }
    Ok(())
}

/// Host boundary before product-code edits: a task that owes a brief must have
/// committed it. A small task owes none; the completion gate holds it to its paths.
pub fn guard(root: &Path, id: &str) -> Result<Value> {
    // A record that exists but does not parse is reported as itself, not as a missing brief.
    let owed = match crate::task_record::read(root, id) {
        Ok(task) => owed(&task),
        Err(e) if root.join(format!("harness/tasks/{id}.json")).is_file() => return Err(e),
        Err(_) => false,
    };
    check(root, id, owed, false, None, None)?;
    if !owed {
        return Ok(json!({"ok":true,"task":id,"tier":"small",
            "scope":"Small task: no brief required; completion and handoff check its paths"}));
    }
    let path = format!("{}/{BRIEF}", directory(id));
    if !history::paths(root, "HEAD")?.contains(&path) {
        return Err(
            format!("commit {BRIEF} before editing product code; it is not committed").into(),
        );
    }
    Ok(json!({"ok":true,"task":id,
        "scope":"The brief exists and is committed; its usefulness needs human review"}))
}

/// Render the task's pages to PNG tiles under target/ with a local Chrome, so an
/// agent can look at what it wrote. Needs Node and Chrome; reports, never judges.
pub fn screenshots(root: &Path, id: &str) -> Result<Value> {
    let dir = dir(root, id)?;
    let mut pages: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|_| format!("no artifact folder {}", directory(id)))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    if pages.is_empty() {
        return Err(format!("no HTML pages in {}", directory(id)).into());
    }
    pages.sort();
    let out = root.join("target/artifact-screenshots").join(id);
    if out.exists() {
        fs::remove_dir_all(&out)?;
    }
    let output = Command::new("node")
        .arg(root.join("scripts/artifact-screenshots.cjs"))
        .arg(&out)
        .args(&pages)
        .current_dir(root)
        .output()
        .map_err(|e| format!("screenshots need Node on PATH: {e}"))?;
    serde_json::from_slice(&output.stdout).map_err(|_| {
        format!(
            "screenshot script failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into()
    })
}
