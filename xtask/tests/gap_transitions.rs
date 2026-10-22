#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;
use serde_json::json;
use support::Fixture;

#[test]
fn gap_transitions_require_exact_rationale_and_verified_closure() {
    let f = Fixture::new();
    let mut gap = Fixture::gap();
    gap["state"] = json!("closed");
    gap["resolution"] = json!("resolution.txt");
    f.write("harness/gaps.json", json!([gap.clone()]));
    assert_eq!(f.run(&["gaps", "--base", "HEAD"])["ok"], false);
    f.receipt();
    assert_eq!(f.run(&["gaps", "--base", "HEAD"])["ok"], false);
    f.evidence();
    assert_eq!(
        f.run(&["gaps", "--base", "HEAD", "--report", "target/report.json"])["ok"],
        true
    );
    gap["evidence"] = json!("Changed claim");
    f.write("harness/gaps.json", json!([gap]));
    f.evidence();
    assert_eq!(
        f.run(&["gaps", "--base", "HEAD", "--report", "target/report.json"])["ok"],
        false
    );
}
#[test]
fn deletions_removed_scopes_and_requirement_weakening_need_receipts() {
    for kind in ["delete", "scope", "requirement"] {
        let f = Fixture::new();
        let mut gap = Fixture::gap();
        match kind {
            "scope" => gap["blocks"] = json!([]),
            "requirement" => gap["requirements"] = json!([]),
            _ => {}
        }
        f.write(
            "harness/gaps.json",
            if kind == "delete" {
                json!([])
            } else {
                json!([gap])
            },
        );
        assert_eq!(f.run(&["gaps", "--base", "HEAD"])["ok"], false);
        f.receipt();
        assert_eq!(f.run(&["gaps", "--base", "HEAD"])["ok"], true);
    }
}
#[test]
fn manual_closure_and_policy_changes_need_explicit_review_records() {
    let f = Fixture::new();
    let mut gap = Fixture::gap();
    gap["state"] = json!("closed");
    gap["requirements"] = json!([]);
    gap["resolution"] = json!("resolution.txt");
    f.write("harness/gaps.json", json!([gap]));
    let mut policy = f.read("harness/readiness.json");
    policy["requirements"][0]["tests"] = json!(["weaker_test"]);
    f.write("harness/readiness.json", policy);
    f.receipt();
    assert_eq!(f.run(&["gaps", "--base", "HEAD"])["ok"], true);
    let mut receipt = f.read("harness/transitions/task-1.json");
    receipt["changes"][0]["manual_justification"] = json!(null);
    f.write("harness/transitions/task-1.json", receipt);
    assert_eq!(f.run(&["gaps", "--base", "HEAD"])["ok"], false);
}
#[test]
fn receipts_are_append_only_and_comparison_uses_merge_base() {
    let f = Fixture::new();
    f.receipt();
    f.git(&["add", "harness/transitions"]);
    f.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "commit",
        "-qm",
        "receipt",
    ]);
    f.text("harness/transitions/task-1.json", "{}");
    assert_eq!(f.run(&["gaps", "--base", "HEAD"])["ok"], false);
    assert_eq!(
        f.run(&["gaps", "--base", "nonexistent-revision"])["ok"],
        false
    );
}

#[test]
fn divergent_target_uses_common_ancestor_not_latest_target_commit() {
    let f = Fixture::new();
    let ancestor = f.git(&["rev-parse", "HEAD"]);
    f.git(&["checkout", "-qb", "feature"]);
    f.text("implementation.rs", "// feature work\n");
    f.git(&["add", "."]);
    f.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "commit",
        "-qm",
        "feature",
    ]);
    f.git(&["checkout", "-qb", "target-branch", &ancestor]);
    let mut gap = Fixture::gap();
    gap["blocks"] = json!([]);
    f.write("harness/gaps.json", json!([gap]));
    f.git(&["add", "."]);
    f.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "commit",
        "-qm",
        "target changed",
    ]);
    f.git(&["checkout", "feature"]);
    let diff = f.run(&["gaps", "--base", "target-branch"]);
    assert_eq!(diff["base_commit"], ancestor);
    assert_eq!(diff["required_changes"], json!([]));
    assert_eq!(diff["ok"], true);
}
