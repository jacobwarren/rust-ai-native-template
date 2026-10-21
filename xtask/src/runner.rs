use crate::config::{Result, Step};
use crate::repo;
use serde::Serialize;
use serde_json::{Value, json};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Passed,
    Failed,
    Timeout,
    Error,
}

#[derive(Debug, Serialize)]
pub struct Outcome {
    pub id: String,
    pub argv: Vec<String>,
    pub status: Status,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub stdout: PathBuf,
    pub stderr: PathBuf,
}

pub fn run_dir(root: &Path) -> Result<PathBuf> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = root
        .join("target/harness/runs")
        .join(format!("{time}-{}", std::process::id()));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn execute(root: &Path, dir: &Path, step: &Step) -> Result<Outcome> {
    execute_with(root, dir, step, None, None)
}

/// Workflow sessions may provide file-backed stdin and share an explicit isolated state root.
pub fn execute_with(
    root: &Path,
    dir: &Path,
    step: &Step,
    input: Option<&Path>,
    state: Option<&Path>,
) -> Result<Outcome> {
    if !cfg!(unix) {
        return Err("bounded process groups currently require Linux or macOS".into());
    }
    let executable = match step.program() {
        "@self" => std::env::current_exe()?,
        "@app" => root.join("target/debug/starter-app"),
        name => PathBuf::from(name),
    };
    let stdout = dir.join(format!("{}.stdout.log", step.id()));
    let stderr = dir.join(format!("{}.stderr.log", step.id()));
    let state = state
        .map(Path::to_path_buf)
        .unwrap_or_else(|| dir.join("state").join(step.id()));
    fs::create_dir_all(&state)?;
    let owns_group = std::env::var_os("HARNESS_IN_PROCESS_GROUP").is_none();
    let mut command = Command::new(executable);
    command
        .args(step.args())
        .current_dir(root)
        .stdin(match input {
            Some(path) => Stdio::from(File::open(path)?),
            None => Stdio::null(),
        })
        .stdout(File::create(&stdout)?)
        .stderr(File::create(&stderr)?)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env(
            "HARNESS_RUN_ID",
            dir.file_name().ok_or("missing run directory")?,
        )
        .env("HARNESS_WORKTREE_ID", repo::worktree_id(root))
        .env("APP_DATA_DIR", state)
        .env("HARNESS_IN_PROCESS_GROUP", "1")
        .env("RUSTDOCFLAGS", "-D warnings")
        .env("NO_COLOR", "1");
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        if owns_group {
            command.process_group(0);
        }
    }
    let start = Instant::now();
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            fs::write(&stderr, format!("cannot spawn command: {error}\n"))?;
            return Ok(Outcome {
                id: step.id().to_owned(),
                argv: step.argv(),
                status: Status::Error,
                error: Some(error.to_string()),
                exit_code: None,
                duration_ms: start.elapsed().as_millis(),
                stdout,
                stderr,
            });
        }
    };
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() >= Duration::from_secs(step.timeout_seconds()) {
            timed_out = true;
            if owns_group {
                terminate_group(child.id())?;
            } else {
                child.kill()?;
            }
            break child.wait()?;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    // Finite checks must not leave background processes after their leader exits.
    if owns_group && !timed_out {
        terminate_group(child.id())?;
    }
    Ok(Outcome {
        id: step.id().to_owned(),
        argv: step.argv(),
        status: if timed_out {
            Status::Timeout
        } else if status.success() {
            Status::Passed
        } else {
            Status::Failed
        },
        error: None,
        exit_code: status.code(),
        duration_ms: start.elapsed().as_millis(),
        stdout,
        stderr,
    })
}

fn terminate_group(pid: u32) -> Result<()> {
    // Safe Rust delegates POSIX signal delivery to the platform's utility.
    // A nonzero status is normal when the entire group already exited.
    Command::new("/bin/kill")
        .args(["-s", "KILL", "--", &format!("-{pid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(())
}

fn save_report(dir: &Path, report: &Value) -> Result<()> {
    let tmp = dir.join("report.json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(report)?)?;
    fs::rename(tmp, dir.join("report.json"))?;
    Ok(())
}

pub fn profile(root: &Path, name: &str) -> Result<Value> {
    let config = crate::config::load(root)?;
    let steps = config
        .profiles
        .get(name)
        .ok_or_else(|| format!("unknown profile: {name}"))?;
    let dir = run_dir(root)?;
    let before = repo::fingerprint(root)?;
    let mut report = json!({
        "schema_version":1,"kind":"verification","ok":false,"complete":false,"profile":name,
        "git":repo::git(root),"source_sha256_before":before,"source_sha256_after":null,
        "source_changed":null,"worktree_id":repo::worktree_id(root),
        "report_path":dir.join("report.json"),"checks":[],"errors":[],
        "scope":"Configured checks only; not proof of arbitrary correctness or security"
    });
    save_report(&dir, &report)?;
    let mut outcomes = Vec::new();
    let mut ok = true;
    for step in steps {
        eprintln!("checking {}", step.id());
        let outcome = execute(root, &dir, step).unwrap_or_else(|error| Outcome {
            id: step.id().into(),
            argv: step.argv(),
            status: Status::Error,
            error: Some(error.to_string()),
            exit_code: None,
            duration_ms: 0,
            stdout: dir.join(format!("{}.stdout.log", step.id())),
            stderr: dir.join(format!("{}.stderr.log", step.id())),
        });
        ok &= outcome.status == Status::Passed;
        outcomes.push(outcome);
        report["checks"] = serde_json::to_value(&outcomes)?;
        save_report(&dir, &report)?;
    }
    match repo::fingerprint(root) {
        Ok(after) => {
            ok &= before == after;
            report["source_changed"] = json!(before != after);
            report["source_sha256_after"] = json!(after);
        }
        Err(error) => {
            ok = false;
            report["errors"] = json!([format!("final source inspection failed: {error}")]);
        }
    }
    report["complete"] = json!(true);
    report["ok"] = json!(ok);
    save_report(&dir, &report)?;
    Ok(report)
}
