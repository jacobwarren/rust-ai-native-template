//! Git comparison helpers. Resolve caller revisions to object IDs before use.
use crate::{config::Result, repo};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path, process::Command};

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    if !output.status.success() {
        return Err(format!(
            "Git history unavailable: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
pub fn merge_base(root: &Path, revision: &str) -> Result<String> {
    let commit = String::from_utf8(git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{revision}^{{commit}}"),
        ],
    )?)?;
    Ok(
        String::from_utf8(git(root, &["merge-base", commit.trim(), "HEAD"])?)?
            .trim()
            .to_owned(),
    )
}
pub fn paths(root: &Path, base: &str) -> Result<Vec<String>> {
    String::from_utf8(git(root, &["ls-tree", "-r", "--name-only", "-z", base])?)?
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| Ok(s.to_owned()))
        .collect()
}
pub fn read(root: &Path, base: &str, path: &str) -> Result<Option<Vec<u8>>> {
    if !paths(root, base)?.iter().any(|p| p == path) {
        return Ok(None);
    }
    Ok(Some(git(root, &["show", &format!("{base}:{path}")])?))
}
/// Paths changed by each of the branch's own commits since `base`, oldest first.
/// Merge commits are skipped so changes merged in from the target do not count.
pub fn commits(root: &Path, base: &str) -> Result<Vec<Vec<String>>> {
    let range = format!("{base}..HEAD");
    let out = String::from_utf8(git(
        root,
        &[
            "-c",
            "core.quotePath=false",
            "log",
            "--first-parent",
            "--no-merges",
            "--no-renames",
            "--reverse",
            "--name-only",
            "-z",
            "--format=%x01",
            &range,
        ],
    )?)?;
    Ok(out
        .split('\u{1}')
        .skip(1)
        .map(|c| {
            c.split('\0')
                .map(|p| p.trim_start_matches('\n'))
                .filter(|p| !p.is_empty())
                .map(String::from)
                .collect()
        })
        .collect())
}
/// Tracked paths with uncommitted changes, plus untracked paths that are not ignored.
pub fn uncommitted(root: &Path) -> Result<Vec<String>> {
    let out = String::from_utf8(git(
        root,
        &[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--no-renames",
        ],
    )?)?;
    Ok(out
        .split('\0')
        .filter(|e| e.len() > 3)
        .map(|e| e[3..].to_owned())
        .collect())
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn json_digest<T: serde::Serialize + ?Sized>(value: &T) -> Result<String> {
    Ok(digest(&serde_json::to_vec(value)?))
}
pub fn current(root: &Path) -> Result<BTreeMap<String, String>> {
    repo::files(root)?
        .iter()
        .map(|p| {
            Ok((
                p.to_string_lossy().replace('\\', "/"),
                digest(&std::fs::read(root.join(p))?),
            ))
        })
        .collect()
}
pub fn before(root: &Path, base: &str) -> Result<BTreeMap<String, String>> {
    paths(root, base)?
        .iter()
        .map(|p| {
            Ok((
                p.clone(),
                digest(&git(root, &["show", &format!("{base}:{p}")])?),
            ))
        })
        .collect()
}
pub fn records(root: &Path, directory: &str) -> Result<Vec<String>> {
    Ok(repo::files(root)?
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .filter(|p| p.starts_with(&format!("{directory}/")) && p.ends_with(".json"))
        .collect())
}
