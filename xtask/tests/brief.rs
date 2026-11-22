#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `task-artifact next` and the page commands: one brief, one author pass, no review.
mod support;
use serde_json::{Value, json};
use std::process::Command;
use support::Fixture;

fn raw(f: &Fixture, args: &[&str]) -> (Option<i32>, Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(&f.0)
        .output()
        .unwrap();
    (o.status.code(), serde_json::from_slice(&o.stdout).unwrap())
}

#[test]
fn next_walks_a_standard_task_to_its_brief_and_no_further() {
    let f = Fixture::new();
    let (_, v) = raw(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["next"]["action"], "record", "{v}");
    let mut r = f.standard("partial");
    r["prompt"] = json!({"task":"add a board command","area":"the engine","task_noun":"ignored"});
    f.write("harness/tasks/TASK-1.json", r);
    f.text("docs/plans/TASK-1.md", "# ticket\n");
    let (code, v) = raw(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(
        (code, v["next"]["action"].as_str()),
        (Some(0), Some("author")),
        "{v}"
    );
    assert_eq!(v["next"]["unfilled"], json!([]), "{v}");
    let text = v["next"]["handover"].as_str().unwrap();
    for line in [
        "Write harness/task-artifacts/TASK-1/brief.html.",
        "Prompt: I'm about to add a board command in the engine. Before I write code, give me one page",
        "Exemplar: examples/thariq/08-implementation-plan.html",
        "Task context: harness/tasks/TASK-1.json, docs/plans/TASK-1.md",
        "Rules: Keep the file at or under 20 KB",
        "labelled as the agent's call.",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    assert!(!text.contains('{'), "{text}");
    // Written but not committed: read it and commit it. No review is named anywhere.
    f.brief();
    let (_, v) = raw(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["next"]["action"], "commit", "{v}");
    assert!(!v.to_string().to_lowercase().contains("review "), "{v}");
    f.commit("brief");
    let (_, v) = raw(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["next"]["action"], "complete", "{v}");
    assert!(
        v["next"]["say"]
            .as_str()
            .unwrap()
            .contains("answer the record's four review questions"),
        "{v}"
    );
    // A brief over its size is advice with exit 3, never a block.
    f.text(
        "harness/task-artifacts/TASK-1/brief.html",
        &format!(
            "<!doctype html><head>{}</head><p>{}</p>",
            support::STYLE,
            "word ".repeat(4100)
        ),
    );
    let (code, v) = raw(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(code, Some(3), "{v}");
    let advice = v["advisories"][0].as_str().unwrap();
    assert!(
        advice.starts_with("brief.html is 20") && advice.contains("stays within 20000"),
        "{advice}"
    );
    // Missing prompt values stay visible as braces, and next says which.
    let g = Fixture::new();
    let mut r = g.standard("partial");
    r["prompt"] = json!({"task":"<verb phrase>","area":""});
    g.write("harness/tasks/TASK-1.json", r);
    let (_, v) = raw(&g, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["next"]["unfilled"], json!(["task", "area"]), "{v}");
    assert!(
        v["next"]["handover"]
            .as_str()
            .unwrap()
            .contains("I'm about to {task} in {area}."),
        "{v}"
    );
    assert!(
        v["next"]["say"]
            .as_str()
            .unwrap()
            .starts_with("First fill the handover's braces for task, area"),
        "{v}"
    );
    // A small task is told to build.
    g.write("harness/tasks/TASK-1.json", g.record("partial"));
    let (_, v) = raw(&g, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(
        (v["tier"].as_str(), v["next"]["action"].as_str()),
        (Some("small"), Some("build")),
        "{v}"
    );
}

#[test]
fn the_page_commands_are_style_check_guard_next_and_screenshot() {
    let f = Fixture::new();
    let (_, v) = raw(&f, &["help", "task-artifact"]);
    let listed: Vec<String> = v["commands"]
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.split(' ').nth(1).unwrap().to_string())
        .collect();
    let mut want = vec!["check", "guard", "next", "screenshot", "style"];
    let mut got: Vec<&str> = listed.iter().map(String::as_str).collect();
    got.sort_unstable();
    want.sort_unstable();
    assert_eq!(got, want, "{v}");
    let (code, v) = raw(&f, &["task-artifact", "verdict", "--task", "TASK-1"]);
    assert_eq!(code, Some(1), "{v}");
    assert!(
        v["error"]
            .as_str()
            .unwrap()
            .starts_with("wrong arguments for task-artifact"),
        "{v}"
    );
    assert_eq!(v["usage"].as_array().unwrap().len(), 5, "{v}");
}
