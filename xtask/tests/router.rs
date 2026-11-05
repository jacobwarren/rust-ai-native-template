#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! task-artifact next and verdict: the page loop as commands, advising and never gating.
mod support;
use serde_json::Value;
use std::process::Command;
use support::Fixture;
use support::hooks::*;

const PAGE: &str = "harness/task-artifacts/TASK-1/blindspots.html";

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
fn step(f: &Fixture) -> (String, String, u64) {
    let (code, v) = next(f);
    assert_eq!(code, 0, "{v}");
    let n = &v["next"];
    (
        n["action"].as_str().unwrap().into(),
        n["page"].as_str().unwrap_or("").into(),
        n["round"].as_u64().unwrap_or(0),
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
            "2",
            "--notes",
            "findings",
        ],
    )
}
fn started() -> Fixture {
    let f = Fixture::new();
    let (_, v) = next(&f);
    assert_eq!(v["next"]["action"], "record", "{v}");
    f.write("harness/tasks/TASK-1.json", f.record("partial"));
    f.run(&["task-artifact", "style", "--task", "TASK-1"]);
    f
}

#[test]
fn next_walks_author_review_rewrite_and_commit() {
    let f = started();
    assert_eq!(step(&f), ("author".into(), "blindspots.html".into(), 1));
    f.page("blindspots.html");
    assert_eq!(step(&f), ("review".into(), "blindspots.html".into(), 1));
    assert_eq!(verdict(&f, "blindspots.html", "revise")["ok"], true);
    assert_eq!(step(&f), ("rewrite".into(), "blindspots.html".into(), 2));
    let again = verdict(&f, "blindspots.html", "pass");
    assert!(
        again["error"]
            .as_str()
            .unwrap()
            .contains("already reviewed"),
        "{again}"
    );
    f.page("blindspots.html");
    assert_eq!(step(&f), ("review".into(), "blindspots.html".into(), 2));
    let pass = verdict(&f, "blindspots.html", "pass");
    assert_eq!(pass["recorded"]["round"], 2, "{pass}");
    assert_eq!(pass["recorded"]["sha256"].as_str().unwrap().len(), 64);
    assert_eq!(step(&f), ("commit".into(), "blindspots.html".into(), 2));
    f.commit("blindspots");
    assert_eq!(step(&f), ("author".into(), "interventions.html".into(), 1));
    // A hand edit after the pass is advice with exit 3, never a block.
    f.text(
        PAGE,
        &format!(
            "{}<p>edited</p>",
            std::fs::read_to_string(f.0.join(PAGE)).unwrap()
        ),
    );
    let (code, v) = next(&f);
    assert_eq!(code, 3, "{v}");
    assert!(
        v["advisories"][0]
            .as_str()
            .unwrap()
            .contains("edited after its Pass"),
        "{v}"
    );
    assert_eq!(v["next"]["action"], "commit", "{v}");
    // Completion does not read the log.
    let log = f.read("harness/task-artifacts/TASK-1/reviews.json");
    assert_eq!(log["rows"].as_array().unwrap().len(), 2);
}

#[test]
fn two_rewrites_end_in_the_best_version() {
    let f = started();
    f.page("blindspots.html");
    let first = std::fs::read(f.0.join(PAGE)).unwrap();
    for round in 1..=3 {
        assert_eq!(step(&f).0, "review");
        assert_eq!(
            verdict(&f, "blindspots.html", "revise")["recorded"]["round"],
            round
        );
        if round < 3 {
            assert_eq!(
                step(&f),
                ("rewrite".into(), "blindspots.html".into(), round + 1)
            );
            f.page("blindspots.html");
        }
    }
    assert_eq!(step(&f).0, "best");
    f.page("blindspots.html");
    let spent = verdict(&f, "blindspots.html", "pass");
    assert!(
        spent["error"].as_str().unwrap().contains("two rewrites"),
        "{spent}"
    );
    let unreviewed = verdict(&f, "blindspots.html", "best");
    assert!(
        unreviewed["error"].as_str().unwrap().contains("restore"),
        "{unreviewed}"
    );
    // verdict kept every reviewed version under target/.
    let kept =
        f.0.join("target/artifact-reviews/TASK-1/blindspots.round1.html");
    assert_eq!(std::fs::read(&kept).unwrap(), first);
    std::fs::copy(&kept, f.0.join(PAGE)).unwrap();
    assert_eq!(step(&f).0, "best");
    assert_eq!(
        verdict(&f, "blindspots.html", "best")["recorded"]["round"],
        1
    );
    let twice = verdict(&f, "blindspots.html", "best");
    assert!(
        twice["error"]
            .as_str()
            .unwrap()
            .contains("already recorded"),
        "{twice}"
    );
    assert_eq!(step(&f), ("commit".into(), "blindspots.html".into(), 1));
    let bad = verdict(&f, "notes.html", "pass");
    assert!(
        bad["error"].as_str().unwrap().contains("not a task page"),
        "{bad}"
    );
    let word = verdict(&f, "plan.html", "approve");
    assert!(
        word["error"]
            .as_str()
            .unwrap()
            .contains("pass, revise or best"),
        "{word}"
    );
}

#[test]
fn next_reaches_completion_and_reads_tasks_from_before_the_log() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    let (code, v) = next(&f);
    assert_eq!(
        (code, v["next"]["action"].as_str()),
        (0, Some("complete")),
        "{v}"
    );
    let mut small = f.record("partial");
    small["tier"] = "small".into();
    f.write("harness/tasks/TASK-1.json", small);
    assert_eq!(step(&f).0, "build");
}

#[test]
fn the_artifact_author_cannot_write_the_verdict_log() {
    let f = Fixture::new();
    let log = hook(
        &f,
        "author",
        "claude",
        &edit(&f, "harness/task-artifacts/TASK-1/reviews.json"),
        None,
    );
    assert_eq!(log.code, 2);
    assert!(
        log.stderr
            .contains("only cargo xtask task-artifact verdict"),
        "{}",
        log.stderr
    );
    let page = edit(&f, "harness/task-artifacts/TASK-1/plan.html");
    assert_eq!(hook(&f, "author", "claude", &page, None).code, 0);
}

#[test]
fn a_crlf_page_that_git_calls_clean_reads_as_committed() {
    let f = started();
    f.text(".gitattributes", "* text=auto eol=lf\n");
    f.commit("attributes");
    f.text(
        PAGE,
        &format!(
            "<!doctype html>\r\n<head>{}</head>\r\n<p>page</p>\r\n",
            support::STYLE
        ),
    );
    assert_eq!(verdict(&f, "blindspots.html", "pass")["ok"], true);
    f.commit("blindspots");
    assert!(f.git(&["status", "--porcelain"]).is_empty());
    assert_eq!(step(&f), ("author".into(), "interventions.html".into(), 1));
    // Another clone checks the page out with LF; it is the version that passed.
    let lf = std::fs::read_to_string(f.0.join(PAGE))
        .unwrap()
        .replace("\r\n", "\n");
    f.text(PAGE, &lf);
    let (code, v) = next(&f);
    assert_eq!(code, 0, "{v}");
}

#[test]
fn an_edit_after_review_shows_on_any_page() {
    let f = started();
    f.page("interventions.html");
    assert_eq!(verdict(&f, "interventions.html", "pass")["ok"], true);
    f.page("interventions.html");
    // The next step is still the first page, but the later edit is reported.
    let (code, v) = next(&f);
    assert_eq!(
        (code, v["next"]["page"].as_str()),
        (3, Some("blindspots.html")),
        "{v}"
    );
    assert!(
        v["advisories"][0]
            .as_str()
            .unwrap()
            .starts_with("interventions.html was edited"),
        "{v}"
    );
}

#[test]
fn commit_step_names_the_page_whose_row_is_uncommitted() {
    let f = started();
    f.page("blindspots.html");
    assert_eq!(verdict(&f, "blindspots.html", "pass")["ok"], true);
    f.commit("blindspots");
    f.page("interventions.html");
    f.commit("interventions draft");
    assert_eq!(verdict(&f, "interventions.html", "pass")["ok"], true);
    // Both pages are committed; only the log row for interventions is not.
    assert_eq!(step(&f), ("commit".into(), "interventions.html".into(), 1));
    f.commit("interventions verdict");
    assert_eq!(step(&f), ("author".into(), "interview.html".into(), 1));
}

#[test]
fn verdict_records_findings_and_model() {
    let f = started();
    // A log written before the fields existed still parses and is left as written.
    let old = r#"{"schema_version":1,"rows":[{"page":"blindspots.html","round":1,"verdict":"revise","sha256":"x","reviewer":"r","notes":"n"}]}"#;
    f.text("harness/task-artifacts/TASK-1/reviews.json", old);
    f.page("blindspots.html");
    let v = verdict(&f, "blindspots.html", "pass");
    assert_eq!(v["recorded"]["findings"], 2, "{v}");
    assert_eq!(v["recorded"]["model"], "claude-sonnet-5-5", "{v}");
    let log =
        std::fs::read_to_string(f.0.join("harness/task-artifacts/TASK-1/reviews.json")).unwrap();
    assert!(!log.contains("null"), "{log}");
    assert_eq!(log.matches("\"findings\": ").count(), 1, "{log}");
    let args = |findings: &str, model: &str| {
        run_raw(
            &f,
            &[
                "task-artifact",
                "verdict",
                "--task",
                "TASK-1",
                "--page",
                "plan.html",
                "--verdict",
                "pass",
                "--reviewer",
                "r",
                "--model",
                model,
                "--findings",
                findings,
                "--notes",
                "n",
            ],
        )
    };
    f.page("plan.html");
    assert!(
        args("two", "m")["error"]
            .as_str()
            .unwrap()
            .contains("--findings two")
    );
    assert!(
        args("1", "gpt 5")["error"]
            .as_str()
            .unwrap()
            .contains("--model")
    );
    assert_eq!(args("0", "gpt-5.1")["recorded"]["model"], "gpt-5.1");
    // The old six-flag form is gone: a verdict without a count or model is refused.
    let short = run_raw(
        &f,
        &[
            "task-artifact",
            "verdict",
            "--task",
            "TASK-1",
            "--page",
            "plan.html",
            "--verdict",
            "pass",
            "--reviewer",
            "r",
            "--notes",
            "n",
        ],
    );
    assert!(
        short["error"]
            .as_str()
            .unwrap()
            .contains("--model and --findings are required"),
        "{short}"
    );
    let long = "m".repeat(65);
    assert!(
        args("1", &long)["error"]
            .as_str()
            .unwrap()
            .contains("at most 64")
    );
}
