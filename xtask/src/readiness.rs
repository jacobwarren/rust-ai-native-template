//! Readiness is computed from current evidence, never from an agent's completion label.
use crate::{
    config::{self, Result, valid_id},
    repo,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Component, Path},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    schema_version: u32,
    requirements: Vec<Requirement>,
    scopes: std::collections::BTreeMap<String, Vec<String>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Requirement {
    id: String,
    implementation: Vec<String>,
    tests: Vec<String>,
    workflows: Vec<String>,
}
#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Gap {
    id: String,
    rank: u32,
    state: String,
    blocks: Vec<String>,
    impact: String,
    evidence: String,
    next_action: String,
    resolution: Option<String>,
}

fn local_file(root: &Path, name: &str) -> bool {
    let p = Path::new(name);
    !p.as_os_str().is_empty()
        && p.components().all(|c| matches!(c, Component::Normal(_)))
        && root.join(p).is_file()
}

fn gaps(root: &Path) -> Result<Vec<Gap>> {
    let mut gaps: Vec<Gap> = config::read(&root.join("harness/gaps.json"))?;
    let mut ids = BTreeSet::new();
    for gap in &gaps {
        if !valid_id(&gap.id)
            || !ids.insert(&gap.id)
            || !["open", "closed"].contains(&gap.state.as_str())
            || gap.impact.trim().is_empty()
            || gap.evidence.trim().is_empty()
            || gap.next_action.trim().is_empty()
            || (gap.state == "closed"
                && !gap.resolution.as_ref().is_some_and(|p| local_file(root, p)))
        {
            return Err("gap needs unique id, state, impact, evidence, next action and a checked-in resolution when closed".into());
        }
    }
    gaps.sort_by_key(|g| g.rank);
    Ok(gaps)
}

pub fn list_gaps(root: &Path) -> Result<Value> {
    Ok(json!({"ok":true,"gaps":gaps(root)?}))
}

pub fn assess(root: &Path, evidence: Option<&Path>, release: Option<&str>) -> Result<Value> {
    let policy: Policy = config::read(&root.join("harness/readiness.json"))?;
    if policy.schema_version != 1 || policy.requirements.is_empty() || policy.scopes.is_empty() {
        return Err("invalid/empty readiness policy".into());
    }
    let known: BTreeSet<_> = policy.requirements.iter().map(|r| r.id.as_str()).collect();
    if known.len() != policy.requirements.len() {
        return Err("duplicate requirement".into());
    }
    for r in &policy.requirements {
        if !valid_id(&r.id)
            || r.implementation.is_empty()
            || r.tests.is_empty()
            || r.workflows.is_empty()
        {
            return Err("requirement needs id, implementation, tests and workflows".into());
        }
    }
    for (scope, ids) in &policy.scopes {
        if !valid_id(scope) || ids.is_empty() || ids.iter().any(|id| !known.contains(id.as_str())) {
            return Err("invalid release scope/requirements".into());
        }
    }
    if release.is_some_and(|s| !policy.scopes.contains_key(s)) {
        return Err("unknown release scope".into());
    }
    let gaps = gaps(root)?;
    if gaps
        .iter()
        .any(|g| g.blocks.iter().any(|s| !policy.scopes.contains_key(s)))
    {
        return Err("gap names unknown release scope".into());
    }
    let mut errors = Vec::new();
    let mut passed_tests = BTreeSet::new();
    let mut passed_workflows = BTreeSet::new();
    if let Some(path) = evidence {
        match verify_evidence(root, path) {
            Ok((tests, workflows)) => {
                passed_tests = tests;
                passed_workflows = workflows;
            }
            Err(e) => errors.push(e.to_string()),
        }
    } else {
        errors.push("no CI evidence supplied".into());
    }
    let mut requirements = Vec::new();
    for r in &policy.requirements {
        let implemented = r.implementation.iter().all(|p| local_file(root, p));
        let missing_tests: Vec<_> = r
            .tests
            .iter()
            .filter(|t| !passed_tests.contains(t.as_str()))
            .collect();
        let missing_workflows: Vec<_> = r
            .workflows
            .iter()
            .filter(|w| !passed_workflows.contains(w.as_str()))
            .collect();
        let verified = implemented
            && errors.is_empty()
            && missing_tests.is_empty()
            && missing_workflows.is_empty();
        requirements.push(json!({"id":r.id,"state":if verified {"verified"} else if implemented {"implemented"} else {"missing"},
            "missing_tests":missing_tests,"missing_workflows":missing_workflows}));
    }
    let scopes: Vec<_> = policy.scopes.iter().map(|(scope, ids)| {
        let blockers: Vec<_> = gaps.iter().filter(|g| g.state == "open" && g.blocks.contains(scope)).map(|g| &g.id).collect();
        let verified = ids.iter().all(|id| requirements.iter().any(|r| r["id"] == *id && r["state"] == "verified"));
        json!({"id":scope,"state":if verified && blockers.is_empty() {"release_ready"} else {"blocked"},"blockers":blockers,"required":ids})
    }).collect();
    let ok = release.is_none_or(|scope| {
        scopes
            .iter()
            .any(|s| s["id"] == scope && s["state"] == "release_ready")
    });
    Ok(
        json!({"ok":ok,"kind":"readiness","evidence":evidence,"source_sha256":repo::fingerprint(root)?,
        "requirements":requirements,"scopes":scopes,"evidence_errors":errors,"scope":"Local evidence for the named scope only. No deployment, signature, hosting policy or cross-platform attestation."}),
    )
}

fn verify_evidence(root: &Path, path: &Path) -> Result<(BTreeSet<String>, BTreeSet<String>)> {
    let report: Value = config::read(path)?;
    let source = repo::fingerprint(root)?;
    if report["kind"] != "verification"
        || report["profile"] != "ci"
        || report["complete"] != true
        || report["ok"] != true
        || report["source_changed"] != false
        || report["source_sha256_before"] != source
        || report["source_sha256_after"] != source
    {
        return Err("CI evidence is stale, incomplete, failed or not the ci profile".into());
    }
    let configuration = config::load(root)?;
    let required = configuration
        .profiles
        .get("ci")
        .ok_or("missing ci profile")?;
    let checks = report["checks"].as_array().ok_or("missing checks")?;
    if checks.len() != required.len() {
        return Err("CI check set differs from current policy".into());
    }
    for step in required {
        let matched: Vec<_> = checks.iter().filter(|c| c["id"] == step.id()).collect();
        if matched.len() != 1
            || matched[0]["status"] != "passed"
            || matched[0]["exit_code"] != 0
            || matched[0]["argv"] != json!(step.argv())
        {
            return Err(format!("missing/mismatched/failed CI step {}", step.id()).into());
        }
    }
    let log_for = |id: &str| -> Result<String> {
        let c = checks
            .iter()
            .find(|c| c["id"] == id)
            .ok_or("required evidence check absent")?;
        Ok(std::fs::read_to_string(
            c["stdout"].as_str().ok_or("missing stdout path")?,
        )?)
    };
    let tests = log_for("tests")?
        .lines()
        .filter_map(|line| {
            line.strip_prefix("test ")?
                .strip_suffix(" ... ok")
                .map(String::from)
        })
        .collect();
    let workflows: Value = serde_json::from_str(&log_for("workflows")?)?;
    if workflows["kind"] != "workflows"
        || workflows["complete"] != true
        || workflows["ok"] != true
        || workflows["source_sha256"] != source
        || workflows["source_sha256_after"] != source
    {
        return Err("workflow evidence is stale/incomplete/failed".into());
    }
    let cases = workflows["cases"]
        .as_array()
        .ok_or("missing workflow cases")?;
    let mut passed = BTreeSet::new();
    for case in cases {
        let id = case["id"].as_str().ok_or("missing workflow id")?;
        if case["ok"] != true || case["boundary"] != "process" || !passed.insert(id.to_owned()) {
            return Err("invalid workflow result".into());
        }
    }
    Ok((tests, passed))
}
