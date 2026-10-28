#![allow(dead_code)]
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::Command};
pub struct Fixture(pub PathBuf);
impl Fixture {
    pub fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let f = loop {
            let serial = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "starter-completion-{stamp}-{}-{serial}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => break Self(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("cannot allocate test fixture: {e}"),
            }
        };
        fs::create_dir_all(f.0.join("target")).unwrap();
        f.text("Cargo.toml", "[workspace]\n");
        f.text("implementation.rs", "// baseline\n");
        f.text("resolution.txt", "Reviewed operational evidence\n");
        f.write("harness/config.json", json!({"schema_version":1,"max_rust_lines":400,"layers":{"tooling":{"dependencies":[],"external":[]}},
            "profiles":{"ci":[{"id":"tests","argv":["test-tool"],"timeout_seconds":10},{"id":"workflows","argv":["workflow-tool"],"timeout_seconds":10}]}}));
        f.write("harness/readiness.json", json!({"schema_version":1,"requirements":[{"id":"behavior","implementation":["implementation.rs"],"tests":["independent_test"],"workflows":["journey"]}],"scopes":{"template":["behavior"],"product":["behavior"]}}));
        f.write("harness/gaps.json", json!([Self::gap()]));
        f.git(&["init", "-q"]);
        f.git(&["add", "."]);
        f.git(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "baseline",
        ]);
        f
    }
    pub fn gap() -> Value {
        json!({"id":"remaining","rank":0,"state":"open","blocks":["product"],"impact":"missing behavior",
            "evidence":"observed failing case","next_action":"implement it","resolution":null,"requirements":["behavior"]})
    }
    pub fn text(&self, p: &str, s: &str) {
        let path = self.0.join(p);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, s).unwrap();
    }
    pub fn write(&self, p: &str, v: Value) {
        self.text(p, &v.to_string());
    }
    pub fn read(&self, p: &str) -> Value {
        serde_json::from_slice(&fs::read(self.0.join(p)).unwrap()).unwrap()
    }
    pub fn git(&self, args: &[&str]) -> String {
        let o = Command::new("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8(o.stdout).unwrap().trim().into()
    }
    pub fn run(&self, args: &[&str]) -> Value {
        let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(o.status.success(), v["ok"] == true, "{v}");
        v
    }
    pub fn task(&self, status: &str) -> Value {
        let task = json!({"schema_version":4,"id":"TASK-1","work_item":"https://tracker.example/TASK-1","status":status,
            "requirements":["behavior"],"related_gaps":[{"id":"remaining","disposition":"unaffected","rationale":"Different work remains"}],
            "gap_review":"Reviewed relevant blockers", "discoveries":[],"discovery_review":"No material discoveries",
            "review":{"acceptance":"Named behavior is implemented, or explicitly pending in remaining work.","verification":"Reviewed relevant checks and identified no additional pre-existing defect.","assumptions":"No unverified assumption was needed for this change.","coverage":"Reviewed the affected boundary; no additional material coverage gap found."},
            "documentation":{"updated":[],"rationale":"No public contract change"},"summary":"Implemented accepted behavior",
            "remaining":if status == "complete" {vec![]} else {vec!["Finish the failing behavior test"]}});
        if !self
            .0
            .join("harness/task-artifacts/TASK-1/record.json")
            .exists()
        {
            self.write("harness/tasks/TASK-1.json", task.clone());
            self.write("target/before.json", json!({"outcome":"Observed behavior", "unknowns":"Recovery boundary", "assumptions":"Single writer", "alternatives":"Preserve rejected edits", "verification":"Independent state assertion", "references":["implementation.rs"]}));
            assert_eq!(
                self.run(&[
                    "task-artifact",
                    "start",
                    "--task",
                    "TASK-1",
                    "--input",
                    "target/before.json"
                ])["ok"],
                true
            );
            self.write("target/discovery.json", self.discovery());
            assert_eq!(
                self.run(&[
                    "task-artifact",
                    "discover",
                    "--task",
                    "TASK-1",
                    "--input",
                    "target/discovery.json"
                ])["ok"],
                true
            );
            self.write("target/note.json", json!({"observation":"Inspected state", "decision":"Preserve committed state", "effect":"Rejected edits do not mutate", "references":["implementation.rs"]}));
            assert_eq!(
                self.run(&[
                    "task-artifact",
                    "note",
                    "--task",
                    "TASK-1",
                    "--input",
                    "target/note.json"
                ])["ok"],
                true
            );
            self.write("target/after.json", json!({"change":"State preservation", "demonstration":"Rejected input leaves state unchanged", "verification":"Independent behavior test", "risks":"No platform qualification", "references":["implementation.rs"], "quiz":[{"question":"Does rejected input mutate state?", "options":["No","Yes"], "answer":0,"explanation":"Validation precedes mutation", "references":["implementation.rs"]}]}));
            assert_eq!(
                self.run(&[
                    "task-artifact",
                    "publish",
                    "--task",
                    "TASK-1",
                    "--input",
                    "target/after.json"
                ])["ok"],
                true
            );
        }
        task
    }
    pub fn discovery(&self) -> Value {
        fn localize(v: &mut Value) {
            match v {
                Value::Object(m) => {
                    for (k, v) in m {
                        if k == "references" {
                            *v = json!(["implementation.rs"]);
                        } else if k == "source" {
                            *v = json!("implementation.rs");
                        } else {
                            localize(v);
                        }
                    }
                }
                Value::Array(a) => {
                    for v in a {
                        localize(v);
                    }
                }
                _ => (),
            }
        }
        let mut d: Value =
            serde_json::from_str(include_str!("../../../examples/task-discovery.json")).unwrap();
        localize(&mut d);
        d
    }
    pub fn evidence(&self) {
        let source = self.run(&["env"])["source_sha256"].clone();
        self.text("target/tests.log", "test independent_test ... ok\n");
        self.write("target/workflows.log", json!({"kind":"workflows","complete":true,"ok":true,"source_sha256":source,"source_sha256_after":source,"cases":[{"id":"journey","boundary":"process","ok":true}]}));
        self.write("target/report.json", json!({"kind":"verification","profile":"ci","complete":true,"ok":true,"source_changed":false,"source_sha256_before":source,"source_sha256_after":source,"checks":[
            {"id":"tests","argv":["test-tool"],"status":"passed","exit_code":0,"stdout":self.0.join("target/tests.log")},
            {"id":"workflows","argv":["workflow-tool"],"status":"passed","exit_code":0,"stdout":self.0.join("target/workflows.log")}] }));
    }
    pub fn receipt(&self) {
        let diff = self.run(&["gaps", "--base", "HEAD"]);
        let changes: Vec<_> = diff["required_changes"]
            .as_array()
            .expect("diff")
            .iter()
            .map(|c| {
                json!({
            "change":c,"rationale":"Reviewed intentional change","evidence":["resolution.txt"],
            "manual_justification":"Operational closure reviewed against the linked evidence"})
            })
            .collect();
        self.write(
            "harness/transitions/task-1.json",
            json!({"schema_version":1,"base_commit":diff["base_commit"],"changes":changes}),
        );
    }
    pub fn complete(&self) -> Value {
        self.run(&[
            "complete",
            "--task",
            "TASK-1",
            "--base",
            "HEAD",
            "--report",
            "target/report.json",
        ])
    }
    pub fn handoff(&self) -> Value {
        self.run(&[
            "handoff",
            "--base",
            "HEAD",
            "--report",
            "target/report.json",
        ])
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
