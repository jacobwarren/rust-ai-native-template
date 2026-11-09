#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! cargo xtask help: help for one command, usage on wrong arguments, and
//! descriptions that name who runs a command.
mod support;
use serde_json::Value;
use support::Fixture;

fn keys(reply: &Value) -> Vec<String> {
    reply["commands"]
        .as_object()
        .unwrap_or_else(|| panic!("no commands: {reply}"))
        .keys()
        .cloned()
        .collect()
}

fn error(reply: &Value) -> &str {
    reply["error"]
        .as_str()
        .unwrap_or_else(|| panic!("no error: {reply}"))
}

#[test]
fn help_narrows_to_the_commands_asked() {
    let f = Fixture::new();
    let one = f.run(&["task-artifact", "next", "--help"]);
    assert_eq!(keys(&one), ["task-artifact next --task ID"]);
    // A help flag typed mid-call still narrows to the command words before the flags.
    let mid = f.run(&["task-artifact", "next", "--help", "--task", "X"]);
    assert_eq!(keys(&mid), ["task-artifact next --task ID"]);
    let family = keys(&f.run(&["help", "task-artifact"]));
    assert_eq!(family.len(), 6, "{family:?}");
    assert!(family.iter().all(|k| k.starts_with("task-artifact ")));
    // These arms take any word, so --help must be read before them.
    for (words, key) in [
        (["check", "--help"], "check ci"),
        (["workflow", "--help"], "workflow [ID]"),
        (["release", "-h"], "release SCOPE --report PATH"),
    ] {
        assert!(keys(&f.run(&words)).contains(&key.to_owned()), "{words:?}");
    }
    let all = f.run(&["help"]);
    assert_eq!(keys(&all), keys(&f.run(&[])));
    assert_eq!(keys(&f.run(&["--help"])), keys(&all));
    let none = f.run(&["nonsense", "--help"]);
    assert!(error(&none).contains("cargo xtask help"), "{none}");
    assert!(none.get("usage").is_none(), "{none}");
}

#[test]
fn wrong_arguments_print_the_matching_usage() {
    let f = Fixture::new();
    let wrong = f.run(&["complete", "--task", "X"]);
    assert_eq!(
        error(&wrong),
        "wrong arguments for complete; usage: complete --task ID --base REV --report PATH; flags go in the order shown"
    );
    assert_eq!(
        wrong["usage"][0],
        "complete --task ID --base REV --report PATH"
    );
    // A typo in the second word gets every command under the first.
    let typo = f.run(&["task-artifact", "nxt", "--task", "X"]);
    assert_eq!(typo["usage"].as_array().unwrap().len(), 6, "{typo}");
    // No flags in the usage, no clause about their order.
    let check = f.run(&["check"]);
    assert_eq!(
        error(&check),
        "wrong arguments for check; usage: check fast, check ci"
    );
    let unknown = f.run(&["nonsense"]);
    assert_eq!(
        error(&unknown),
        "unknown command nonsense; run cargo xtask help"
    );
    assert!(unknown.get("usage").is_none(), "{unknown}");
}

#[test]
fn help_flags_after_run_belong_to_the_application() {
    let f = Fixture::new();
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["run", "--", "--help"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    let reply: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(reply.get("commands").is_none(), "{reply}");
    assert!(reply.get("usage").is_none(), "{reply}");
}

#[test]
fn agent_stop_help_names_the_hook_that_runs() {
    let f = Fixture::new();
    let stop = f.run(&["agent-stop", "--help"]);
    let does = stop["commands"]["agent-stop --task ID --base REV --report PATH"]
        .as_str()
        .unwrap();
    assert!(
        does.contains("The default stop hook runs complete"),
        "{does}"
    );
    let complete = f.run(&["complete", "--help"]);
    let does = complete["commands"]["complete --task ID --base REV --report PATH"]
        .as_str()
        .unwrap();
    assert!(does.contains("the default stop hook runs this"), "{does}");
    let hook = include_str!("../../scripts/agent-complete.sh");
    assert!(
        hook.contains("$xtask complete --task"),
        "the stop hook no longer runs complete"
    );
    assert!(
        !hook.contains("agent-stop"),
        "the stop hook runs agent-stop again"
    );
}
