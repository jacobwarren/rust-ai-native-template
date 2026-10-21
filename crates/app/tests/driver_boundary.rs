#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Exercise the real executable's transport, not just the in-process session.
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

fn run(input: &[u8]) -> (i32, Vec<Value>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_starter-app"))
        .arg("driver")
        .env_remove("APP_DATA_DIR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input);
    }
    let o = child.wait_with_output().unwrap();
    let values = String::from_utf8(o.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    (o.status.code().unwrap(), values)
}

#[test]
fn oversize_transport_frame_closes_without_executing_tail() {
    let mut input = vec![b' '; starter_engine::MAX_BYTES + 1];
    input.extend_from_slice(
        b"\n{\"schema_version\":1,\"id\":1,\"action\":{\"method\":\"inspect\"}}\n",
    );
    let (code, values) = run(&input);
    assert_eq!(code, 2);
    assert_eq!(values.len(), 1);
    assert_eq!(values[0]["error"]["code"], "capacity");
}

#[test]
fn malformed_frame_is_reported_and_next_complete_frame_still_runs() {
    let (code, values) =
        run(b"not json\n{\"schema_version\":1,\"id\":2,\"action\":{\"method\":\"inspect\"}}\n");
    assert_eq!(code, 2);
    assert_eq!(values.len(), 2);
    assert_eq!(values[0]["error"]["code"], "invalid_request");
    assert_eq!(values[1]["id"], 2);
    assert_eq!(values[1]["result"]["tasks"], json!([]));
}

#[test]
fn script_and_stdin_are_identical_through_real_process_entrypoints() {
    let input=b"{\"schema_version\":1,\"id\":1,\"action\":{\"method\":\"execute\",\"command\":{\"command\":\"add\",\"title\":\"Same\"}}}\n{\"schema_version\":1,\"id\":2,\"action\":{\"method\":\"capture\"}}\n";
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let p = std::env::temp_dir().join(format!(
        "starter-script-{stamp}-{}.jsonl",
        std::process::id()
    ));
    fs::write(&p, input).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_starter-app"))
        .arg("script")
        .arg(&p)
        .env_remove("APP_DATA_DIR")
        .output()
        .unwrap();
    fs::remove_file(p).unwrap();
    assert!(result.status.success());
    let actual: Vec<Value> = String::from_utf8(result.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let (code, expected) = run(input);
    assert_eq!(code, 0);
    assert_eq!(actual, expected);
}
