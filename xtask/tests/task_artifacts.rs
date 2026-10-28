#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;
use serde_json::json;
use support::Fixture;
const RECORD: &str = "harness/task-artifacts/TASK-1/record.json";
const HTML: &str = "harness/task-artifacts/TASK-1/index.html";
fn render(f: &Fixture) {
    assert_eq!(
        f.run(&["task-artifact", "render", "--task", "TASK-1"])["ok"],
        true
    );
}
fn check(f: &Fixture) -> serde_json::Value {
    f.run(&["task-artifact", "check", "--task", "TASK-1"])
}

#[test]
fn artifact_lifecycle_preserves_preflight_and_requires_current_html() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    let original = f.read(RECORD);
    assert_eq!(check(&f)["ok"], true);
    let html = std::fs::read_to_string(f.0.join(HTML)).unwrap();
    for (page, expected) in [
        ("unknowns.html", "Recovery boundary"),
        ("implementation.html", "Preserve committed state"),
        ("buy-in.html", "Rejected input leaves state unchanged"),
        ("quiz.html", "Validation precedes mutation"),
    ] {
        assert!(html.contains(&format!("href=\"{page}\"")));
        let path = f.0.join("harness/task-artifacts/TASK-1").join(page);
        let page_html = std::fs::read_to_string(&path).unwrap();
        assert!(page_html.contains(expected));
        assert!(page_html.starts_with("<!doctype html>"));
        std::fs::remove_file(&path).unwrap();
        assert_eq!(check(&f)["ok"], false);
        render(&f);
    }
    assert!(!html.contains("data-answer="));
    let quiz =
        std::fs::read_to_string(f.0.join("harness/task-artifacts/TASK-1/quiz.html")).unwrap();
    assert!(quiz.contains("<article class=question data-answer=\"0\">"));
    assert!(quiz.contains("<legend class=sr-only>"));
    assert!(!quiz.contains("position:sticky"));
    assert_eq!(
        f.run(&[
            "task-artifact",
            "start",
            "--task",
            "TASK-1",
            "--input",
            "target/before.json"
        ])["ok"],
        false
    );
    assert_eq!(
        f.run(&[
            "task-artifact",
            "note",
            "--task",
            "TASK-1",
            "--input",
            "target/note.json"
        ])["ok"],
        true
    );
    assert_eq!(f.read(RECORD)["before"], original["before"]);
    assert_eq!(f.read(RECORD)["notes"].as_array().unwrap().len(), 2);
    assert_eq!(check(&f)["ok"], false); // published brief predates the new note
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    assert_eq!(f.handoff()["ok"], true);
    assert_eq!(
        f.run(&[
            "task-artifact",
            "publish",
            "--task",
            "TASK-1",
            "--input",
            "target/after.json"
        ])["ok"],
        true
    );
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    assert_eq!(check(&f)["ok"], true);
    f.text(HTML, "<html>stale</html>");
    assert_eq!(check(&f)["ok"], false);
    render(&f);
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
    std::fs::remove_file(f.0.join(HTML)).unwrap();
    assert_eq!(f.complete()["ok"], false);
    render(&f);
    f.evidence();
    assert_eq!(f.complete()["ok"], true);
}
#[test]
fn artifact_rejects_invalid_quiz_and_escapes_authored_content() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    let before = std::fs::read(f.0.join(RECORD)).unwrap();
    let mut after = f.read("target/after.json");
    after["quiz"][0]["answer"] = json!(9);
    f.write("target/invalid.json", after.clone());
    assert_eq!(
        f.run(&[
            "task-artifact",
            "publish",
            "--task",
            "TASK-1",
            "--input",
            "target/invalid.json"
        ])["ok"],
        false
    );
    assert_eq!(std::fs::read(f.0.join(RECORD)).unwrap(), before);
    after["quiz"][0]["answer"] = json!(0);
    after["change"] = json!("</script><img src=x onerror=alert(1)>");
    f.write("target/invalid.json", after.clone());
    assert_eq!(
        f.run(&[
            "task-artifact",
            "publish",
            "--task",
            "TASK-1",
            "--input",
            "target/invalid.json"
        ])["ok"],
        true
    );
    let html =
        std::fs::read_to_string(f.0.join("harness/task-artifacts/TASK-1/buy-in.html")).unwrap();
    assert!(html.contains("&lt;/script&gt;&lt;img"));
    assert!(!html.contains("<img src=x"));
    for refs in [
        json!(["javascript:alert(1)"]),
        json!(["../implementation.rs"]),
        json!(["target/after.json"]),
    ] {
        after["references"] = refs;
        f.write("target/invalid.json", after.clone());
        assert_eq!(
            f.run(&[
                "task-artifact",
                "publish",
                "--task",
                "TASK-1",
                "--input",
                "target/invalid.json"
            ])["ok"],
            false
        );
    }
    assert_eq!(
        f.run(&["task-artifact", "render", "--task", "../escape"])["ok"],
        false
    );
}
#[test]
fn artifact_history_cannot_be_rewritten_even_without_task_edits() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    f.git(&["add", "harness"]);
    f.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "commit",
        "-qm",
        "checkpoint",
    ]);
    let mut record = f.read(RECORD);
    record["before"]["unknowns"] = json!("Rewritten hindsight");
    f.write(RECORD, record);
    render(&f);
    assert_eq!(f.handoff()["ok"], false);
}
#[test]
fn legacy_records_cannot_bypass_mandatory_artifacts() {
    let f = Fixture::new();
    let mut task = f.task("partial");
    task["schema_version"] = json!(2);
    f.write("harness/tasks/TASK-1.json", task);
    assert_eq!(f.run(&["coordination"])["ok"], true);
    assert_eq!(f.handoff()["ok"], false);
}
#[test]
fn partial_preflight_is_reviewable_but_not_complete() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    let mut record = f.read(RECORD);
    record["notes"] = json!([]);
    record["publications"] = json!([]);
    f.write(RECORD, record);
    render(&f);
    assert_eq!(f.handoff()["ok"], true);
    f.write("harness/tasks/TASK-1.json", f.task("complete"));
    f.evidence();
    assert_eq!(f.complete()["ok"], false);
}
#[test]
fn concurrent_fixtures_have_independent_directories() {
    let threads: Vec<_> = (0..16)
        .map(|i| {
            std::thread::spawn(move || {
                let f = Fixture::new();
                f.text("marker", &i.to_string());
                (f, i)
            })
        })
        .collect();
    let fixtures: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    let paths: std::collections::BTreeSet<_> = fixtures.iter().map(|(f, _)| f.0.clone()).collect();
    assert_eq!(paths.len(), 16);
    for (f, i) in fixtures {
        assert_eq!(
            std::fs::read_to_string(f.0.join("marker")).unwrap(),
            i.to_string()
        );
    }
}

#[test]
fn artifact_invalid_input_and_lock_leave_record_unchanged() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    let original = std::fs::read(f.0.join(RECORD)).unwrap();
    for quiz in [
        json!([]),
        json!([{"question":"Q", "options":["same","same"], "answer":0,"explanation":"why","references":["implementation.rs"]}]),
    ] {
        let mut after = f.read("target/after.json");
        after["quiz"] = quiz;
        f.write("target/bad.json", after);
        assert_eq!(
            f.run(&[
                "task-artifact",
                "publish",
                "--task",
                "TASK-1",
                "--input",
                "target/bad.json"
            ])["ok"],
            false
        );
        assert_eq!(std::fs::read(f.0.join(RECORD)).unwrap(), original);
    }
    let mut note = f.read("target/note.json");
    note["extra"] = json!(true);
    f.write("target/bad.json", note);
    assert_eq!(
        f.run(&[
            "task-artifact",
            "note",
            "--task",
            "TASK-1",
            "--input",
            "target/bad.json"
        ])["ok"],
        false
    );
    f.text("harness/task-artifacts/TASK-1/.lock", "active writer");
    assert_eq!(
        f.run(&[
            "task-artifact",
            "note",
            "--task",
            "TASK-1",
            "--input",
            "target/note.json"
        ])["ok"],
        false
    );
    assert_eq!(std::fs::read(f.0.join(RECORD)).unwrap(), original);
    assert_eq!(
        std::fs::read_to_string(f.0.join("harness/task-artifacts/TASK-1/.lock")).unwrap(),
        "active writer"
    );
}

#[cfg(unix)]
#[test]
fn artifact_symlinks_and_orphans_are_rejected() {
    let f = Fixture::new();
    f.write("harness/tasks/TASK-1.json", f.task("partial"));
    std::fs::remove_file(f.0.join(HTML)).unwrap();
    std::os::unix::fs::symlink(f.0.join("implementation.rs"), f.0.join(HTML)).unwrap();
    assert_eq!(
        f.run(&["task-artifact", "render", "--task", "TASK-1"])["ok"],
        false
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("implementation.rs")).unwrap(),
        "// baseline\n"
    );
    std::fs::remove_file(f.0.join(HTML)).unwrap();
    render(&f);
    f.text("harness/task-artifacts/orphan/index.html", "orphan");
    assert_eq!(f.handoff()["ok"], false);
}
