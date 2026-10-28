//! Opt-in adapter for apply_patch hook events; shell and MCP writers need separate adapters.
use crate::{artifact_workflow, authored, config::Result};
use serde_json::{Value, json};
use std::{
    io::{self, Read},
    path::Path,
};
pub fn run(root: &Path, id: &str, point: &str) -> Result<Value> {
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
                let path = Path::new(p);
                let relative = if path.is_absolute() {
                    path.strip_prefix(root)?.to_string_lossy().into_owned()
                } else {
                    p.into()
                };
                authored::safe(root, &relative)?;
                paths.push(relative);
            }
        }
    }
    if paths.is_empty() {
        return Err("no recognized patch paths; cannot classify boundary".into());
    }
    let administrative = |p: &str| {
        [
            "harness/task-artifacts/",
            "harness/tasks/",
            "harness/transitions/",
            "docs/plans/",
        ]
        .iter()
        .any(|s| p.starts_with(s))
            || ["harness/docs.json", "harness/gaps.json"].contains(&p)
    };
    if paths.iter().all(|p| administrative(p)) {
        return Ok(
            json!({"ok":true,"scope":"Artifact/administrative edit; source boundary not crossed"}),
        );
    }
    artifact_workflow::guard(root, id, point)
}
