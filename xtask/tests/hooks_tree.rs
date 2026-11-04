#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Default hooks: host-local files and protected host config.
mod support;
use serde_json::json;
use support::Fixture;
use support::hooks::*;

#[test]
fn host_local_files_do_not_change_the_tree() {
    let f = Fixture::new();
    let before = f.run(&["env"])["source_sha256"].clone();
    f.text(
        ".claude/settings.local.json",
        "{\"permissions\":{\"allow\":[\"Bash(ls:*)\"]}}",
    );
    f.text(".claude/worktrees/w/x.rs", "fn main() {}\n");
    assert_eq!(f.run(&["env"])["source_sha256"], before);
    assert_eq!(f.handoff()["ok"], true);
    assert_eq!(
        f.run(&["gaps", "--base", "HEAD"])["protected_files"],
        json!([])
    );

    // Only the exact root paths are host-local; the same name elsewhere is source.
    f.text("docs/.claude/settings.local.json", "{}");
    assert_ne!(f.run(&["env"])["source_sha256"], before);
    let handoff = run_raw(&f, &["handoff", "--base", "HEAD"]);
    assert_eq!(handoff["ok"], false, "{handoff}");
}

#[test]
fn committed_local_settings_are_still_reported() {
    let f = Fixture::new();
    f.text(".claude/settings.local.json", "{}");
    f.git(&["add", "-f", ".claude/settings.local.json"]);
    f.commit("commit a host-local file by mistake");
    let base = f.git(&["rev-parse", "HEAD"]);
    f.text("implementation.rs", "// changed\n");
    let gaps = run_raw(&f, &["gaps", "--base", &base]);
    assert!(
        gaps["protected_files"]
            .as_array()
            .unwrap()
            .contains(&json!(".claude/settings.local.json")),
        "{gaps}"
    );
}
