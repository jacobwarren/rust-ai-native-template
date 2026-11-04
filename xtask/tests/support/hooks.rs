//! Drive the default hook scripts with hand-written events.
use super::Fixture;
use serde_json::{Value, json};
use std::process::Command;

pub fn run_raw(f: &Fixture, args: &[&str]) -> Value {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(&f.0)
        .output()
        .unwrap();
    serde_json::from_slice(&o.stdout).unwrap()
}

pub const HOOK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../scripts/agent-hook.sh");

pub struct Outcome {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn hook(f: &Fixture, mode: &str, host: &str, event: &Value, path: Option<&str>) -> Outcome {
    use std::io::Write;
    let mut cmd = Command::new("bash");
    cmd.args([HOOK, mode, host]).current_dir(&f.0);
    // The check finds xtask under CARGO_TARGET_DIR, which the harness sets per run.
    cmd.env_remove("CARGO_TARGET_DIR");
    if let Some(path) = path {
        cmd.env("PATH", path);
    }
    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(event.to_string().as_bytes())
        .unwrap();
    let o = child.wait_with_output().unwrap();
    Outcome {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

pub fn event(f: &Fixture, tool: &str, input: Value) -> Value {
    json!({"session_id":"s","hook_event_name":"PreToolUse","cwd":f.0,"tool_name":tool,"tool_input":input})
}

pub fn edit(f: &Fixture, rel: &str) -> Value {
    event(
        f,
        "Edit",
        json!({"file_path":f.0.join(rel),"old_string":"a","new_string":"b"}),
    )
}

pub fn patch(f: &Fixture, body: &str) -> Value {
    event(
        f,
        "apply_patch",
        json!({"command":format!("*** Begin Patch\n{body}\n*** End Patch")}),
    )
}

pub fn shell(f: &Fixture, command: &str) -> Value {
    event(f, "Bash", json!({"command":command}))
}

pub fn decision(o: &Outcome) -> Value {
    let v: Value = serde_json::from_str(&o.stdout).unwrap_or(Value::Null);
    v["hookSpecificOutput"]["permissionDecision"].clone()
}

pub const STOP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../scripts/agent-complete.sh");

pub fn stop(f: &Fixture, xtask: &str) -> Outcome {
    let o = Command::new("bash")
        .arg(STOP)
        .current_dir(&f.0)
        .env("HARNESS_BASE", "HEAD")
        .env("HARNESS_REPORT", "target/report.json")
        .env("HARNESS_XTASK", xtask)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    Outcome {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

pub fn stub(f: &Fixture, name: &str, body: &str) -> String {
    let path = f.0.join("target/stub-bin").join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    Command::new("chmod").arg("+x").arg(&path).status().unwrap();
    path.display().to_string()
}

pub fn check_fixture(cargo: &str, xtask_body: &str) -> (Fixture, String) {
    let f = Fixture::new();
    f.text(
        "crates/app/Cargo.toml",
        "[package]\nname = \"starter-app\"\nversion = \"0.1.0\"\n",
    );
    f.text("crates/app/src/main.rs", "fn main() {}\n");
    let log = f.0.join("target/cargo.log");
    stub(
        &f,
        "cargo",
        &format!("echo \"$@\" >> '{}'\n{cargo}", log.display()),
    );
    let xtask = f.0.join("target/debug/xtask");
    std::fs::create_dir_all(xtask.parent().unwrap()).unwrap();
    std::fs::write(&xtask, format!("#!/bin/sh\n{xtask_body}\n")).unwrap();
    Command::new("chmod")
        .arg("+x")
        .arg(&xtask)
        .status()
        .unwrap();
    let path = format!(
        "{}:{}",
        f.0.join("target/stub-bin").display(),
        std::env::var("PATH").unwrap()
    );
    (f, path)
}
