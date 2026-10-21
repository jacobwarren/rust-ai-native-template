mod checks;
mod config;
mod golden;
mod readiness;
mod repo;
mod runner;
mod scenarios;
mod workflows;

use config::Result;
use runner::Status;
use serde_json::{Value, json};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match dispatch() {
        Ok(value) => {
            let ok = value["ok"].as_bool().unwrap_or(true);
            println!("{value:#}");
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            println!("{}", json!({"ok":false,"error":error.to_string()}));
            ExitCode::FAILURE
        }
    }
}

fn dispatch() -> Result<Value> {
    let root = repo::root()?.canonicalize()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["help"] => Ok(json!({"ok":true,"commands":{
            "doctor":"Check local tools and configuration",
            "map":"Workspace package and layer inventory",
            "check fast":"Short edit feedback loop; emits evidence JSON",
            "check ci":"Full configured pre-merge checks; emits evidence JSON",
            "architecture":"Validate all declared workspace dependency edges",
            "docs":"Validate catalog, AGENTS size, and local file links",
            "scenario":"Build and replay application boundary cases",
            "workflow [id]":"Run independent stateful process workflows; capture requests, replies and golden comparisons",
            "readiness [--report path]":"Compute implemented and verified requirements from source-bound evidence",
            "release scope --report path":"Fail unless the named scope is ready; never publishes",
            "gaps":"List ranked evidence-backed gaps",
            "run -- greet Ada":"Run example with isolated state and JSON logs",
            "env":"Show worktree-specific context"}})),
        ["doctor"] => {
            config::load(&root)?;
            let mut versions = serde_json::Map::new();
            let mut ok = cfg!(unix) && std::path::Path::new("/bin/kill").is_file();
            for (id, tool, flags) in [
                ("rustc", "rustc", vec!["--version"]),
                ("cargo", "cargo", vec!["--version"]),
                ("rustfmt", "cargo", vec!["fmt", "--version"]),
                ("clippy", "cargo", vec!["clippy", "--version"]),
            ] {
                match Command::new(tool).args(flags).output() {
                    Ok(output) if output.status.success() => {
                        versions.insert(
                            id.into(),
                            json!(String::from_utf8_lossy(&output.stdout).trim()),
                        );
                    }
                    _ => {
                        ok = false;
                        versions.insert(
                            id.into(),
                            json!("missing: install pinned toolchain and components using rustup"),
                        );
                    }
                }
            }
            Ok(
                json!({"ok":ok,"versions":versions,"process_platform":"Linux/macOS required","git":repo::git(&root)}),
            )
        }
        ["map"] => {
            let metadata = repo::metadata(&root)?;
            let packages: Vec<Value> = metadata["packages"]
                .as_array()
                .ok_or("missing packages")?
                .iter()
                .map(|p| {
                    json!({"name":p["name"],"manifest":p["manifest_path"],
                    "harness":p["metadata"]["harness"],"dependencies":p["dependencies"]})
                })
                .collect();
            Ok(json!({"ok":true,"packages":packages}))
        }
        ["check", profile] => runner::profile(&root, profile),
        ["architecture"] => checks::architecture(&root),
        ["docs"] => checks::docs(&root),
        ["scenario"] => scenarios::run(&root),
        ["workflow"] => workflows::run(&root, None),
        ["workflow", id] => workflows::run(&root, Some(id)),
        ["gaps"] => readiness::list_gaps(&root),
        ["readiness"] => readiness::assess(&root, None, None),
        ["readiness", "--report", path] => {
            readiness::assess(&root, Some(std::path::Path::new(path)), None)
        }
        ["release", scope, "--report", path] => {
            readiness::assess(&root, Some(std::path::Path::new(path)), Some(scope))
        }
        ["env"] => Ok(
            json!({"ok":true,"root":root,"worktree_id":repo::worktree_id(&root),
            "artifacts":root.join("target/harness"),"source_sha256":repo::fingerprint(&root)?}),
        ),
        ["run", "--", tail @ ..] if !tail.is_empty() => {
            let dir = runner::run_dir(&root)?;
            let build = runner::execute(
                &root,
                &dir,
                &config::Step::new(
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
            let mut argv = vec!["@app".into()];
            argv.extend(tail.iter().map(|s| (*s).to_owned()));
            let outcome =
                runner::execute(&root, &dir, &config::Step::new("app".into(), argv, 30)?)?;
            Ok(
                json!({"ok":outcome.status == Status::Passed,"process":outcome,
                "application_stdout":std::fs::read_to_string(&outcome.stdout)?}),
            )
        }
        _ => Err("unknown command or extra arguments; run cargo xtask help".into()),
    }
}
