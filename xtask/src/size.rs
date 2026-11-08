//! The small tier: a change small enough, and far enough from policy, to need no
//! task pages. The gate measures the branch against its base so a task cannot
//! declare itself small and then grow. See docs/ARTIFACTS.md "Task size".
use crate::{artifacts, config::Result, history, transitions};
use std::path::Path;

/// Most changed lines (added plus deleted) a small task may have outside
/// administrative paths. Changing it is a policy change with a receipt.
pub const SMALL_LINES: u64 = 100;

/// Paths that define the ceremony itself; editing them always needs a standard
/// task. The ceremony skills, in either host, are added by `ceremony_skill`;
/// other skills are ordinary paths.
const CEREMONY: [&str; 5] = [
    "docs/ARTIFACTS.md",
    "docs/skills/author-artifact.md",
    "docs/skills/review-artifact.md",
    ".claude/agents/",
    ".codex/agents/",
];

/// Whether a change to this path needs a standard task whatever its size.
pub fn forces_standard(path: &str) -> bool {
    transitions::protected(path)
        || path == "Cargo.toml"
        || path.ends_with("/Cargo.toml")
        || transitions::ceremony_skill(path)
        || CEREMONY
            .iter()
            .any(|c| path == *c || (c.ends_with('/') && path.starts_with(c)))
}

/// Changed lines per path against `base`, including uncommitted and untracked
/// files. `None` marks a binary file. Moves count on both sides.
fn changes(root: &Path, base: &str) -> Result<Vec<(String, Option<u64>)>> {
    let mut out = Vec::new();
    let diff = String::from_utf8(history::git(
        root,
        &[
            "-c",
            "core.quotePath=false",
            "diff",
            "--numstat",
            "--no-renames",
            "-z",
            base,
        ],
    )?)?;
    for entry in diff.split('\0').filter(|e| !e.is_empty()) {
        let mut parts = entry.splitn(3, '\t');
        let (Some(added), Some(deleted), Some(path)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(format!("unreadable diff entry: {entry}").into());
        };
        let lines = match (added.parse::<u64>(), deleted.parse::<u64>()) {
            (Ok(a), Ok(d)) => Some(a + d),
            _ => None,
        };
        out.push((path.to_owned(), lines));
    }
    let untracked = String::from_utf8(history::git(
        root,
        &[
            "-c",
            "core.quotePath=false",
            "ls-files",
            "-o",
            "--exclude-standard",
            "-z",
        ],
    )?)?;
    for path in untracked.split('\0').filter(|p| !p.is_empty()) {
        let bytes = std::fs::read(root.join(path))?;
        let head = &bytes[..bytes.len().min(8000)];
        let lines = if head.contains(&0) {
            None
        } else {
            // Count lines as Git does: a final line without a newline still counts.
            let newlines = bytes.iter().filter(|&&b| b == b'\n').count() as u64;
            Some(newlines + u64::from(bytes.last().is_some_and(|&b| b != b'\n')))
        };
        out.push((path.to_owned(), lines));
    }
    Ok(out)
}

/// Refuses a small task whose branch is too large or touches a path that needs
/// the full ceremony. `base` is the merge base.
pub fn check_small(root: &Path, base: &str) -> Result<()> {
    let mut total = 0;
    let mut problems = Vec::new();
    for (path, lines) in changes(root, base)? {
        if artifacts::administrative(&path) {
            continue;
        }
        if forces_standard(&path) {
            problems.push(format!("{path} is a policy, dependency or ceremony path"));
        }
        match lines {
            Some(n) => total += n,
            None => problems.push(format!("{path} is a binary file")),
        }
    }
    if total > SMALL_LINES {
        problems.push(format!(
            "{total} changed lines exceed the small limit of {SMALL_LINES}"
        ));
    }
    if problems.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{}; a small task changes at most {SMALL_LINES} lines and none of these paths. Narrow the change, or restart it as a standard task on a new branch with its discovery pages committed before code (docs/ARTIFACTS.md)",
        problems.join("; ")
    )
    .into())
}
