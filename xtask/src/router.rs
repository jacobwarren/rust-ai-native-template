//! The page loop as commands: `next` says which page to write, review, rewrite or
//! commit, from the task record, the pages and the verdict log; `verdict` appends one
//! review to that log. Both advise. Completion never reads the log (docs/ARTIFACTS.md).
use crate::{
    artifacts::{self, DISCOVERY, FINAL, NOTES},
    config::{Result, valid_id},
    history,
    task_record::{self, Tier},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fs, path::Path};

/// The verdict log, committed in the task's page folder with the pages it judges.
pub const LOG: &str = "reviews.json";
/// Reviews a page may take: the first, then one after each of two rewrites.
const REVIEWS: usize = 3;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Pass,
    Revise,
    /// The version committed after two rewrites, when no review passed.
    Best,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub page: String,
    /// The version reviewed: 1 is the first draft, 3 the second rewrite.
    pub round: usize,
    pub verdict: Verdict,
    /// SHA-256 of the page as reviewed, so a later edit shows.
    pub sha256: String,
    pub reviewer: String,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Log {
    pub schema_version: u32,
    pub rows: Vec<Row>,
}

fn log_path(id: &str) -> String {
    format!("{}/{LOG}", artifacts::directory(id))
}

fn parse(bytes: &[u8]) -> Result<Log> {
    let log: Log = serde_json::from_slice(bytes).map_err(|e| format!("{LOG}: {e}"))?;
    if log.schema_version != 1 {
        return Err(format!("{LOG}: schema_version must be 1").into());
    }
    Ok(log)
}

/// The working log, or None before the task's first verdict.
fn read(root: &Path, id: &str) -> Result<Option<Log>> {
    match fs::read(root.join(log_path(id))) {
        Ok(bytes) => parse(&bytes).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn pages() -> impl Iterator<Item = &'static str> {
    DISCOVERY.into_iter().chain([NOTES]).chain(FINAL)
}

fn page_sha(root: &Path, id: &str, page: &str) -> Result<Option<String>> {
    if !artifacts::exists(root, id, page)? {
        return Ok(None);
    }
    let bytes = fs::read(root.join(artifacts::directory(id)).join(page))?;
    Ok(Some(history::digest(&bytes)))
}

/// Append one review of a page to the task's verdict log.
pub fn verdict(
    root: &Path,
    id: &str,
    page: &str,
    verdict: &str,
    reviewer: &str,
    notes: &str,
) -> Result<Value> {
    task_record::read(root, id)?;
    if !pages().any(|p| p == page) {
        return Err(format!("{page} is not a task page; see docs/ARTIFACTS.md").into());
    }
    let verdict: Verdict = serde_json::from_value(json!(verdict))
        .map_err(|_| format!("verdict {verdict} must be pass, revise or best"))?;
    if !valid_id(reviewer) {
        return Err("--reviewer names who reviewed, in letters, digits, - and _".into());
    }
    let sha = page_sha(root, id, page)?.ok_or(format!("{page} does not exist"))?;
    let mut log = read(root, id)?.unwrap_or(Log {
        schema_version: 1,
        rows: Vec::new(),
    });
    let rows: Vec<&Row> = log.rows.iter().filter(|r| r.page == page).collect();
    let revisions = rows.iter().filter(|r| r.verdict == Verdict::Revise).count();
    let round = match verdict {
        Verdict::Best => {
            if revisions < REVIEWS || rows.last().is_some_and(|r| r.verdict != Verdict::Revise) {
                return Err(format!(
                    "best is for a page whose {REVIEWS} reviews all asked for revision; {page} has {revisions}"
                )
                .into());
            }
            rows.iter()
                .find(|r| r.sha256 == sha)
                .ok_or(format!(
                    "{page} differs from every reviewed version; restore the best one first"
                ))?
                .round
        }
        _ => {
            if rows.last().is_some_and(|r| r.sha256 == sha) {
                return Err(format!("{page} was already reviewed in this exact version").into());
            }
            if revisions >= REVIEWS {
                return Err(format!(
                    "{page} has used its two rewrites; commit the best version and record --verdict best"
                )
                .into());
            }
            revisions + 1
        }
    };
    let row = Row {
        page: page.into(),
        round,
        verdict,
        sha256: sha,
        reviewer: reviewer.into(),
        notes: notes.trim().into(),
    };
    log.rows.push(row.clone());
    fs::write(
        root.join(log_path(id)),
        format!("{}\n", serde_json::to_string_pretty(&log)?),
    )?;
    Ok(json!({"ok":true,"task":id,"recorded":row,
        "next":format!("commit {page} with {} when the page is final; cargo xtask task-artifact next --task {id}", log_path(id))}))
}

fn step(action: &str, page: &str, round: usize, say: String) -> Option<Value> {
    Some(json!({"action":action,"page":page,"round":round,"say":say}))
}

/// The next step for one page, or None when it is reviewed and committed.
fn page_step(root: &Path, id: &str, page: &str, log: Option<&Log>) -> Result<Option<Value>> {
    let rows: Vec<&Row> = log
        .map(|l| l.rows.iter().filter(|r| r.page == page).collect())
        .unwrap_or_default();
    let revisions = rows.iter().filter(|r| r.verdict == Verdict::Revise).count();
    let Some(sha) = page_sha(root, id, page)? else {
        return Ok(step(
            "author",
            page,
            revisions + 1,
            format!(
                "Start a fresh artifact-author on {page} with its prompt and exemplar (docs/ARTIFACTS.md) and the task context."
            ),
        ));
    };
    // Git's view, not a hash of the blob: eol=lf normalises a CRLF page that status calls clean.
    let dirty = history::uncommitted(root)?;
    let path = format!("{}/{page}", artifacts::directory(id));
    let committed = !dirty.contains(&path);
    let review = |round: usize| {
        step(
            "review",
            page,
            round,
            format!(
                "Start a fresh artifact-reviewer, never the author, on {page}; then record it: cargo xtask task-artifact verdict --task {id} --page {page} --verdict pass|revise --reviewer NAME --notes TEXT"
            ),
        )
    };
    match rows.last() {
        // Tasks from before the log: a committed page counts as reviewed elsewhere.
        None if log.is_none() && committed => return Ok(None),
        None => return Ok(review(1)),
        Some(last) if last.verdict == Verdict::Revise && last.sha256 != sha => {
            return Ok(review(revisions + 1));
        }
        Some(_)
            if revisions >= REVIEWS
                && rows.last().is_some_and(|r| r.verdict == Verdict::Revise) =>
        {
            return Ok(step(
                "best",
                page,
                revisions,
                format!(
                    "Two rewrites are spent. Restore the best reviewed version of {page}, record it with --verdict best, and commit it with its open findings noted."
                ),
            ));
        }
        Some(last) if last.verdict == Verdict::Revise => {
            return Ok(step(
                "rewrite",
                page,
                revisions + 1,
                format!(
                    "Start a fresh artifact-author to rewrite {page} from the round {} findings (rewrite {revisions} of 2).",
                    last.round
                ),
            ));
        }
        Some(_) => {}
    }
    if committed && !dirty.contains(&log_path(id)) {
        return Ok(None);
    }
    Ok(step(
        "commit",
        page,
        rows.last().map_or(1, |r| r.round),
        format!("Commit {page} and {} together.", log_path(id)),
    ))
}

/// What the page loop says to do next for a task. Advises; never gates.
pub fn next(root: &Path, id: &str) -> Result<Value> {
    if !valid_id(id) {
        return Err("invalid task ID".into());
    }
    if !root.join(format!("harness/tasks/{id}.json")).is_file() {
        return Ok(
            json!({"ok":true,"task":id,"next":{"action":"record","say":format!(
            "Write harness/tasks/{id}.json first; the write-ticket skill drafts the ticket and a partial record stub.")},
            "advisories":[]}),
        );
    }
    let task = task_record::read(root, id)?;
    if task.tier == Tier::Small {
        return Ok(
            json!({"ok":true,"task":id,"tier":"small","next":{"action":"build","say":
            "Small task: no pages. Build, commit, run cargo xtask check ci, then complete; the gate measures the size."},
            "advisories":[]}),
        );
    }
    let log = read(root, id)?;
    let mut advisories = Vec::new();
    if let (Some(log), Some(committed)) = (&log, history::read(root, "HEAD", &log_path(id))?)
        && parse(&committed).is_ok_and(|c| !log.rows.starts_with(&c.rows))
    {
        advisories.push(format!(
            "{LOG} no longer starts with its committed rows; record verdicts only with task-artifact verdict"
        ));
    }
    let mut found = None;
    for page in DISCOVERY {
        found = page_step(root, id, page, log.as_ref())?;
        if found.is_some() {
            break;
        }
    }
    if found.is_none() {
        found = if artifacts::exists(root, id, NOTES)? {
            page_step(root, id, NOTES, log.as_ref())?
        } else {
            step("build", NOTES, 1, "Discovery is done. Build and commit the first product slice, then write implementation.html and commit it before the last product commit.".into())
        };
    }
    for page in FINAL {
        if found.is_some() {
            break;
        }
        found = if artifacts::exists(root, id, page)? {
            page_step(root, id, page, log.as_ref())?
        } else {
            step(
                "author",
                page,
                1,
                format!(
                    "Finish and commit the product work and its notes; then start a fresh artifact-author on {page}, which must be committed at or after the last product commit."
                ),
            )
        };
    }
    // Every page, not only those before the next step: a hand edit anywhere shows.
    for row in log.iter().flat_map(|l| &l.rows) {
        let latest = log
            .iter()
            .flat_map(|l| &l.rows)
            .rfind(|r| r.page == row.page);
        if latest == Some(row)
            && row.verdict != Verdict::Revise
            && page_sha(root, id, &row.page)?.is_some_and(|sha| sha != row.sha256)
        {
            advisories.push(format!(
                "{} was edited after its {:?} verdict; review it again or list the hand edit in the task record",
                row.page, row.verdict
            ));
        }
    }
    let source = if log.is_some() {
        format!("from {}", log_path(id))
    } else {
        "from each page's review".into()
    };
    let next = found.unwrap_or_else(|| json!({"action":"complete","say":format!(
        "Every page is reviewed and committed. Copy each page's final verdict and open notes {source} into review.verification, run cargo xtask check ci > target/verification/ci.json, then cargo xtask complete.")}));
    Ok(json!({"ok":true,"task":id,"next":next,"advisories":advisories}))
}
