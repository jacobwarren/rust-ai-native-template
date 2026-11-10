//! The small tier, the default: a task that owes no pages. It may be any size,
//! but not touch the paths where a mistake changes what every later task is held
//! to; those owe a brief first. The gate reads the branch against its base, so a
//! task cannot declare itself small and then edit a gate. See docs/ARTIFACTS.md.
use crate::{artifacts, config::Result, history, transitions};
use std::path::Path;

/// Paths that define the ceremony itself; editing them always needs a standard
/// task and its brief. The ceremony skills, in either host, are added by `ceremony_skill`;
/// other skills are ordinary paths.
const CEREMONY: [&str; 5] = [
    "docs/ARTIFACTS.md",
    "docs/skills/author-artifact.md",
    "docs/skills/review-artifact.md",
    ".claude/agents/",
    ".codex/agents/",
];

/// Protected files that record what the product must do. Product work edits them
/// with every feature, so they need their transition receipt but not a brief.
const PRODUCT_ACCEPTANCE: [&str; 5] = [
    "harness/workflows.json",
    "harness/scenarios.json",
    "harness/readiness.json",
    "harness/goldens/",
    "docs/PRODUCT.md",
];

/// Whether a change to this path needs a standard task whatever its size.
pub fn forces_standard(path: &str) -> bool {
    let acceptance = PRODUCT_ACCEPTANCE
        .iter()
        .any(|p| path == *p || (p.ends_with('/') && path.starts_with(p)));
    (transitions::protected(path) && !acceptance)
        || path == "Cargo.toml"
        || path.ends_with("/Cargo.toml")
        || transitions::ceremony_skill(path)
        || CEREMONY
            .iter()
            .any(|c| path == *c || (c.ends_with('/') && path.starts_with(c)))
}

/// Paths changed against `base`, including uncommitted and untracked files.
fn changed(root: &Path, base: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for args in [
        &[
            "-c",
            "core.quotePath=false",
            "diff",
            "--name-only",
            "--no-renames",
            "-z",
            base,
        ][..],
        &[
            "-c",
            "core.quotePath=false",
            "ls-files",
            "-o",
            "--exclude-standard",
            "-z",
        ][..],
    ] {
        let text = String::from_utf8(history::git(root, args)?)?;
        out.extend(text.split('\0').filter(|p| !p.is_empty()).map(String::from));
    }
    Ok(out)
}

/// Refuses a small task whose branch touches a path that owes a brief. `base` is
/// the merge base.
pub fn check_small(root: &Path, base: &str) -> Result<()> {
    let problems: Vec<String> = changed(root, base)?
        .into_iter()
        .filter(|path| !artifacts::administrative(path) && forces_standard(path))
        .map(|path| format!("{path} is a policy, dependency or ceremony path"))
        .collect();
    if problems.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{}; a task without pages changes none of these paths. Narrow the change, or restart it as a standard task on a new branch with its brief committed before code (docs/ARTIFACTS.md)",
        problems.join("; ")
    )
    .into())
}
