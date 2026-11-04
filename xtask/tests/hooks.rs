#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Default agent hooks: what counts as the tree, the edit guard, the post-edit
//! check and the claims-only stop gate. See harness/task-artifacts/default-hooks/.
mod support;
use serde_json::{Value, json};
use std::process::Command;
use support::Fixture;

fn run_raw(f: &Fixture, args: &[&str]) -> Value {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(&f.0)
        .output()
        .unwrap();
    serde_json::from_slice(&o.stdout).unwrap()
}

#[test]
fn host_local_files_do_not_change_the_tree() {
    let f = Fixture::new();
    let before = f.run(&["env"])["source_sha256"].clone();
    f.text(
        ".claude/settings.local.json",
        "{\"permissions\":{\"allow\":[\"Bash(ls:*)\"]}}",
    );
    f.text(".claude/worktrees/w/x.rs", "fn main() {}\n");
    assert_eq!(f.run(&["env"])["source_sha256"], before);
    assert_eq!(f.handoff()["ok"], true);
    assert_eq!(
        f.run(&["gaps", "--base", "HEAD"])["protected_files"],
        json!([])
    );

    // Only the exact root paths are host-local; the same name elsewhere is source.
    f.text("docs/.claude/settings.local.json", "{}");
    assert_ne!(f.run(&["env"])["source_sha256"], before);
    let handoff = run_raw(&f, &["handoff", "--base", "HEAD"]);
    assert_eq!(handoff["ok"], false, "{handoff}");
}

#[test]
fn committed_local_settings_are_still_reported() {
    let f = Fixture::new();
    f.text(".claude/settings.local.json", "{}");
    f.git(&["add", "-f", ".claude/settings.local.json"]);
    f.commit("commit a host-local file by mistake");
    let base = f.git(&["rev-parse", "HEAD"]);
    f.text("implementation.rs", "// changed\n");
    let gaps = run_raw(&f, &["gaps", "--base", &base]);
    assert!(
        gaps["protected_files"]
            .as_array()
            .unwrap()
            .contains(&json!(".claude/settings.local.json")),
        "{gaps}"
    );
}

const HOOK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../scripts/agent-hook.sh");

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn hook(f: &Fixture, mode: &str, host: &str, event: &Value, path: Option<&str>) -> Outcome {
    use std::io::Write;
    let mut cmd = Command::new("bash");
    cmd.args([HOOK, mode, host]).current_dir(&f.0);
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

fn event(f: &Fixture, tool: &str, input: Value) -> Value {
    json!({"session_id":"s","hook_event_name":"PreToolUse","cwd":f.0,"tool_name":tool,"tool_input":input})
}
fn edit(f: &Fixture, rel: &str) -> Value {
    event(
        f,
        "Edit",
        json!({"file_path":f.0.join(rel),"old_string":"a","new_string":"b"}),
    )
}
fn patch(f: &Fixture, body: &str) -> Value {
    event(
        f,
        "apply_patch",
        json!({"command":format!("*** Begin Patch\n{body}\n*** End Patch")}),
    )
}
fn shell(f: &Fixture, command: &str) -> Value {
    event(f, "Bash", json!({"command":command}))
}
fn decision(o: &Outcome) -> Value {
    let v: Value = serde_json::from_str(&o.stdout).unwrap_or(Value::Null);
    v["hookSpecificOutput"]["permissionDecision"].clone()
}

#[test]
fn guard_asks_claude_and_blocks_codex_on_golden_edits() {
    let f = Fixture::new();
    let claude = hook(
        &f,
        "guard",
        "claude",
        &edit(&f, "harness/goldens/kept.json"),
        None,
    );
    assert_eq!(
        (claude.code, decision(&claude)),
        (0, json!("ask")),
        "{}",
        claude.stdout
    );
    assert!(claude.stdout.contains("harness/goldens/kept.json"));
    let codex = hook(
        &f,
        "guard",
        "codex",
        &patch(&f, "*** Update File: harness/goldens/kept.json\n@@\n-a\n+b"),
        None,
    );
    assert_eq!(codex.code, 2);
    assert!(
        codex.stderr.contains("harness/goldens/kept.json"),
        "{}",
        codex.stderr
    );
}

#[test]
fn guard_reads_every_patch_path_and_blocks_what_it_cannot_read() {
    let f = Fixture::new();
    for line in [
        "*** Add File: harness/goldens/new.json\n+{}",
        "*** Delete File: harness/goldens/kept.json",
        "*** Update File: src/a.rs\n*** Move to: harness/goldens/b.json",
    ] {
        assert_eq!(
            hook(&f, "guard", "codex", &patch(&f, line), None).code,
            2,
            "{line}"
        );
    }
    let hunk = hook(
        &f,
        "guard",
        "codex",
        &patch(
            &f,
            "*** Update File: src/a.rs\n@@\n+*** Update File: harness/goldens/x",
        ),
        None,
    );
    assert_eq!((hunk.code, hunk.stdout.trim()), (0, "{}"));
    let garbage = event(&f, "apply_patch", json!({"command":"garbage"}));
    assert_eq!(hook(&f, "guard", "codex", &garbage, None).code, 2);
    let no_path = event(&f, "Edit", json!({"old_string":"a"}));
    assert_eq!(hook(&f, "guard", "claude", &no_path, None).code, 2);
    let traversal = edit(&f, "src/../harness/goldens/kept.json");
    assert_eq!(hook(&f, "guard", "claude", &traversal, None).code, 2);
}

#[test]
fn guard_notes_policy_files_and_passes_product_files() {
    let f = Fixture::new();
    let o = hook(&f, "guard", "claude", &edit(&f, "xtask/src/repo.rs"), None);
    let v: Value = serde_json::from_str(&o.stdout).unwrap();
    assert_eq!(o.code, 0);
    assert!(
        v["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .contains("transition receipt"),
        "{v}"
    );
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], Value::Null);
    let codex = hook(
        &f,
        "guard",
        "codex",
        &patch(&f, "*** Update File: docs/COMPLETION.md"),
        None,
    );
    assert!(
        codex.stdout.contains("additionalContext"),
        "{}",
        codex.stdout
    );
    for rel in ["crates/app/src/main.rs", "crates/app/Cargo.toml"] {
        let o = hook(&f, "guard", "claude", &edit(&f, rel), None);
        assert_eq!((o.code, o.stdout.trim()), (0, "{}"), "{rel}");
    }
    let outside = event(
        &f,
        "Write",
        json!({"file_path":"/tmp/x","content":"\"file_path\": \"harness/goldens/a\""}),
    );
    let o = hook(&f, "guard", "claude", &outside, None);
    assert_eq!((o.code, o.stdout.trim()), (0, "{}"));
}

#[test]
fn guard_asks_before_the_dependency_allowlist_changes() {
    let f = Fixture::new();
    let claude = hook(
        &f,
        "guard",
        "claude",
        &edit(&f, "harness/config.json"),
        None,
    );
    assert_eq!((claude.code, decision(&claude)), (0, json!("ask")));
    let codex = hook(
        &f,
        "guard",
        "codex",
        &patch(&f, "*** Update File: harness/config.json"),
        None,
    );
    assert_eq!(codex.code, 2);
    assert!(codex.stderr.contains("allowlists"), "{}", codex.stderr);
}

#[test]
fn cargo_add_asks_through_rules_not_the_guard() {
    let f = Fixture::new();
    for command in ["cargo add serde", "cargo remove serde"] {
        let o = hook(&f, "guard", "claude", &shell(&f, command), None);
        assert_eq!((o.code, o.stdout.trim()), (0, "{}"), "{command}");
    }
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let claude: Value =
        serde_json::from_slice(&std::fs::read(format!("{root}/.claude/settings.json")).unwrap())
            .unwrap();
    let asks = claude["permissions"]["ask"].as_array().unwrap();
    assert!(
        asks.contains(&json!("Bash(cargo add *)")) && asks.contains(&json!("Bash(cargo remove *)"))
    );
    let rules = std::fs::read_to_string(format!("{root}/.codex/rules/default.rules")).unwrap();
    assert!(
        rules.contains(r#"pattern = ["cargo", ["add", "remove"]],"#)
            && rules.contains(r#"decision = "prompt""#),
        "{rules}"
    );
}

#[test]
fn guard_catches_shell_writes_to_goldens_but_not_mentions() {
    let f = Fixture::new();
    for command in [
        "cp target/workflows/rejected-edit-0-5.candidate.json harness/goldens/kept.json",
        "rm -r harness/goldens",
        "echo {} > harness/goldens/a.json",
        "mv harness/goldens harness/g2",
        "sed -i '' s/a/b/ harness/goldens/kept.json",
        "echo x | tee -a harness/goldens/kept.json",
        "git checkout -- harness/goldens/kept.json",
    ] {
        let claude = hook(&f, "guard", "claude", &shell(&f, command), None);
        assert_eq!(
            decision(&claude),
            json!("ask"),
            "{command}: {}",
            claude.stdout
        );
        assert_eq!(
            hook(&f, "guard", "codex", &shell(&f, command), None).code,
            2,
            "{command}"
        );
    }
    for command in [
        "cat harness/goldens/kept.json",
        "git diff harness/goldens",
        "git commit -m \"refresh harness/goldens/kept.json\"",
        "cat > crates/app/tests/fixture.rs <<'EOF'\n> harness/goldens/a.json\nEOF",
        "cp harness/goldens/kept.json /tmp/kept.json",
    ] {
        let o = hook(&f, "guard", "codex", &shell(&f, command), None);
        assert_eq!(
            (o.code, o.stdout.trim()),
            (0, "{}"),
            "{command}: {}",
            o.stderr
        );
    }
}

#[test]
fn guard_refuses_pr_merge_and_force_push_spellings() {
    let f = Fixture::new();
    for command in [
        "git push --force origin x",
        "gh pr merge 3",
        "git push origin x --force",
        "git push origin +x",
        "git push --force-with-lease",
        "git -C . push --force",
        "gh api -X PUT repos/o/r/pulls/3/merge",
    ] {
        for host in ["claude", "codex"] {
            let o = hook(&f, "guard", host, &shell(&f, command), None);
            assert_eq!(o.code, 2, "{host} {command}");
            assert!(o.stderr.contains("need the owner"), "{}", o.stderr);
        }
    }
    for command in [
        "git push origin x",
        "git merge main",
        "git push -u origin x",
    ] {
        assert_eq!(
            hook(&f, "guard", "codex", &shell(&f, command), None).code,
            0,
            "{command}"
        );
    }
}

#[test]
fn guard_never_runs_cargo() {
    let f = Fixture::new();
    let bin = f.0.join("target/stub-bin");
    std::fs::create_dir_all(&bin).unwrap();
    let marker = f.0.join("target/cargo-ran");
    let stub = bin.join("cargo");
    std::fs::write(&stub, format!("#!/bin/sh\ntouch '{}'\n", marker.display())).unwrap();
    Command::new("chmod").arg("+x").arg(&stub).status().unwrap();
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
    for e in [
        edit(&f, "harness/goldens/kept.json"),
        edit(&f, "xtask/src/repo.rs"),
        edit(&f, "crates/app/src/main.rs"),
        patch(&f, "*** Update File: harness/config.json"),
        shell(&f, "cargo add serde"),
        shell(&f, "git push --force"),
    ] {
        for host in ["claude", "codex"] {
            hook(&f, "guard", host, &e, Some(&path));
        }
    }
    assert!(!marker.exists(), "the guard ran cargo");
}
