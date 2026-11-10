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
    // The same change as a standard task still needs its pages.
    f.write("harness/tasks/TASK-1.json", f.record("complete"));
    f.commit("standard record");
    f.evidence();
    let v = complete_at(&f, &base);
    assert_eq!(v["ok"], false);
    assert!(errors(&v).contains("missing blindspots.html"), "{v}");
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
        "docs/skills/review-artifact.md",
        ".agents/skills/review-artifact/SKILL.md",
        ".agents/skills/write-ticket/SKILL.md",
        ".agents/skills/author-artifact/SKILL.md",
        ".claude/skills/author-artifact/SKILL.md",
        ".claude/skills/review-artifact/SKILL.md",
        ".claude/skills/write-ticket/SKILL.md",
        ".agents/skills/Review-Artifact/SKILL.md",
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
    f.text(".claude/skills/review-artifact/SKILL.md", "one line\n");
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
    // A page a small task chooses to write must still carry the stylesheet.
    f.text("harness/task-artifacts/TASK-1/plan.html", "<p>no style</p>");
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["coordination"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    assert!(!o.status.success());
    // A standard record without pages fails the same checks.
    std::fs::remove_dir_all(f.0.join("harness/task-artifacts")).unwrap();
    f.write("harness/tasks/TASK-1.json", f.record("complete"));
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["task-artifact", "guard", "--task", "TASK-1"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    assert!(!o.status.success());
}

#[test]
fn tier_is_strict_and_an_old_record_defaults_to_the_full_set() {
    let f = Fixture::new();
    let mut r = f.record("partial");
    r["tier"] = json!("tiny");
    f.write("harness/tasks/TASK-1.json", r);
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["coordination"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        !o.status.success() && out.contains("unknown variant"),
        "{out}"
    );
    // The guard reports a bad tier as itself, never as missing pages.
    for bad in [json!("Small"), Value::Null] {
        let mut r = f.record("partial");
        r["tier"] = bad;
        f.write("harness/tasks/TASK-1.json", r);
        let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["task-artifact", "guard", "--task", "TASK-1"])
            .current_dir(&f.0)
            .output()
            .unwrap();
        let out = String::from_utf8_lossy(&o.stdout);
        assert!(
            !o.status.success() && !out.contains("missing blindspots"),
            "{out}"
        );
    }
    // A schema 6 record with no tier keeps its meaning: the eight-page set.
    f.write("harness/tasks/TASK-1.json", f.record("complete"));
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["task-artifact", "check", "--task", "TASK-1"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&o.stdout).contains("missing blindspots.html"));
}
