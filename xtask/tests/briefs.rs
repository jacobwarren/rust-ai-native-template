#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The page briefs carry the reviewer's fixed checks, and the author brief has a
//! rule under each check's name, in both hosts (docs/EVALUATION.md has why).
use std::path::Path;

/// The checks a sampled review missed about half the time in the calibration.
const CHECKS: [&str; 4] = ["consistent", "provenance", "cited", "ticket"];
const HOSTS: [&str; 2] = [".claude/skills", ".agents/skills"];

fn brief(host: &str, skill: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(host)
        .join(skill)
        .join("SKILL.md");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The first cell of each row in the table headed `| Check |`.
fn check_rows(review: &str) -> Vec<String> {
    let mut lines = review
        .lines()
        .map(str::trim)
        .skip_while(|l| !l.starts_with("| Check |"));
    lines.next().expect("a table headed | Check |");
    lines
        .skip(1) // the | --- | separator
        .take_while(|l| l.starts_with('|'))
        .map(|l| l.split('|').nth(1).unwrap_or_default().trim().to_string())
        .collect()
}

#[test]
fn reviewer_brief_runs_the_fixed_checks() {
    for host in HOSTS {
        let review = brief(host, "review-artifact");
        // The table cell, not the bare word: "cited" already sits inside "uncited".
        assert_eq!(check_rows(&review), CHECKS, "{host}");
        assert!(review.contains("Fixed checks:"), "{host}: the report line");
        assert!(review.contains("FINDINGS: N"), "{host}");
    }
}

#[test]
fn author_brief_answers_each_fixed_check() {
    for host in HOSTS {
        let author = brief(host, "author-artifact");
        for check in check_rows(&brief(host, "review-artifact")) {
            assert!(
                author.contains(&format!("- **{check}**:")),
                "{host}: the author brief has no rule for {check}"
            );
        }
    }
}
