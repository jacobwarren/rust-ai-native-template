//! Opt-in adapter for apply_patch hook events; shell and MCP writers need separate adapters.
use crate::{artifacts, config::Result};
use serde_json::{Value, json};
use std::{
    io::{self, Read},
    path::Path,
};
pub fn run(root: &Path, id: &str) -> Result<Value> {
    let mut input = String::new();
    io::stdin().take(1_048_577).read_to_string(&mut input)?;
    if input.len() > 1_048_576 {
        return Err("hook event exceeds 1 MiB".into());
    }
    let event: Value = serde_json::from_str(&input)?;
    if event["tool_name"] != "apply_patch" {
        return Err(
            "adapter only supports apply_patch; do not assume other tools are covered".into(),
        );
    }
    let patch = event["tool_input"]["command"]
        .as_str()
        .ok_or("missing apply_patch command")?;
    let mut paths = vec![];
    for line in patch.lines() {
        for prefix in [
            "*** Add File: ",
            "*** Update File: ",
            "*** Delete File: ",
            "*** Move to: ",
        ] {
            if let Some(p) = line.strip_prefix(prefix) {
                paths.push(artifacts::relative(root, p)?);
            }
        }
    }
    if paths.is_empty() {
        return Err("no recognized patch paths; cannot classify boundary".into());
    }
    if paths.iter().all(|p| artifacts::administrative(p)) {
        return Ok(
            json!({"ok":true,"scope":"Artifact/administrative edit; source boundary not crossed"}),
        );
    }
    artifacts::guard(root, id)
}
