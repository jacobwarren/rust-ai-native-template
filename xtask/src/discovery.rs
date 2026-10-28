//! Typed discovery content: five mandatory tools, with honest empty/settled states.
use crate::{
    config::{Result, valid_id},
    tracking,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Discovery {
    pub scope: String,
    pub references: Vec<String>,
    pub blindspots: Vec<Blindspot>,
    pub no_blindspots_reason: String,
    pub interventions: Vec<Intervention>,
    pub interview: Interview,
    pub reference: Reference,
    pub plan: Plan,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Blindspot {
    pub id: String,
    pub title: String,
    pub category: String,
    pub observation: String,
    pub consequence: String,
    pub constraint: String,
    pub references: Vec<String>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Intervention {
    pub id: String,
    pub title: String,
    pub effort: u8,
    pub approach: String,
    pub benefit: String,
    pub tradeoff: String,
    pub references: Vec<String>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OptionItem {
    pub id: String,
    pub label: String,
    pub consequence: String,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SettledAnswer {
    pub value: String,
    pub basis: String,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InterviewQuestion {
    pub id: String,
    pub prompt: String,
    pub why: String,
    pub blocking: bool,
    pub options: Vec<OptionItem>,
    pub settled: Option<SettledAnswer>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Interview {
    pub questions: Vec<InterviewQuestion>,
    pub no_questions_reason: String,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub id: String,
    pub behavior: String,
    pub source_excerpt: String,
    pub target_excerpt: String,
    pub difference: String,
    pub expected: String,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub name: String,
    pub revision: String,
    pub source: String,
    pub target: String,
    pub mappings: Vec<Mapping>,
    pub missing_reason: String,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanOption {
    pub id: String,
    pub label: String,
    pub tradeoff: String,
    pub preview: String,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanChoice {
    pub id: String,
    pub title: String,
    pub why: String,
    pub recommended: String,
    pub options: Vec<PlanOption>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub choices: Vec<PlanChoice>,
    pub steps: Vec<String>,
    pub mechanical: Vec<String>,
}
fn fields(values: &[&str]) -> Result<()> {
    if values.iter().any(|s| !tracking::nonempty(s)) {
        return Err("discovery requires nonempty evidence, explanations and choices".into());
    }
    Ok(())
}
fn ids<'a>(values: impl Iterator<Item = &'a str>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for id in values {
        if !valid_id(id) || !seen.insert(id) {
            return Err("invalid or duplicate discovery ID".into());
        }
    }
    Ok(())
}
fn refs(root: &Path, values: &[String]) -> Result<()> {
    if values.is_empty()
        || !tracking::unique(values)
        || values.iter().any(|r| !tracking::reference(root, r))
    {
        return Err("discovery references require source files or HTTPS URLs".into());
    }
    Ok(())
}
pub fn validate(root: &Path, d: &Discovery) -> Result<()> {
    fields(&[&d.scope])?;
    refs(root, &d.references)?;
    ids(d.blindspots.iter().map(|x| x.id.as_str()))?;
    if d.blindspots.is_empty() {
        fields(&[&d.no_blindspots_reason])?;
    }
    for b in &d.blindspots {
        fields(&[
            &b.title,
            &b.category,
            &b.observation,
            &b.consequence,
            &b.constraint,
        ])?;
        refs(root, &b.references)?;
    }
    if d.interventions.len() < 2 {
        return Err("compare at least two intervention approaches, including leaving behavior unchanged when appropriate".into());
    }
    ids(d.interventions.iter().map(|x| x.id.as_str()))?;
    for i in &d.interventions {
        fields(&[&i.title, &i.approach, &i.benefit, &i.tradeoff])?;
        refs(root, &i.references)?;
        if !(1..=5).contains(&i.effort) {
            return Err("intervention effort must be 1..5".into());
        }
    }
    ids(d.interview.questions.iter().map(|x| x.id.as_str()))?;
    if d.interview.questions.is_empty() {
        fields(&[&d.interview.no_questions_reason])?;
    }
    for q in &d.interview.questions {
        fields(&[&q.prompt, &q.why])?;
        if q.options.len() < 2 {
            return Err("interview questions need at least two alternatives".into());
        }
        ids(q.options.iter().map(|x| x.id.as_str()))?;
        for o in &q.options {
            fields(&[&o.label, &o.consequence])?;
        }
        if let Some(a) = &q.settled {
            fields(&[&a.value, &a.basis])?;
        }
    }
    let r = &d.reference;
    if r.mappings.is_empty() {
        fields(&[&r.missing_reason])?;
    } else {
        fields(&[&r.name, &r.revision, &r.source, &r.target])?;
        if !tracking::reference(root, &r.source) {
            return Err("reference source must be a source file or HTTPS URL".into());
        }
    }
    ids(r.mappings.iter().map(|x| x.id.as_str()))?;
    for m in &r.mappings {
        fields(&[
            &m.behavior,
            &m.source_excerpt,
            &m.target_excerpt,
            &m.difference,
            &m.expected,
        ])?;
    }
    let p = &d.plan;
    if p.choices.is_empty()
        || p.steps.is_empty()
        || !tracking::unique(&p.steps)
        || !tracking::unique(&p.mechanical)
    {
        return Err(
            "plan requires consequential choices and nonempty distinct execution steps".into(),
        );
    }
    ids(p.choices.iter().map(|x| x.id.as_str()))?;
    for c in &p.choices {
        fields(&[&c.title, &c.why])?;
        ids(c.options.iter().map(|x| x.id.as_str()))?;
        if c.options.len() < 2 || !c.options.iter().any(|o| o.id == c.recommended) {
            return Err("plan choice needs alternatives and a valid recommendation".into());
        }
        for o in &c.options {
            fields(&[&o.label, &o.tradeoff, &o.preview])?;
        }
    }
    Ok(())
}
