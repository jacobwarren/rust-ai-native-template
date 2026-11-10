#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Archived history may leave the tree, and the small frictions a fresh agent hits:
//! where the report lands, what a failed check says, what a record must carry.
mod support;
use serde_json::{Value, json};
use std::process::Command;
use support::Fixture;

fn xtask(f: &Fixture, args: &[&str]) -> (bool, Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(&f.0)
        .output()
        .unwrap();
    (
        o.status.success(),
        serde_json::from_slice(&o.stdout).unwrap(),
    )
}

fn manifest(f: &Fixture, commit: &str, paths: &[&str]) {
    f.write(
        "harness/archive.json",
        json!({"schema_version":1,"archives":[{"name":"showcase/history","commit":commit,
            "reason":"Earlier tasks kept for reference","paths":paths}]}),
    );
}

/// A merged task with its record, one page and a receipt, then a later task that archives it.
fn merged_task() -> (Fixture, String) {
    let f = Fixture::new();
    let mut old = f.record("partial");
    old["id"] = json!("OLD-1");
    f.write("harness/tasks/OLD-1.json", old);
    f.text(
        "harness/task-artifacts/OLD-1/blindspots.html",
        &format!("<!doctype html><head>{}</head><p>old</p>", support::STYLE),
    );
    f.write(
        "harness/transitions/old-1.json",
        json!({"schema_version":1,"base_commit":"0".repeat(40),"changes":[]}),
    );
    f.commit("merged task");
    let base = f.git(&["rev-parse", "HEAD"]);
    let mut r = f.record("partial");
    r["schema_version"] = json!(7);
    r["tier"] = json!("small");
    f.write("harness/tasks/TASK-1.json", r);
    for p in [
        "harness/tasks/OLD-1.json",
        "harness/task-artifacts/OLD-1/blindspots.html",
        "harness/transitions/old-1.json",
    ] {
        std::fs::remove_file(f.0.join(p)).unwrap();
    }
    (f, base)
}

#[test]
fn archived_history_may_be_deleted_and_nothing_else() {
    let (f, base) = merged_task();
    // No manifest: both gates refuse the deletions.
    let (ok, v) = xtask(&f, &["gaps", "--base", &base]);
    assert!(
        !ok && v
            .to_string()
            .contains("transition history changed or deleted"),
        "{v}"
    );
    manifest(&f, &base, &["harness/transitions/old-1.json"]);
    let (_, v) = xtask(&f, &["handoff", "--base", &base]);
    let text = v.to_string();
    assert!(
        text.contains("task history deleted: harness/tasks/OLD-1.json"),
        "{v}"
    );
    assert!(text.contains("artifact has no task record: OLD-1"), "{v}");
    // Every deleted path listed, by file or by folder: the deletions pass.
    manifest(
        &f,
        &base,
        &[
            "harness/tasks/OLD-1.json",
            "harness/task-artifacts/OLD-1/",
            "harness/transitions/old-1.json",
        ],
    );
    let (_, v) = xtask(&f, &["handoff", "--base", &base]);
    let text = v.to_string();
    assert!(!text.contains("task history deleted"), "{v}");
    assert!(!text.contains("artifact has no task record"), "{v}");
    assert!(!text.contains("transition history"), "{v}");
    assert_eq!(xtask(&f, &["coordination"]).1["archives"], 1);
    // A listed path the commit does not hold is refused by name.
    let first = f.git(&["rev-list", "--max-parents=0", "HEAD"]);
    manifest(&f, &first, &["harness/tasks/OLD-1.json"]);
    let (ok, v) = xtask(&f, &["handoff", "--base", &base]);
    assert!(!ok && v.to_string().contains("does not hold it"), "{v}");
    // So is a commit this repository does not have.
    manifest(&f, &"a".repeat(40), &["harness/tasks/OLD-1.json"]);
    let (ok, v) = xtask(&f, &["handoff", "--base", &base]);
    assert!(
        !ok && v.to_string().contains("is not in this repository"),
        "{v}"
    );
    // A commit that holds an older version of the record does not keep this one.
    let (g, _) = merged_task();
    g.git(&["checkout", "-q", "HEAD", "--", "."]);
    let older = g.git(&["rev-parse", "HEAD"]);
    let mut edited = g.record("partial");
    edited["id"] = json!("OLD-1");
    edited["summary"] = json!("Edited after the archive commit");
    g.write("harness/tasks/OLD-1.json", edited);
    g.commit("record edited later");
    let later = g.git(&["rev-parse", "HEAD"]);
    std::fs::remove_file(g.0.join("harness/tasks/OLD-1.json")).unwrap();
    manifest(&g, &older, &["harness/tasks/OLD-1.json"]);
    let (ok, v) = xtask(&g, &["handoff", "--base", &later]);
    assert!(!ok && v.to_string().contains("changed after commit"), "{v}");
    // A tree unpacked without that history still validates: the commit is read
    // only when a deletion relies on it.
    manifest(&g, &"b".repeat(40), &["harness/tasks/GONE.json"]);
    assert_eq!(xtask(&g, &["coordination"]).1["archives"], 1);
    // A path still in the tree is not archived, and a short commit id is not an id.
    manifest(&f, &base, &["harness/tasks/TASK-1.json"]);
    let (ok, v) = xtask(&f, &["coordination"]);
    assert!(!ok && v.to_string().contains("still in the tree"), "{v}");
    manifest(&f, &base[..12], &["harness/tasks/OLD-1.json"]);
    let (ok, v) = xtask(&f, &["coordination"]);
    assert!(!ok && v.to_string().contains("full commit id"), "{v}");
    // A receipt that changed, rather than left, is still refused.
    let (f, base) = merged_task();
    manifest(
        &f,
        &base,
        &[
            "harness/tasks/OLD-1.json",
            "harness/task-artifacts/OLD-1/",
            "harness/transitions/old-1.json",
        ],
    );
    f.write(
        "harness/transitions/old-1.json",
        json!({"schema_version":1,"base_commit":"1".repeat(40),"changes":[]}),
    );
    let (ok, v) = xtask(&f, &["gaps", "--base", &base]);
    assert!(
        !ok && v
            .to_string()
            .contains("transition history changed or deleted"),
        "{v}"
    );
}

fn profile(f: &Fixture, argv: Value) {
    f.write(
        "harness/config.json",
        json!({"schema_version":1,"max_rust_lines":400,"layers":{"tooling":{"dependencies":[],"external":[]}},
            "profiles":{"ci":[{"id":"step","argv":argv,"timeout_seconds":10}]}}),
    );
    f.commit("profile");
}

#[test]
fn a_profile_run_writes_its_report_where_the_docs_say() {
    let f = Fixture::new();
    profile(&f, json!(["true"]));
    std::fs::remove_dir_all(f.0.join("target")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["check", "ci"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    assert!(o.status.success());
    let written = std::fs::read(f.0.join("target/verification/ci.json")).unwrap();
    assert_eq!(written, o.stdout);
    let v: Value = serde_json::from_slice(&written).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["checks"][0].get("output_tail").is_none(), "{v}");
}

#[test]
fn a_failed_check_shows_the_end_of_its_output() {
    let f = Fixture::new();
    profile(
        &f,
        json!([
            "sh",
            "-c",
            "seq 1 100; echo 'test board::rename ... FAILED'; echo 'left: 1 right: 2' >&2; exit 1"
        ]),
    );
    let (ok, v) = xtask(&f, &["check", "ci"]);
    assert!(!ok);
    let tail = v["checks"][0]["output_tail"].as_str().unwrap();
    assert!(tail.contains("test board::rename ... FAILED"), "{tail}");
    assert!(tail.contains("left: 1 right: 2"), "{tail}");
    // Only the end: the first lines of a long log stay in the log file.
    assert!(
        !tail.contains("\n1\n") && tail.contains("\n100\n"),
        "{tail}"
    );
}

#[test]
fn a_complete_record_needs_no_remaining_field() {
    let f = Fixture::new();
    let mut r = f.record("complete");
    r["tier"] = json!("small");
    r.as_object_mut().unwrap().remove("remaining");
    f.write("harness/tasks/TASK-1.json", r);
    assert_eq!(xtask(&f, &["coordination"]).1["ok"], true);
    let mut r = f.record("partial");
    r["tier"] = json!("small");
    r.as_object_mut().unwrap().remove("remaining");
    f.write("harness/tasks/TASK-1.json", r);
    let (ok, v) = xtask(&f, &["coordination"]);
    assert!(
        !ok && v["error"]
            .as_str()
            .unwrap()
            .contains("lists its next steps in `remaining`"),
        "{v}"
    );
}

#[test]
fn next_reports_a_standard_page_over_its_size_limit() {
    let f = Fixture::new();
    let mut r = f.record("partial");
    r["schema_version"] = json!(7);
    f.write("harness/tasks/TASK-1.json", r);
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    f.page("brief.html");
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["task-artifact", "next", "--task", "TASK-1"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(
        v["next"]["handover"]
            .as_str()
            .unwrap()
            .contains("at most 20 KB as a file"),
        "{v}"
    );
    f.text(
        "harness/task-artifacts/TASK-1/brief.html",
        &format!(
            "<!doctype html><head>{}</head><p>{}</p>",
            support::STYLE,
            "word ".repeat(4100)
        ),
    );
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["task-artifact", "next", "--task", "TASK-1"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(3));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let advice = v["advisories"][0].as_str().unwrap();
    assert!(
        advice.starts_with("brief.html is 20") && advice.contains("stays within 20000"),
        "{advice}"
    );
}

#[test]
fn a_late_brief_on_a_merge_commit_points_at_head() {
    let f = Fixture::new();
    let main = f.git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let base = f.git(&["rev-parse", "HEAD"]);
    f.git(&["switch", "-qc", "work"]);
    let mut r = f.record("partial");
    r["schema_version"] = json!(7);
    f.write("harness/tasks/TASK-1.json", r);
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    // A correctly ordered branch: brief, then code.
    f.page("brief.html");
    f.commit("brief");
    f.text("implementation.rs", "// slice 1\n");
    f.commit("code");
    let head = f.git(&["rev-parse", "HEAD"]);
    f.git(&["switch", "-q", &main]);
    f.text("other.rs", "// the target moved\n");
    f.commit("target moved");
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
    let (_, v) = xtask(&f, &["handoff", "--base", &base]);
    let text = v.to_string();
    assert!(
        text.contains("before the first product-code commit; HEAD is a merge commit"),
        "{v}"
    );
    assert!(text.contains("handoff --head"), "{v}");
    let (_, v) = xtask(&f, &["handoff", "--base", &base, "--head", &head]);
    assert!(
        !v.to_string()
            .contains("before the first product-code commit"),
        "{v}"
    );
}
