//! `task-artifact next`: the one next step toward a task's brief, with the prompt
//! to hand its author. It advises and never gates.
use crate::{
    artifacts::{self, BRIEF, BRIEF_BYTES},
    config::{Result, valid_id},
    history,
    task_record::{self, Prompt},
};
use serde_json::{Value, json};
use std::path::Path;

/// The brief's prompt; docs/ARTIFACTS.md quotes it, and a test keeps them equal.
pub const PROMPT: &str = "I'm about to {task} in {area}. Before I write code, give me one page: the unknowns in this part of the codebase that the ticket does not show, each with the code that proves it, then the plan, leading with the decisions I'm most likely to tweak and burying the mechanical work at the bottom.";
/// What keeps a brief short and self-sufficient; quoted in the guide too.
pub const RULES: &str = "Keep the file at or under 20 KB, the pasted stylesheet included: five unknowns at most, each proven by code or captured output, and one decision per screen. No person will answer questions before the code is written, so put each question you would have asked on the page with your recommendation, labelled as the agent's call.";
/// The page the brief is modelled on.
pub const EXEMPLAR: &str = "examples/thariq/08-implementation-plan.html";

/// A value the ticket stub left as its placeholder, such as `<the code area>`, is no value.
fn given(value: Option<&String>) -> Option<&str> {
    let v = value?.trim();
    (!v.is_empty() && !(v.starts_with('<') && v.ends_with('>'))).then_some(v)
}

/// The text to hand the brief's author unchanged, and the record fields under
/// `prompt` that have no value, whose braces stay visible in it.
fn handover(root: &Path, id: &str, prompt: Option<&Prompt>) -> (String, Vec<&'static str>) {
    let mut filled = PROMPT.to_string();
    let mut unfilled = Vec::new();
    for (brace, field, value) in [
        ("{task}", "task", prompt.and_then(|p| p.task.as_ref())),
        ("{area}", "area", prompt.and_then(|p| p.area.as_ref())),
    ] {
        match given(value) {
            Some(v) => filled = filled.replace(brace, v),
            None => unfilled.push(field),
        }
    }
    let mut context = vec![format!("harness/tasks/{id}.json")];
    let ticket = format!("docs/plans/{id}.md");
    if root.join(&ticket).is_file() {
        context.push(ticket);
    }
    let text = [
        format!("Write {}/{BRIEF}.", artifacts::directory(id)),
        format!("Prompt: {filled}"),
        format!("Exemplar: {EXEMPLAR}"),
        format!("Task context: {}", context.join(", ")),
        format!("Rules: {RULES}"),
        "Follow the author-artifact skill and return what it asks for.".to_string(),
    ]
    .join("\n");
    (text, unfilled)
}

/// What to do next for a task. Advises; never gates.
pub fn next(root: &Path, id: &str) -> Result<Value> {
    if !valid_id(id) {
        return Err("invalid task ID".into());
    }
    if !root.join(format!("harness/tasks/{id}.json")).is_file() {
        return Ok(
            json!({"ok":true,"task":id,"advisories":[],"next":{"action":"record","say":format!(
            "Write harness/tasks/{id}.json first; the write-ticket skill drafts the ticket and a partial record stub.")}}),
        );
    }
    let task = task_record::read(root, id)?;
    let finish = "answer the record's four review questions, run cargo xtask check ci (it writes target/verification/ci.json), then cargo xtask complete";
    if !artifacts::owed(&task) {
        return Ok(
            json!({"ok":true,"task":id,"tier":"small","advisories":[],"next":{"action":"build","say":format!(
            "No brief owed. Build, test and commit, {finish}. A change to a policy, dependency or ceremony path needs a standard task and its brief first.")}}),
        );
    }
    let path = format!("{}/{BRIEF}", artifacts::directory(id));
    let mut advisories = Vec::new();
    let next = if !artifacts::exists(root, id)? {
        let (text, unfilled) = handover(root, id, task.prompt.as_ref());
        let fill = if unfilled.is_empty() {
            String::new()
        } else {
            format!(
                "First fill the handover's braces for {}: set them under prompt in harness/tasks/{id}.json and run next again, or replace them by hand. Then: ",
                unfilled.join(", ")
            )
        };
        json!({"action":"author","page":BRIEF,"handover":text,"unfilled":unfilled,"say":format!(
            "{fill}Start a fresh artifact-author on {BRIEF} and give it the handover verbatim. If your host has no subagents, write it in a turn of its own.")})
    } else {
        let size = std::fs::metadata(root.join(&path))?.len();
        if size > BRIEF_BYTES {
            advisories.push(format!(
                "{BRIEF} is {size} bytes; a brief stays within {BRIEF_BYTES} (docs/ARTIFACTS.md)"
            ));
        }
        if history::uncommitted(root)?.contains(&path) {
            json!({"action":"commit","page":BRIEF,"say":format!(
                "Read {BRIEF}, check what it found against the code, change the plan where it is right, then commit it before any product code.")})
        } else {
            json!({"action":"complete","say":format!(
                "The brief is committed. Build, test and commit the work, {finish}.")})
        }
    };
    Ok(json!({"ok":true,"task":id,"tier":"standard","next":next,"advisories":advisories}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_guide_quotes_the_prompt_and_rules_next_hands_over() {
        let guide = include_str!("../../docs/ARTIFACTS.md").replace("\n> ", " ");
        let guide = guide.replace('\n', " ");
        assert!(
            guide.contains(PROMPT),
            "docs/ARTIFACTS.md must quote the brief's prompt"
        );
        assert!(
            guide.contains(RULES),
            "docs/ARTIFACTS.md must quote the brief's rules"
        );
        assert!(RULES.contains(&format!("{} KB", BRIEF_BYTES / 1000)));
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        assert!(root.join(EXEMPLAR).is_file());
    }
}
