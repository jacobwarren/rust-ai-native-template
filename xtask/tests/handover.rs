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
        text.contains("Answering rule: When no one answers the discovery pages"),
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

fn piped(f: &Fixture, page: &str, verdict: &str, input: &[u8]) -> Value {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
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
            "-",
        ])
        .current_dir(&f.0)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    serde_json::from_slice(&child.wait_with_output().unwrap().stdout).unwrap()
}
fn refused(v: &Value, words: &str) {
    assert!(
        v["error"].as_str().is_some_and(|e| e.contains(words)),
        "{words}: {v}"
    );
}

#[test]
fn verdict_reads_full_notes_from_stdin() {
    let f = started();
    f.pages(&["blindspots.html", "interventions.html"]);
    // 64 KiB exactly is accepted; one byte more is too long, even mid-character.
    let full = "é".repeat(32 * 1024);
    let v = piped(&f, "blindspots.html", "pass", full.as_bytes());
    assert_eq!(
        v["recorded"]["notes"].as_str().unwrap().len(),
        64 * 1024,
        "{}",
        v["error"]
    );
    refused(
        &piped(
            &f,
            "interventions.html",
            "pass",
            format!("a{full}").as_bytes(),
        ),
        "more than 64 KiB",
    );
    refused(
        &piped(&f, "interventions.html", "pass", b""),
        "read nothing",
    );
    refused(
        &piped(&f, "interventions.html", "pass", b" \n\t"),
        "read nothing",
    );
    refused(
        &piped(&f, "interventions.html", "pass", &[0x66, 0xff, 0xfe]),
        "UTF-8",
    );
    // Any other value is the notes as written.
    let literal = run_raw(
        &f,
        &[
            "task-artifact",
            "verdict",
            "--task",
            "TASK-1",
            "--page",
            "interventions.html",
            "--verdict",
            "pass",
            "--reviewer",
            "r",
            "--model",
            "m",
            "--findings",
            "0",
            "--notes",
            "-x",
        ],
    );
    assert_eq!(literal["recorded"]["notes"], "-x", "{literal}");
}

#[test]
fn review_and_rewrite_steps_carry_their_prompt() {
    let f = started();
    with_prompt(&f, values());
    f.page("blindspots.html");
    let (step, text) = handover(&f);
    assert_eq!(step["action"], "review", "{step}");
    assert!(
        step["say"].as_str().unwrap().ends_with("--notes -"),
        "{step}"
    );
    for line in [
        "Review harness/task-artifacts/TASK-1/blindspots.html.",
        "The prompt it answers: I'm about to add a board command in the engine.",
        "Exemplar: examples/thariq/01-blindspot-pass.html",
        "Pages before it: none",
        "FINDINGS: N",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    // Findings with backticks and $ arrive through stdin as written.
    let reply = "REVISE\n1. `page_step` drops $PAGE and \"quotes\".\nFINDINGS: 1\n";
    assert_eq!(
        piped(&f, "blindspots.html", "revise", reply.as_bytes())["ok"],
        true
    );
    let (step, text) = handover(&f);
    assert_eq!(step["action"], "rewrite", "{step}");
    assert!(
        text.contains(
            "Rewrite harness/task-artifacts/TASK-1/blindspots.html in place (rewrite 1 of 2)."
        ),
        "{text}"
    );
    assert!(
        text.contains(&format!(
            "Round 1 findings; address every one:\n{}",
            reply.trim()
        )),
        "{text}"
    );
    assert!(!text.contains("target/"), "{text}");
    f.page("blindspots.html");
    let (step, text) = handover(&f);
    assert_eq!(
        (step["action"].as_str(), step["round"].as_u64()),
        (Some("review"), Some(2))
    );
    assert!(
        text.contains("rewritten; judge the whole page again"),
        "{text}"
    );
}
