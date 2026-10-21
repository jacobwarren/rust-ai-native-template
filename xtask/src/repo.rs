use crate::config::Result;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn root() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    cwd.ancestors()
        .find(|p| p.join("harness/config.json").is_file() && p.join("Cargo.toml").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| "run from inside the starter workspace".into())
}

pub fn files(root: &Path) -> Result<Vec<PathBuf>> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if [".git", "target", ".DS_Store", "__pycache__"].contains(&name.as_ref())
                || name.ends_with(".pyc")
                || name == ".env"
                || name.starts_with(".env.") && name != ".env.example"
            {
                continue;
            }
            if entry.file_type()?.is_symlink() {
                return Err(format!(
                    "source symlink {} is unsupported; use a real file",
                    path.display()
                )
                .into());
            }
            if entry.file_type()?.is_dir() {
                walk(base, &path, out)?;
            } else {
                out.push(path.strip_prefix(base)?.to_path_buf());
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort();
    Ok(out)
}

pub fn fingerprint(root: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    for path in files(root)? {
        let name = path.to_string_lossy().replace('\\', "/");
        let bytes = std::fs::read(root.join(&path))?;
        hasher.update((name.len() as u64).to_le_bytes());
        hasher.update(name.as_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn git(root: &Path) -> Value {
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output();
    let status = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(root)
        .output();
    match (head, status) {
        (Ok(head), Ok(status)) if head.status.success() && status.status.success() => json!({
            "head":String::from_utf8_lossy(&head.stdout).trim(),
            "dirty":!status.stdout.is_empty()
        }),
        _ => json!({"head":null,"dirty":null}),
    }
}

pub fn worktree_id(root: &Path) -> String {
    let hash = Sha256::digest(root.to_string_lossy().as_bytes());
    format!("{hash:x}")[..12].to_owned()
}

pub fn metadata(root: &Path) -> Result<Value> {
    let output = Command::new("cargo")
        .args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
