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
    std::fs::write(f.0.join(PAGE), first).unwrap();
    assert_eq!(
        verdict(&f, "blindspots.html", "best")["recorded"]["round"],
        1
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
