//! The same admission gate is callable from an agent hook and PR CI.
use crate::{
    artifacts,
    config::{self, Result},
    history, readiness, repo,
    task_record::{self, GapState, Status},
    tracking, transitions,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

pub fn task(
    root: &Path,
    id: &str,
    base: &str,
    report: Option<&Path>,
    must_complete: bool,
) -> Result<Value> {
    let task = task_record::read(root, id)?;
    let artifact_result = artifacts::check(root, id, task.status == Status::Complete, Some(base));
    let current = match tracking::load(root) {
        Ok(gaps) => gaps,
        Err(e)
            if task.status == Status::Blocked
                && !must_complete
                && task_record::review_errors(&task).is_empty()
                && artifact_result.is_ok() =>
        {
            return Ok(json!({"ok":true,"complete":false,"task":task.id,
                "reconciliation_pending":true,"gap_register_error":e.to_string(),
                "summary":task.summary,"remaining":task.remaining}));
        }
        Err(e) => return Err(e),
    };
    let old = tracking::load_from(|p| history::read(root, base, p))?;
    let mut errors = task_record::review_errors(&task);
    if let Err(e) = artifact_result {
        errors.push(format!("task HTML artifact: {e}"));
    }
    if must_complete && task.status != Status::Complete {
        errors.push("task is partial/blocked; use handoff, do not claim completion".into());
    }
    let assessment = readiness::assess(root, report.filter(|p| p.is_file()), None)?;
    if task.status == Status::Complete
        && !assessment["evidence_errors"]
            .as_array()
            .is_some_and(|es| es.is_empty())
    {
        errors.push("complete task requires current full-CI evidence".into());
    }
    for id in &task.requirements {
        if !assessment["requirements"]
            .as_array()
            .is_some_and(|rs| rs.iter().any(|r| r["id"] == *id))
        {
            errors.push(format!("unknown acceptance requirement {id}"));
        } else if task.status == Status::Complete
            && !tracking::requirement_verified(&assessment, id)
        {
            errors.push(format!("acceptance requirement {id} is not verified"));
        }
    }
    for related in &task.related_gaps {
        let gap = current.iter().find(|g| g.id == related.id);
        let previous = old.iter().find(|g| g.id == related.id);
        let valid = match related.disposition {
            GapState::Resolved => gap
                .is_some_and(|g| g.state == "closed" && tracking::closure_verified(g, &assessment)),
            GapState::Updated => match (gap, previous) {
                (Some(g), Some(p)) => history::json_digest(g)? != history::json_digest(p)?,
                (Some(_), None) => true,
                _ => false,
            },
            GapState::Unaffected => match (gap, previous) {
                (Some(g), Some(p)) => history::json_digest(g)? == history::json_digest(p)?,
                _ => false,
            },
            GapState::Retired => gap.is_none() && previous.is_some(),
        };
        if !valid {
            errors.push(format!(
                "gap {} does not support its declared disposition",
                related.id
            ));
        }
    }
    for d in &task.discoveries {
        let valid = if let Some(id) = d.record.strip_prefix("gap:") {
            current.iter().any(|g| g.id == id)
        } else {
            tracking::reference(root, &d.record)
        };
        if !valid {
            errors.push(format!("discovery has no durable record: {}", d.record));
        }
    }
    for p in &task.documentation.updated {
        if history::read(root, base, p)?.as_deref() == Some(std::fs::read(root.join(p))?.as_slice())
        {
            errors.push(format!("documentation marked updated but unchanged: {p}"));
        }
    }
    Ok(
        json!({"ok":errors.is_empty(),"task":task.id,"work_item":task.work_item,
        "complete":task.status == Status::Complete && errors.is_empty(),"summary":task.summary,
        "remaining":task.remaining,"errors":errors}),
    )
}

pub fn complete(root: &Path, id: &str, revision: &str, report: &Path) -> Result<Value> {
    let base = history::merge_base(root, revision)?;
    let transitions = transitions::check(root, &base, Some(report))?;
    let task = task(root, id, &base, Some(report), true)?;
    Ok(
        json!({"ok":transitions["ok"] == true && task["ok"] == true,"kind":"completion",
        "base_commit":base,"source_sha256":repo::fingerprint(root)?,"task":task,"transitions":transitions}),
    )
}

pub fn handoff(root: &Path, revision: &str, report: Option<&Path>) -> Result<Value> {
    let base = history::merge_base(root, revision)?;
    let before = history::before(root, &base)?;
    let after = history::current(root)?;
    let changed = before != after;
    let records = history::records(root, "harness/tasks")?;
    let mut results = Vec::new();
    let mut covered = BTreeSet::new();
    let mut errors = Vec::new();
    for path in before.keys().filter(|p| p.starts_with("harness/tasks/")) {
        if !after.contains_key(path) {
            errors.push(format!("task history deleted: {path}"));
        }
    }
    for path in records.iter().filter(|p| {
        let id = Path::new(p)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let prefix = format!("{}/", artifacts::directory(id));
        before.get(*p) != after.get(*p)
            || before
                .keys()
                .chain(after.keys())
                .any(|name| name.starts_with(&prefix) && before.get(name) != after.get(name))
    }) {
        let id = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("invalid task filename")?;
        let record = task_record::read(root, id)?;
        covered.extend(record.related_gaps.iter().map(|g| g.id.clone()));
        covered.extend(
            record
                .discoveries
                .iter()
                .filter_map(|d| d.record.strip_prefix("gap:").map(String::from)),
        );
        results.push(task(root, id, &base, report, false)?);
    }
    if changed && results.is_empty() {
        errors.push("changed source requires an added/updated task reconciliation record".into());
    }
    for path in before
        .keys()
        .chain(after.keys())
        .filter(|p| p.starts_with("harness/task-artifacts/"))
    {
        if before.get(path) != after.get(path) {
            let id = path.split('/').nth(2).unwrap_or("");
            if !after.contains_key(&format!("harness/tasks/{id}.json")) {
                errors.push(format!("artifact has no task record: {id}"));
            }
        }
    }
    let old = tracking::load_from(|p| history::read(root, &base, p))?;
    let new = tracking::load(root)?;
    for id in old
        .iter()
        .chain(&new)
        .map(|g| &g.id)
        .collect::<BTreeSet<_>>()
    {
        let a = old
            .iter()
            .find(|g| g.id == *id)
            .map(history::json_digest)
            .transpose()?;
        let b = new
            .iter()
            .find(|g| g.id == *id)
            .map(history::json_digest)
            .transpose()?;
        if a != b && !covered.contains(id) {
            errors.push(format!(
                "changed gap {id} is absent from task reconciliation"
            ));
        }
    }
    let transitions = transitions::check(root, &base, report)?;
    Ok(
        json!({"ok":errors.is_empty() && results.iter().all(|r| r["ok"] == true) && transitions["ok"] == true,
        "kind":"handoff","base_commit":base,"source_sha256":repo::fingerprint(root)?,
        "all_tasks_complete":!results.is_empty() && results.iter().all(|r| r["complete"] == true),
        "tasks":results,"transitions":transitions,"errors":errors}),
    )
}

pub fn validate(root: &Path) -> Result<Value> {
    let tracking = tracking::validate(root)?;
    let mut ids = Vec::new();
    for path in history::records(root, "harness/tasks")? {
        let id = Path::new(&path)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("invalid task filename")?;
        let record = task_record::read(root, id)?;
        if record.schema_version >= 6 {
            artifacts::check(root, id, record.status == Status::Complete, None)?;
        }
        ids.push(id.to_owned());
    }
    // Old records are retained but their evidence is not re-asserted on every edit.
    let _: Value = config::read(&root.join("harness/readiness.json"))?;
    Ok(json!({"ok":true,"tasks":ids,"tracking":tracking}))
}

/// A stop hook permits an explicit blocked handoff even when the gap register needs repair.
/// It never turns that exception into a completion or PR reconciliation claim.
pub fn agent_stop(root: &Path, id: &str, revision: &str, report: &Path) -> Result<Value> {
    let record = task_record::read(root, id)?;
    if record.status == Status::Complete {
        return complete(root, id, revision, report);
    }
    let base = history::merge_base(root, revision)?;
    let task = task(root, id, &base, Some(report), false)?;
    if task["reconciliation_pending"] == true {
        return Ok(
            json!({"ok":true,"complete":false,"kind":"blocked_handoff","task":task,
            "warning":"Stopping is allowed; gap reconciliation and PR admission remain blocked."}),
        );
    }
    let transitions = transitions::check(root, &base, report.is_file().then_some(report))?;
    Ok(json!({"ok":task["ok"] == true && transitions["ok"] == true,
        "complete":false,"kind":"handoff","task":task,"transitions":transitions}))
}
