#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! A successful process and self-reported success cannot substitute for expected state.
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn workflow_grader_rejects_wrong_state_and_missing_golden_from_successful_app() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let f = Fixture(
        std::env::temp_dir().join(format!("starter-workflow-{stamp}-{}", std::process::id())),
    );
    fs::create_dir_all(f.0.join("harness/goldens")).unwrap();
    fs::create_dir_all(f.0.join("src")).unwrap();
    fs::write(
        f.0.join("Cargo.toml"),
        "[workspace]\n[package]\nname=\"starter-app\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
    )
    .unwrap();
    fs::write(
        f.0.join("Cargo.lock"),
        "version = 4\n[[package]]\nname = \"starter-app\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(f.0.join("harness/config.json"), "{}").unwrap();
    fs::write(f.0.join("src/main.rs"),r#"
        fn main() {
            println!("{{\"schema_version\":1,\"id\":1,\"ok\":true,\"result\":{{\"snapshot\":{{\"tasks\":[]}}}}}}");
            let id=std::env::var("HARNESS_RUN_ID").unwrap();
            eprintln!("{{\"event\":\"driver_response\",\"run_id\":\"{id}\"}}");
        }
    "#).unwrap();
    fs::write(f.0.join("harness/goldens/correct.json"), r#"{"tasks":[]}"#).unwrap();
    fs::write(
        f.0.join("harness/goldens/wrong.json"),
        r#"{"tasks":["expected"]}"#,
    )
    .unwrap();
    let cases:Vec<_>=["correct","wrong","missing"].into_iter().map(|id|json!({"id":id,"boundary":"process","criterion":"must match independently expected state",
        "sessions":[{"transport":"stdin","requests":[{"schema_version":1,"id":1,"action":{"method":"capture"}}],"exit_code":0,
        "expect":[{"kind":"golden","path":format!("harness/goldens/{id}.json"),"pointer":"/result/snapshot"}]}]})).collect();
    fs::write(
        f.0.join("harness/workflows.json"),
        serde_json::to_vec(&cases).unwrap(),
    )
    .unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(&f.0)
        .args(["workflow"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(!o.status.success(), "{report}");
    assert_eq!(report["complete"], true, "{report}");
    assert_eq!(report["cases"][0]["ok"], true, "{report}");
    assert_eq!(report["cases"][1]["ok"], false);
    assert_eq!(report["cases"][2]["ok"], false);
    assert!(!f.0.join("harness/goldens/missing.json").exists());
    let candidate = report["cases"][2]["sessions"][0]["checks"][0]["candidate"]
        .as_str()
        .unwrap();
    assert!(PathBuf::from(candidate).is_file());
}
