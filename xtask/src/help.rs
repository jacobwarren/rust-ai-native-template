//! The command table, read by help requests and by wrong-argument errors.
use crate::config::Result;
use serde_json::{Map, Value, json};
use std::fmt;

/// Every dispatched command: its usage and what it does. Literal words are
/// lowercase, placeholders UPPERCASE, optional trailing parts in [ ], and
/// alternatives joined by |.
pub const TABLE: &[(&str, &str)] = &[
    ("doctor", "Check local tools and configuration"),
    ("map", "Workspace package and layer inventory"),
    (
        "check fast",
        "Short edit feedback loop; emits evidence JSON",
    ),
    (
        "check ci",
        "Full configured pre-merge checks; emits evidence JSON",
    ),
    (
        "architecture",
        "Validate all declared workspace dependency edges",
    ),
    (
        "docs",
        "Validate catalog, AGENTS size, and local file links",
    ),
    ("scenario", "Build and replay application boundary cases"),
    (
        "workflow [ID]",
        "Run independent stateful process workflows; capture requests, replies and golden comparisons",
    ),
    (
        "readiness [--report PATH]",
        "Compute implemented and verified requirements from source-bound evidence",
    ),
    (
        "release SCOPE --report PATH",
        "Fail unless the named scope is ready; never publishes",
    ),
    (
        "gaps [--base REV [--report PATH]]",
        "Read known project gaps (does not generate them); with --base, validate the branch's transitions",
    ),
    (
        "task-artifact style --task ID",
        "Create the task's artifact folder and write the shared stylesheet block to paste into each page (docs/ARTIFACTS.md)",
    ),
    (
        "task-artifact check --task ID",
        "Check the task's required pages exist and carry the stylesheet inline",
    ),
    (
        "task-artifact screenshot --task ID",
        "Render the task's pages to PNG tiles under target/artifact-screenshots with local Chrome; reports console errors, phone overflow and network requests",
    ),
    (
        "task-artifact guard --task ID",
        "Fail until the five discovery pages are committed; for host hooks before product edits",
    ),
    (
        "task-artifact next --task ID",
        "Say which page to write, review, rewrite or commit next; exit 3 adds advice, such as a page edited after review",
    ),
    (
        "task-artifact verdict --task ID --page PAGE --verdict pass|revise|best --reviewer NAME --model MODEL --findings N --notes TEXT|-",
        "Append one page review, with the page's SHA-256, finding count and reviewer model, to the task's reviews.json; --notes - reads the notes from stdin",
    ),
    (
        "artifact-hook --task ID",
        "Hook adapter for Codex apply_patch events: reads one event on stdin and refuses product edits until the discovery pages are committed; run by scripts/agent-artifact-write.sh",
    ),
    (
        "coordination",
        "Validate task record structure without demanding completion",
    ),
    (
        "agent-stop --task ID --base REV --report PATH",
        "One task's stop verdict, by hand: complete's gate when the record says complete, otherwise a partial or blocked handoff check that accepts a missing report. The default stop hook runs complete, not this",
    ),
    (
        "complete --task ID --base REV --report PATH",
        "Gate a completion claim with reconciliation and current evidence; the default stop hook runs this for each changed task record that says complete",
    ),
    (
        "handoff --base REV [--report PATH]",
        "Check changed tasks; partial/blocked handoffs are explicitly allowed",
    ),
    (
        "run -- ARGS...",
        "Build starter-app and run it with ARGS, such as greet Ada, with isolated state and JSON logs; xtask never reads ARGS, --help included",
    ),
    ("env", "Show worktree-specific context"),
    (
        "help [WORDS]",
        "Print every command, or those that start with WORDS; WORDS --help or -h does the same",
    ),
];

/// Arguments no dispatch arm accepted, with the usage of the commands that
/// start with the same words.
#[derive(Debug)]
pub struct UsageError {
    /// The command words that matched, or the unknown first word.
    pub command: String,
    /// Matching keys of TABLE in table order; empty for an unknown command.
    pub usage: Vec<&'static str>,
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.usage.is_empty() {
            return write!(f, "unknown command {}; run cargo xtask help", self.command);
        }
        write!(
            f,
            "wrong arguments for {}; usage: {}",
            self.command,
            self.usage.join(", ")
        )?;
        // Dispatch matches fixed slices, so flags must come in the order shown.
        let flags = self.usage.iter().any(|u| {
            u.split(' ')
                .any(|t| t.trim_start_matches('[').starts_with("--") && t.len() > 2)
        });
        if flags {
            write!(f, "; flags go in the order shown")?;
        }
        Ok(())
    }
}

impl std::error::Error for UsageError {}

/// A help request: `--help` or `-h` anywhere before the first `--`. It
/// returns the command words typed before the first flag. Words after `--`
/// belong to the application that `run` starts.
pub fn requested(args: &[String]) -> Option<Vec<&str>> {
    let head = args.split(|a| a == "--").next().unwrap_or_default();
    head.iter().any(|a| a == "--help" || a == "-h").then(|| {
        head.iter()
            .map(String::as_str)
            .take_while(|a| !a.starts_with('-'))
            .collect()
    })
}

/// Every entry, or those that start with `words`; when none does, the usage error.
pub fn narrowed(words: &[&str]) -> Result<Value> {
    let entries: Vec<_> = TABLE.iter().filter(|e| starts(e.0, words)).collect();
    if entries.is_empty() {
        return Err(usage_error(words).into());
    }
    let commands: Map<String, Value> = entries
        .into_iter()
        .map(|(usage, does)| ((*usage).to_owned(), json!(does)))
        .collect();
    Ok(json!({"ok":true,"commands":commands}))
}

/// The error for arguments no arm accepted: the entries that start with the
/// longest run of the typed words.
pub fn usage_error(words: &[&str]) -> UsageError {
    for n in (1..=words.len()).rev() {
        let usage: Vec<&'static str> = TABLE
            .iter()
            .filter(|e| starts(e.0, &words[..n]))
            .map(|e| e.0)
            .collect();
        if let Some(first) = usage.first() {
            // Name only the typed words that matched a literal command word.
            let named = tokens(first)
                .zip(&words[..n])
                .take_while(|(t, _)| !t.starts_with('-') && !placeholder(t))
                .count();
            return UsageError {
                command: words[..named.max(1)].join(" "),
                usage,
            };
        }
    }
    UsageError {
        command: words.first().copied().unwrap_or_default().to_owned(),
        usage: Vec::new(),
    }
}

/// The words of a usage key, with optional brackets dropped.
fn tokens(usage: &str) -> impl Iterator<Item = &str> {
    usage
        .split(' ')
        .map(|t| t.trim_matches(|c| c == '[' || c == ']'))
}

fn starts(usage: &str, words: &[&str]) -> bool {
    let tokens: Vec<&str> = tokens(usage).collect();
    tokens.len() >= words.len() && tokens.iter().zip(words).all(|(t, w)| fits(t, w))
}

/// An UPPERCASE placeholder, such as ID or ARGS...
fn placeholder(token: &str) -> bool {
    let name = token.trim_end_matches("...");
    !name.is_empty() && name.chars().all(|c| c.is_ascii_uppercase())
}

/// A literal takes itself; a placeholder takes any word that is not a flag;
/// alternatives joined by | take any of theirs.
fn fits(token: &str, word: &str) -> bool {
    token.split('|').any(|alt| {
        if placeholder(alt) {
            !word.starts_with("--")
        } else {
            alt == word
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_reads_as_a_usage() {
        for (usage, _) in TABLE {
            for t in tokens(usage) {
                assert!(
                    t.split('|').all(|a| placeholder(a)
                        || a == "-"
                        || a == "--"
                        || a.chars().all(|c| c.is_ascii_lowercase() || c == '-')),
                    "{usage}: {t}"
                );
            }
        }
        let wrong = usage_error(&["task-artifact", "nxt", "--task", "X"]);
        assert_eq!(wrong.command, "task-artifact");
        assert_eq!(wrong.usage.len(), 6);
        let check = usage_error(&["check"]);
        assert_eq!(
            check.to_string(),
            "wrong arguments for check; usage: check fast, check ci"
        );
    }
}
