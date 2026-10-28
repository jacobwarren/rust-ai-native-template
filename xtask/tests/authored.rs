#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;
use serde_json::{Value, json};
use support::Fixture;
const DIR: &str = "harness/task-artifacts/TASK-1";
fn manifest(f: &Fixture, variant: bool) -> Value {
    assert_eq!(
        f.run(&["task-artifact", "style", "--task", "TASK-1"])["ok"],
        true
    );
    let mut files = vec![];
    for purpose in [
        "blindspots",
        "interventions",
        "interview",
        "reference",
        "plan",
        "implementation",
        "buy-in",
        "quiz",
    ] {
        let name = format!("{purpose}-bespoke.html");
        let content = if variant {
            format!(
                "<!doctype html><html><title>{purpose}</title><body><canvas id=simulation></canvas><script>window.choice={{color:'purple'}};</script></body></html>"
            )
        } else {
            format!(
                "<!doctype html><html><body><svg viewBox='0 0 100 50'><text x='2' y='20'>{purpose}</text></svg><button>Copy custom response</button></body></html>"
            )
        };
        let content = content.replace("<html>", "<html><head><link rel=\"stylesheet\" href=\"foundation.css\" data-artifact-foundation=\"1\"></head>");
        f.text(&format!("{DIR}/{name}"), &content);
        files.push(json!({"purpose":purpose,"path":name,"description":"Task-specific exploratory interface","evidence":["implementation.rs"],"feedback":"Copy the selected values as a prompt or arbitrary JSON.","verification":"Fixture asserts registry behavior; no visual claim."}));
    }
    json!({"summary":"Register original task-authored interfaces", "files":files})
}
fn raw_register(f: &Fixture, m: Value) -> Value {
    f.write("target/manifest.json", m);
    f.run(&[
        "task-artifact",
        "register",
        "--task",
        "TASK-1",
        "--input",
        "target/manifest.json",
    ])
}
fn register(f: &Fixture, mut m: Value) -> Value {
    use sha2::{Digest, Sha256};
    let plan_path = m["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["purpose"] == "plan")
        .unwrap()["path"]
        .as_str()
        .unwrap();
    if let Ok(plan) = std::fs::read(f.0.join(format!("{DIR}/{plan_path}"))) {
        f.write(&format!("{DIR}/plan-evidence.json"), json!({"schema_version":1,"plan_sha256":format!("{:x}",Sha256::digest(plan)),"test_command":"fixture: registry contract only","cases":[{"choice":"A","before":"initial","after":"outcome A","response":"Choose A"},{"choice":"B","before":"initial","after":"outcome B","response":"Choose B"}]}));
    }
    let mut entry = m["files"][0].clone();
    entry["purpose"] = json!("supporting");
    entry["path"] = json!("plan-evidence.json");
    m["files"].as_array_mut().unwrap().push(entry);
    m["plan_evidence"] = json!("plan-evidence.json");
    if !f.0.join(format!("{DIR}/authored.json")).exists() {
        let mut before = m.clone();
        before["files"].as_array_mut().unwrap().retain(|s| {
            !["implementation", "buy-in", "quiz"].contains(&s["purpose"].as_str().unwrap())
        });
        let result = raw_register(f, before);
        if result["ok"] != true {
            return result;
        }
        if !m["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["purpose"] == "implementation")
        {
            return result;
        }
        f.text("implementation.rs", "// implementation checkpoint change\n");
    }
    if m["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["purpose"] == "buy-in")
    {
        let mut during = m.clone();
        during["files"]
            .as_array_mut()
            .unwrap()
            .retain(|s| !["buy-in", "quiz"].contains(&s["purpose"].as_str().unwrap()));
        let result = raw_register(f, during);
        if result["ok"] != true {
            return result;
        }
    }
    raw_register(f, m)
}
fn setup() -> Fixture {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    f
}
#[test]
fn authored_layouts_are_unconstrained_and_templates_cannot_overwrite_them() {
    let f = setup();
    let a = manifest(&f, false);
    assert_eq!(register(&f, a)["ok"], true);
    let original = std::fs::read(f.0.join(format!("{DIR}/blindspots-bespoke.html"))).unwrap();
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    assert_eq!(
        f.run(&["task-artifact", "render", "--task", "TASK-1"])["ok"],
        false
    );
    assert_eq!(
        std::fs::read(f.0.join(format!("{DIR}/blindspots-bespoke.html"))).unwrap(),
        original
    );
    let b = manifest(&f, true);
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
    assert_eq!(register(&f, b)["ok"], true);
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    assert_eq!(
        std::fs::read(f.0.join(format!(
            "{DIR}/authored-revisions/1/blindspots-bespoke.html"
        )))
        .unwrap(),
        original
    );
}
#[test]
fn authored_feedback_accepts_open_payloads_and_rejects_stale_envelopes() {
    let f = setup();
    let m = manifest(&f, true);
    let c = register(&f, m.clone());
    let payload = json!({"drawing":{"points":[[1,2],[8,9]],"color":"purple"},"custom_reason":"Prefer easing B"});
    let response = json!({"schema_version":1,"task":"TASK-1","revision_sha256":c["revision_sha256"],"feedback_base_sha256":c["feedback_base_sha256"],"actor":"reviewer","payload":payload});
    f.write("target/response.json", response);
    let args = [
        "task-artifact",
        "decide",
        "--task",
        "TASK-1",
        "--input",
        "target/response.json",
    ];
    assert_eq!(f.run(&args)["ok"], true);
    let current = std::fs::read(f.0.join(format!("{DIR}/authored.json"))).unwrap();
    assert_eq!(f.run(&args)["ok"], false);
    assert_eq!(
        std::fs::read(f.0.join(format!("{DIR}/authored.json"))).unwrap(),
        current
    );
    let l = f.read(&format!("{DIR}/authored.json"));
    assert_eq!(l["feedback"][0]["payload"], payload);
    f.evidence();
    assert_eq!(f.complete()["ok"], false); // feedback must be reconciled in a new artifact revision
    assert_eq!(register(&f, m)["ok"], true);
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
}
#[test]
fn authored_contract_rejects_missing_purposes_paths_and_history_edits() {
    let f = setup();
    let mut m = manifest(&f, false);
    let original = m.clone();
    m["files"].as_array_mut().unwrap().remove(0);
    assert_eq!(register(&f, m)["ok"], false);
    let mut m = original.clone();
    m["files"][0]["path"] = json!("../../outside.html");
    assert_eq!(register(&f, m)["ok"], false);
    assert_eq!(register(&f, original)["ok"], true);
    f.git(&["add", "harness"]);
    f.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "commit",
        "-qm",
        "registered",
    ]);
    f.text(
        &format!("{DIR}/authored-revisions/1/plan-bespoke.html"),
        "changed snapshot",
    );
    assert_eq!(f.handoff()["ok"], false);
}
#[test]
fn authored_partial_requires_discovery_and_final_requires_lifecycle() {
    let f = setup();
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    let mut m = manifest(&f, false);
    m["files"].as_array_mut().unwrap().truncate(5);
    assert_eq!(register(&f, m)["ok"], true);
    assert_eq!(f.handoff()["ok"], true);
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
}
#[cfg(unix)]
#[test]
fn authored_registration_rejects_symlink_assets_without_writing_ledger() {
    let f = setup();
    let m = manifest(&f, false);
    let p = f.0.join(format!("{DIR}/plan-bespoke.html"));
    std::fs::remove_file(&p).unwrap();
    std::os::unix::fs::symlink(f.0.join("implementation.rs"), p).unwrap();
    assert_eq!(register(&f, m)["ok"], false);
    assert!(!f.0.join(format!("{DIR}/authored.json")).exists());
}

#[test]
fn shared_style_is_required_and_bundled_for_nested_and_supporting_pages() {
    let f = setup();
    let mut m = manifest(&f, false);
    m["files"][0]["path"] = json!("deep/risk.html");
    f.text(&format!("{DIR}/deep/risk.html"), "<!doctype html><html><head><link rel=\"stylesheet\" href=\"../foundation.css\" data-artifact-foundation=\"1\"></head><body><svg></svg></body></html>");
    let mut extra = m["files"][0].clone();
    extra["purpose"] = json!("supporting");
    extra["path"] = json!("index.html");
    m["files"].as_array_mut().unwrap().push(extra);
    f.text(
        &format!("{DIR}/index.html"),
        "<!doctype html><html>Missing style</html>",
    );
    assert_eq!(register(&f, m.clone())["ok"], false);
    f.text(&format!("{DIR}/index.html"), "<!doctype html><html><head><link rel=\"stylesheet\" href=\"foundation.css\" data-artifact-foundation=\"1\"></head><body>Custom navigation</body></html>");
    let css = std::fs::read(f.0.join(format!("{DIR}/foundation.css"))).unwrap();
    std::fs::remove_file(f.0.join(format!("{DIR}/foundation.css"))).unwrap();
    assert_eq!(register(&f, m.clone())["ok"], false);
    f.text(&format!("{DIR}/foundation.css"), "body { color: red; }");
    assert_eq!(register(&f, m.clone())["ok"], false);
    assert_eq!(
        f.run(&["task-artifact", "style", "--task", "TASK-1"])["ok"],
        true
    );
    assert_eq!(register(&f, m)["ok"], true);
    assert_eq!(
        std::fs::read(f.0.join(format!("{DIR}/authored-revisions/1/foundation.css"))).unwrap(),
        css
    );
    assert!(
        f.0.join(format!("{DIR}/authored-revisions/1/deep/../foundation.css"))
            .is_file()
    );
    assert!(
        String::from_utf8(css)
            .unwrap()
            .contains("prefers-reduced-motion")
    );
    f.text(&format!("{DIR}/foundation.css"), "changed after review");
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
}

#[test]
fn shared_style_upgrade_preserves_historical_css_and_custom_html() {
    use sha2::{Digest, Sha256};
    let f = setup();
    let m = manifest(&f, true);
    assert_eq!(register(&f, m.clone())["ok"], true);
    // Model a ledger produced by a prior release with a different foundation.
    // The current verifier must read historical bytes, not compare with today's CSS.
    let old = "/* historical v0 */ body { color: navy; }";
    f.text(&format!("{DIR}/authored-revisions/1/foundation.css"), old);
    f.text(&format!("{DIR}/foundation.css"), old);
    let mut ledger = f.read(&format!("{DIR}/authored.json"));
    for (i, r) in ledger["revisions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        f.text(
            &format!("{DIR}/authored-revisions/{}/foundation.css", i + 1),
            old,
        );
        let css = r["files"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|s| s["file"]["path"] == "foundation.css")
            .unwrap();
        css["sha256"] = json!(format!("{:x}", Sha256::digest(old.as_bytes())));
    }
    f.write(&format!("{DIR}/authored.json"), ledger);
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    let html_path = f.0.join(format!("{DIR}/plan-bespoke.html"));
    let html = std::fs::read(&html_path).unwrap();
    assert_eq!(
        f.run(&["task-artifact", "style", "--task", "TASK-1"])["ok"],
        true
    );
    assert_eq!(std::fs::read(&html_path).unwrap(), html);
    f.evidence();
    assert_eq!(f.complete()["ok"], false); // working CSS requires a new registration
    assert_eq!(register(&f, m)["ok"], true);
    assert_eq!(
        std::fs::read_to_string(f.0.join(format!("{DIR}/authored-revisions/1/foundation.css")))
            .unwrap(),
        old
    );
    assert_ne!(
        std::fs::read_to_string(f.0.join(format!("{DIR}/authored-revisions/5/foundation.css")))
            .unwrap(),
        old
    );
    assert_eq!(
        std::fs::read(f.0.join(format!("{DIR}/authored-revisions/5/plan-bespoke.html"))).unwrap(),
        html
    );
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    f.text(
        &format!("{DIR}/authored-revisions/1/foundation.css"),
        "tampered",
    );
    assert_eq!(f.complete()["ok"], false);
}

#[cfg(unix)]
#[test]
fn shared_style_install_rejects_symlink_and_active_writer() {
    let f = setup();
    let m = manifest(&f, false);
    f.text(&format!("{DIR}/authored.lock"), "another writer");
    assert_eq!(
        f.run(&["task-artifact", "style", "--task", "TASK-1"])["ok"],
        false
    );
    std::fs::remove_file(f.0.join(format!("{DIR}/authored.lock"))).unwrap();
    let css = f.0.join(format!("{DIR}/foundation.css"));
    std::fs::remove_file(&css).unwrap();
    std::os::unix::fs::symlink(f.0.join("implementation.rs"), css).unwrap();
    assert_eq!(
        f.run(&["task-artifact", "style", "--task", "TASK-1"])["ok"],
        false
    );
    assert_eq!(register(&f, m)["ok"], false);
    assert_eq!(
        std::fs::read_to_string(f.0.join("implementation.rs")).unwrap(),
        "// baseline\n"
    );
}
