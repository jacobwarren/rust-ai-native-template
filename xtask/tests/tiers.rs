#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! A small task needs no pages; the gate reads its branch so it stays off policy paths.
mod support;
use serde_json::{Value, json};
use std::process::Command;
use support::Fixture;

fn small(f: &Fixture, status: &str) -> Value {
    let mut r = f.record(status);
    r["tier"] = json!("small");
    r
}

fn complete_at(f: &Fixture, base: &str) -> Value {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "complete",
            "--task",
            "TASK-1",
            "--base",
            base,
            "--report",
            "target/report.json",
        ])
        .current_dir(&f.0)
        .output()
        .unwrap();
    serde_json::from_slice(&o.stdout).unwrap()
}

fn handoff_at(f: &Fixture, base: &str) -> Value {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["handoff", "--base", base])
        .current_dir(&f.0)
        .output()
        .unwrap();
    serde_json::from_slice(&o.stdout).unwrap()
}

fn errors(v: &Value) -> String {
    v.to_string()
}

#[test]
fn small_task_completes_with_no_pages() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.text("implementation.rs", "// fixed\nfn fixed() {}\n");
    f.write("harness/tasks/TASK-1.json", small(&f, "complete"));
    f.commit("small fix");
    f.evidence();
    let v = complete_at(&f, &base);
    assert_eq!(v["ok"], true, "{v}");
    assert!(!f.0.join("harness/task-artifacts/TASK-1").exists());
    // The same change as a standard task needs its brief.
    f.write("harness/tasks/TASK-1.json", f.standard("complete"));
    f.commit("standard record");
    f.evidence();
    let v = complete_at(&f, &base);
    assert_eq!(v["ok"], false);
    assert!(errors(&v).contains("missing brief.html"), "{v}");
}

#[test]
fn a_pageless_task_is_refused_on_policy_paths_only() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.write("harness/tasks/TASK-1.json", small(&f, "partial"));
    // Size never refuses: 500 lines, a move and a binary file are all a small task.
    f.text("src/lib.rs", &"// line\n".repeat(500));
    f.commit("500 lines");
    f.git(&["mv", "src/lib.rs", "src/moved.rs"]);
    std::fs::write(f.0.join("src/blob.bin"), [0u8, 1, 2, 0]).unwrap();
    f.commit("a move and a binary");
    let v = handoff_at(&f, &base);
    assert!(!errors(&v).contains("task tier small"), "{v}");
    // A gate's own source does, committed or not yet tracked.
    f.text("xtask/src/new_gate.rs", "// one line\n");
    let v = handoff_at(&f, &base);
    assert_eq!(v["ok"], false);
    let e = errors(&v);
    assert!(
        e.contains("xtask/src/new_gate.rs is a policy, dependency or ceremony path"),
        "{v}"
    );
    assert!(
        e.contains("restart it as a standard task on a new branch with its brief"),
        "{v}"
    );
    // A blocked handoff may stop while the gap register is broken, but not on a policy path.
    let mut blocked = small(&f, "blocked");
    blocked["remaining"] = json!(["Repair the gap register"]);
    f.write("harness/tasks/TASK-1.json", blocked);
    std::fs::remove_file(f.0.join("harness/gaps.json")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "agent-stop",
            "--task",
            "TASK-1",
            "--base",
            &base,
            "--report",
            "target/none.json",
        ])
        .current_dir(&f.0)
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        !o.status.success() && !out.contains("blocked_handoff"),
        "{out}"
    );
}

#[test]
fn protected_cargo_and_ceremony_paths_force_standard() {
    for path in [
        "xtask/src/main.rs",
        "scripts/agent-hook.sh",
        "crates/app/Cargo.toml",
        "docs/ARTIFACTS.md",
        "docs/skills/author-artifact.md",
        ".agents/skills/write-ticket/SKILL.md",
        ".agents/skills/author-artifact/SKILL.md",
        ".claude/skills/author-artifact/SKILL.md",
        ".claude/skills/write-ticket/SKILL.md",
        ".agents/skills/Author-Artifact/SKILL.md",
        ".agents/skills/README.md",
        ".claude/settings.json",
        ".claude/commands/new.md",
        ".claude/agents/artifact-author.md",
        ".codex/agents/artifact-author.toml",
    ] {
        let f = Fixture::new();
        let base = f.git(&["rev-parse", "HEAD"]);
        f.write("harness/tasks/TASK-1.json", small(&f, "partial"));
        f.text(path, "one line\n");
        let v = handoff_at(&f, &base);
        assert!(
            errors(&v).contains(&format!("{path} is a policy, dependency or ceremony path")),
            "{path}: {v}"
        );
    }
    // Product acceptance files change with every feature: a receipt, but no brief.
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.write("harness/tasks/TASK-1.json", small(&f, "partial"));
    for path in [
        "harness/workflows.json",
        "harness/scenarios.json",
        "harness/goldens/kept.json",
        "docs/PRODUCT.md",
    ] {
        f.text(path, "one line\n");
    }
    let v = handoff_at(&f, &base);
    assert!(!errors(&v).contains("task tier small"), "{v}");
    assert!(errors(&v).contains("transition rationale"), "{v}");
    // Administrative paths and an ordinary docs edit stay small.
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.write("harness/tasks/TASK-1.json", small(&f, "partial"));
    f.text("docs/GUIDE.md", "a fix\n");
    let v = handoff_at(&f, &base);
    assert!(!errors(&v).contains("task tier small"), "{v}");
}

#[test]
fn unrelated_skills_stay_small() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    f.write("harness/tasks/TASK-1.json", small(&f, "partial"));
    // A skill outside the ceremony, its seed and a new skill, in both hosts.
    for path in [
        ".claude/skills/investigate/SKILL.md",
        ".agents/skills/investigate/SKILL.md",
        "docs/skills/investigate.md",
        ".claude/skills/new-skill/SKILL.md",
        ".agents/skills/new-skill/SKILL.md",
    ] {
        f.text(path, "one line\n");
    }
    f.commit("skills");
    let v = handoff_at(&f, &base);
    assert!(!errors(&v).contains("task tier small"), "{v}");
    assert!(!errors(&v).contains("transition rationale"), "{v}");
    // The same edit to a ceremony skill needs a receipt as well as the standard tier.
    f.text(".claude/skills/author-artifact/SKILL.md", "one line\n");
    f.commit("ceremony skill");
    let v = handoff_at(&f, &base);
    assert!(errors(&v).contains("task tier small"), "{v}");
    assert!(errors(&v).contains("transition rationale"), "{v}");
}

#[test]
fn every_page_check_reads_the_tier() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", small(&f, "complete"));
    // coordination, task-artifact check and the edit guard need no pages for a small task.
    assert_eq!(f.run(&["coordination"])["ok"], true);
    assert_eq!(
        f.run(&["task-artifact", "check", "--task", "TASK-1"])["ok"],
        true
    );
    assert_eq!(
        f.run(&["task-artifact", "guard", "--task", "TASK-1"])["tier"],
        "small"
    );
    // The opt-in apply_patch hook asks the same guard.
    let mut hook = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["artifact-hook", "--task", "TASK-1"])
        .current_dir(&f.0)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    std::io::Write::write_all(
        hook.stdin.as_mut().unwrap(),
        br#"{"tool_name":"apply_patch","tool_input":{"command":"*** Begin Patch\n*** Update File: implementation.rs\n*** End Patch"}}"#,
    )
    .unwrap();
    assert!(hook.wait().unwrap().success());
    // A brief a small task chooses to write must still carry the stylesheet.
    f.text(
        "harness/task-artifacts/TASK-1/brief.html",
        "<p>no style</p>",
    );
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["coordination"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    assert!(!o.status.success());
    // A standard record without its brief fails the same checks.
    std::fs::remove_dir_all(f.0.join("harness/task-artifacts")).unwrap();
    f.write("harness/tasks/TASK-1.json", f.standard("complete"));
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["task-artifact", "guard", "--task", "TASK-1"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    assert!(!o.status.success());
}

#[test]
fn tier_is_small_or_standard() {
    let f = Fixture::new();
    let coordination = |r: Value| {
        f.write("harness/tasks/TASK-1.json", r);
        let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["coordination"])
            .current_dir(&f.0)
            .output()
            .unwrap();
        (
            o.status.success(),
            String::from_utf8_lossy(&o.stdout).into_owned(),
        )
    };
    // The full tier is gone, by name; so is anything else that is not one of the two.
    for bad in [json!("full"), json!("tiny"), json!("Small"), Value::Null] {
        let mut r = f.record("partial");
        r["tier"] = bad.clone();
        let (ok, out) = coordination(r);
        assert!(!ok && !out.contains("missing brief"), "{bad}: {out}");
        if bad.is_string() {
            assert!(out.contains("expected `small` or `standard`"), "{out}");
        }
    }
    // Records written when a full tier chose its pages by reader still parse.
    for reader in ["none", "human"] {
        let mut r = f.standard("partial");
        r["reader"] = json!(reader);
        r["prompt"] = json!({"task":"t","task_noun":"n","area":"a","problem":"p","reference":"r"});
        assert!(coordination(r).0, "{reader}");
    }
    // No tier means small: nothing is owed.
    assert!(coordination(f.record("complete")).0);
}

#[test]
fn a_record_below_schema_7_cannot_claim() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    let mut old = f.standard("complete");
    old["schema_version"] = json!(6);
    f.write("harness/tasks/TASK-1.json", old);
    // It is history: readable, and owing nothing, whatever tier it names.
    assert_eq!(f.run(&["coordination"])["ok"], true);
    f.text("implementation.rs", "// changed\n");
    f.commit("work under an old record");
    f.evidence();
    let v = complete_at(&f, &base);
    assert_eq!(v["ok"], false);
    assert!(
        errors(&v).contains("Upgrade this active task to schema_version 7"),
        "{v}"
    );
    assert!(!errors(&v).contains("brief.html"), "{v}");
}
