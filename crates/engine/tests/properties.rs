#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Deterministic generated-sequence properties against a deliberately simpler Vec model.
//! Seeds and failing operation indices are reproducible. This generator does not shrink.
use serde_json::json;
use starter_engine::{Command, MAX_COMMANDS, Session};

type State = (u64, Vec<(u64, String, bool)>);
#[test]
fn generated_edits_match_reference_model_and_replay() {
    for seed in 1..=64u64 {
        let mut random = seed;
        let mut actual = Session::default();
        let mut reference: State = (1, vec![]);
        let mut history: Vec<State> = vec![];
        let mut accepted = 0;
        for step in 0..100 {
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let id = (random >> 8) % 20;
            let choice = (random >> 32) % 4;
            let command = match choice {
                0 => Command::Add {
                    title: format!("Task é {step}"),
                },
                1 => Command::Complete { id },
                2 => Command::Remove { id },
                _ => Command::Undo {},
            };
            let old = reference.clone();
            let valid = match &command {
                Command::Add { title } if reference.1.len() < 64 => {
                    reference.1.push((reference.0, title.clone(), false));
                    reference.0 += 1;
                    true
                }
                Command::Complete { id } => {
                    if let Some(task) = reference.1.iter_mut().find(|t| t.0 == *id && !t.2) {
                        task.2 = true;
                        true
                    } else {
                        false
                    }
                }
                Command::Remove { id } => {
                    if let Some(pos) = reference.1.iter().position(|t| t.0 == *id) {
                        reference.1.remove(pos);
                        true
                    } else {
                        false
                    }
                }
                Command::Undo {} => {
                    if let Some(before) = history.pop() {
                        reference = before;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if valid {
                accepted += 1;
                if !matches!(command, Command::Undo {}) {
                    history.push(old);
                }
            }
            let result = actual.execute(command);
            assert_eq!(result.is_ok(), valid, "seed {seed}, step {step}");
            let tasks: Vec<_> = reference
                .1
                .iter()
                .map(|(id, title, done)| json!({"id":id,"title":title,"done":done}))
                .collect();
            assert_eq!(
                actual.inspect(),
                json!({"tasks":tasks,"next_id":reference.0,"revision":accepted,"can_undo":!history.is_empty()}),
                "seed {seed}, step {step}"
            );
            let mut replay = Session::default();
            replay.replay(actual.journal().to_vec()).unwrap();
            assert_eq!(
                replay.inspect(),
                actual.inspect(),
                "seed {seed}, step {step}"
            );
        }
    }
}

#[test]
fn failed_replay_and_invalid_edit_preserve_all_observable_state() {
    let mut s = Session::default();
    s.execute(Command::Add {
        title: "Keep".into(),
    })
    .unwrap();
    let before = s.inspect();
    let journal = s.journal().to_vec();
    assert!(s.execute(Command::Add { title: "\n".into() }).is_err());
    assert!(
        s.replay(vec![
            Command::Add {
                title: "Replace".into()
            },
            Command::Remove { id: 99 }
        ])
        .is_err()
    );
    assert_eq!(s.inspect(), before);
    assert_eq!(s.journal(), journal);
    s.execute(Command::Undo {}).unwrap();
    assert_eq!(s.inspect()["tasks"], json!([]));
}

#[test]
fn journal_limit_cannot_be_bypassed_by_undo_or_replay() {
    let mut s = Session::default();
    for _ in 0..MAX_COMMANDS / 2 {
        s.execute(Command::Add { title: "x".into() }).unwrap();
        s.execute(Command::Undo {}).unwrap();
    }
    let before = s.inspect();
    assert!(
        s.execute(Command::Add {
            title: "extra".into()
        })
        .is_err()
    );
    assert!(s.replay(vec![Command::Undo {}; MAX_COMMANDS + 1]).is_err());
    assert_eq!(s.inspect(), before);
}
