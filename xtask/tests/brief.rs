#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! A standard task's brief: one author pass, a review only when its owner wants one.
mod support;
use serde_json::{Value, json};
use std::process::Command;
use support::Fixture;

fn record(f: &Fixture, status: &str, tier: Option<&str>, reader: Option<&str>) -> Value {
    let mut r = f.record(status);
    r["schema_version"] = json!(7);
    if let Some(tier) = tier {
        r["tier"] = json!(tier);
    }
    if let Some(reader) = reader {
        r["reader"] = json!(reader);
    }
    r
}

fn xtask(f: &Fixture, args: &[&str]) -> Value {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(&f.0)
        .output()
        .unwrap();
    serde_json::from_slice(&o.stdout).unwrap()
}

#[test]
fn next_offers_review_of_a_brief_without_requiring_it() {
    let f = Fixture::new();
    let mut r = record(&f, "partial", Some("standard"), None);
    r["prompt"] =
        json!({"task":"add a board command","task_noun":"the board command","area":"the engine"});
    f.write("harness/tasks/TASK-1.json", r);
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    let v = xtask(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["pages"], json!(["brief.html"]), "{v}");
    assert_eq!(v["next"]["action"], "author", "{v}");
    let text = v["next"]["handover"].as_str().unwrap();
    for line in [
        "Write harness/task-artifacts/TASK-1/brief.html.",
        "Prompt: I'm about to add a board command in the engine. Before I write code, give me one page",
        "Exemplar: examples/thariq/08-implementation-plan.html",
        "Pages so far: none",
        "No reader: No person will answer this task's pages.",
        "Length: A standard page is short: at most 20 KB as a file",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    assert!(!text.contains('{'), "{text}");
    f.page("brief.html");
    let v = xtask(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["next"]["action"], "review", "{v}");
    assert_eq!(v["next"]["optional"], true, "{v}");
    let say = v["next"]["say"].as_str().unwrap();
    assert!(
        say.starts_with("Optional: review brief.html when it changes a gate"),
        "{say}"
    );
    assert!(
        say.contains("otherwise commit brief.html and go on"),
        "{say}"
    );
    // The report is no longer a page, and a full-set page is not this task's.
    for page in ["report.html", "quiz.html"] {
        let stray = xtask(
            &f,
            &[
                "task-artifact",
                "verdict",
                "--task",
                "TASK-1",
                "--page",
                page,
                "--verdict",
                "pass",
                "--reviewer",
                "r",
                "--model",
                "m",
                "--findings",
                "0",
                "--notes",
                "n",
            ],
        );
        assert!(
            stray
                .to_string()
                .contains("not a task page this task owes (brief.html)"),
            "{stray}"
        );
    }
    // Committed without a review, the brief is done: the work and the record are next.
    f.commit("brief, unreviewed");
    let v = xtask(&f, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["next"]["action"], "complete", "{v}");
    assert!(
        v["next"]["say"]
            .as_str()
            .unwrap()
            .contains("answer the record's four review questions"),
        "{v}"
    );
    assert_eq!(
        f.run(&["task-artifact", "guard", "--task", "TASK-1"])["ok"],
        true
    );
    // A review that was asked for is still walked: revise, rewrite, review again.
    let g = Fixture::new();
    g.write(
        "harness/tasks/TASK-1.json",
        record(&g, "partial", Some("standard"), None),
    );
    g.run(&["task-artifact", "style", "--task", "TASK-1"]);
    g.page("brief.html");
    let revise = xtask(
        &g,
        &[
            "task-artifact",
            "verdict",
            "--task",
            "TASK-1",
            "--page",
            "brief.html",
            "--verdict",
            "revise",
            "--reviewer",
            "r",
            "--model",
            "m",
            "--findings",
            "1",
            "--notes",
            "n",
        ],
    );
    assert_eq!(revise["ok"], true, "{revise}");
    let v = xtask(&g, &["task-artifact", "next", "--task", "TASK-1"]);
    assert_eq!(v["next"]["action"], "rewrite", "{v}");
}
