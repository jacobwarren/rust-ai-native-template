#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Black-box tests: a green harness must mean its declared checks really passed.
#![cfg(unix)]
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rust-harness-test-{id}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("harness")).expect("fixture dir");
        fs::write(path.join("Cargo.toml"), "[workspace]\n").expect("fixture marker");
        let config = json!({"schema_version":1,"max_rust_lines":400,
            "layers":{"tooling":{"dependencies":[],"external":[]}},
            "profiles":{"probe":[{"id":"probe","argv":["sh","-c","printf ok"],"timeout_seconds":5}]}});
        fs::write(path.join("harness/config.json"), config.to_string()).expect("config");
        Self(path)
    }

    fn command(&self, args: &[&str]) -> (bool, Value) {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            // Test the outer runner's real group behavior, even from inside CI.
            .env_remove("HARNESS_IN_PROCESS_GROUP")
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("run fixture");
        let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!(
                "invalid JSON: {error}; stdout={}; stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.success(), value)
    }

    fn profile(&self, steps: Value) {
        let path = self.0.join("harness/config.json");
        let mut config: Value =
            serde_json::from_slice(&fs::read(&path).expect("config")).expect("json");
        config["profiles"]["probe"] = steps;
        fs::write(path, config.to_string()).expect("write profile");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn successful_profile_records_evidence() {
    let fixture = Fixture::new();
    let (success, report) = fixture.command(&["check", "probe"]);
    assert!(success, "{report}");
    assert_eq!(report["checks"][0]["status"], "passed");
    assert_eq!(
        report["source_sha256_before"],
        report["source_sha256_after"]
    );
    let path = report["report_path"].as_str().expect("path");
    assert!(PathBuf::from(path).is_file());
}

#[test]
fn failure_and_spawn_errors_cannot_turn_green() {
    let fixture = Fixture::new();
    fixture.profile(json!([
        {"id":"bad","argv":["sh","-c","exit 7"],"timeout_seconds":5},
        {"id":"missing","argv":["/definitely-no-such-executable"],"timeout_seconds":5},
        {"id":"last","argv":["sh","-c","exit 0"],"timeout_seconds":5}
    ]));
    let (success, report) = fixture.command(&["check", "probe"]);
    assert!(!success);
    assert_eq!(report["checks"][0]["exit_code"], 7);
    assert_eq!(report["checks"][1]["status"], "error");
    assert_eq!(report["checks"][2]["status"], "passed");
}

#[test]
fn timeout_is_bounded_and_distinct_from_failure() {
    let fixture = Fixture::new();
    fixture
        .profile(json!([{"id":"hung","argv":["sh","-c","sleep 30 & wait"],"timeout_seconds":1}]));
    let start = std::time::Instant::now();
    let (success, report) = fixture.command(&["check", "probe"]);
    assert!(!success);
    assert_eq!(report["checks"][0]["status"], "timeout");
    assert!(start.elapsed().as_secs() < 10);
}

#[test]
fn source_changes_fail_even_when_commands_pass() {
    let fixture = Fixture::new();
    fixture.profile(json!([{"id":"mutate","argv":["sh","-c","printf changed > source.rs"],"timeout_seconds":5}]));
    let (success, report) = fixture.command(&["check", "probe"]);
    assert!(!success);
    assert_eq!(report["checks"][0]["status"], "passed");
    assert_eq!(report["source_changed"], true);
}

#[test]
fn infrastructure_failure_is_recorded_and_other_steps_still_run() {
    let fixture = Fixture::new();
    fixture.profile(json!([
        {"id":"prepare","argv":["sh","-c","mkdir -p target/harness/runs/$HARNESS_RUN_ID/broken.stdout.log"],"timeout_seconds":5},
        {"id":"broken","argv":["sh","-c","exit 0"],"timeout_seconds":5},
        {"id":"last","argv":["sh","-c","exit 0"],"timeout_seconds":5}
    ]));
    let (success, report) = fixture.command(&["check", "probe"]);
    assert!(!success);
    assert_eq!(report["complete"], true);
    assert_eq!(report["checks"][1]["status"], "error");
    assert!(report["checks"][1]["error"].is_string());
    assert_eq!(report["checks"][2]["status"], "passed");
}

#[test]
fn final_source_inspection_failure_still_writes_a_failed_report() {
    let fixture = Fixture::new();
    fixture.profile(
        json!([{"id":"link","argv":["sh","-c","ln -s Cargo.toml link.rs"],"timeout_seconds":5}]),
    );
    let (success, report) = fixture.command(&["check", "probe"]);
    assert!(!success);
    assert_eq!(report["complete"], true);
    assert!(report["source_sha256_after"].is_null());
    assert!(!report["errors"].as_array().expect("errors").is_empty());
    assert!(PathBuf::from(report["report_path"].as_str().expect("path")).is_file());
}

#[test]
fn broken_document_link_is_actionable() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("AGENTS.md"), "[missing](missing.md)\n").expect("doc");
    fs::write(
        fixture.0.join("harness/docs.json"),
        json!([{
            "path":"AGENTS.md","owner":"test","status":"active","verified_by":"test"
        }])
        .to_string(),
    )
    .expect("catalog");
    let (success, report) = fixture.command(&["docs"]);
    assert!(!success);
    assert!(
        report["errors"]
            .as_array()
            .expect("errors")
            .iter()
            .any(|e| e.as_str().is_some_and(|s| s.starts_with("DOC006")))
    );
}

#[test]
fn empty_profile_and_unknown_arguments_fail_closed() {
    let fixture = Fixture::new();
    fixture.profile(json!([]));
    assert!(!fixture.command(&["check", "probe"]).0);
    assert!(!fixture.command(&["architecture", "--typo"]).0);
}

#[test]
fn empty_scenario_suite_is_an_error() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("harness/scenarios.json"), "[]").expect("scenarios");
    let (success, report) = fixture.command(&["scenario"]);
    assert!(!success);
    assert!(report["error"].as_str().expect("message").contains("empty"));
}

#[test]
fn scenario_grading_catches_a_wrong_answer_from_a_successful_process() {
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.0.join("src")).expect("src");
    fs::write(
        fixture.0.join("Cargo.toml"),
        "[workspace]\n[package]\nname = \"starter-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    fs::write(
        fixture.0.join("Cargo.lock"),
        "version = 4\n[[package]]\nname = \"starter-app\"\nversion = \"0.1.0\"\n",
    )
    .expect("lockfile");
    fs::write(
        fixture.0.join("src/main.rs"),
        r#"
        fn main() {
            println!("{{\"message\":\"actual\"}}");
            let id = std::env::var("HARNESS_RUN_ID").expect("run id");
            eprintln!("{{\"event\":\"done\",\"run_id\":\"{id}\"}}");
        }
    "#,
    )
    .expect("source");
    fs::write(fixture.0.join("harness/scenarios.json"), json!([
        {"id":"correct","args":[],"exit_code":0,"stdout_json":{"message":"actual"},"stderr_event":"done"},
        {"id":"wrong","args":[],"exit_code":0,"stdout_json":{"message":"wrong"},"stderr_event":"done"},
        {"id":"wrong-exit","args":[],"exit_code":2,"stdout_json":{"message":"actual"},"stderr_event":"done"}
    ]).to_string()).expect("cases");
    let (success, report) = fixture.command(&["scenario"]);
    assert!(!success, "{report}");
    assert_eq!(report["cases"][0]["ok"], true, "{report}");
    assert_eq!(report["cases"][1]["ok"], false);
    assert_eq!(report["cases"][2]["ok"], false);
}
