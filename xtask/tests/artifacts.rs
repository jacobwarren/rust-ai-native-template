#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The gate checks presence and commit order of task pages, never their usefulness.
mod support;
use serde_json::{Value, json};
use std::{io::Write, process::Command, process::Stdio};
use support::{ALL, DISCOVERY, Fixture};

fn setup() -> (Fixture, String) {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.write("harness/tasks/TASK-1.json", f.record("complete"));
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    (f, base)
}
fn code(f: &Fixture, n: u32) {
    f.text("implementation.rs", &format!("// slice {n}\n"));
}
fn complete_from(f: &Fixture, base: &str) -> Value {
    f.evidence();
    f.run(&[
        "complete",
        "--task",
        "TASK-1",
        "--base",
        base,
        "--report",
        "target/report.json",
    ])
}
fn errors(v: &Value) -> String {
    v["task"]["errors"].to_string()
}

#[test]
fn completion_requires_eight_pages_carrying_the_stylesheet_inline() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.record("complete"));
    f.evidence();
    assert!(errors(&f.complete()).contains("missing blindspots.html"));
    let style = f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    let block = std::fs::read_to_string(f.0.join(style["style"].as_str().unwrap())).unwrap();
    assert!(block.starts_with("<style data-artifact-foundation=\"1\">") && block.contains("--ink"));
    assert!(
        !f.0.join("harness/task-artifacts/TASK-1/foundation.css")
            .exists()
    );
    f.pages(&DISCOVERY);
    f.commit("discovery");
    f.evidence();
    assert!(errors(&f.complete()).contains("missing implementation.html"));
    f.pages(&ALL);
    f.text(
        "harness/task-artifacts/TASK-1/quiz.html",
        "<!doctype html><link rel=\"stylesheet\" href=\"foundation.css\"><p>sibling file</p>",
    );
    f.commit("pages");
    f.evidence();
    assert!(errors(&f.complete()).contains("quiz.html must carry the shared stylesheet inline"));
    f.page("quiz.html");
    f.commit("fix quiz");
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    assert_eq!(
        f.run(&["task-artifact", "check", "--task", "TASK-1"])["ok"],
        true
    );
}

#[test]
fn discovery_must_be_committed_before_the_first_code_commit() {
    let (f, base) = setup();
    code(&f, 1);
    f.commit("code first");
    f.pages(&DISCOVERY);
    f.commit("discovery after code");
    f.pages(&["implementation.html", "buy-in.html", "quiz.html"]);
    f.commit("rest");
    let late = complete_from(&f, &base);
    assert_eq!(late["ok"], false);
    assert!(errors(&late).contains("before the first product-code commit"));

    let (f, base) = setup();
    f.pages(&DISCOVERY);
    f.commit("discovery");
    code(&f, 1);
    f.page("implementation.html");
    f.commit("first slice with notes");
    f.pages(&["buy-in.html", "quiz.html"]);
    f.commit("final pages");
    assert_eq!(complete_from(&f, &base)["ok"], true);
}

#[test]
fn notes_must_land_during_the_work_and_final_pages_after_it() {
    let (f, base) = setup();
    f.pages(&DISCOVERY);
    f.commit("discovery");
    code(&f, 1);
    f.commit("slice 1");
    code(&f, 2);
    f.commit("slice 2");
    f.pages(&["implementation.html", "buy-in.html", "quiz.html"]);
    f.commit("notes written at the end");
    let late = complete_from(&f, &base);
    assert!(errors(&late).contains("implementation.html must be committed during the work"));

    let (f, base) = setup();
    f.pages(&DISCOVERY);
    f.commit("discovery");
    code(&f, 1);
    f.page("implementation.html");
    f.commit("slice 1 with notes");
    code(&f, 2);
    f.commit("slice 2");
    f.pages(&["buy-in.html", "quiz.html"]);
    f.commit("final pages");
    assert_eq!(complete_from(&f, &base)["ok"], true);
    code(&f, 3);
    f.commit("code after final pages");
    let stale = complete_from(&f, &base);
    assert!(errors(&stale).contains("buy-in.html must be updated at or after"));
    f.pages(&["buy-in.html", "quiz.html"]);
    f.commit("refresh final pages");
    assert_eq!(complete_from(&f, &base)["ok"], true);
}

#[test]
fn completion_requires_committed_code_and_pages() {
    let (f, base) = setup();
    f.pages(&DISCOVERY);
    f.commit("discovery");
    code(&f, 1);
    f.pages(&["implementation.html", "buy-in.html", "quiz.html"]);
    f.commit("work");
    assert_eq!(complete_from(&f, &base)["ok"], true);
    f.page("buy-in.html");
    let dirty = complete_from(&f, &base);
    assert!(errors(&dirty).contains("commit product changes"));
    f.commit("buy-in edit");
    code(&f, 2);
    assert!(errors(&complete_from(&f, &base)).contains("implementation.rs"));
    // Task records, gaps and reports stay editable without a commit.
    let mut record = f.record("complete");
    record["summary"] = json!("Summary edited after the last commit");
    f.write("harness/tasks/TASK-1.json", record);
    f.git(&["checkout", "--", "implementation.rs"]);
    assert_eq!(complete_from(&f, &base)["ok"], true);
}

fn hook(f: &Fixture, paths: &[&str]) -> Value {
    let patch: String = paths
        .iter()
        .map(|p| format!("*** Update File: {p}\n"))
        .collect();
    let event = json!({"tool_name":"apply_patch","tool_input":{"command":patch}});
    let mut child = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["artifact-hook", "--task", "TASK-1"])
        .current_dir(&f.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(event.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.success(), v["ok"] == true, "{v}");
    v
}

#[test]
fn guard_blocks_code_edits_until_discovery_is_committed() {
    let (f, _) = setup();
    let guard = ["task-artifact", "guard", "--task", "TASK-1"];
    assert_eq!(f.run(&guard)["ok"], false);
    f.pages(&DISCOVERY);
    assert!(f.run(&guard)["error"].to_string().contains("not committed"));
    assert_eq!(hook(&f, &["implementation.rs"])["ok"], false);
    let pages = hook(&f, &["harness/task-artifacts/TASK-1/plan.html"]);
    assert_eq!(pages["ok"], true);
    assert_eq!(hook(&f, &["../outside.rs"])["ok"], false);
    f.commit("discovery");
    assert_eq!(f.run(&guard)["ok"], true);
    assert_eq!(hook(&f, &["implementation.rs"])["ok"], true);
}

#[test]
fn merged_target_commits_do_not_count_as_task_code() {
    let (f, _) = setup();
    let target = f.git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    f.git(&["checkout", "-qb", "task"]);
    f.pages(&DISCOVERY);
    f.commit("discovery");
    code(&f, 1);
    f.pages(&["implementation.html", "buy-in.html", "quiz.html"]);
    f.commit("work");
    f.git(&["checkout", "-q", &target]);
    f.text("resolution.txt", "Target branch moved on\n");
    f.commit("unrelated target change");
    f.git(&["checkout", "-q", "task"]);
    f.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "merge",
        "-q",
        "--no-edit",
        &target,
    ]);
    let merged = complete_from(&f, &target);
    assert_eq!(merged["ok"], true, "{merged}");
}

#[test]
fn concurrent_fixtures_have_independent_directories() {
    let threads: Vec<_> = (0..16)
        .map(|i| {
            std::thread::spawn(move || {
                let f = Fixture::new();
                f.text("marker", &i.to_string());
                (f, i)
            })
        })
        .collect();
    let fixtures: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    let paths: std::collections::BTreeSet<_> = fixtures.iter().map(|(f, _)| f.0.clone()).collect();
    assert_eq!(paths.len(), 16);
    for (f, i) in fixtures {
        assert_eq!(
            std::fs::read_to_string(f.0.join("marker")).unwrap(),
            i.to_string()
        );
    }
}

#[test]
fn screenshots_need_pages_and_a_valid_task() {
    let (f, _) = setup();
    let shot = |id: &str| f.run(&["task-artifact", "screenshot", "--task", id]);
    assert!(
        shot("TASK-1")["error"]
            .to_string()
            .contains("no HTML pages")
    );
    assert!(
        shot("TASK-2")["error"]
            .to_string()
            .contains("no artifact folder")
    );
    assert!(
        shot("../TASK-1")["error"]
            .to_string()
            .contains("invalid task ID")
    );
}
