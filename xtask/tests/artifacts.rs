#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The brief: a standard task commits one page before its code. The gate checks
//! presence, a floor of text and commit order, never usefulness.
mod support;
use serde_json::{Value, json};
use std::{io::Write, process::Command, process::Stdio};
use support::Fixture;

fn setup() -> (Fixture, String) {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.write("harness/tasks/TASK-1.json", f.standard("complete"));
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    (f, base)
}
fn code(f: &Fixture, n: u32) {
    f.text("implementation.rs", &format!("// slice {n}\n"));
}
fn raw(f: &Fixture, args: &[&str]) -> (Option<i32>, Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(&f.0)
        .output()
        .unwrap();
    (o.status.code(), serde_json::from_slice(&o.stdout).unwrap())
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
fn a_standard_task_owes_a_brief_before_code() {
    let (f, base) = setup();
    assert!(errors(&complete_from(&f, &base)).contains("missing brief.html"));
    let style = f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    let block = std::fs::read_to_string(f.0.join(style["style"].as_str().unwrap())).unwrap();
    assert!(block.starts_with("<style data-artifact-foundation=\"1\">") && block.contains("--ink"));
    // A page that loads its style from beside it is not self-contained.
    f.text(
        "harness/task-artifacts/TASK-1/brief.html",
        "<!doctype html><link rel=\"stylesheet\" href=\"foundation.css\"><p>sibling file</p>",
    );
    f.commit("sibling style");
    assert!(
        errors(&complete_from(&f, &base)).contains("brief.html must carry the shared stylesheet")
    );
    // A page that is only the stylesheet, however large, is a placeholder.
    f.text(
        "harness/task-artifacts/TASK-1/brief.html",
        &format!(
            "<!doctype html><head>{}<style>.filler::after {{ content: \"{}\"; }}</style></head><p>Soon.</p>",
            support::STYLE,
            "x".repeat(5000)
        ),
    );
    f.commit("placeholder");
    assert!(
        errors(&complete_from(&f, &base))
            .contains("brief.html holds 5 characters outside its tags and styles")
    );
    f.brief();
    f.commit("brief");
    code(&f, 1);
    f.commit("code");
    assert_eq!(complete_from(&f, &base)["ok"], true);
    // Nothing is owed after the code: a late fix needs no page touched.
    code(&f, 2);
    f.commit("late fix");
    assert_eq!(complete_from(&f, &base)["ok"], true);
    let v = f.run(&["task-artifact", "check", "--task", "TASK-1"]);
    assert_eq!(v["brief_owed"], true, "{v}");

    // Written after the code, the brief is refused, for a partial task too.
    let (f, base) = setup();
    f.write("harness/tasks/TASK-1.json", f.standard("partial"));
    code(&f, 1);
    f.commit("code first");
    f.brief();
    f.commit("brief after code");
    let late = "brief.html must be committed before the first product-code commit";
    let (_, v) = raw(&f, &["handoff", "--base", &base]);
    assert!(v.to_string().contains(late), "{v}");
    f.write("harness/tasks/TASK-1.json", f.standard("complete"));
    assert!(errors(&complete_from(&f, &base)).contains(late));
}

#[test]
fn coordination_accepts_a_started_task_but_checks_a_brief_it_has() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.standard("partial"));
    assert_eq!(f.run(&["coordination"])["ok"], true);
    f.text(
        "harness/task-artifacts/TASK-1/brief.html",
        "<!doctype html><link rel=\"stylesheet\" href=\"foundation.css\">",
    );
    assert!(
        f.run(&["coordination"])["error"]
            .to_string()
            .contains("brief.html must carry the shared stylesheet inline")
    );
    std::fs::remove_dir_all(f.0.join("harness/task-artifacts")).unwrap();
    f.write("harness/tasks/TASK-1.json", f.standard("complete"));
    assert!(
        f.run(&["coordination"])["error"]
            .to_string()
            .contains("missing brief.html")
    );
    // A small task owes none, but one it writes is still checked.
    f.write("harness/tasks/TASK-1.json", f.record("complete"));
    assert_eq!(f.run(&["coordination"])["ok"], true);
}

#[test]
fn completion_requires_committed_code_and_brief() {
    let (f, base) = setup();
    f.brief();
    f.commit("brief");
    code(&f, 1);
    f.commit("work");
    assert_eq!(complete_from(&f, &base)["ok"], true);
    f.brief();
    assert!(errors(&complete_from(&f, &base)).contains("commit product changes"));
    f.commit("brief edit");
    code(&f, 2);
    assert!(errors(&complete_from(&f, &base)).contains("implementation.rs"));
    // Task records, gaps and reports stay editable without a commit.
    let mut record = f.standard("complete");
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
fn guard_blocks_code_edits_until_the_brief_is_committed() {
    let (f, _) = setup();
    let guard = ["task-artifact", "guard", "--task", "TASK-1"];
    assert_eq!(f.run(&guard)["ok"], false);
    f.brief();
    assert!(f.run(&guard)["error"].to_string().contains("not committed"));
    assert_eq!(hook(&f, &["implementation.rs"])["ok"], false);
    let page = hook(&f, &["harness/task-artifacts/TASK-1/brief.html"]);
    assert_eq!(page["ok"], true);
    assert_eq!(hook(&f, &["../outside.rs"])["ok"], false);
    f.commit("brief");
    assert_eq!(f.run(&guard)["ok"], true);
    assert_eq!(hook(&f, &["implementation.rs"])["ok"], true);
    // A small task has nothing to wait for.
    f.write("harness/tasks/TASK-1.json", f.record("partial"));
    std::fs::remove_dir_all(f.0.join("harness/task-artifacts")).unwrap();
    assert_eq!(f.run(&guard)["tier"], "small");
}

#[test]
fn merged_target_commits_do_not_count_as_task_code() {
    let (f, _) = setup();
    let target = f.git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    f.git(&["checkout", "-qb", "task"]);
    f.brief();
    f.commit("brief");
    code(&f, 1);
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

fn merge_into(f: &Fixture, main: &str) {
    f.git(&["switch", "-q", main]);
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
        "Merge work",
    ]);
}

#[test]
fn handoff_reads_the_branchs_commits_from_a_pull_request_merge() {
    let late = "brief.html must be committed before the first product-code commit";
    // A late brief, read from a host's merge commit after the target moved.
    let f = Fixture::new();
    let main = f.git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let base = f.git(&["rev-parse", "HEAD"]);
    f.git(&["switch", "-qc", "work"]);
    f.write("harness/tasks/TASK-1.json", f.standard("partial"));
    code(&f, 1);
    f.commit("code first");
    f.brief();
    f.commit("brief after code");
    let head = f.git(&["rev-parse", "HEAD"]);
    f.git(&["switch", "-q", &main]);
    f.text("other.rs", "// merged by someone else\n");
    f.commit("target moved");
    merge_into(&f, &main);
    let (_, v) = raw(&f, &["handoff", "--base", &base, "--head", &head]);
    assert!(v.to_string().contains(late), "{v}");
    // When the merge sits directly on the base, the branch is found without a head.
    let f = Fixture::new();
    let main = f.git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let base = f.git(&["rev-parse", "HEAD"]);
    f.git(&["switch", "-qc", "work"]);
    f.write("harness/tasks/TASK-1.json", f.standard("partial"));
    code(&f, 1);
    f.commit("code first");
    f.brief();
    f.commit("brief after code");
    merge_into(&f, &main);
    let (_, v) = raw(&f, &["handoff", "--base", &base]);
    assert!(v.to_string().contains(late), "{v}");
    // A correct branch on a moved target needs the head, and the error says so.
    let f = Fixture::new();
    let main = f.git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let base = f.git(&["rev-parse", "HEAD"]);
    f.git(&["switch", "-qc", "work"]);
    f.write("harness/tasks/TASK-1.json", f.standard("partial"));
    f.brief();
    f.commit("brief");
    code(&f, 1);
    f.commit("code");
    let head = f.git(&["rev-parse", "HEAD"]);
    f.git(&["switch", "-q", &main]);
    f.text("other.rs", "// the target moved\n");
    f.commit("target moved");
    merge_into(&f, &main);
    let (_, v) = raw(&f, &["handoff", "--base", &base]);
    let text = v.to_string();
    assert!(
        text.contains("HEAD is a merge commit") && text.contains("handoff --head"),
        "{v}"
    );
    let (_, v) = raw(&f, &["handoff", "--base", &base, "--head", &head]);
    assert!(!v.to_string().contains(late), "{v}");
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
fn screenshots_need_a_page_and_a_valid_task() {
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
