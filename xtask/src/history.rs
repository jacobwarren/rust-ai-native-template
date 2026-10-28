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
