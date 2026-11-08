#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The artifact author's own hook and the subagent and skill definitions it relies on.
mod support;
use serde_json::{Value, json};
use std::path::Path;
use support::Fixture;
use support::hooks::*;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

#[test]
fn author_mode_allows_only_the_artifact_folder_and_outside_paths() {
    let f = Fixture::new();
    for (path, allowed) in [
        ("harness/task-artifacts/TASK-1/plan.html", true),
        ("harness/task-artifacts/plan.html", false),
        ("target/debug/xtask", false),
        ("xtask/src/main.rs", false),
        ("crates/app/src/main.rs", false),
        (
            "harness/task-artifacts/TASK-1/../../../xtask/src/main.rs",
            false,
        ),
    ] {
        let o = hook(&f, "author", "claude", &edit(&f, path), None);
        assert_eq!(o.code == 0, allowed, "{path}: {}", o.stderr);
        if !allowed {
            assert!(o.stderr.contains("edits only task pages"), "{}", o.stderr);
        }
    }
    let outside = event(
        &f,
        "Write",
        json!({"file_path":"/tmp/notes.txt","content":"x"}),
    );
    assert_eq!(hook(&f, "author", "claude", &outside, None).code, 0);
    // Where Git ignores case, a differently spelled root is still the checkout.
    f.git(&["config", "core.ignorecase", "true"]);
    let upper = f.0.to_str().unwrap().to_uppercase();
    let cased = event(
        &f,
        "Edit",
        json!({"file_path":format!("{upper}/xtask/src/main.rs")}),
    );
    assert_eq!(hook(&f, "author", "claude", &cased, None).code, 2);
    f.git(&["config", "--unset", "core.ignorecase"]);
    // Known limit: a symlink outside the checkout reads as outside (the hook never
    // touches the disk); Bash and the default guard are the other line.
    let link = f.0.with_extension("link");
    std::os::unix::fs::symlink(f.0.join("xtask"), &link).unwrap();
    let via = event(&f, "Edit", json!({"file_path":link.join("src/main.rs")}));
    assert_eq!(hook(&f, "author", "claude", &via, None).code, 0);
    std::fs::remove_file(&link).unwrap();
    // Bash is not this hook's job: the default guard and the brief cover it.
    assert_eq!(
        hook(&f, "author", "claude", &shell(&f, "rm -rf xtask"), None).code,
        0
    );
    // A linked worktree may not reach back into the main checkout.
    let wt = f.0.with_extension("wt");
    f.git(&["worktree", "add", "-q", "--detach", wt.to_str().unwrap()]);
    let back = json!({"tool_name":"Edit","cwd":wt,"tool_input":{"file_path":f.0.join("xtask/src/main.rs")}});
    let o = hook(&f, "author", "claude", &back, None);
    assert_eq!(o.code, 2, "{}", o.stderr);
    assert!(o.stderr.contains("main checkout"), "{}", o.stderr);
    f.git(&["worktree", "remove", "--force", wt.to_str().unwrap()]);
}

#[test]
fn agent_definitions_preload_their_skills_and_limit_writes() {
    let root = Path::new(ROOT);
    let author = std::fs::read_to_string(root.join(".claude/agents/artifact-author.md")).unwrap();
    assert!(author.contains("name: artifact-author") && author.contains("model: opus"));
    assert!(author.contains("  - author-artifact"), "{author}");
    assert!(
        author.contains("scripts/agent-hook.sh\" author claude"),
        "{author}"
    );
    let reviewer =
        std::fs::read_to_string(root.join(".claude/agents/artifact-reviewer.md")).unwrap();
    assert!(reviewer.contains("model: sonnet") && reviewer.contains("  - review-artifact"));
    let tools = reviewer.lines().find(|l| l.starts_with("tools:")).unwrap();
    assert!(
        !tools.contains("Write") && !tools.contains("Edit"),
        "{tools}"
    );
    for (agent, skill) in [
        ("artifact-author", "author-artifact"),
        ("artifact-reviewer", "review-artifact"),
    ] {
        let toml =
            std::fs::read_to_string(root.join(format!(".codex/agents/{agent}.toml"))).unwrap();
        assert!(toml.contains(&format!("name = \"{agent}\"")), "{toml}");
        assert!(
            toml.contains(&format!("${skill}")),
            "{agent} must invoke {skill}"
        );
        for host in [".claude/skills", ".agents/skills"] {
            assert!(
                root.join(host).join(skill).join("SKILL.md").is_file(),
                "{host}/{skill}"
            );
        }
    }
}

fn docs(f: &Fixture) -> Value {
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("docs")
        .current_dir(&f.0)
        .output()
        .unwrap();
    serde_json::from_slice(&o.stdout).unwrap()
}

#[test]
fn skill_copies_must_match_and_name_their_folder() {
    let f = Fixture::new();
    let skill = "---\nname: demo\ndescription: A demo skill.\n---\n\nBody.\n";
    let mut catalog = vec![];
    for host in [".claude/skills", ".agents/skills"] {
        f.text(&format!("{host}/demo/SKILL.md"), skill);
        catalog.push(json!({"path":format!("{host}/demo/SKILL.md"),"owner":"o","status":"active","verified_by":"test"}));
    }
    f.write("harness/docs.json", json!(catalog));
    assert_eq!(docs(&f)["ok"], true, "{}", docs(&f));
    f.text(
        ".agents/skills/demo/SKILL.md",
        &skill.replace("Body.", "Edited body."),
    );
    let v = docs(&f);
    assert!(v.to_string().contains("DOC007 skills/demo/SKILL.md"), "{v}");
    let wrong = skill.replace("name: demo", "name: other");
    f.text(".claude/skills/demo/SKILL.md", &wrong);
    f.text(".agents/skills/demo/SKILL.md", &wrong);
    let v = docs(&f);
    assert!(
        v.to_string()
            .contains("DOC008 .claude/skills/demo/SKILL.md"),
        "{v}"
    );
    assert!(!v.to_string().contains("DOC007"), "{v}");
    // An unquoted ": " in the description breaks strict YAML parsers.
    let colon = skill.replace("A demo skill.", "Demo: a skill.");
    f.text(".claude/skills/demo/SKILL.md", &colon);
    f.text(".agents/skills/demo/SKILL.md", &colon);
    assert!(
        docs(&f)
            .to_string()
            .contains("DOC010 .claude/skills/demo/SKILL.md")
    );
    let quoted = skill.replace("A demo skill.", "\"Demo: a skill.\"");
    f.text(".claude/skills/demo/SKILL.md", &quoted);
    f.text(".agents/skills/demo/SKILL.md", &quoted);
    assert_eq!(docs(&f)["ok"], true, "{}", docs(&f));
}

#[test]
fn free_skills_stay_plain_text() {
    let f = Fixture::new();
    let skill = "---\nname: demo\ndescription: >\n  A demo skill.\n---\n\nBody.\n";
    let mut catalog = vec![];
    for name in ["demo", "author-artifact"] {
        for host in [".claude/skills", ".agents/skills"] {
            let path = format!("{host}/{name}/SKILL.md");
            f.text(&path, &skill.replace("demo", name));
            catalog.push(json!({"path":path,"owner":"o","status":"active","verified_by":"test"}));
        }
    }
    f.write("harness/docs.json", json!(catalog));
    assert_eq!(docs(&f)["ok"], true, "{}", docs(&f));
    for changed in [
        skill.replace("---\n\n", "hooks:\n  Stop: []\n---\n\n"),
        skill.replace("---\n\n", "allowed-tools: Bash\n---\n\n"),
        skill.replace("---\n\n", "name: other\n---\n\n"),
        skill.replace("Body.", "Run !`date` first."),
        skill.replace("Body.", "```!\ndate\n```"),
    ] {
        for host in [".claude/skills", ".agents/skills"] {
            f.text(&format!("{host}/demo/SKILL.md"), &changed);
            f.text(
                &format!("{host}/author-artifact/SKILL.md"),
                &changed.replace("demo", "author-artifact"),
            );
        }
        let v = docs(&f).to_string();
        assert!(
            v.contains("DOC011 .claude/skills/demo/SKILL.md")
                || v.contains("DOC012 .claude/skills/demo/SKILL.md"),
            "{changed}: {v}"
        );
        // A ceremony skill may set more; its edits need a receipt instead.
        assert!(
            !v.contains(".claude/skills/author-artifact/SKILL.md: frontmatter"),
            "{v}"
        );
        assert!(!v.contains("DOC012 .claude/skills/author-artifact"), "{v}");
    }
    // Only text files, and no host folder below the root.
    for host in [".claude/skills", ".agents/skills"] {
        f.text(&format!("{host}/demo/SKILL.md"), skill);
        f.text(&format!("{host}/demo/run.sh"), "date\n");
    }
    f.text("crates/app/.claude/skills/helper/SKILL.md", skill);
    let v = docs(&f).to_string();
    assert!(v.contains("DOC013 .claude/skills/demo/run.sh"), "{v}");
    assert!(
        v.contains("DOC014 crates/app/.claude/skills/helper/SKILL.md"),
        "{v}"
    );
}

#[test]
fn investigate_skill_matches_its_seed() {
    let f = Fixture::new();
    let seed = "# Repository investigation skill\n\nStart with AGENTS.md.\n";
    let skill = format!("---\nname: investigate\ndescription: Investigate.\n---\n\n{seed}");
    let mut catalog = vec![
        json!({"path":"docs/skills/investigate.md","owner":"o","status":"active","verified_by":"test"}),
    ];
    f.text("docs/skills/investigate.md", seed);
    for host in [".claude/skills", ".agents/skills"] {
        f.text(&format!("{host}/investigate/SKILL.md"), &skill);
        catalog.push(json!({"path":format!("{host}/investigate/SKILL.md"),"owner":"o","status":"active","verified_by":"test"}));
    }
    f.write("harness/docs.json", json!(catalog));
    assert_eq!(docs(&f)["ok"], true, "{}", docs(&f));
    // GEPA edits the seed; the skill must follow, or the hosts read other words.
    f.text(
        "docs/skills/investigate.md",
        &seed.replace("AGENTS.md", "the README"),
    );
    let v = docs(&f);
    assert!(
        v.to_string().contains("DOC009 skills/investigate/SKILL.md"),
        "{v}"
    );
}
