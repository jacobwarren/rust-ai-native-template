#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use support::Fixture;
const DIR: &str = "harness/task-artifacts/TASK-1";
fn setup() -> Fixture {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    assert_eq!(
        f.run(&["task-artifact", "style", "--task", "TASK-1"])["ok"],
        true
    );
    f
}
fn manifest(f: &Fixture, stage: usize) -> Value {
    let mut files = vec![];
    let purposes = [
        "blindspots",
        "interventions",
        "interview",
        "reference",
        "plan",
        "implementation",
        "buy-in",
        "quiz",
    ];
    for p in purposes.iter().take([5, 6, 8][stage]) {
        let name = format!("{p}-custom.html");
        if !f.0.join(format!("{DIR}/{name}")).exists() {
            f.text(&format!("{DIR}/{name}"),&format!("<!doctype html><html><head><link rel=\"stylesheet\" href=\"foundation.css\" data-artifact-foundation=\"1\"></head><body>{p}</body></html>"));
        }
        files.push(json!({"purpose":p,"path":name,"description":"Fixture contract","evidence":["implementation.rs"],"feedback":"Declared fixture response","verification":"Structural admission only"}));
    }
    let plan = std::fs::read(f.0.join(format!("{DIR}/plan-custom.html"))).unwrap();
    f.write(&format!("{DIR}/plan-evidence.json"),json!({"schema_version":1,"plan_sha256":format!("{:x}",Sha256::digest(plan)),"test_command":"fixture only; no semantic execution claim","cases":[{"choice":"reject","before":"unset","after":"error; undo unchanged","response":"Reject same title"},{"choice":"noop","before":"unset","after":"success; undo unchanged","response":"Accept without mutation"}]}));
    files.push(json!({"purpose":"supporting","path":"plan-evidence.json","description":"Fixture evidence","evidence":["implementation.rs"],"feedback":"Correct fixture","verification":"Structural evidence validation"}));
    json!({"summary":"Checkpoint fixture","files":files,"plan_evidence":"plan-evidence.json"})
}
fn register(f: &Fixture, m: Value) -> Value {
    f.write("target/manifest.json", m);
    let result = f.run(&[
        "task-artifact",
        "register",
        "--task",
        "TASK-1",
        "--input",
        "target/manifest.json",
    ]);
    eprintln!("register result: {result}");
    result
}
fn guard(f: &Fixture, point: &str) -> Value {
    f.run(&[
        "task-artifact",
        "guard",
        "--task",
        "TASK-1",
        "--point",
        point,
    ])
}
#[test]
fn checkpoints_reject_final_only_and_require_separate_current_notes() {
    let f = setup();
    let final_m = manifest(&f, 2);
    assert_eq!(register(&f, final_m.clone())["ok"], false);
    assert!(!f.0.join(format!("{DIR}/authored.json")).exists());
    let pre = manifest(&f, 0);
    assert_eq!(register(&f, pre.clone())["ok"], true);
    assert_eq!(guard(&f, "before-edit")["ok"], true);
    assert_eq!(register(&f, final_m.clone())["ok"], false);
    let during = manifest(&f, 1);
    assert_eq!(register(&f, during.clone())["ok"], false);
    f.text("implementation.rs", "// implemented slice\n");
    assert_eq!(guard(&f, "after-edit")["ok"], false);
    assert_eq!(register(&f, during.clone())["ok"], true);
    assert_eq!(guard(&f, "before-final")["ok"], true);
    assert_eq!(register(&f, pre)["ok"], false);
    f.text("implementation.rs", "// another slice\n");
    assert_eq!(register(&f, during.clone())["ok"], false);
    assert_eq!(register(&f, final_m.clone())["ok"], false);
    let p = f.0.join(format!("{DIR}/implementation-custom.html"));
    let s = std::fs::read_to_string(&p).unwrap();
    std::fs::write(
        p,
        s.replace("</body>", "<p>New slice and validation</p></body>"),
    )
    .unwrap();
    assert_eq!(register(&f, during)["ok"], true);
    assert_eq!(register(&f, final_m)["ok"], true);
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    f.text("implementation.rs", "// stale final\n");
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
}
#[test]
fn plan_evidence_rejects_inert_choices_missing_exports_and_stale_html() {
    let f = setup();
    let m = manifest(&f, 0);
    let p = format!("{DIR}/plan-evidence.json");
    let original = f.read(&p);
    for broken in ["inert", "same", "empty", "stale"] {
        let mut v = original.clone();
        match broken {
            "inert" => v["cases"][0]["after"] = v["cases"][0]["before"].clone(),
            "same" => v["cases"][1]["after"] = v["cases"][0]["after"].clone(),
            "empty" => v["cases"][0]["response"] = json!(""),
            _ => v["plan_sha256"] = json!("0".repeat(64)),
        }
        f.write(&p, v);
        assert_eq!(register(&f, m.clone())["ok"], false, "{broken}");
    }
    f.write(&p, original);
    assert_eq!(register(&f, m)["ok"], true);
    let p =
        f.0.join(format!("{DIR}/authored-revisions/1/plan-evidence.json"));
    std::fs::write(p, "{}").unwrap();
    assert_eq!(guard(&f, "before-edit")["ok"], false);
}
fn hook(f: &Fixture, patch: &str, point: &str) -> Value {
    let mut c = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["artifact-hook", "--task", "TASK-1", "--point", point])
        .current_dir(&f.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin
        .take()
        .unwrap()
        .write_all(
            json!({"tool_name":"apply_patch","tool_input":{"command":patch}})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    serde_json::from_slice(&c.wait_with_output().unwrap().stdout).unwrap()
}
#[test]
fn patch_hook_blocks_source_without_checkpoint_and_allows_artifact_repairs() {
    let f = setup();
    let source = "*** Begin Patch\n*** Update File: implementation.rs\n*** End Patch";
    assert_eq!(hook(&f, source, "before-edit")["ok"], false);
    assert_eq!(
        hook(
            &f,
            "*** Add File: harness/task-artifacts/TASK-1/plan-custom.html",
            "before-edit"
        )["ok"],
        true
    );
    assert_eq!(
        hook(&f, "*** Add File: ../escape.rs", "before-edit")["ok"],
        false
    );
    let pre = manifest(&f, 0);
    assert_eq!(register(&f, pre)["ok"], true);
    assert_eq!(hook(&f, source, "before-edit")["ok"], true);
    f.text("implementation.rs", "// changed\n");
    assert_eq!(hook(&f, source, "after-edit")["ok"], false);
    assert_eq!(
        hook(&f, "*** Update File: docs/plans/new.md", "before-edit")["ok"],
        true
    );
    let during = manifest(&f, 1);
    assert_eq!(register(&f, during)["ok"], true);
    assert_eq!(hook(&f, source, "before-edit")["ok"], true);
    assert_eq!(
        hook(
            &f,
            "*** Move to: implementation.rs\n*** Update File: harness/task-artifacts/TASK-1/plan-custom.html",
            "before-edit"
        )["ok"],
        true
    );
}

#[test]
fn new_task_cannot_bypass_checkpoints_with_legacy_renderer() {
    let f = setup();
    let mut task = f.task("complete");
    task["schema_version"] = json!(5);
    f.write("harness/tasks/TASK-1.json", task);
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
    let pre = manifest(&f, 0);
    assert_eq!(register(&f, pre)["ok"], true);
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
    f.text("implementation.rs", "// current implementation\n");
    let during = manifest(&f, 1);
    assert_eq!(register(&f, during)["ok"], true);
    let final_m = manifest(&f, 2);
    assert_eq!(register(&f, final_m)["ok"], true);
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
}
