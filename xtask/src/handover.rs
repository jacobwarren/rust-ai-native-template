//! What `task-artifact next` hands a page's author or reviewer: the page's prompt
//! from docs/ARTIFACTS.md with its braces filled from the task record, the
//! exemplar, the output path and the pages written so far. The guide is compiled
//! in, so the prompt text has one source and a fixture repository needs no copy.
use crate::{
    artifacts::{self, BRIEF},
    config::Result,
    router::{Log, Verdict},
    task_record::{Prompt, Reader, Task, Tier},
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
    artifacts::known()
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

/// The rule a guide section quotes, as one line.
fn quoted(heading: &str) -> Result<String> {
    let section = GUIDE
        .split("\n## ")
        .find(|s| s.starts_with(heading))
        .ok_or(format!("docs/ARTIFACTS.md has no section {heading}"))?;
    let rule: Vec<&str> = section
        .lines()
        .filter_map(|l| l.strip_prefix("> "))
        .collect();
    if rule.is_empty() {
        return Err(format!("docs/ARTIFACTS.md's section {heading} must quote its rule").into());
    }
    Ok(rule.join(" "))
}

/// The quoted rule for answering the interview when no one replies.
pub fn answering_rule() -> Result<String> {
    quoted("Replies come back into the work")
}

/// The quoted rule for a task whose pages no person will answer.
pub fn no_reader_rule() -> Result<String> {
    quoted("Pages need a reader")
}

/// The quoted length rule for the standard tier's brief.
pub fn standard_rule() -> Result<String> {
    quoted("The standard page")
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

/// Fill a prompt's braces. A prompt that opens "I'm about to {task}" takes the
/// verb phrase; the others take the noun phrase.
fn fill(row: &Row, prompt: Option<&Prompt>) -> (String, Vec<&'static str>) {
    let none = Prompt::default();
    let p = prompt.unwrap_or(&none);
    let task = if row.prompt.starts_with("I'm about to {task}") {
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
pub fn handover(root: &Path, id: &str, page: &str, task: &Task, to: For) -> Result<Handover> {
    let prompt = task.prompt.as_ref();
    let owed = artifacts::Pages::of(task).all();
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
    let position = owed.iter().position(|p| *p == page).unwrap_or(0);
    for r in rows.iter().filter(|r| owed[..position].contains(&r.file)) {
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
    if page == "interview.html" || (page == "buy-in.html" && task.reader() == Reader::Human) {
        lines.push(format!("Answering rule: {}", answering_rule()?));
    }
    // Without a reader no interview is written: its questions belong to these pages.
    if task.reader() == Reader::None && ["plan.html", "buy-in.html", BRIEF].contains(&page) {
        lines.push(format!("No reader: {}", no_reader_rule()?));
    }
    if task.tier() == Tier::Standard {
        lines.push(format!("Length: {}", standard_rule()?));
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

/// Advice for each standard page whose file is over the size its hand-over states.
pub fn size_advisories(root: &Path, id: &str, task: &Task) -> Vec<String> {
    if task.tier() != Tier::Standard {
        return Vec::new();
    }
    let limit = artifacts::STANDARD_PAGE_BYTES;
    artifacts::Pages::of(task)
        .all()
        .into_iter()
        .filter_map(|page| {
            let size = std::fs::metadata(root.join(artifacts::directory(id)).join(page))
                .ok()?
                .len();
            (size > limit).then(|| {
                format!(
                    "{page} is {size} bytes; a standard page stays within {limit} (docs/ARTIFACTS.md, The standard page)"
                )
            })
        })
        .collect()
}

/// Add the hand-over to a step that starts a subagent: author, build (the notes'
/// author), review and rewrite. Other steps are left as they are.
pub fn attach(
    root: &Path,
    id: &str,
    task: &Task,
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
    let h = handover(root, id, &page, task, to)?;
    // The note goes first: a review step's say ends with the verdict command to copy.
    if !h.unfilled.is_empty() {
        let say = step["say"].as_str().unwrap_or_default();
        step["say"] = format!(
            "First fill the handover's braces for {}: set them under prompt in harness/tasks/{id}.json and run next again, or replace them by hand. Then: {say}",
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
            // Only the two pages that open a task take {task} as a verb phrase.
            assert_eq!(
                row.prompt.starts_with("I'm about to {task}"),
                ["blindspots.html", "brief.html"].contains(&row.file),
                "{}",
                row.file
            );
        }
        assert!(answering_rule().unwrap().contains("the agent's call"));
        assert!(no_reader_rule().unwrap().contains("the agent's call"));
        // The quoted limit is the one `next` measures.
        assert!(
            standard_rule()
                .unwrap()
                .contains(&format!("{} KB", artifacts::STANDARD_PAGE_BYTES / 1000))
        );
    }
}
