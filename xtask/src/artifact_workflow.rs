//! Ordered, content-bound checkpoints. These are audit records, not trusted timestamps.
use crate::{
    authored::{self, Ledger, Revision, Snapshot},
    config::{self, Result},
    history, repo, tracking,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Discovery,
    Implementation,
    Final,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub phase: Phase,
    pub work_sha256: String,
    pub plan_evidence: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanEvidence {
    schema_version: u32,
    plan_sha256: String,
    test_command: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    choice: String,
    before: String,
    after: String,
    response: String,
}

pub fn work_hash(root: &Path) -> Result<String> {
    let mut entries = Vec::new();
    for p in repo::files(root)? {
        let name = p.to_string_lossy().replace('\\', "/");
        if [
            "harness/task-artifacts/",
            "harness/tasks/",
            "harness/transitions/",
            "docs/plans/",
        ]
        .iter()
        .any(|s| name.starts_with(s))
            || ["harness/docs.json", "harness/gaps.json"].contains(&name.as_str())
        {
            continue;
        }
        entries.push((name, history::digest(&fs::read(root.join(p))?)));
    }
    history::json_digest(&entries)
}
fn has(files: &[Snapshot], purpose: &str) -> bool {
    files.iter().any(|s| s.file.purpose == purpose)
}
fn notes(files: &[Snapshot]) -> Option<&str> {
    files
        .iter()
        .find(|s| s.file.purpose == "implementation")
        .map(|s| s.sha256.as_str())
}
fn phase(files: &[Snapshot]) -> Result<Phase> {
    if has(files, "buy-in") || has(files, "quiz") {
        if !has(files, "buy-in") || !has(files, "quiz") || !has(files, "implementation") {
            return Err(
                "final publication needs implementation notes, buy-in and quiz together".into(),
            );
        }
        Ok(Phase::Final)
    } else if has(files, "implementation") {
        Ok(Phase::Implementation)
    } else {
        Ok(Phase::Discovery)
    }
}
fn ordered(previous: &[Revision], files: &[Snapshot], cp: &Checkpoint) -> Result<()> {
    if cp.phase != phase(files)? || cp.work_sha256.len() != 64 {
        return Err("invalid checkpoint phase or work identity".into());
    }
    let last = previous.last().and_then(|r| r.checkpoint.as_ref());
    match cp.phase {
        Phase::Discovery if last.is_some_and(|c| c.phase != Phase::Discovery) => {
            return Err("cannot backdate discovery after implementation".into());
        }
        Phase::Implementation => {
            let before = previous
                .iter()
                .find_map(|r| {
                    r.checkpoint
                        .as_ref()
                        .filter(|c| c.phase == Phase::Discovery)
                })
                .ok_or("register discovery before implementation notes")?;
            if before.work_sha256 == cp.work_sha256 {
                return Err("implementation checkpoint needs work changed since discovery".into());
            }
            if let Some(r) = previous.last()
                && let Some(old) = &r.checkpoint
                && old.work_sha256 != cp.work_sha256
                && notes(&r.files) == notes(files)
            {
                return Err("changed work needs updated implementation notes".into());
            }
        }
        Phase::Final => {
            let note = previous
                .iter()
                .rev()
                .find(|r| {
                    r.checkpoint
                        .as_ref()
                        .is_some_and(|c| c.phase == Phase::Implementation)
                })
                .ok_or("register a separate implementation checkpoint before final publication")?;
            if note
                .checkpoint
                .as_ref()
                .is_none_or(|c| c.work_sha256 != cp.work_sha256)
                || notes(&note.files) != notes(files)
            {
                return Err("final publication needs a current implementation checkpoint; update notes and register without buy-in/quiz first".into());
            }
        }
        _ => {}
    }
    Ok(())
}
fn evidence(root: &Path, id: &str, prefix: &str, files: &[Snapshot], report: &str) -> Result<()> {
    let plan = files
        .iter()
        .find(|s| s.file.purpose == "plan")
        .ok_or("missing plan")?;
    if !files
        .iter()
        .any(|s| s.file.purpose == "supporting" && s.file.path == report)
    {
        return Err("plan_evidence must name a registered supporting JSON report".into());
    }
    let e: PlanEvidence = config::read(&authored::path(root, id, &format!("{prefix}{report}"))?)?;
    let mut choices = BTreeSet::new();
    let mut outcomes = BTreeSet::new();
    if e.schema_version != 1
        || e.plan_sha256 != plan.sha256
        || !tracking::nonempty(&e.test_command)
        || e.cases.len() < 2
    {
        return Err(
            "plan evidence needs current HTML hash, test command and two exercised choices".into(),
        );
    }
    for c in e.cases {
        if !tracking::nonempty(&c.choice)
            || !tracking::nonempty(&c.before)
            || !tracking::nonempty(&c.after)
            || !tracking::nonempty(&c.response)
            || c.before == c.after
            || !choices.insert(c.choice)
        {
            return Err(
                "plan choices must change a visible consequence and produce a usable response"
                    .into(),
            );
        }
        outcomes.insert(c.after);
    }
    if outcomes.len() < 2 {
        return Err("plan alternatives need distinct consequences".into());
    }
    Ok(())
}
pub fn prepare(
    root: &Path,
    id: &str,
    ledger: &Ledger,
    files: &[Snapshot],
    report: Option<&str>,
) -> Result<Option<Checkpoint>> {
    if ledger.schema_version == 1 {
        return Ok(None);
    }
    let cp = Checkpoint {
        phase: phase(files)?,
        work_sha256: work_hash(root)?,
        plan_evidence: report
            .ok_or("new workflow requires plan_evidence; see docs/ARTIFACT-CHECKPOINTS.md")?
            .into(),
    };
    ordered(&ledger.revisions, files, &cp)?;
    evidence(root, id, "", files, &cp.plan_evidence)?;
    Ok(Some(cp))
}
pub fn history_check(root: &Path, id: &str, l: &Ledger) -> Result<()> {
    if l.schema_version == 1 {
        return Ok(());
    }
    for (i, r) in l.revisions.iter().enumerate() {
        let cp = r.checkpoint.as_ref().ok_or("missing ordered checkpoint")?;
        ordered(&l.revisions[..i], &r.files, cp)?;
        evidence(
            root,
            id,
            &format!("authored-revisions/{}/", i + 1),
            &r.files,
            &cp.plan_evidence,
        )?;
    }
    Ok(())
}
pub fn final_check(root: &Path, l: &Ledger) -> Result<()> {
    if l.schema_version == 1 {
        return Ok(());
    }
    let cp = l
        .revisions
        .last()
        .and_then(|r| r.checkpoint.as_ref())
        .ok_or("missing checkpoint")?;
    if cp.phase != Phase::Final || cp.work_sha256 != work_hash(root)? {
        return Err("completion requires final publication for current work after implementation checkpoint".into());
    }
    Ok(())
}
pub fn guard(root: &Path, id: &str, point: &str) -> Result<Value> {
    if !["before-edit", "after-edit", "before-final"].contains(&point) {
        return Err("unknown artifact boundary".into());
    }
    authored::check(root, id, false, None)?;
    let l = authored::read(root, id)?;
    if l.schema_version != 2 {
        return Err("boundary guard requires an ordered schema-2 ledger; legacy records do not attest to sequence".into());
    }
    let cp = l
        .revisions
        .last()
        .and_then(|r| r.checkpoint.as_ref())
        .ok_or("missing checkpoint")?;
    if cp.work_sha256 != work_hash(root)? {
        return Err("work changed since checkpoint: append implementation notes and register a separate implementation revision before continuing".into());
    }
    if point == "before-final" && cp.phase != Phase::Implementation {
        return Err("before-final requires an implementation checkpoint".into());
    }
    Ok(
        json!({"ok":true,"task":id,"point":point,"phase":cp.phase,"scope":"Checks observed bytes at this boundary; host must intercept writes for ordering guarantees"}),
    )
}

/// New task records cannot select a legacy renderer to bypass ordered checkpoints.
pub fn admission(root: &Path, id: &str) -> Result<()> {
    if crate::task_record::read(root, id)?.schema_version >= 5
        && (!authored::exists(root, id) || authored::read(root, id)?.schema_version != 2)
    {
        return Err(
            "schema-5 tasks require an ordered authored ledger; register discovery first".into(),
        );
    }
    Ok(())
}
