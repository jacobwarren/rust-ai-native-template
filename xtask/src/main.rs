mod artifact_hook;
mod artifacts;
mod checks;
mod completion;
mod config;
mod golden;
mod handover;
mod history;
mod readiness;
mod repo;
mod router;
mod runner;
mod scenarios;
mod size;
mod task_record;
mod tracking;
mod transitions;
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
            // Advice that deserves a look but blocks nothing, from task-artifact next.
            let advised = value["advisories"]
                .as_array()
                .is_some_and(|a| !a.is_empty());
            if ok && advised {
                ExitCode::from(3)
            } else if ok {
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

/// Most bytes `verdict --notes -` reads: a long review, far short of a runaway pipe.
const NOTES_CAP: usize = 64 * 1024;

/// Notes piped to `verdict --notes -`. The length is checked on bytes before
/// decoding, so an over-long reply cut inside a character still reads as too long.
fn piped_notes() -> Result<String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(NOTES_CAP as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > NOTES_CAP {
        return Err("--notes - read more than 64 KiB from stdin; pipe the review itself".into());
    }
    let notes = String::from_utf8(bytes).map_err(|_| "--notes - needs UTF-8 on stdin")?;
    if notes.trim().is_empty() {
        return Err("--notes - read nothing from stdin; pipe the reviewer's reply".into());
    }
    Ok(notes)
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
            "gaps":"Read known project gaps (does not generate them); --base REV [--report PATH] validates transitions",
            "task-artifact style --task ID":"Create the task's artifact folder and write the shared stylesheet block to paste into each page (docs/ARTIFACTS.md)",
            "task-artifact check --task ID":"Check the task's required pages exist and carry the stylesheet inline",
            "task-artifact screenshot --task ID":"Render the task's pages to PNG tiles under target/artifact-screenshots with local Chrome; reports console errors, phone overflow and network requests",
            "task-artifact guard --task ID":"Fail until the five discovery pages are committed; for host hooks before product edits",
            "task-artifact next --task ID":"Say which page to write, review, rewrite or commit next; exit 3 adds advice, such as a page edited after review",
            "task-artifact verdict --task ID --page PAGE --verdict pass|revise|best --reviewer NAME --model MODEL --findings N --notes TEXT|-":"Append one page review, with the page's SHA-256, finding count and reviewer model, to the task's reviews.json; --notes - reads the notes from stdin",
            "coordination":"Validate task record structure without demanding completion",
            "agent-stop --task ID --base REV --report PATH":"Hook gate: completion or explicit partial/blocked handoff",
            "complete --task ID --base REV --report PATH":"Gate a completion claim with reconciliation and current evidence",
            "handoff --base REV [--report PATH]":"Check changed tasks; partial/blocked handoffs are explicitly allowed",
            "run -- greet Ada":"Run example with isolated state and JSON logs",
            "env":"Show worktree-specific context"}})),
        ["artifact-hook", "--task", id] => artifact_hook::run(&root, id),
        ["agent-stop", "--task", id, "--base", base, "--report", path] => {
            completion::agent_stop(&root, id, base, std::path::Path::new(path))
        }
        ["task-artifact", "style", "--task", id] => artifacts::install(&root, id),
        ["task-artifact", "guard", "--task", id] => artifacts::guard(&root, id),
        ["task-artifact", "next", "--task", id] => router::next(&root, id),
        [
            "task-artifact",
            "verdict",
            "--task",
            id,
            "--page",
            page,
            "--verdict",
            verdict,
            "--reviewer",
            reviewer,
            "--model",
            model,
            "--findings",
            findings,
            "--notes",
            notes,
        ] => {
            // "-" reads the reviewer's whole reply from stdin, where shell quoting cannot cut it.
            let piped = if *notes == "-" {
                Some(piped_notes()?)
            } else {
                None
            };
            router::verdict(
                &root,
                id,
                &router::Review {
                    page,
                    verdict,
                    reviewer,
                    notes: piped.as_deref().unwrap_or(notes),
                    findings,
                    model,
                },
            )
        }
        ["task-artifact", "verdict", ..] => Err(
            "task-artifact verdict takes --task ID --page PAGE --verdict pass|revise|best --reviewer NAME --model MODEL --findings N --notes TEXT|-, in that order; --model and --findings are required"
                .into(),
        ),
        ["task-artifact", "screenshot", "--task", id] => artifacts::screenshots(&root, id),
        ["task-artifact", "check", "--task", id] => {
            let task = task_record::read(&root, id)?;
            if task.tier == task_record::Tier::Small {
                artifacts::check_written(&root, id)?;
            } else {
                artifacts::check(
                    &root,
                    id,
                    task.status == task_record::Status::Complete,
                    None,
                )?;
            }
            Ok(json!({"ok":true,"task":id}))
        }
        ["coordination"] => completion::validate(&root),
        ["complete", "--task", id, "--base", base, "--report", path] => {
            completion::complete(&root, id, base, std::path::Path::new(path))
        }
        ["handoff", "--base", base] => completion::handoff(&root, base, None),
        ["handoff", "--base", base, "--report", path] => {
            completion::handoff(&root, base, Some(std::path::Path::new(path)))
        }
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
        ["gaps"] => tracking::list(&root),
        ["gaps", "--base", base] => transitions::check(&root, base, None),
        ["gaps", "--base", base, "--report", path] => {
            transitions::check(&root, base, Some(std::path::Path::new(path)))
        }
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
