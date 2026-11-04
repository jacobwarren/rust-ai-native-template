#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Admission tests use synthetic trusted reports; this is not a signing/authentication test.
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        // Tests in this binary run in parallel; the clock alone can repeat on macOS.
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let serial = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "starter-readiness-{stamp}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(path.join("harness")).unwrap();
        fs::create_dir_all(path.join("target")).unwrap();
        fs::write(path.join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::write(path.join("implementation.rs"), "// implemented\n").unwrap();
        let f = Self(path);
        f.write("harness/config.json",json!({"schema_version":1,"max_rust_lines":400,"layers":{"tooling":{"dependencies":[],"external":[]}},
            "profiles":{"ci":[{"id":"tests","argv":["test-tool"],"timeout_seconds":10},{"id":"workflows","argv":["workflow-tool"],"timeout_seconds":10}]}}));
        f.write("harness/readiness.json",json!({"schema_version":1,"requirements":[{"id":"behavior","implementation":["implementation.rs"],"tests":["independent_test"],"workflows":["journey"]}],"scopes":{"template":["behavior"],"product":["behavior"]}}));
        f.write("harness/gaps.json",json!([{"id":"real-product","rank":0,"state":"open","blocks":["product"],"impact":"missing product acceptance","evidence":"example only","next_action":"define acceptance","resolution":null}]));
        f
    }
    fn write(&self, p: &str, value: Value) {
        fs::write(self.0.join(p), value.to_string()).unwrap();
    }
    fn run(&self, args: &[&str]) -> Value {
        let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(o.status.success(), v["ok"] == true, "{v}");
        v
    }
    fn evidence(&self) -> Value {
        let source = self.run(&["env"])["source_sha256"].clone();
        fs::write(
            self.0.join("target/tests.log"),
            "test independent_test ... ok\n",
        )
        .unwrap();
        self.write("target/workflows.log",json!({"kind":"workflows","complete":true,"ok":true,"source_sha256":source,"source_sha256_after":source,"cases":[{"id":"journey","boundary":"process","ok":true}]}));
        json!({"kind":"verification","profile":"ci","complete":true,"ok":true,"source_changed":false,"source_sha256_before":source,"source_sha256_after":source,"checks":[
            {"id":"tests","argv":["test-tool"],"status":"passed","exit_code":0,"stdout":self.0.join("target/tests.log")},
            {"id":"workflows","argv":["workflow-tool"],"status":"passed","exit_code":0,"stdout":self.0.join("target/workflows.log")}]})
    }
    fn release(&self, report: Value) -> Value {
        self.write("target/report.json", report);
        self.run(&["release", "template", "--report", "target/report.json"])
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn readiness_rejects_stale_incomplete_and_missing_workflow_evidence() {
    let f = Fixture::new();
    let report = f.evidence();
    assert_eq!(f.release(report.clone())["ok"], true);
    assert_eq!(
        f.run(&["release", "product", "--report", "target/report.json"])["ok"],
        false
    );
    let mut incomplete = report.clone();
    incomplete["complete"] = json!(false);
    assert_eq!(f.release(incomplete)["ok"], false);
    let mut stale = report.clone();
    stale["source_sha256_before"] = json!("old");
    assert_eq!(f.release(stale)["ok"], false);
    let mut missing = report.clone();
    missing["checks"] = json!([]);
    assert_eq!(f.release(missing)["ok"], false);
    let mut workflows: Value =
        serde_json::from_slice(&fs::read(f.0.join("target/workflows.log")).unwrap()).unwrap();
    workflows["cases"] = json!([]);
    f.write("target/workflows.log", workflows);
    assert_eq!(f.release(report)["ok"], false);
    let ready = f.evidence();
    fs::write(f.0.join("implementation.rs"), "// changed after evidence\n").unwrap();
    assert_eq!(f.release(ready)["ok"], false);
}

#[test]
fn no_evidence_never_claims_verified_or_release_ready() {
    let f = Fixture::new();
    let r = f.run(&["readiness"]);
    assert_eq!(r["requirements"][0]["state"], "implemented");
    assert!(
        r["scopes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["state"] == "blocked")
    );
}
