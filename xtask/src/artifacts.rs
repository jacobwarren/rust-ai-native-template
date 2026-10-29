//! Task HTML artifacts: eight self-contained agent-authored pages and their commit order.
//! Checks establish presence and order only; people judge usefulness against the exemplars.
use crate::{
    config::{Result, valid_id},
    history,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

pub const DISCOVERY: [&str; 5] = [
    "blindspots.html",
    "interventions.html",
    "interview.html",
    "reference.html",
    "plan.html",
];
pub const NOTES: &str = "implementation.html";
pub const FINAL: [&str; 2] = ["buy-in.html", "quiz.html"];
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

/// Create the task's folder and write the current stylesheet as a block to paste into each
/// page's head. Inline styles keep a page whole in any viewer and unchanged by later edits.
pub fn install(root: &Path, id: &str) -> Result<Value> {
    fs::create_dir_all(dir(root, id)?)?;
    fs::create_dir_all(root.join("target"))?;
    fs::write(root.join(BLOCK), format!("{MARKER}\n{CSS}</style>\n"))?;
    Ok(
        json!({"ok":true,"task":id,"folder":directory(id),"style":BLOCK,
        "next":"Paste the style block first in each page's head, then write the pages described in docs/ARTIFACTS.md."}),
    )
}

/// Discovery pages are always required; the full set when the task claims completion.
/// With a base revision, completion also checks the order of the branch's commits.
pub fn check(root: &Path, id: &str, final_stage: bool, base: Option<&str>) -> Result<()> {
    let dir = dir(root, id)?;
    let mut pages = DISCOVERY.to_vec();
    if final_stage {
        pages.push(NOTES);
        pages.extend(FINAL);
    }
    for page in pages {
        let path = dir.join(page);
        if path
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return Err(format!("{page} must not be a symlink").into());
        }
        let text = fs::read_to_string(&path)
            .map_err(|_| format!("missing {page}; see docs/ARTIFACTS.md"))?;
        if !text.contains(MARKER) {
            return Err(format!(
                "{page} must carry the shared stylesheet inline; paste the block from cargo xtask task-artifact style --task ID"
            )
            .into());
        }
    }
    if let (true, Some(base)) = (final_stage, base) {
        order(root, id, base)?;
    }
    Ok(())
}

fn order(root: &Path, id: &str, base: &str) -> Result<()> {
    let prefix = format!("{}/", directory(id));
    let dirty: Vec<String> = history::uncommitted(root)?
        .into_iter()
        .filter(|p| p.starts_with(&prefix) || !administrative(p))
        .collect();
    if !dirty.is_empty() {
        return Err(format!(
            "commit product changes and this task's pages before completion; order is read from commits: {}",
            dirty.join(", ")
        )
        .into());
    }
    let commits = history::commits(root, base)?;
    let code: Vec<usize> = (0..commits.len())
        .filter(|&i| commits[i].iter().any(|p| !administrative(p)))
        .collect();
    let (Some(&first), Some(&last)) = (code.first(), code.last()) else {
        return Ok(());
    };
    let touched = |page: &str| -> Vec<usize> {
        let path = format!("{prefix}{page}");
        (0..commits.len())
            .filter(|&i| commits[i].contains(&path))
            .collect()
    };
    for page in DISCOVERY {
        let at_base = history::read(root, base, &format!("{prefix}{page}"))?.is_some();
        if !at_base && !touched(page).first().is_some_and(|&i| i < first) {
            return Err(
                format!("{page} must be committed before the first product-code commit").into(),
            );
        }
    }
    // With one code commit, notes in that same commit are as early as notes can be.
    let notes = touched(NOTES).first().copied();
    if !notes.is_some_and(|i| i < last || (code.len() == 1 && i == last)) {
        return Err(format!(
            "{NOTES} must be committed during the work, before the last product-code commit"
        )
        .into());
    }
    for page in FINAL {
        if !touched(page).last().is_some_and(|&i| i >= last) {
            return Err(
                format!("{page} must be updated at or after the last product-code commit").into(),
            );
        }
    }
    Ok(())
}

/// Host boundary before product-code edits: the discovery pages must already be committed.
pub fn guard(root: &Path, id: &str) -> Result<Value> {
    check(root, id, false, None)?;
    let committed = history::paths(root, "HEAD")?;
    for page in DISCOVERY {
        if !committed.contains(&format!("{}/{page}", directory(id))) {
            return Err(format!(
                "commit the five discovery pages before editing product code; {page} is not committed"
            )
            .into());
        }
    }
    Ok(json!({"ok":true,"task":id,
        "scope":"Discovery pages exist and are committed; their usefulness needs human review"}))
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
