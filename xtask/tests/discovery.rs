#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;
use serde_json::{Value, json};
use support::Fixture;
const RECORD: &str = "harness/task-artifacts/TASK-1/record.json";
fn setup() -> Fixture {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    f
}
fn command(f: &Fixture, action: &str, input: Value) -> Value {
    f.write("target/input.json", input);
    f.run(&[
        "task-artifact",
        action,
        "--task",
        "TASK-1",
        "--input",
        "target/input.json",
    ])
}
fn feedback(f: &Fixture, changes: Value) -> Value {
    let state = f.run(&["task-artifact", "render", "--task", "TASK-1"]);
    json!({"schema_version":1,"task":"TASK-1","discovery_sha256":state["discovery_sha256"],"feedback_base_sha256":state["feedback_base_sha256"],"actor":"CLI regression","responses":changes})
}
#[test]
fn five_discovery_artifacts_are_required_and_escape_authored_content() {
    let f = setup();
    let path = f.0.join("harness/task-artifacts/TASK-1");
    for (name, title) in [
        ("blindspots.html", "Blindspot pass"),
        ("interventions.html", "Brainstorm the intervention"),
        ("interview.html", "The interview"),
        ("reference.html", "Point at a reference"),
        ("plan.html", "The tweakable plan"),
    ] {
        let content = std::fs::read_to_string(path.join(name)).unwrap();
        assert!(content.contains(title));
        assert!(content.contains("Export decisions (.json)"));
        std::fs::remove_file(path.join(name)).unwrap();
        assert_eq!(
            f.run(&["task-artifact", "check", "--task", "TASK-1"])["ok"],
            false
        );
        assert_eq!(
            f.run(&["task-artifact", "render", "--task", "TASK-1"])["ok"],
            true
        );
    }
    let mut d = f.discovery();
    d["scope"] = json!("</script><img src=x onerror=alert(1)>");
    assert_eq!(command(&f, "discover", d)["ok"], true);
    let html = std::fs::read_to_string(path.join("plan.html")).unwrap();
    assert!(!html.contains("<img src=x"));
    assert!(html.contains("\\u003c/script\\u003e"));
    let mut record = f.read(RECORD);
    record["discoveries"] = json!([]);
    f.write(RECORD, record);
    f.run(&["task-artifact", "render", "--task", "TASK-1"]);
    assert_eq!(f.handoff()["ok"], false);
}
#[test]
fn feedback_import_rejects_stale_wrong_task_and_invalid_choices_atomically() {
    let f = setup();
    let original = std::fs::read(f.0.join(RECORD)).unwrap();
    let valid = feedback(
        &f,
        json!({"plan:model":"typed","intervention:typed":"selected"}),
    );
    for bad in [
        json!({"plan:model":"invented"}),
        json!({"unknown:key":"x"}),
        json!({"interview:future":" "}),
    ] {
        let mut input = valid.clone();
        input["responses"] = bad;
        assert_eq!(command(&f, "decide", input)["ok"], false);
        assert_eq!(std::fs::read(f.0.join(RECORD)).unwrap(), original);
    }
    let mut wrong = valid.clone();
    wrong["task"] = json!("another-task");
    assert_eq!(command(&f, "decide", wrong)["ok"], false);
    assert_eq!(command(&f, "decide", valid.clone())["ok"], true);
    let updated = std::fs::read(f.0.join(RECORD)).unwrap();
    assert_eq!(command(&f, "decide", valid)["ok"], false);
    assert_eq!(std::fs::read(f.0.join(RECORD)).unwrap(), updated);
    assert_eq!(
        f.read(RECORD)["feedback"][0]["responses"]["plan:model"],
        "typed"
    );
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
    assert_eq!(
        command(&f, "publish", f.read("target/after.json"))["ok"],
        true
    );
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
}
#[test]
fn discovery_revisions_preserve_history_and_do_not_reuse_old_answers() {
    let f = setup();
    let old = feedback(&f, json!({"interview:future":"A mock"}));
    assert_eq!(command(&f, "decide", old.clone())["ok"], true);
    let mut d = f.discovery();
    d["interview"]["questions"][2]["blocking"] = json!(true);
    assert_eq!(command(&f, "discover", d)["ok"], true);
    assert_eq!(f.read(RECORD)["discoveries"].as_array().unwrap().len(), 2);
    assert_eq!(command(&f, "decide", old)["ok"], false);
    command(&f, "publish", f.read("target/after.json"));
    f.evidence();
    assert_eq!(f.complete()["ok"], false); // previous revision's answer does not resolve this one
    let answer = feedback(&f, json!({"interview:future":"A domain explainer"}));
    assert_eq!(command(&f, "decide", answer)["ok"], true);
    command(&f, "publish", f.read("target/after.json"));
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    f.git(&["add", "harness"]);
    f.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "commit",
        "-qm",
        "checkpoint",
    ]);
    let mut record = f.read(RECORD);
    record["discoveries"][0]["scope"] = json!("Changed old discovery");
    f.write(RECORD, record);
    assert_eq!(f.handoff()["ok"], false);
}
#[test]
fn discovery_requires_structured_sections_but_allows_documented_settled_states() {
    let f = setup();
    let mut d = f.discovery();
    d.as_object_mut().unwrap().remove("reference");
    assert_eq!(command(&f, "discover", d)["ok"], false);
    let mut d = f.discovery();
    d["plan"]["choices"][0]["recommended"] = json!("missing");
    assert_eq!(command(&f, "discover", d)["ok"], false);
    let mut d = f.discovery();
    d["interview"]["questions"] = json!([]);
    d["interview"]["no_questions_reason"] =
        json!("All required decisions are specified in the task.");
    d["reference"]["mappings"] = json!([]);
    d["reference"]["missing_reason"] =
        json!("No equivalent implementation was found; use the explicit acceptance contract.");
    d["blindspots"] = json!([]);
    d["no_blindspots_reason"] = json!("Inspected the named scope and found no material unknowns.");
    assert_eq!(command(&f, "discover", d)["ok"], true);
}
