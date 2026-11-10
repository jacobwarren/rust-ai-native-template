//! Task HTML pages: self-contained agent-authored pages and their commit order. A
//! standard task owes a brief and a report, a full task the eight-page set.
//! Checks establish presence, a floor of text and order; people judge usefulness.
use crate::{
    config::{Result, valid_id},
    history,
    task_record::{Reader, Task, Tier},
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
/// The standard tier's two pages: one before code, one after it.
pub const BRIEF: &str = "brief.html";
pub const REPORT: &str = "report.html";
/// The two pages written only when a person will answer them.
const FOR_A_READER: [&str; 2] = ["interview.html", "quiz.html"];
/// Fewest characters, outside tags and styles, a required page may hold. Script
/// text counts: an interactive page builds its content from data in a script.
/// A floor against placeholders, not a measure of usefulness.
pub const MIN_TEXT: usize = 1200;
/// Largest file a standard page should be, stylesheet included. Advice from
/// `next`, never a gate; docs/ARTIFACTS.md quotes it to authors and reviewers.
pub const STANDARD_PAGE_BYTES: u64 = 20_000;

/// The pages one task owes, by when they are due.
#[derive(Default)]
pub struct Pages {
    /// Committed before the first product-code commit.
    pub before: Vec<&'static str>,
    /// Committed during the work, before the last product-code commit.
    pub notes: Option<&'static str>,
    /// Committed at or after the last product-code commit.
    pub after: Vec<&'static str>,
}

impl Pages {
    pub fn of(task: &Task) -> Self {
        Self::for_tier(task.tier(), task.reader())
    }
    pub fn for_tier(tier: Tier, reader: Reader) -> Self {
        let wanted = |p: &&str| reader == Reader::Human || !FOR_A_READER.contains(p);
        match tier {
            Tier::Small => Self::default(),
            Tier::Standard => Self {
                before: vec![BRIEF],
                notes: None,
                after: vec![REPORT],
            },
            Tier::Full => Self {
                before: DISCOVERY.into_iter().filter(wanted).collect(),
                notes: Some(NOTES),
                after: FINAL.into_iter().filter(wanted).collect(),
            },
        }
    }
    pub fn all(&self) -> Vec<&'static str> {
        (self.before.iter().copied())
            .chain(self.notes)
            .chain(self.after.iter().copied())
            .collect()
    }
    pub fn is_empty(&self) -> bool {
        self.all().is_empty()
    }
}

/// Every page name any tier uses, in the order the guide's prompt tables list them.
pub fn known() -> Vec<&'static str> {
    [BRIEF, REPORT]
        .into_iter()
        .chain(DISCOVERY)
        .chain([NOTES])
        .chain(FINAL)
        .collect()
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

/// The pages due before code are always required; the whole set when the task
/// claims completion. With a base revision the order of the branch's commits is
/// checked too: before-code pages at any stage, the rest at completion. `head`
/// names the branch tip when HEAD is a merge commit built by a code host.
pub fn check(
    root: &Path,
    id: &str,
    pages: &Pages,
    final_stage: bool,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<()> {
    check_written(root, id)?;
    let dir = dir(root, id)?;
    let mut required = pages.before.clone();
    if final_stage {
        required.extend(pages.notes);
        required.extend(&pages.after);
    }
    for page in required {
        if !verify(&dir, page)? {
            return Err(format!("missing {page}; see docs/ARTIFACTS.md").into());
        }
        let text = visible_text(&fs::read_to_string(dir.join(page))?);
        if text < MIN_TEXT {
            return Err(format!(
                "{page} holds {text} characters outside its tags and styles; a required page needs at least {MIN_TEXT} to be more than a placeholder"
            )
            .into());
        }
    }
    if let Some(base) = base {
        order(root, id, pages, base, head, final_stage)?;
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

/// Checks the pages a task has written so far, so a task in progress validates
/// before its pages exist.
pub fn check_written(root: &Path, id: &str) -> Result<()> {
    let dir = dir(root, id)?;
    for page in known() {
        verify(&dir, page)?;
    }
    Ok(())
}

/// Whether a task page exists and carries the inline stylesheet.
pub fn exists(root: &Path, id: &str, page: &str) -> Result<bool> {
    verify(&dir(root, id)?, page)
}

/// Whether the page exists, after rejecting a symlink or a page without the inline stylesheet.
fn verify(dir: &Path, page: &str) -> Result<bool> {
    let path = dir.join(page);
    let Ok(meta) = path.symlink_metadata() else {
        return Ok(false);
    };
    if meta.file_type().is_symlink() {
        return Err(format!("{page} must not be a symlink").into());
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("cannot read {page}: {e}"))?;
    if !text.contains(MARKER) {
        return Err(format!(
            "{page} must carry the shared stylesheet inline; paste the block from cargo xtask task-artifact style --task ID"
        )
        .into());
    }
    Ok(true)
}

/// Completion reads commits, so product changes and the task's pages must be committed.
pub fn committed(root: &Path, id: &str) -> Result<()> {
    let prefix = format!("{}/", directory(id));
    let dirty: Vec<String> = history::uncommitted(root)?
        .into_iter()
        .filter(|p| p.starts_with(&prefix) || !administrative(p))
        .collect();
    if !dirty.is_empty() {
        return Err(format!(
            "commit product changes, and the task's pages, before completion; completion reads commits: {}",
            dirty.join(", ")
        )
        .into());
    }
    Ok(())
}

fn order(
    root: &Path,
    id: &str,
    pages: &Pages,
    base: &str,
    head: Option<&str>,
    final_stage: bool,
) -> Result<()> {
    if final_stage {
        committed(root, id)?;
    }
    let prefix = format!("{}/", directory(id));
    let commits = history::commits(root, base, head)?;
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
    for page in &pages.before {
        let at_base = history::read(root, base, &format!("{prefix}{page}"))?.is_some();
        if !at_base && !touched(page).first().is_some_and(|&i| i < first) {
            // Read from a host's merge commit without the branch tip, the target's
            // own commits can look like this branch's code.
            let hint = if head.is_none() && history::head_is_merge(root)? {
                "; HEAD is a merge commit, so if it is a pull request's checkout pass the branch tip as handoff --head"
            } else {
                ""
            };
            return Err(format!(
                "{page} must be committed before the first product-code commit{hint}"
            )
            .into());
        }
    }
    if !final_stage {
        return Ok(());
    }
    // With one code commit, notes in that same commit are as early as notes can be.
    if let Some(notes) = pages.notes
        && !touched(notes)
            .first()
            .is_some_and(|&i| i < last || (code.len() == 1 && i == last))
    {
        return Err(format!(
            "{notes} must be committed during the work, before the last product-code commit"
        )
        .into());
    }
    for page in &pages.after {
        if !touched(page).last().is_some_and(|&i| i >= last) {
            return Err(
                format!("{page} must be updated at or after the last product-code commit").into(),
            );
        }
    }
    Ok(())
}

/// Host boundary before product-code edits: the pages due before code must already
/// be committed. A small task needs no pages; the completion gate holds it to its size.
pub fn guard(root: &Path, id: &str) -> Result<Value> {
    // A record that exists but does not parse is reported as itself, not as missing pages.
    let pages = match crate::task_record::read(root, id) {
        Ok(task) => Pages::of(&task),
        Err(e) if root.join(format!("harness/tasks/{id}.json")).is_file() => return Err(e),
        Err(_) => Pages::for_tier(Tier::Standard, Reader::None),
    };
    if pages.is_empty() {
        check_written(root, id)?;
        return Ok(json!({"ok":true,"task":id,"tier":"small",
            "scope":"Small task: no pages required; completion and handoff check its size"}));
    }
    check(root, id, &pages, false, None, None)?;
    let committed = history::paths(root, "HEAD")?;
    for page in &pages.before {
        if !committed.contains(&format!("{}/{page}", directory(id))) {
            return Err(format!(
                "commit the pages due before code ({}) before editing product code; {page} is not committed",
                pages.before.join(", ")
            )
            .into());
        }
    }
    Ok(json!({"ok":true,"task":id,
        "scope":"The pages due before code exist and are committed; their usefulness needs human review"}))
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
