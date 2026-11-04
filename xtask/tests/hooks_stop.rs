#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Default hooks: the claims-only stop gate, post-edit check and host configs.
mod support;
use serde_json::{Value, json};
use support::Fixture;
use support::hooks::*;

#[test]
fn stop_allows_turns_that_claim_nothing() {
    let f = Fixture::new();
    let mut task = f.task("partial");
    task["review"] = json!({"acceptance":"","verification":"","assumptions":"","coverage":""});
    f.write("harness/tasks/TASK-1.json", task);
    f.text("stray.txt", "scratch\n");
    let o = stop(&f, env!("CARGO_BIN_EXE_xtask"));
    assert_eq!(o.code, 0, "{}", o.stderr);
}

#[test]
fn stop_refuses_an_unverified_claim_and_allows_a_verified_one() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    let refused = stop(&f, env!("CARGO_BIN_EXE_xtask"));
    assert_eq!(refused.code, 2);
    assert!(
        refused.stderr.contains("Completion claim refused"),
        "{}",
        refused.stderr
    );
    assert!(refused.stderr.contains("TASK-1"), "{}", refused.stderr);
    f.evidence();
    let allowed = stop(&f, env!("CARGO_BIN_EXE_xtask"));
    assert_eq!(allowed.code, 0, "{}", allowed.stderr);
    // A compact spelling of the claim is still a claim.
    f.text("harness/tasks/TASK-1.json", &f.task("complete").to_string());
    assert!(
        !std::fs::read_to_string(f.0.join("harness/tasks/TASK-1.json"))
            .unwrap()
            .contains("\"status\": ")
    );
    std::fs::remove_file(f.0.join("target/report.json")).unwrap();
    assert_eq!(stop(&f, env!("CARGO_BIN_EXE_xtask")).code, 2);
}

#[test]
fn stop_blocks_a_claim_when_the_gate_cannot_run() {
    let f = Fixture::new();
    let broken = stub(
        &f,
        "xtask-broken",
        "echo 'error: cannot update the lock file' >&2; exit 101",
    );
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    assert_eq!(stop(&f, &broken).code, 0);
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    let o = stop(&f, &broken);
    assert_eq!(o.code, 2);
    assert!(
        o.stderr.contains("could not run (exit 101)"),
        "{}",
        o.stderr
    );
}

#[test]
fn hook_configs_wire_existing_scripts_and_no_subagent_stop() {
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
    for (config, guard_matcher) in [
        (
            ".claude/settings.json",
            "Edit|Write|MultiEdit|NotebookEdit|Bash",
        ),
        (".codex/hooks.json", "^(apply_patch|Bash)$"),
    ] {
        let v: Value = serde_json::from_slice(&std::fs::read(root.join(config)).unwrap()).unwrap();
        let hooks = v["hooks"].as_object().unwrap();
        let mut events: Vec<_> = hooks.keys().map(String::as_str).collect();
        events.sort_unstable();
        assert_eq!(events, ["PostToolUse", "PreToolUse", "Stop"], "{config}");
        assert_eq!(hooks["PreToolUse"][0]["matcher"], guard_matcher, "{config}");
        for entry in hooks.values().flat_map(|e| e.as_array().unwrap()) {
            for h in entry["hooks"].as_array().unwrap() {
                let command = h["command"].as_str().unwrap();
                let script = command
                    .split("scripts/")
                    .nth(1)
                    .unwrap()
                    .split('"')
                    .next()
                    .unwrap();
                assert!(
                    root.join("scripts").join(script).is_file(),
                    "{config}: {command}"
                );
            }
        }
    }
    let claude: Value =
        serde_json::from_slice(&std::fs::read(root.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert_eq!(
        claude["hooks"]["PreToolUse"][0]["hooks"][0]["onFailure"],
        "block"
    );
    let example: Value = serde_json::from_slice(
        &std::fs::read(root.join("examples/codex-artifact-hooks.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        example["hooks"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ["PreToolUse"]
    );
}

#[test]
fn post_edit_check_reports_errors_from_the_changed_crate() {
    let ok_xtask = "echo '{\"ok\":true,\"errors\":[]}'";
    let (f, path) = check_fixture(
        "case \"$*\" in *'check -q --locked -p starter-app'*) echo 'error[E0425]: cannot find value' >&2; exit 101;; esac",
        ok_xtask,
    );
    let o = hook(
        &f,
        "check",
        "claude",
        &edit(&f, "crates/app/src/main.rs"),
        Some(&path),
    );
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("E0425"), "{}", o.stderr);
    let log = std::fs::read_to_string(f.0.join("target/cargo.log")).unwrap();
    assert!(log.contains("check -q --locked -p starter-app"), "{log}");
    assert!(log.contains("fmt --check -p starter-app"), "{log}");

    let (f, path) = check_fixture(
        "exit 0",
        "echo '{\"ok\":false,\"errors\":[\"SIZE001 crates/app/src/main.rs: 401 lines exceeds 400\"]}'; exit 1",
    );
    let o = hook(
        &f,
        "check",
        "codex",
        &patch(&f, "*** Update File: crates/app/src/main.rs"),
        Some(&path),
    );
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("SIZE001"), "{}", o.stderr);

    let (f, path) = check_fixture("exit 0", ok_xtask);
    let o = hook(
        &f,
        "check",
        "claude",
        &edit(&f, "crates/app/src/main.rs"),
        Some(&path),
    );
    assert_eq!((o.code, o.stdout.trim()), (0, "{}"), "{}", o.stderr);
    let docs = hook(
        &f,
        "check",
        "claude",
        &edit(&f, "docs/HARNESS.md"),
        Some(&path),
    );
    assert_eq!((docs.code, docs.stdout.trim()), (0, "{}"));
}

#[test]
fn post_edit_check_uses_the_built_xtask_when_the_lock_is_stale() {
    let (f, path) = check_fixture(
        "echo 'error: cannot update the lock file because --locked was passed' >&2; exit 101",
        "echo '{\"ok\":false,\"errors\":[\"ARCH004 starter-app -> rand: external dependency not allowed for this layer\"]}'; exit 1",
    );
    let o = hook(
        &f,
        "check",
        "claude",
        &edit(&f, "crates/app/Cargo.toml"),
        Some(&path),
    );
    assert_eq!(o.code, 2);
    assert!(
        o.stderr.contains("ARCH004 starter-app -> rand"),
        "{}",
        o.stderr
    );
    assert!(
        o.stderr.contains("cannot update the lock file"),
        "{}",
        o.stderr
    );
    assert!(o.stderr.contains("older binary"), "{}", o.stderr);
    std::fs::remove_file(f.0.join("target/debug/xtask")).unwrap();
    let o = hook(
        &f,
        "check",
        "claude",
        &edit(&f, "crates/app/Cargo.toml"),
        Some(&path),
    );
    assert!(
        o.stderr.contains("xtask architecture was not checked"),
        "{}",
        o.stderr
    );
}
