#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Synthetic reports test admission, not report authenticity or agent honesty.
mod support;
use serde_json::json;
use support::Fixture;

#[test]
fn completion_requires_acceptance_and_fresh_evidence_but_not_closing_unrelated_gaps() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    assert_eq!(f.complete()["ok"], false);
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    assert_eq!(f.handoff()["all_tasks_complete"], true);
    f.text("implementation.rs", "// changed after verification\n");
    assert_eq!(f.complete()["ok"], false);
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    let mut task = f.task("complete");
    task["requirements"] = json!(["invented"]);
    f.write("harness/tasks/TASK-1.json", task);
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
}
#[test]
fn partial_and_blocked_handoffs_are_honest_and_do_not_claim_completion() {
    for status in ["partial", "blocked"] {
        let f = Fixture::new();
        f.write("harness/tasks/TASK-1.json", f.task(status));
        let h = f.run(&["handoff", "--base", "HEAD"]);
        assert_eq!(h["ok"], true);
        assert_eq!(h["all_tasks_complete"], false);
        assert_eq!(f.complete()["ok"], false);
        let mut task = f.task(status);
        task["remaining"] = json!([]);
        f.write("harness/tasks/TASK-1.json", task);
        assert_eq!(f.handoff()["ok"], false);
    }
}
#[test]
fn reconciliation_rejects_missing_records_false_closure_and_missing_discoveries() {
    let f = Fixture::new();
    f.text("implementation.rs", "// edit\n");
    f.evidence();
    assert_eq!(f.handoff()["ok"], false);
    let mut task = f.task("complete");
    task["related_gaps"][0]["disposition"] = json!("resolved");
    f.write("harness/tasks/TASK-1.json", task.clone());
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
    task["related_gaps"][0]["disposition"] = json!("unaffected");
    task["discoveries"] = json!([{"summary":"New blocker","record":"gap:missing"}]);
    f.write("harness/tasks/TASK-1.json", task);
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
}
#[test]
fn changed_gap_must_be_accounted_for_and_updated_means_changed() {
    let f = Fixture::new();
    let mut task = f.task("complete");
    task["related_gaps"][0]["disposition"] = json!("updated");
    f.write("harness/tasks/TASK-1.json", task.clone());
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
    let mut gap = Fixture::gap();
    gap["next_action"] = json!("Investigate the reproducer");
    f.write("harness/gaps.json", json!([gap]));
    f.evidence();
    assert_eq!(f.handoff()["ok"], true);
    task["related_gaps"] = json!([]);
    f.write("harness/tasks/TASK-1.json", task);
    f.evidence();
    assert_eq!(f.handoff()["ok"], false);
}
#[test]
fn strict_task_records_reject_unknown_fields_traversal_and_false_doc_updates() {
    let f = Fixture::new();
    let mut task = f.task("complete");
    task["approved"] = json!(true);
    f.write("harness/tasks/TASK-1.json", task);
    assert_eq!(f.run(&["coordination"])["ok"], false);
    let mut task = f.task("complete");
    task["documentation"]["updated"] = json!(["resolution.txt"]);
    f.write("harness/tasks/TASK-1.json", task);
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
    assert_eq!(
        f.run(&[
            "complete",
            "--task",
            "../TASK-1",
            "--base",
            "HEAD",
            "--report",
            "target/report.json"
        ])["ok"],
        false
    );
}

#[test]
fn hook_allows_register_repair_handoff_without_allowing_pr_or_completion() {
    let f = Fixture::new();
    std::fs::remove_file(f.0.join("harness/gaps.json")).unwrap();
    f.write("harness/tasks/TASK-1.json", f.task("blocked"));
    let stop = f.run(&[
        "agent-stop",
        "--task",
        "TASK-1",
        "--base",
        "HEAD",
        "--report",
        "target/missing.json",
    ]);
    assert_eq!(stop["ok"], true);
    assert_eq!(stop["complete"], false);
    assert_eq!(stop["task"]["reconciliation_pending"], true);
    assert_eq!(f.handoff()["ok"], false);
    assert_eq!(f.complete()["ok"], false);
}

#[test]
fn hook_rejects_unverified_completion_and_accepts_explicit_partial_handoff() {
    let f = Fixture::new();
    let args = [
        "agent-stop",
        "--task",
        "TASK-1",
        "--base",
        "HEAD",
        "--report",
        "target/report.json",
    ];
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    assert_eq!(f.run(&args)["ok"], false);
    f.evidence();
    assert_eq!(f.run(&args)["ok"], true);
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    std::fs::remove_file(f.0.join("target/report.json")).unwrap();
    assert_eq!(f.run(&args)["ok"], true);
    assert_eq!(f.run(&args)["complete"], false);
}
