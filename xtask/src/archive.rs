//! History that left the tree. `harness/archive.json` names the commit that still
//! holds each archived path, so task records, pages and receipts can be deleted
//! without losing them. A deletion the manifest does not cover is still refused.
use crate::{
    config::{self, Result},
    history, tracking,
};
use serde::Deserialize;
use std::path::Path;

pub const FILE: &str = "harness/archive.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    archives: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// A branch or tag that names the commit, for people; the gate reads `commit`.
    pub name: String,
    pub commit: String,
    pub reason: String,
    /// Files, or folders ending in `/`, that the commit holds and the tree no longer does.
    pub paths: Vec<String>,
}

/// The manifest's entries; none when the file does not exist.
pub fn load(root: &Path) -> Result<Vec<Entry>> {
    if !root.join(FILE).is_file() {
        return Ok(Vec::new());
    }
    let manifest: Manifest = config::read(&root.join(FILE))?;
    let valid = manifest.schema_version == 1
        && manifest.archives.iter().all(|e| {
            tracking::nonempty(&e.name)
                && tracking::nonempty(&e.reason)
                && e.commit.len() == 40
                && e.commit.chars().all(|c| c.is_ascii_hexdigit())
                && !e.paths.is_empty()
                && tracking::unique(&e.paths)
                && e.paths
                    .iter()
                    .all(|p| tracking::source_path(p.trim_end_matches('/')))
        });
    if !valid {
        return Err(format!(
            "{FILE}: each archive needs a name, a reason, a full commit id and unique repository paths"
        )
        .into());
    }
    Ok(manifest.archives)
}

fn lists(entry: &Entry, path: &str) -> bool {
    entry
        .paths
        .iter()
        .any(|p| p == path || (p.ends_with('/') && path.starts_with(p.as_str())))
}

fn blob(root: &Path, commit: &str, path: &str) -> Result<String> {
    Ok(String::from_utf8(history::git(
        root,
        &["rev-parse", "--verify", &format!("{commit}:{path}")],
    )?)?
    .trim()
    .to_owned())
}

/// Whether a path deleted since `base` is archived: a manifest entry lists it and
/// that entry's commit holds the same content the base did. A listed path the
/// commit lacks or holds differently, or a commit this repository does not have,
/// is an error, never a quiet pass. The commit is read only here, when a deletion
/// relies on it, so a tree unpacked without history still validates.
pub fn covers(root: &Path, entries: &[Entry], base: &str, path: &str) -> Result<bool> {
    let Some(entry) = entries.iter().find(|e| lists(e, path)) else {
        return Ok(false);
    };
    history::paths(root, &entry.commit).map_err(|_| {
        format!(
            "{FILE}: archive commit {} ({}) is not in this repository; fetch it",
            entry.commit, entry.name
        )
    })?;
    let kept = blob(root, &entry.commit, path).map_err(|_| {
        format!(
            "{FILE}: {path} is listed under {} but commit {} does not hold it",
            entry.name, entry.commit
        )
    })?;
    if kept != blob(root, base, path)? {
        return Err(format!(
            "{FILE}: {path} changed after commit {}; archive a commit that holds the version being deleted",
            entry.commit
        )
        .into());
    }
    Ok(true)
}

/// Structure check for `coordination`: a listed path is gone from the tree, since
/// a path still present is not archived. Returns how many archives are listed.
pub fn validate(root: &Path) -> Result<usize> {
    let entries = load(root)?;
    for entry in &entries {
        for path in &entry.paths {
            // Git keeps no empty folder, so one left behind on disk is not in the tree.
            let on_disk = root.join(path.trim_end_matches('/'));
            let present = on_disk.is_file()
                || std::fs::read_dir(&on_disk).is_ok_and(|mut entries| entries.next().is_some());
            if present {
                return Err(format!(
                    "{FILE}: {path} is listed as archived but is still in the tree"
                )
                .into());
            }
        }
    }
    Ok(entries.len())
}
