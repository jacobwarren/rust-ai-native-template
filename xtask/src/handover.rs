//! What `task-artifact next` hands a page's author or reviewer: the page's prompt
//! from docs/ARTIFACTS.md with its braces filled from the task record, the
//! exemplar, the output path and the pages written so far. The guide is compiled
//! in, so the prompt text has one source and a fixture repository needs no copy.
use crate::{
    artifacts::{self, DISCOVERY, FINAL, NOTES},
    config::Result,
    router::{Log, Verdict},
    task_record::Prompt,
};
use serde_json::{Value, json};
use std::path::Path;

const GUIDE: &str = include_str!("../../docs/ARTIFACTS.md");
/// The braces a prompt may hold, each with the record field that fills it.
/// `{task}` is a verb phrase in the blindspot prompt and a noun phrase elsewhere.
const BRACES: [&str; 4] = ["{task}", "{area}", "{problem}", "{reference}"];

/// One row of the guide's prompt table.
pub struct Row {
    pub file: &'static str,
    pub prompt: &'static str,
    /// Repository path of the exemplar page.
    pub exemplar: &'static str,
}

fn pages() -> Vec<&'static str> {
    DISCOVERY.into_iter().chain([NOTES]).chain(FINAL).collect()
}

/// The prompt table, keyed by its File column, in page order.
pub fn rows() -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    for line in GUIDE.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // | When | Page | File | Prompt | Exemplar |: seven pieces around the pipes.
        let [_, _, _, file, prompt, exemplar, _] = cells[..] else {
            continue;
        };
        let Some(prompt) = prompt.strip_prefix('"').and_then(|p| p.strip_suffix('"')) else {
            continue;
        };
        let exemplar = exemplar
            .split_once("](")
            .and_then(|(_, link)| link.strip_suffix(')'))
            .and_then(|link| link.strip_prefix("../"))
            .ok_or(format!(
                "docs/ARTIFACTS.md: {file}'s exemplar must be a link into examples/"
            ))?;
        rows.push(Row {
            file,
            prompt,
            exemplar,
        });
    }
    let files: Vec<&str> = rows.iter().map(|r| r.file).collect();
    if files != pages() {
        return Err(format!(
            "docs/ARTIFACTS.md's prompt table must list {} in its File column, in that order",
            pages().join(", ")
        )
        .into());
    }
    Ok(rows)
}

/// The quoted rule for answering the interview when no one replies.
pub fn answering_rule() -> Result<String> {
    let section = GUIDE
        .split("\n## ")
        .find(|s| s.starts_with("Replies come back into the work"))
        .ok_or("docs/ARTIFACTS.md has no section on replies")?;
    let rule: Vec<&str> = section
        .lines()
        .filter_map(|l| l.strip_prefix("> "))
        .collect();
    if rule.is_empty() {
        return Err("docs/ARTIFACTS.md's replies section must quote its answering rule".into());
    }
    Ok(rule.join(" "))
}

/// Who receives the hand-over.
pub enum For<'a> {
    Author,
    Review {
        round: usize,
    },
    Rewrite {
        of: usize,
        round: usize,
        notes: &'a str,
    },
}

pub struct Handover {
    pub text: String,
    /// Record fields under `prompt` with no value; their braces stay in the text.
    pub unfilled: Vec<&'static str>,
}

/// A value the stub left as its placeholder, such as `<Problem, one sentence>`, is no value.
fn given(value: Option<&String>) -> Option<&str> {
    let v = value?.trim();
    (!v.is_empty() && !(v.starts_with('<') && v.ends_with('>'))).then_some(v)
}

/// Fill a prompt's braces. The blindspot prompt alone opens "I'm about to {task}",
/// so it takes the verb phrase; the others take the noun phrase.
fn fill(row: &Row, prompt: Option<&Prompt>) -> (String, Vec<&'static str>) {
    let none = Prompt::default();
    let p = prompt.unwrap_or(&none);
    let task = if row.file == DISCOVERY[0] {
        ("task", &p.task)
    } else {
        ("task_noun", &p.task_noun)
    };
    let fields = [
        task,
        ("area", &p.area),
        ("problem", &p.problem),
        ("reference", &p.reference),
    ];
    let mut text = row.prompt.to_string();
    let mut unfilled = Vec::new();
    for (brace, (field, value)) in BRACES.iter().zip(fields) {
        if !text.contains(brace) {
            continue;
        }
        match given(value.as_ref()) {
            Some(v) => text = text.replace(brace, v),
            None => unfilled.push(field),
        }
    }
    (text, unfilled)
}

/// The text to hand the subagent unchanged, for one page of a task.
pub fn handover(
    root: &Path,
    id: &str,
    page: &str,
    prompt: Option<&Prompt>,
    to: For,
) -> Result<Handover> {
    let rows = rows()?;
    let index = rows
        .iter()
        .position(|r| r.file == page)
        .ok_or(format!("{page} is not a task page"))?;
    let row = &rows[index];
    let (filled, unfilled) = fill(row, prompt);
    let path = format!("{}/{page}", artifacts::directory(id));
    let mut context = vec![format!("harness/tasks/{id}.json")];
    let ticket = format!("docs/plans/{id}.md");
    if root.join(&ticket).is_file() {
        context.push(ticket);
    }
    let mut before = Vec::new();
    for r in &rows[..index] {
        if artifacts::exists(root, id, r.file)? {
            before.push(r.file);
        }
    }
    let before = if before.is_empty() {
        "none".to_string()
    } else {
        before.join(", ")
    };
    let mut lines = match &to {
        For::Author => vec![format!("Write {path}."), format!("Prompt: {filled}")],
        For::Review { round } => vec![
            if *round > 1 {
                format!("Review {path}, rewritten; judge the whole page again.")
            } else {
                format!("Review {path}.")
            },
            format!("The prompt it answers: {filled}"),
        ],
        For::Rewrite { of, .. } => vec![
            format!("Rewrite {path} in place (rewrite {of} of 2)."),
            format!("Prompt: {filled}"),
        ],
    };
    lines.push(format!("Exemplar: {}", row.exemplar));
    lines.push(format!("Task context: {}", context.join(", ")));
    lines.push(match to {
        For::Review { .. } => format!("Pages before it: {before}"),
        _ => format!("Pages so far: {before}"),
    });
    if page == "interview.html" || page == "buy-in.html" {
        lines.push(format!("When no one answers: {}", answering_rule()?));
    }
    match to {
        For::Rewrite { round, notes, .. } => {
            lines.push(format!("Round {round} findings; address every one:"));
            lines.push(notes.trim().to_string());
            lines.push("Follow the author-artifact skill and return what it asks for.".into());
        }
        For::Author => {
            lines.push("Follow the author-artifact skill and return what it asks for.".into());
        }
        For::Review { .. } => lines.push(
            "Follow the review-artifact skill: PASS or REVISE on the first line, numbered findings, and a last line FINDINGS: N counting every finding you listed.".into(),
        ),
    }
    Ok(Handover {
        text: lines.join("\n"),
        unfilled,
    })
}

/// Add the hand-over to a step that starts a subagent: author, build (the notes'
/// author), review and rewrite. Other steps are left as they are.
pub fn attach(
    root: &Path,
    id: &str,
    prompt: Option<&Prompt>,
    log: Option<&Log>,
    step: &mut Value,
) -> Result<()> {
    let (Some(action), Some(page)) = (step["action"].as_str(), step["page"].as_str()) else {
        return Ok(());
    };
    let page = page.to_string();
    let rows: Vec<_> = log
        .map(|l| l.rows.iter().filter(|r| r.page == page).collect())
        .unwrap_or_default();
    let revisions = rows.iter().filter(|r| r.verdict == Verdict::Revise).count();
    let to = match (action, rows.last()) {
        ("author" | "build", _) => For::Author,
        ("review", _) => For::Review {
            round: step["round"].as_u64().map_or(1, |r| r as usize),
        },
        ("rewrite", Some(last)) => For::Rewrite {
            of: revisions,
            round: last.round,
            notes: &last.notes,
        },
        _ => return Ok(()),
    };
    let h = handover(root, id, &page, prompt, to)?;
    if !h.unfilled.is_empty() {
        let say = step["say"].as_str().unwrap_or_default();
        step["say"] = format!(
            "{say} First fill its braces for {}: set them under prompt in harness/tasks/{id}.json and run next again, or replace them in the handover by hand.",
            h.unfilled.join(", ")
        )
        .into();
    }
    step["handover"] = h.text.into();
    step["unfilled"] = json!(h.unfilled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_prompts_match_the_artifacts_guide() {
        let rows = rows().unwrap();
        assert_eq!(rows.iter().map(|r| r.file).collect::<Vec<_>>(), pages());
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        for row in &rows {
            assert!(root.join(row.exemplar).is_file(), "{}", row.exemplar);
            // Every brace in a prompt is one fill() knows.
            let mut rest = row.prompt;
            while let Some(start) = rest.find('{') {
                let end = rest[start..].find('}').unwrap() + start + 1;
                assert!(BRACES.contains(&&rest[start..end]), "{}", &rest[start..end]);
                rest = &rest[end..];
            }
            // Only the blindspot prompt takes {task} as a verb phrase.
            assert_eq!(
                row.prompt.starts_with("I'm about to {task}"),
                row.file == "blindspots.html",
                "{}",
                row.file
            );
        }
        assert!(answering_rule().unwrap().contains("the agent's call"));
    }
}
