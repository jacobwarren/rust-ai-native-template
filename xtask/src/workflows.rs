//! Independent process-boundary acceptance. Expectations never come from app self-grading.
use crate::{
    config::{self, Result, Step, valid_id},
    golden, repo,
    runner::{self, Status},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Workflow {
    id: String,
    boundary: String,
    criterion: String,
    sessions: Vec<Session>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Session {
    transport: String,
    requests: Vec<Value>,
    exit_code: i32,
    expect: Vec<Expected>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Expected {
    Exact {
        value: Value,
    },
    Fields {
        pointers: std::collections::BTreeMap<String, Value>,
    },
    Golden {
        path: String,
        pointer: String,
    },
}

pub fn run(root: &Path, selected: Option<&str>) -> Result<Value> {
    let cases: Vec<Workflow> = config::read(&root.join("harness/workflows.json"))?;
    validate(root, &cases)?;
    if selected.is_some_and(|id| !cases.iter().any(|c| c.id == id)) {
        return Err("unknown workflow id".into());
    }
    let before = repo::fingerprint(root)?;
    let dir = runner::run_dir(root)?;
    let path = dir.join("workflows.json");
    let mut report = json!({"schema_version":1,"kind":"workflows","complete":false,"ok":false,
        "source_sha256":before,"report_path":path,"cases":[],"scope":"public process protocol and filesystem; no GUI or remote platform claim"});
    checkpoint(&path, &report)?;
    let result = (|| -> Result<()> {
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
        report["build"] = serde_json::to_value(&build)?;
        if build.status != Status::Passed {
            return Err("application build failed".into());
        }
        let mut results = Vec::new();
        for case in cases
            .iter()
            .filter(|c| selected.is_none_or(|id| id == c.id))
        {
            let state = dir.join("state").join(&case.id);
            let mut sessions = Vec::new();
            for (index, session) in case.sessions.iter().enumerate() {
                let id = format!("{}-{index}", case.id);
                let input = dir.join(format!("{id}.requests.jsonl"));
                let lines = session
                    .requests
                    .iter()
                    .map(serde_json::to_string)
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .join("\n")
                    + "\n";
                std::fs::write(&input, lines)?;
                let argv = if session.transport == "stdin" {
                    vec!["@app".into(), "driver".into()]
                } else {
                    vec![
                        "@app".into(),
                        "script".into(),
                        input.to_string_lossy().into_owned(),
                    ]
                };
                let outcome = runner::execute_with(
                    root,
                    &dir,
                    &Step::new(id.clone(), argv, 15)?,
                    Some(&input),
                    Some(&state),
                )?;
                let text = std::fs::read_to_string(&outcome.stdout)?;
                let actual: std::result::Result<Vec<Value>, _> =
                    text.lines().map(serde_json::from_str).collect();
                let mut checks = Vec::new();
                let mut ok = outcome.status != Status::Timeout
                    && outcome.status != Status::Error
                    && outcome.exit_code == Some(session.exit_code);
                if let Ok(actual) = actual {
                    ok &= actual.len() == session.expect.len();
                    for (n, expectation) in session.expect.iter().enumerate() {
                        let got = actual.get(n).unwrap_or(&Value::Null);
                        let check = match expectation {
                            Expected::Exact { value } => {
                                json!({"ok":got == value,"expected":value,"actual":got})
                            }
                            Expected::Fields { pointers } => {
                                json!({"ok": pointers.iter().all(|(p,v)| got.pointer(p) == Some(v)),"expected_fields":pointers,"actual":got})
                            }
                            Expected::Golden { path, pointer } => {
                                if got["ok"] != true || got.pointer(pointer).is_none() {
                                    json!({"ok":false,"error":"missing successful response/snapshot","actual":got})
                                } else {
                                    golden::compare(
                                        root,
                                        path,
                                        got.pointer(pointer).ok_or("missing snapshot")?,
                                        &dir.join(format!("{id}-{n}.candidate.json")),
                                    )?
                                }
                            }
                        };
                        ok &= check["ok"] == true;
                        checks.push(check);
                    }
                } else {
                    ok = false;
                }
                let events = std::fs::read_to_string(&outcome.stderr)?;
                let events: std::result::Result<Vec<Value>, _> =
                    events.lines().map(serde_json::from_str).collect();
                let run_id = dir.file_name().ok_or("missing run id")?.to_string_lossy();
                let telemetry = events.is_ok_and(|e| {
                    e.len() == session.requests.len()
                        && e.iter().all(|e| {
                            e["event"] == "driver_response" && e["run_id"] == run_id.as_ref()
                        })
                });
                ok &= telemetry;
                sessions.push(json!({"ok":ok,"transport":session.transport,"input":input,"process":outcome,"checks":checks,"telemetry_ok":telemetry}));
            }
            results.push(json!({"id":case.id,"criterion":case.criterion,"boundary":case.boundary,"ok":sessions.iter().all(|s|s["ok"]==true),"sessions":sessions}));
            report["cases"] = json!(results);
            checkpoint(&path, &report)?;
        }
        let after = repo::fingerprint(root)?;
        report["source_sha256_after"] = json!(after);
        report["ok"] = json!(after == before && results.iter().all(|c| c["ok"] == true));
        Ok(())
    })();
    if let Err(error) = result {
        report["error"] = json!(error.to_string());
        report["ok"] = json!(false);
    }
    report["complete"] = json!(true);
    checkpoint(&path, &report)?;
    Ok(report)
}

fn validate(root: &Path, cases: &[Workflow]) -> Result<()> {
    if cases.is_empty() {
        return Err("workflow suite must not be empty".into());
    }
    let mut ids = BTreeSet::new();
    for case in cases {
        if !valid_id(&case.id)
            || !ids.insert(&case.id)
            || case.boundary != "process"
            || case.criterion.trim().is_empty()
            || case.sessions.is_empty()
        {
            return Err(
                "workflow needs unique id, process boundary, criterion and sessions".into(),
            );
        }
        for s in &case.sessions {
            if !["stdin", "script"].contains(&s.transport.as_str())
                || s.requests.is_empty()
                || s.expect.len() != s.requests.len()
            {
                return Err(
                    "workflow sessions need known transport and one expectation per request".into(),
                );
            }
            for e in &s.expect {
                match e {
                    Expected::Fields { pointers }
                        if pointers.is_empty()
                            || !pointers.contains_key("/ok")
                            || pointers.keys().any(|p| !p.starts_with('/')) =>
                    {
                        return Err("field expectations need /ok and JSON pointers".into());
                    }
                    Expected::Golden { path, pointer } => {
                        golden::baseline_path(root, path)?;
                        if !pointer.starts_with('/') {
                            return Err("golden needs JSON pointer".into());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn checkpoint(path: &Path, report: &Value) -> Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(report)?)?;
    std::fs::rename(tmp, path)?;
    Ok(())
}
