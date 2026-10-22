#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Discovery is agent judgment. These tests cover reconciliation, not completeness of thought.
mod support;
use serde_json::json;
use support::Fixture;

#[test]
fn prompt_only_work_needs_no_ticket_or_new_gap() {
    let f = Fixture::new();
    let original = std::fs::read(f.0.join("harness/gaps.json")).unwrap();
    let mut task = f.task("complete");
    task.as_object_mut().unwrap().remove("work_item");
    task["related_gaps"] = json!([]);
    task["gap_review"] =
        json!("Reviewed the project register; no gaps are associated with this task.");
    f.write("harness/tasks/TASK-1.json", task);
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    assert_eq!(f.handoff()["ok"], true);
    let gaps = f.run(&["gaps"]);
    assert_eq!(gaps["scope"], "project");
    assert_eq!(gaps["generates_gaps"], false);
    assert_eq!(
        std::fs::read(f.0.join("harness/gaps.json")).unwrap(),
        original
    );
}

#[test]
fn opaque_ticket_ids_do_not_change_completion_semantics() {
    let f = Fixture::new();
    for reference in [
        "JIRA-142",
        "GitHub issue #7",
        "https://unreachable.invalid/TASK-1",
    ] {
        let mut task = f.task("complete");
        task["work_item"] = json!(reference);
        f.write("harness/tasks/TASK-1.json", task);
        f.evidence();
        assert_eq!(f.complete()["ok"], true);
    }
}

#[test]
fn stop_hook_returns_specific_questions_for_missing_review() {
    let f = Fixture::new();
    let mut task = f.task("complete");
    task["review"] = json!({});
    f.write("harness/tasks/TASK-1.json", task);
    f.evidence();
    let result = f.run(&[
        "agent-stop",
        "--task",
        "TASK-1",
        "--base",
        "HEAD",
        "--report",
        "target/report.json",
    ]);
    assert_eq!(result["ok"], false);
    let errors = result["task"]["errors"].as_array().unwrap();
    for field in ["acceptance", "verification", "assumptions", "coverage"] {
        assert!(
            errors
                .iter()
                .any(|e| e.as_str().unwrap().starts_with(&format!("review.{field}:")))
        );
    }
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
}

#[test]
fn material_discovery_requires_durable_record_but_can_remain_open_after_task() {
    let f = Fixture::new();
    let mut task = f.task("complete");
    task["discoveries"] =
        json!([{"summary":"A distinct recovery path needs validation.","record":"gap:recovery"}]);
    task["review"]["coverage"] =
        json!("Found an out-of-scope recovery gap; recorded as gap:recovery.");
    f.write("harness/tasks/TASK-1.json", task);
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
    let mut recovery = Fixture::gap();
    recovery["id"] = json!("recovery");
    f.write("harness/gaps.json", json!([Fixture::gap(), recovery]));
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    assert_eq!(f.handoff()["ok"], true);
    assert_eq!(
        f.run(&["release", "product", "--report", "target/report.json"])["ok"],
        false
    );
}

#[test]
fn historical_v1_records_are_readable_but_cannot_make_new_completion_claims() {
    let f = Fixture::new();
    let mut task = f.task("complete");
    task["schema_version"] = json!(1);
    task.as_object_mut().unwrap().remove("review");
    f.write("harness/tasks/TASK-1.json", task);
    f.evidence();
    assert_eq!(f.run(&["coordination"])["ok"], true);
    assert_eq!(f.complete()["ok"], false);
}

#[test]
fn removed_external_authority_fails_explicitly_instead_of_using_a_stale_local_copy() {
    let f = Fixture::new();
    f.write(
        "harness/tracking.json",
        json!({"schema_version":1,"mode":"external"}),
    );
    let result = f.run(&["gaps"]);
    assert_eq!(result["ok"], false);
    assert!(
        result["error"]
            .as_str()
            .unwrap()
            .contains("migrate reviewed gaps")
    );
    f.evidence();
    assert_eq!(
        f.run(&["release", "template", "--report", "target/report.json"])["ok"],
        false
    );
}
