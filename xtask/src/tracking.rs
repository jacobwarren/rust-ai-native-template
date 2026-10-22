//! Project engineering gaps, maintained by people/agents; ticket references are opaque.
use crate::config::{Result, valid_id};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Component, Path},
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Gap {
    pub id: String,
    pub rank: u32,
    pub state: String,
    pub blocks: Vec<String>,
    pub impact: String,
    pub evidence: String,
    pub next_action: String,
    pub resolution: Option<String>,
    #[serde(default)]
    pub requirements: Vec<String>,
    #[serde(default)]
    pub ticket: Option<String>,
}

pub fn local_path(name: &str) -> bool {
    !name.is_empty()
        && Path::new(name)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}
pub fn source_path(name: &str) -> bool {
    local_path(name)
        && Path::new(name).components().all(|c| {
            let name = c.as_os_str().to_string_lossy();
            ![".git", "target", ".DS_Store", "__pycache__"].contains(&name.as_ref())
                && !name.ends_with(".pyc")
                && name != ".env"
                && !(name.starts_with(".env.") && name != ".env.example")
        })
}
pub fn source_file(root: &Path, name: &str) -> bool {
    source_path(name) && root.join(name).is_file()
}
pub fn nonempty(s: &str) -> bool {
    !s.trim().is_empty()
}
pub fn reference(root: &Path, s: &str) -> bool {
    (s.starts_with("https://") && s.len() > 8 && !s.chars().any(char::is_whitespace))
        || source_file(root, s)
}
pub fn unique(values: &[String]) -> bool {
    values.iter().all(|s| nonempty(s))
        && values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

/// Read the project register, including historical versions for transition checks.
pub fn load_from(mut read: impl FnMut(&str) -> Result<Option<Vec<u8>>>) -> Result<Vec<Gap>> {
    // Never silently interpret an old external-authority installation as local.
    if let Some(bytes) = read("harness/tracking.json")? {
        let old: Value = serde_json::from_slice(&bytes)?;
        if old["schema_version"] != 1 || old["mode"] != "repository" {
            return Err("external tracker authority was removed: migrate reviewed gaps into harness/gaps.json and remove harness/tracking.json; see docs/COMPLETION.md".into());
        }
    }
    let required = |bytes: Option<Vec<u8>>| -> Result<Vec<u8>> {
        bytes.ok_or_else(|| "missing project gap register or readiness policy".into())
    };
    let mut gaps: Vec<Gap> = serde_json::from_slice(&required(read("harness/gaps.json")?)?)?;
    let policy: Value = serde_json::from_slice(&required(read("harness/readiness.json")?)?)?;
    let mut ids = BTreeSet::new();
    for g in &gaps {
        if !valid_id(&g.id)
            || !ids.insert(g.id.clone())
            || !["open", "closed"].contains(&g.state.as_str())
            || !nonempty(&g.impact)
            || !nonempty(&g.evidence)
            || !nonempty(&g.next_action)
            || !unique(&g.blocks)
            || !unique(&g.requirements)
            || g.blocks.iter().any(|s| policy["scopes"].get(s).is_none())
            || g.requirements.iter().any(|id| {
                !policy["requirements"]
                    .as_array()
                    .is_some_and(|rs| rs.iter().any(|r| r["id"] == *id))
            })
            || g.ticket.as_ref().is_some_and(|s| !nonempty(s))
        {
            return Err(format!("invalid gap {}: check identity, state, fields, requirements, scope and optional reference", g.id).into());
        }
        if g.state == "closed" {
            let resolution = g
                .resolution
                .as_ref()
                .ok_or("closed gap needs a resolution")?;
            if !source_path(resolution) || read(resolution)?.is_none() {
                return Err(format!("gap {} needs an existing resolution record", g.id).into());
            }
        }
    }
    gaps.sort_by_key(|g| (g.rank, g.id.clone()));
    Ok(gaps)
}

pub fn load(root: &Path) -> Result<Vec<Gap>> {
    load_from(|p| match std::fs::read(root.join(p)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    })
}
pub fn list(root: &Path) -> Result<Value> {
    Ok(
        json!({"ok":true,"kind":"gap_register","scope":"project","generates_gaps":false,"gaps":load(root)?}),
    )
}
pub fn requirement_verified(assessment: &Value, id: &str) -> bool {
    assessment["requirements"]
        .as_array()
        .is_some_and(|rs| rs.iter().any(|r| r["id"] == id && r["state"] == "verified"))
}
pub fn closure_verified(gap: &Gap, assessment: &Value) -> bool {
    gap.requirements
        .iter()
        .all(|id| requirement_verified(assessment, id))
}
pub fn validate(root: &Path) -> Result<Value> {
    list(root)
}
