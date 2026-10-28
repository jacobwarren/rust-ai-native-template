//! Explicit import of browser proposals. Hashes prevent stale cross-page overwrites.
use crate::{config::Result, discovery::Discovery, history, task_artifact::Artifact, tracking};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Feedback {
    pub schema_version: u32,
    pub task: String,
    pub discovery_sha256: String,
    pub feedback_base_sha256: String,
    pub actor: String,
    pub responses: BTreeMap<String, String>,
}
pub fn discovery_hash(a: &Artifact) -> Result<Option<String>> {
    a.discoveries.last().map(history::json_digest).transpose()
}
pub fn responses(a: &Artifact) -> Result<BTreeMap<String, String>> {
    let hash = discovery_hash(a)?;
    let mut values = BTreeMap::new();
    for f in &a.feedback {
        if Some(&f.discovery_sha256) == hash.as_ref() {
            values.extend(f.responses.clone());
        }
    }
    Ok(values)
}
fn values(d: &Discovery, f: &Feedback) -> Result<()> {
    if f.schema_version != 1 || !tracking::nonempty(&f.actor) || f.responses.is_empty() {
        return Err("feedback needs an actor and explicit responses".into());
    }
    for (key, value) in &f.responses {
        if !tracking::nonempty(value) {
            return Err("feedback responses cannot be blank".into());
        }
        let (kind, id) = key.split_once(':').ok_or("invalid feedback key")?;
        let valid = match kind {
            "blindspot" => {
                d.blindspots.iter().any(|b| b.id == id)
                    && ["include", "investigate", "exclude"].contains(&value.as_str())
            }
            "intervention" => {
                d.interventions.iter().any(|i| i.id == id)
                    && ["selected", "rejected"].contains(&value.as_str())
            }
            "interview" => d.interview.questions.iter().any(|q| q.id == id),
            "reference" => d.reference.mappings.iter().any(|m| m.id == id),
            "plan" => d
                .plan
                .choices
                .iter()
                .any(|c| c.id == id && c.options.iter().any(|o| o.id == *value)),
            _ => false,
        };
        if !valid {
            return Err(format!("unknown or invalid feedback choice: {key}").into());
        }
    }
    Ok(())
}
pub fn import(a: &mut Artifact, f: Feedback) -> Result<()> {
    if f.task != a.task
        || Some(f.discovery_sha256.clone()) != discovery_hash(a)?
        || f.feedback_base_sha256 != history::json_digest(&a.feedback)?
    {
        return Err("feedback is stale or belongs to another task; reopen regenerated artifacts and export again".into());
    }
    values(a.discoveries.last().ok_or("create discovery first")?, &f)?;
    a.feedback.push(f);
    Ok(())
}
pub fn validate(a: &Artifact) -> Result<()> {
    for (i, f) in a.feedback.iter().enumerate() {
        let d = a
            .discoveries
            .iter()
            .find(|d| history::json_digest(d).ok().as_ref() == Some(&f.discovery_sha256))
            .ok_or("feedback references unknown discovery")?;
        if f.task != a.task || f.feedback_base_sha256 != history::json_digest(&a.feedback[..i])? {
            return Err("feedback history chain was rewritten".into());
        }
        values(d, f)?;
    }
    Ok(())
}
pub fn completion(a: &Artifact) -> Result<()> {
    let d = a
        .discoveries
        .last()
        .ok_or("all five discovery tools are mandatory; run task-artifact discover")?;
    let answers = responses(a)?;
    for q in &d.interview.questions {
        if q.blocking
            && q.settled.is_none()
            && !answers.contains_key(&format!("interview:{}", q.id))
        {
            return Err(format!("blocking interview question remains unanswered: {}", q.id).into());
        }
    }
    let p = a
        .publications
        .last()
        .ok_or("publish the final brief after discovery")?;
    if p.discovery_sha256 != discovery_hash(a)?
        || p.feedback_sha256.as_deref() != Some(&history::json_digest(&a.feedback)?)
    {
        return Err("discovery or decisions changed; republish the buy-in brief and quiz".into());
    }
    Ok(())
}
