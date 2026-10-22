//! Review receipts bind rationale to exact before/after objects, not free-form claims.
use crate::{
    config::{self, Result},
    history, readiness,
    tracking::{self, Gap},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub subject: String,
    pub before_sha256: Option<String>,
    pub after_sha256: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    base_commit: String,
    changes: Vec<Explanation>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Explanation {
    change: Change,
    rationale: String,
    evidence: Vec<String>,
    manual_justification: Option<String>,
}

fn protected(path: &str) -> bool {
    [
        "xtask/",
        ".github/",
        "scripts/",
        ".cargo/",
        "harness/goldens/",
    ]
    .iter()
    .any(|p| path.starts_with(p))
        || [
            "harness/config.json",
            "harness/readiness.json",
            "harness/tracking.json",
            "harness/workflows.json",
            "harness/scenarios.json",
            "Cargo.toml",
            "Cargo.lock",
            "docs/PRODUCT.md",
            "docs/READINESS.md",
            "docs/COMPLETION.md",
            "AGENTS.md",
        ]
        .contains(&path)
}
fn sensitive(old: &Gap, new: Option<&Gap>) -> bool {
    let Some(new) = new else {
        return true;
    };
    (old.state == "open" && new.state == "closed")
        || old.blocks.iter().any(|s| !new.blocks.contains(s))
        || old
            .requirements
            .iter()
            .any(|s| !new.requirements.contains(s))
        || (old.state == "closed"
            && (old.resolution != new.resolution || old.requirements != new.requirements))
}

pub fn required(root: &Path, base: &str) -> Result<(Vec<Change>, Vec<Gap>, Vec<String>)> {
    let old = tracking::load_from(|p| history::read(root, base, p))?;
    let new = tracking::load(root)?;
    let mut changes = Vec::new();
    for gap in &old {
        let next = new.iter().find(|g| g.id == gap.id);
        if sensitive(gap, next) {
            changes.push(Change {
                subject: format!("gap:{}", gap.id),
                before_sha256: Some(history::json_digest(gap)?),
                after_sha256: next.map(history::json_digest).transpose()?,
            });
        }
    }
    for gap in new
        .iter()
        .filter(|g| g.state == "closed" && !old.iter().any(|o| o.id == g.id))
    {
        changes.push(Change {
            subject: format!("gap:{}", gap.id),
            before_sha256: None,
            after_sha256: Some(history::json_digest(gap)?),
        });
    }
    let before = history::before(root, base)?;
    let after = history::current(root)?;
    // Existing receipts are append-only. A later correction must be a new record.
    for (path, hash) in before
        .iter()
        .filter(|(p, _)| p.starts_with("harness/transitions/"))
    {
        if after.get(path) != Some(hash) {
            return Err(format!("transition history changed or deleted: {path}").into());
        }
    }
    let before: BTreeMap<_, _> = before.into_iter().filter(|(p, _)| protected(p)).collect();
    let after: BTreeMap<_, _> = after.into_iter().filter(|(p, _)| protected(p)).collect();
    let changed: Vec<_> = before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|p| before.get(*p) != after.get(*p))
        .cloned()
        .collect();
    if !changed.is_empty() {
        changes.push(Change {
            subject: "policy".into(),
            before_sha256: Some(history::json_digest(&before)?),
            after_sha256: Some(history::json_digest(&after)?),
        });
    }
    Ok((changes, new, changed))
}

pub fn check(root: &Path, revision: &str, report: Option<&Path>) -> Result<Value> {
    let base = history::merge_base(root, revision)?;
    let (required, gaps, protected_files) = required(root, &base)?;
    let mut explanations = Vec::new();
    for path in history::records(root, "harness/transitions")? {
        let receipt: Receipt = config::read(&root.join(&path))?;
        if receipt.schema_version != 1 {
            return Err(format!("invalid receipt {path}").into());
        }
        if receipt.base_commit == base {
            explanations.extend(receipt.changes);
        }
    }
    let mut errors = Vec::new();
    let mut seen = BTreeSet::new();
    for e in &explanations {
        if !seen.insert(e.change.subject.clone())
            || !required.contains(&e.change)
            || !tracking::nonempty(&e.rationale)
            || e.evidence.is_empty()
            || !tracking::unique(&e.evidence)
            || !e.evidence.iter().all(|s| tracking::reference(root, s))
        {
            errors.push(format!(
                "invalid, stale, duplicate or unsupported rationale for {}",
                e.change.subject
            ));
        }
    }
    let assessment = report
        .map(|r| readiness::assess(root, Some(r), None))
        .transpose()?;
    for change in &required {
        let explanation = explanations.iter().find(|e| e.change == *change);
        if explanation.is_none() {
            errors.push(format!(
                "missing exact transition rationale: {}",
                change.subject
            ));
        }
        if let Some(id) = change.subject.strip_prefix("gap:")
            && let Some(gap) = gaps.iter().find(|g| g.id == id && g.state == "closed")
        {
            if gap.requirements.is_empty() {
                if !explanation.is_some_and(|e| {
                    e.manual_justification
                        .as_ref()
                        .is_some_and(|s| tracking::nonempty(s))
                }) {
                    errors.push(format!(
                        "manual closure of {id} needs an explicit review justification"
                    ));
                }
            } else if !assessment
                .as_ref()
                .is_some_and(|a| tracking::closure_verified(gap, a))
            {
                errors.push(format!(
                    "closure of {id} needs current verified requirements: {:?}",
                    gap.requirements
                ));
            }
        }
    }
    Ok(
        json!({"ok":errors.is_empty(),"kind":"gap_transitions","base_commit":base,
        "required_changes":required,"protected_files":protected_files,"errors":errors,
        "review":"Rationale is not independent approval. Enforce reviewer authority in hosting rules."}),
    )
}
