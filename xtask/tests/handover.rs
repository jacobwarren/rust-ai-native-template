#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! task-artifact next hands each page's author or reviewer its filled-in prompt.
mod support;
use serde_json::{Value, json};
use std::process::Command;
use support::Fixture;
use support::hooks::*;

fn next(f: &Fixture) -> (i32, Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["task-artifact", "next", "--task", "TASK-1"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    (
        o.status.code().unwrap(),
        serde_json::from_slice(&o.stdout).unwrap(),
    )
}
fn verdict(f: &Fixture, page: &str, verdict: &str) -> Value {
    run_raw(
        f,
        &[
            "task-artifact",
            "verdict",
            "--task",
            "TASK-1",
            "--page",
            page,
            "--verdict",
            verdict,
            "--reviewer",
            "artifact-reviewer",
            "--model",
            "claude-sonnet-5-5",
            "--findings",
            "1",
            "--notes",
            "findings",
        ],
    )
}
fn started() -> Fixture {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.record("partial"));
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    f
}

fn with_prompt(f: &Fixture, prompt: Value) {
    let mut record = f.record("partial");
    record["prompt"] = prompt;
    f.write("harness/tasks/TASK-1.json", record);
}
fn handover(f: &Fixture) -> (Value, String) {
    let (code, v) = next(f);
    assert_eq!(code, 0, "{v}");
    let text = v["next"]["handover"].as_str().unwrap_or("").to_string();
    (v["next"].clone(), text)
}
fn values() -> Value {
    json!({"task":"add a board command","task_noun":"the board command",
        "area":"the engine","problem":"boards cannot be renamed","reference":"The archive command"})
}

#[test]
fn next_hands_over_the_filled_in_prompt() {
    let f = started();
    with_prompt(&f, values());
    let (step, text) = handover(&f);
    assert_eq!(step["unfilled"], json!([]), "{step}");
    for line in [
        "Write harness/task-artifacts/TASK-1/blindspots.html.",
        "Prompt: I'm about to add a board command in the engine. Do a blindspot pass",
        "Exemplar: examples/thariq/01-blindspot-pass.html",
        "Task context: harness/tasks/TASK-1.json",
        "Pages so far: none",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    assert!(!text.contains('{'), "{text}");
    f.page("blindspots.html");
    verdict(&f, "blindspots.html", "pass");
    f.commit("blindspots");
    let (_, text) = handover(&f);
    assert!(
        text.contains("Prompt: Here's the problem: boards cannot be renamed."),
        "{text}"
    );
    assert!(text.contains("Pages so far: blindspots.html"), "{text}");
    f.page("interventions.html");
    verdict(&f, "interventions.html", "pass");
    f.commit("interventions");
    // The interview takes the noun phrase and carries the guide's answering rule.
    let (_, text) = handover(&f);
    assert!(
        text.contains("anything still ambiguous in the board command."),
        "{text}"
    );
    assert!(
        text.contains("When no one answers: When no one answers the discovery pages"),
        "{text}"
    );
}

#[test]
fn build_step_hands_over_the_notes_prompt() {
    let f = started();
    with_prompt(&f, values());
    for page in support::DISCOVERY {
        f.page(page);
        verdict(&f, page, "pass");
    }
    f.commit("discovery");
    let (step, text) = handover(&f);
    assert_eq!(step["action"], "build", "{step}");
    for line in [
        "Write harness/task-artifacts/TASK-1/implementation.html.",
        "Prompt: Keep implementation notes as you build the board command.",
        "Exemplar: examples/thariq/09-implementation-notes.html",
        "Pages so far: blindspots.html, interventions.html, interview.html, reference.html, plan.html",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
}

#[test]
fn missing_values_stay_as_braces() {
    let f = started();
    let (step, text) = handover(&f);
    assert_eq!(step["unfilled"], json!(["task", "area"]), "{step}");
    assert!(text.contains("I'm about to {task} in {area}."), "{text}");
    assert!(
        step["say"]
            .as_str()
            .unwrap()
            .contains("First fill its braces for task, area"),
        "{step}"
    );
    // A stub placeholder is no value; a blank one neither.
    with_prompt(&f, json!({"task":"<verb phrase>","area":"the engine"}));
    let (step, text) = handover(&f);
    assert_eq!(step["unfilled"], json!(["task"]), "{step}");
    assert!(
        text.contains("I'm about to {task} in the engine."),
        "{text}"
    );
}
