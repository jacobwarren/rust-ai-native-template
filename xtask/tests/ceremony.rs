#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The tiers of schema 7: a small task owes no pages, a standard task a brief, a full task
//! the eight-page set, and the interview and quiz only when a person will answer.
mod support;
use serde_json::{Value, json};
use std::process::Command;
use support::{ALL, Fixture};

const SIX: [&str; 6] = [
    "blindspots.html",
    "interventions.html",
    "reference.html",
    "plan.html",
    "implementation.html",
    "buy-in.html",
];

fn record(f: &Fixture, status: &str, tier: Option<&str>, reader: Option<&str>) -> Value {
    let mut r = f.record(status);
    r["schema_version"] = json!(7);
    if let Some(tier) = tier {
        r["tier"] = json!(tier);
    }
    if let Some(reader) = reader {
        r["reader"] = json!(reader);
    }
    r
}

fn xtask(f: &Fixture, args: &[&str]) -> Value {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(&f.0)
        .output()
        .unwrap();
    serde_json::from_slice(&o.stdout).unwrap()
}

fn complete(f: &Fixture, base: &str) -> String {
    f.evidence();
    xtask(
        f,
        &[
            "complete",
            "--task",
            "TASK-1",
            "--base",
            base,
            "--report",
            "target/report.json",
        ],
    )
    .to_string()
}

fn code(f: &Fixture, n: u32) {
    f.text("implementation.rs", &format!("// slice {n}\n"));
}

#[test]
fn a_task_with_no_tier_owes_no_pages_at_any_size() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.write(
        "harness/tasks/TASK-1.json",
        record(&f, "complete", None, None),
    );
    f.text("src/feature.rs", &"// a line of the feature\n".repeat(500));
    f.commit("a 500-line feature and its record");
    let done = complete(&f, &base);
    assert!(done.contains("\"complete\":true"), "{done}");
    assert!(!f.0.join("harness/task-artifacts/TASK-1").exists());
    let v = xtask(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["tier"], "small", "{v}");
    assert!(
        v["next"]["say"]
            .as_str()
            .unwrap()
            .starts_with("No pages owed."),
        "{v}"
    );
}

#[test]
fn a_standard_task_owes_only_a_brief_before_code() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.write(
        "harness/tasks/TASK-1.json",
        record(&f, "complete", Some("standard"), None),
    );
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    code(&f, 1);
    f.commit("code first");
    assert!(complete(&f, &base).contains("missing brief.html"));
    f.page("brief.html");
    f.commit("brief after code");
    assert!(complete(&f, &base).contains("brief.html must be committed before the first"));

    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.write(
        "harness/tasks/TASK-1.json",
        record(&f, "complete", Some("standard"), None),
    );
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    f.page("brief.html");
    f.commit("brief");
    code(&f, 1);
    f.commit("code");
    // Nothing is owed after the code: no report, none of the eight-page set.
    let done = complete(&f, &base);
    assert!(done.contains("\"complete\":true"), "{done}");
    code(&f, 2);
    f.commit("a late fix");
    let done = complete(&f, &base);
    assert!(done.contains("\"complete\":true"), "{done}");
    let v = xtask(&f, &["task-artifact", "check", "--task", "TASK-1"]);
    assert_eq!(v["pages"], json!(["brief.html"]), "{v}");
}

#[test]
fn full_task_writes_the_interview_and_quiz_only_for_a_human_reader() {
    for (reader, pages, ok) in [
        (None, &SIX[..], true),
        (Some("none"), &SIX[..], true),
        (Some("human"), &SIX[..], false),
        (Some("human"), &ALL[..], true),
    ] {
        let f = Fixture::new();
        f.write(
            "harness/tasks/TASK-1.json",
            record(&f, "complete", Some("full"), reader),
        );
        f.run(&["task-artifact", "style", "--task", "TASK-1"]);
        f.pages(pages);
        let v = xtask(&f, &["task-artifact", "check", "--task", "TASK-1"]);
        assert_eq!(v["ok"] == true, ok, "{reader:?}: {v}");
        if !ok {
            assert!(v.to_string().contains("missing interview.html"), "{v}");
        }
    }
    // next never routes a task without a reader to either page.
    let f = Fixture::new();
    f.write(
        "harness/tasks/TASK-1.json",
        record(&f, "partial", Some("full"), None),
    );
    let v = xtask(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["pages"], json!(SIX), "{v}");
    // The plan's author is told where the interview's questions go.
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    f.pages(&["blindspots.html", "interventions.html", "reference.html"]);
    f.commit("discovery so far");
    let v = xtask(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["next"]["page"], "plan.html", "{v}");
    let text = v["next"]["handover"].as_str().unwrap();
    assert!(text.contains("No reader: No person will answer"), "{text}");
    assert!(!text.contains("Length:"), "{text}");
}

#[test]
fn reader_and_tier_are_strict() {
    let f = Fixture::new();
    for (tier, reader, version, ok) in [
        (Some("full"), Some("human"), 7, true),
        (Some("standard"), Some("none"), 7, true),
        (Some("full"), Some("owner"), 7, false),
        (Some("big"), None, 7, false),
        // A schema 6 record predates the reader field.
        (None, Some("human"), 6, false),
        (None, None, 8, false),
    ] {
        let mut r = record(&f, "partial", tier, reader);
        r["schema_version"] = json!(version);
        f.write("harness/tasks/TASK-1.json", r);
        let v = xtask(&f, &["coordination"]);
        assert_eq!(v["ok"] == true, ok, "{tier:?} {reader:?} {version}: {v}");
    }
}

#[test]
fn a_placeholder_page_is_refused() {
    let f = Fixture::new();
    f.write(
        "harness/tasks/TASK-1.json",
        record(&f, "partial", Some("standard"), None),
    );
    f.text(
        "harness/task-artifacts/TASK-1/brief.html",
        &format!(
            "<!doctype html><head>{}<style>.filler::after {{ content: \"{}\"; }}</style></head><p>Soon.</p>",
            support::STYLE,
            "x".repeat(5000)
        ),
    );
    let v = xtask(&f, &["task-artifact", "check", "--task", "TASK-1"]);
    assert!(
        v.to_string()
            .contains("brief.html holds 5 characters outside its tags and styles"),
        "{v}"
    );
    f.page("brief.html");
    assert_eq!(
        f.run(&["task-artifact", "check", "--task", "TASK-1"])["ok"],
        true
    );
}

#[test]
fn a_partial_handoff_checks_that_the_brief_came_before_code() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.write(
        "harness/tasks/TASK-1.json",
        record(&f, "partial", Some("standard"), None),
    );
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    code(&f, 1);
    f.commit("code first");
    f.page("brief.html");
    f.commit("brief after code");
    let v = xtask(&f, &["handoff", "--base", &base]);
    assert!(
        v.to_string()
            .contains("brief.html must be committed before the first product-code commit"),
        "{v}"
    );
}

#[test]
fn handoff_reads_the_branchs_commits_from_a_pull_request_merge() {
    let f = Fixture::new();
    let main = f.git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    f.git(&["switch", "-qc", "work"]);
    f.write(
        "harness/tasks/TASK-1.json",
        record(&f, "partial", Some("standard"), None),
    );
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    code(&f, 1);
    f.commit("code first");
    f.page("brief.html");
    f.commit("brief after code");
    let head = f.git(&["rev-parse", "HEAD"]);
    // The target branch moves on, then the host builds its merge commit on top.
    f.git(&["switch", "-q", &main]);
    let event_base = f.git(&["rev-parse", "HEAD"]);
    f.text("other.rs", "// merged by someone else\n");
    f.commit("target moved");
    let late = "brief.html must be committed before the first product-code commit";
    f.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "merge",
        "-q",
        "--no-ff",
        "work",
        "-m",
        "Merge work into main",
    ]);
    // From the merge commit alone the branch has no commits; the head names them.
    let v = xtask(&f, &["handoff", "--base", &event_base, "--head", &head]);
    assert!(v.to_string().contains(late), "{v}");
    // When the merge sits directly on the base, the branch is found without a head.
    let f2 = Fixture::new();
    let main = f2.git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let base = f2.git(&["rev-parse", "HEAD"]);
    f2.git(&["switch", "-qc", "work"]);
    f2.write(
        "harness/tasks/TASK-1.json",
        record(&f2, "partial", Some("standard"), None),
    );
    f2.run(&["task-artifact", "style", "--task", "TASK-1"]);
    code(&f2, 1);
    f2.commit("code first");
    f2.page("brief.html");
    f2.commit("brief after code");
    f2.git(&["switch", "-q", &main]);
    f2.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "merge",
        "-q",
        "--no-ff",
        "work",
        "-m",
        "Merge work into main",
    ]);
    let v = xtask(&f2, &["handoff", "--base", &base]);
    assert!(v.to_string().contains(late), "{v}");
}
