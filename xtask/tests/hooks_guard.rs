#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Default hooks: the edit and shell guard on Claude Code and Codex.
mod support;
use serde_json::{Value, json};
use std::process::Command;
use support::Fixture;
use support::hooks::*;

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
    // `..` resolves before the tier is chosen, so it neither hides nor invents a golden.
    let traversal = edit(&f, "src/../harness/goldens/kept.json");
    assert_eq!(
        decision(&hook(&f, "guard", "claude", &traversal, None)),
        json!("ask")
    );
    let name = f.0.file_name().unwrap().to_str().unwrap().to_string();
    let back_in = shell(&f, &format!("cp x ../{name}/harness/goldens/kept.json"));
    assert_eq!(hook(&f, "guard", "codex", &back_in, None).code, 2);
    let outside = hook(
        &f,
        "guard",
        "codex",
        &shell(&f, "cp x ../outside.txt"),
        None,
    );
    assert_eq!((outside.code, outside.stdout.trim()), (0, "{}"));
}

#[test]
fn guard_passes_unrelated_skills_without_a_receipt_note() {
    let f = Fixture::new();
    for rel in [
        ".claude/skills/investigate/SKILL.md",
        ".claude/skills/new-skill/SKILL.md",
    ] {
        let o = hook(&f, "guard", "claude", &edit(&f, rel), None);
        assert_eq!((o.code, o.stdout.trim()), (0, "{}"), "{rel}");
    }
    for rel in [
        ".claude/skills/author-artifact/SKILL.md",
        ".claude/skills/Write-Ticket/SKILL.md",
        ".claude/settings.json",
    ] {
        let o = hook(&f, "guard", "claude", &edit(&f, rel), None);
        assert_eq!(o.code, 0, "{rel}");
        assert!(
            o.stdout.contains("transition receipt"),
            "{rel}: {}",
            o.stdout
        );
    }
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
fn guard_asks_before_merges_and_refuses_force_pushes() {
    let f = Fixture::new();
    for command in ["gh pr merge 3", "gh api -X PUT repos/o/r/pulls/3/merge"] {
        let claude = hook(&f, "guard", "claude", &shell(&f, command), None);
        assert_eq!(
            decision(&claude),
            json!("ask"),
            "{command}: {}",
            claude.stdout
        );
        assert!(
            claude.stdout.contains("owner's approval"),
            "{}",
            claude.stdout
        );
        let codex = hook(&f, "guard", "codex", &shell(&f, command), None);
        assert_eq!(codex.code, 2, "{command}");
        assert!(codex.stderr.contains("needs the owner"), "{}", codex.stderr);
    }
    for command in [
        "git push --force origin x",
        "git push origin x --force",
        "git push origin +x",
        "git push --force-with-lease",
        "git -C . push --force",
    ] {
        for host in ["claude", "codex"] {
            let o = hook(&f, "guard", host, &shell(&f, command), None);
            assert_eq!(o.code, 2, "{host} {command}");
            assert!(o.stderr.contains("rewrites history"), "{}", o.stderr);
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

#[test]
fn guard_fails_closed_on_an_internal_error() {
    let f = Fixture::new();
    let script = std::fs::read_to_string(HOOK).unwrap();
    // An unset variable aborts a `set -u` script without running its ERR trap.
    let broken = script.replacen("\nmode=", "\necho \"$undefined_variable\"\nmode=", 1);
    assert_ne!(broken, script);
    let path = f.0.join("target/broken-hook.sh");
    std::fs::write(&path, broken).unwrap();
    for host in ["claude", "codex"] {
        let o = Command::new("bash")
            .args([path.to_str().unwrap(), "guard", host])
            .current_dir(&f.0)
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(2), "{host}");
        assert!(String::from_utf8_lossy(&o.stderr).contains("internal error"));
    }
}

#[test]
fn guard_folds_case_where_git_ignores_case() {
    let f = Fixture::new();
    f.git(&["config", "core.ignorecase", "true"]);
    let o = hook(
        &f,
        "guard",
        "codex",
        &edit(&f, "HARNESS/goldens/kept.json"),
        None,
    );
    assert_eq!(o.code, 2, "{}", o.stdout);
    let o = hook(
        &f,
        "guard",
        "claude",
        &edit(&f, "Harness/Config.json"),
        None,
    );
    assert_eq!(decision(&o), json!("ask"));
    let o = hook(&f, "guard", "claude", &edit(&f, "XTASK/src/main.rs"), None);
    assert!(o.stdout.contains("additionalContext"), "{}", o.stdout);
    // Where Git compares case, or the key is unset, other spellings are other files.
    for setting in [Some("false"), None] {
        match setting {
            Some(v) => f.git(&["config", "core.ignorecase", v]),
            None => f.git(&["config", "--unset", "core.ignorecase"]),
        };
        let o = hook(
            &f,
            "guard",
            "codex",
            &edit(&f, "HARNESS/goldens/kept.json"),
            None,
        );
        assert_eq!(
            (o.code, o.stdout.trim()),
            (0, "{}"),
            "{setting:?}: {}",
            o.stderr
        );
    }
}
