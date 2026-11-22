//! Durable task reconciliation, not a replacement backlog.
use crate::{
    config::{self, Result, valid_id},
    tracking,
};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Complete,
    Partial,
    Blocked,
}
/// How much ceremony a task owes (docs/ARTIFACTS.md "Task size"). A small task,
/// the default, owes no page and stays away from policy paths; the gate reads its
/// branch (size.rs). A standard task owes a brief before code.
#[derive(Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Small,
    Standard,
}
/// The record schema a new claim must use. Earlier records can be read as
/// history but owe nothing and cannot complete.
pub const SCHEMA: u32 = 7;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub schema_version: u32,
    pub id: String,
    pub work_item: Option<String>,
    pub status: Status,
    #[serde(default, rename = "tier", deserialize_with = "named_tier")]
    declared_tier: Option<Tier>,
    /// Written by records from when a full tier chose its pages by reader; ignored.
    #[serde(default, rename = "reader")]
    _reader: Option<serde::de::IgnoredAny>,
    pub requirements: Vec<String>,
    pub related_gaps: Vec<Disposition>,
    pub gap_review: String,
    pub discoveries: Vec<Discovery>,
    pub discovery_review: String,
    #[serde(default)]
    pub review: Option<Review>,
    pub documentation: Documentation,
    pub summary: String,
    /// Next steps; a complete record may leave it out.
    #[serde(default)]
    pub remaining: Vec<String>,
    /// Values for the brief prompt's braces; `next` fills them in (brief.rs).
    #[serde(default)]
    pub prompt: Option<Prompt>,
}
/// A tier that is written must be one of the two; `null` is not "no tier".
fn named_tier<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<Tier>, D::Error> {
    Tier::deserialize(deserializer).map(Some)
}

impl Task {
    /// The ceremony this record owes; a record from an earlier schema owes none.
    pub fn tier(&self) -> Tier {
        if self.schema_version < SCHEMA {
            return Tier::Small;
        }
        self.declared_tier.unwrap_or(Tier::Small)
    }
}
/// Brace values for the brief's prompt. Each is optional and checked by `next`,
/// which leaves a missing one visible, never by `read`. Three more names are
/// accepted from records written for the earlier page set and ignored.
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Prompt {
    /// Verb phrase after "I'm about to".
    pub task: Option<String>,
    pub area: Option<String>,
    #[serde(rename = "task_noun")]
    _task_noun: Option<serde::de::IgnoredAny>,
    #[serde(rename = "problem")]
    _problem: Option<serde::de::IgnoredAny>,
    #[serde(rename = "reference")]
    _reference: Option<serde::de::IgnoredAny>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disposition {
    pub id: String,
    pub disposition: GapState,
    pub rationale: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapState {
    Resolved,
    Updated,
    Unaffected,
    Retired,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Discovery {
    pub summary: String,
    pub record: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Documentation {
    pub updated: Vec<String>,
    pub rationale: String,
}

pub fn read(root: &Path, id: &str) -> Result<Task> {
    if !valid_id(id) {
        return Err("invalid task ID".into());
    }
    let task: Task = config::read(&root.join(format!("harness/tasks/{id}.json")))?;
    if task.status != Status::Complete && task.remaining.is_empty() {
        return Err(format!(
            "task {id}: a partial or blocked record lists its next steps in `remaining`; only a complete record may leave it empty or out"
        )
        .into());
    }
    if !(1..=SCHEMA).contains(&task.schema_version)
        || task.id != id
        || task
            .work_item
            .as_ref()
            .is_some_and(|s| !tracking::nonempty(s))
        || !tracking::nonempty(&task.summary)
        || !tracking::nonempty(&task.gap_review)
        || !tracking::nonempty(&task.discovery_review)
        || !tracking::nonempty(&task.documentation.rationale)
        || !tracking::unique(&task.requirements)
        || !tracking::unique(&task.remaining)
        || !tracking::unique(&task.documentation.updated)
        || !tracking::unique(
            &task
                .related_gaps
                .iter()
                .map(|g| g.id.clone())
                .collect::<Vec<_>>(),
        )
        || task
            .related_gaps
            .iter()
            .any(|g| !valid_id(&g.id) || !tracking::nonempty(&g.rationale))
        || task
            .discoveries
            .iter()
            .any(|d| !tracking::nonempty(&d.summary) || !tracking::nonempty(&d.record))
        || task
            .documentation
            .updated
            .iter()
            .any(|p| !tracking::source_file(root, p))
        || (task.status != Status::Complete && task.remaining.is_empty())
        || (task.status == Status::Complete && task.requirements.is_empty())
    {
        return Err(format!("task {id}: invalid identity, missing reconciliation/acceptance, invalid references, or missing next steps").into());
    }
    Ok(task)
}

/// A bounded review of this task, not a demand to invent gaps or audit the project.
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Review {
    #[serde(default)]
    pub acceptance: String,
    #[serde(default)]
    pub verification: String,
    #[serde(default)]
    pub assumptions: String,
    #[serde(default)]
    pub coverage: String,
}

pub fn review_errors(task: &Task) -> Vec<String> {
    let mut errors = Vec::new();
    if task.schema_version < SCHEMA {
        errors.push(format!("Upgrade this active task to schema_version {SCHEMA} (docs/COMPLETION.md); earlier records are history and cannot make a new claim."));
    }
    let empty = Review::default();
    let review = task.review.as_ref().unwrap_or(&empty);
    for (field, response, question) in [
        (
            "acceptance",
            &review.acceptance,
            "Did any assigned acceptance criterion remain unmet? Explain the result and record remaining work.",
        ),
        (
            "verification",
            &review.verification,
            "Did checks reveal a pre-existing problem or unresolved failure? Link durable findings; transient failures fixed during work need no gap.",
        ),
        (
            "assumptions",
            &review.assumptions,
            "Did this task rely on an unverified assumption? State the evidence or where the unresolved assumption was recorded.",
        ),
        (
            "coverage",
            &review.coverage,
            "Did you discover a material missing test, recovery path or platform check? Record it, or explain why no material shortcoming was found in the inspected scope.",
        ),
    ] {
        if !tracking::nonempty(response) {
            errors.push(format!("review.{field}: {question}"));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::{Status, Task};
    use std::io::Write;
    use std::process::{Command, Stdio};

    /// The Stop hook finds a completion claim with grep; it must agree with
    /// how this module reads the top-level status, in any JSON spelling.
    #[test]
    fn stop_claim_pattern_matches_parsed_status() {
        let script = include_str!("../../scripts/agent-complete.sh");
        let pattern = script
            .lines()
            .find_map(|l| l.strip_prefix("CLAIM='"))
            .and_then(|l| l.strip_suffix('\''))
            .expect("CLAIM pattern in scripts/agent-complete.sh");
        let grep = |text: &str| {
            let mut child = Command::new("grep")
                .args(["-Eq", pattern])
                .stdin(Stdio::piped())
                .spawn()
                .unwrap();
            child
                .stdin
                .take()
                .unwrap()
                .write_all(text.as_bytes())
                .unwrap();
            child.wait().unwrap().success()
        };
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../harness/tasks");
        let mut checked = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let record: serde_json::Value =
                serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
            for status in ["complete", "partial", "blocked"] {
                let mut record = record.clone();
                record["status"] = status.into();
                let compact = record.to_string();
                let spaced = compact.replace("\":", "\" :  ");
                for text in [
                    compact.clone(),
                    serde_json::to_string_pretty(&record).unwrap(),
                    spaced,
                ] {
                    let task: Task = serde_json::from_str(&text).unwrap();
                    assert_eq!(grep(&text), task.status == Status::Complete, "{text}");
                    checked += 1;
                }
            }
        }
        assert!(checked >= 9);
    }

    #[test]
    fn prompt_values_are_optional_and_strict() {
        let base = serde_json::json!({"schema_version":7,"id":"t","work_item":null,"status":"partial",
            "requirements":[],"related_gaps":[],"gap_review":"g","discoveries":[],"discovery_review":"d",
            "documentation":{"updated":[],"rationale":"r"},"summary":"s","remaining":["x"]});
        let parse = |prompt: Option<serde_json::Value>| {
            let mut record = base.clone();
            if let Some(p) = prompt {
                record["prompt"] = p;
            }
            serde_json::from_value::<Task>(record)
        };
        assert!(parse(None).unwrap().prompt.is_none());
        let p = parse(Some(
            serde_json::json!({"task":"add x","area":"","reference":null}),
        ))
        .unwrap()
        .prompt
        .unwrap();
        assert_eq!(p.task.as_deref(), Some("add x"));
        assert_eq!(p.area.as_deref(), Some(""));
        // Names from the earlier page set still parse; nothing reads them.
        assert!(parse(Some(serde_json::json!({"task_noun":"x","problem":"y"}))).is_ok());
        assert!(parse(Some(serde_json::json!({"task_nuon":"x"}))).is_err());
    }
}
