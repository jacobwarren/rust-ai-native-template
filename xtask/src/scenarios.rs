use crate::config::{Result, Step, valid_id};
use crate::runner::{self, Status};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scenario {
    id: String,
    args: Vec<String>,
    exit_code: i32,
    stdout_json: Value,
    stderr_event: String,
}

pub fn run(root: &Path) -> Result<Value> {
    let cases: Vec<Scenario> = crate::config::read(&root.join("harness/scenarios.json"))?;
    if cases.is_empty() {
        return Err("scenario suite must not be empty".into());
    }
    let mut ids = BTreeSet::new();
    for case in &cases {
        if !valid_id(&case.id) || case.id == "build" || !ids.insert(&case.id) {
            return Err(format!("invalid/duplicate/reserved scenario id {}", case.id).into());
        }
    }
    let dir = runner::run_dir(root)?;
    let build = runner::execute(
        root,
        &dir,
        &Step::new(
            "build".into(),
            ["cargo", "build", "--locked", "-p", "starter-app"]
                .map(String::from)
                .to_vec(),
            300,
        )?,
    )?;
    if build.status != Status::Passed {
        return Ok(json!({"ok":false,"build":build}));
    }
    let mut results = Vec::new();
    for case in cases {
        let mut argv = vec!["@app".to_owned()];
        argv.extend(case.args);
        let outcome = runner::execute(root, &dir, &Step::new(case.id.clone(), argv, 10)?)?;
        let stdout: std::result::Result<Value, _> =
            serde_json::from_slice(&std::fs::read(&outcome.stdout)?);
        let stderr = std::fs::read_to_string(&outcome.stderr)?;
        let events: std::result::Result<Vec<Value>, _> =
            stderr.lines().map(serde_json::from_str).collect();
        let run_id = dir.file_name().ok_or("missing run id")?.to_string_lossy();
        let telemetry_ok = events.as_ref().is_ok_and(|events| {
            !events.is_empty()
                && events
                    .iter()
                    .all(|e| e["run_id"].as_str() == Some(run_id.as_ref()))
                && events
                    .iter()
                    .any(|e| e["event"].as_str() == Some(case.stderr_event.as_str()))
        });
        let ok = outcome.status != Status::Timeout
            && outcome.status != Status::Error
            && outcome.exit_code == Some(case.exit_code)
            && stdout
                .as_ref()
                .is_ok_and(|actual| actual == &case.stdout_json)
            && telemetry_ok;
        results.push(json!({"id":case.id,"ok":ok,"expected_exit":case.exit_code,
            "expected_stdout":case.stdout_json,"actual_stdout":stdout.ok(),
            "telemetry_ok":telemetry_ok,"process":outcome}));
    }
    let report = json!({"schema_version":1,"kind":"scenarios",
        "ok":results.iter().all(|r| r["ok"] == true),"cases":results,"report_path":dir.join("scenarios.json")});
    std::fs::write(
        dir.join("scenarios.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(report)
}
