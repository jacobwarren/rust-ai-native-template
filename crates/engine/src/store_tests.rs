use super::*;
use std::process::Command as Process;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "starter-store-{time}-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
// Bytes written by the first journal format, reviewed by hand. Never regenerate them.
const V1: &[u8] = include_bytes!("../tests/fixtures/journal/v1-board.json");
const V1_PRETTY: &[u8] = include_bytes!("../tests/fixtures/journal/v1-board.pretty.json");
const V1_STATE: &str = include_str!("../tests/fixtures/journal/v1-board.expected.json");
// Files a newer build might write, each with a correct digest over its data bytes.
const V2_ADD_ONLY: &[u8] = include_bytes!("../tests/fixtures/journal/v2-add-only.json");
const V2_NEWER: [&[u8]; 4] = [
    V2_ADD_ONLY,
    include_bytes!("../tests/fixtures/journal/v2-new-command.json"),
    include_bytes!("../tests/fixtures/journal/v2-new-field.json"),
    include_bytes!("../tests/fixtures/journal/v2-envelope-key.json"),
];

fn forge(data: &str) -> Vec<u8> {
    format!(
        r#"{{"data":{data},"sha256":"{}"}}"#,
        digest(data.as_bytes())
    )
    .into_bytes()
}

fn load_bytes(bytes: &[u8]) -> (Fixture, Store, Result<Session>) {
    let dir = Fixture::new();
    fs::write(dir.0.join("board.json"), bytes).unwrap();
    let store = Store::open(&dir.0).unwrap();
    let mut session = Session::default();
    let result = store.load(&mut session).map(|()| session);
    (dir, store, result)
}
fn session(title: &str) -> Session {
    let mut s = Session::default();
    s.execute(Command::Add {
        title: title.into(),
    })
    .unwrap();
    s
}

#[test]
fn roundtrip_preserves_state_journal_and_undo() {
    let dir = Fixture::new();
    let store = Store::open(&dir.0).unwrap();
    let mut before = session("Zoë");
    before.execute(Command::Complete { id: 1 }).unwrap();
    store.save(&before).unwrap();
    let mut after = Session::default();
    store.load(&mut after).unwrap();
    assert_eq!(after.inspect(), before.inspect());
    assert_eq!(after.journal(), before.journal());
    before.execute(Command::Undo {}).unwrap();
    after.execute(Command::Undo {}).unwrap();
    assert_eq!(after.inspect(), before.inspect());
}

#[test]
fn boards_saved_by_the_first_format_still_load() {
    let expected: serde_json::Value = serde_json::from_str(V1_STATE).unwrap();
    for bytes in [V1, V1_PRETTY] {
        let (_dir, _store, loaded) = load_bytes(bytes);
        let loaded = loaded.unwrap();
        assert_eq!(loaded.inspect(), expected);
        // The journal keeps the title as typed; replay trims it under today's rules.
        assert_eq!(
            loaded.journal()[0],
            Command::Add {
                title: "  Review  ".into()
            }
        );
    }
    let (dir, store, loaded) = load_bytes(V1);
    let mut loaded = loaded.unwrap();
    store.save(&loaded).unwrap();
    assert_eq!(fs::read(dir.0.join("board.json")).unwrap(), V1);
    loaded.execute(Command::Undo {}).unwrap();
    assert_eq!(loaded.inspect()["tasks"][1]["done"], false);
}

#[test]
fn a_reformatted_v1_board_with_an_edited_title_is_corrupt() {
    let edited = String::from_utf8(V1_PRETTY.to_vec())
        .unwrap()
        .replace("Zoë", "Zoe");
    let (_dir, _store, loaded) = load_bytes(edited.as_bytes());
    assert_eq!(loaded.unwrap_err().code, "corrupt_store");
}

#[test]
fn newer_boards_are_refused_and_never_overwritten() {
    for bytes in V2_NEWER {
        let dir = Fixture::new();
        fs::write(dir.0.join("board.json"), bytes).unwrap();
        let store = Store::open(&dir.0).unwrap();
        let mut live = session("keep");
        let before = live.inspect();
        let err = store.load(&mut live).unwrap_err();
        assert_eq!(err.code, "store_too_new");
        assert!(
            err.message.contains("format 2; this build reads format 1"),
            "{}",
            err.message
        );
        assert_eq!(live.inspect(), before);
        let err = store.save(&live).unwrap_err();
        assert_eq!(err.code, "store_too_new");
        assert!(
            err.message.contains("It was not overwritten"),
            "{}",
            err.message
        );
        assert_eq!(fs::read(dir.0.join("board.json")).unwrap(), bytes);
    }
}

#[test]
fn a_build_reads_every_format_up_to_its_own() {
    let mut s = Session::default();
    assert_eq!(
        Store::load_from(V2_ADD_ONLY, 1, &mut s).unwrap_err().code,
        "store_too_new"
    );
    Store::load_from(V2_ADD_ONLY, 2, &mut s).unwrap();
    assert_eq!(s.inspect()["tasks"][0]["title"], "Groceries");
    assert!(check_format(1, 1, 1).is_ok());
    assert!(check_format(2, 1, 2).is_ok());
    assert_eq!(check_format(2, 1, 1).unwrap_err().code, "store_too_new");
    assert_eq!(check_format(3, 1, 2).unwrap_err().code, "store_too_new");
    assert_eq!(check_format(0, 1, 2).unwrap_err().code, "invalid_store");
    assert!(
        check_format(3, 1, 2)
            .unwrap_err()
            .message
            .contains("reads formats 1 to 2")
    );
}

#[test]
fn saves_write_the_lowest_format_and_hash_the_bytes_they_write() {
    let all = [
        Command::Add { title: "a".into() },
        Command::Complete { id: 1 },
        Command::Remove { id: 1 },
        Command::Undo {},
    ];
    assert_eq!(format_for(&[]), 1);
    assert_eq!(format_for(&all), 1);
    let dir = Fixture::new();
    let store = Store::open(&dir.0).unwrap();
    store
        .save(&session("Zoë \"quoted\" </script> 漢字"))
        .unwrap();
    let bytes = fs::read(dir.0.join("board.json")).unwrap();
    let (peek, format) = Peek::parse(&bytes).unwrap();
    assert_eq!(format, JOURNAL_FORMAT);
    assert_eq!(digest(peek.data.get().as_bytes()), peek.sha256);
}

#[test]
fn saving_replaces_a_board_that_cannot_be_read() {
    let dir = Fixture::new();
    fs::write(dir.0.join("board.json"), b"{\"data\":{\"schema_vers").unwrap();
    let store = Store::open(&dir.0).unwrap();
    store.save(&session("recovered")).unwrap();
    let mut loaded = Session::default();
    store.load(&mut loaded).unwrap();
    assert_eq!(loaded.inspect(), session("recovered").inspect());
}

#[test]
fn failures_before_replace_preserve_committed_bytes() {
    let dir = Fixture::new();
    let store = Store::open(&dir.0).unwrap();
    store.save(&session("old")).unwrap();
    let bytes = fs::read(dir.0.join("board.json")).unwrap();
    for failed_stage in [Stage::BeforeWrite, Stage::AfterSync] {
        let err = store
            .save_with(&session("new"), |stage| {
                if stage == failed_stage {
                    Err(std::io::Error::other("injected disk failure"))
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert_eq!(err.code, "storage_io");
        assert_eq!(fs::read(dir.0.join("board.json")).unwrap(), bytes);
    }
    assert_eq!(
        fs::read_dir(&dir.0).unwrap().count(),
        2,
        "no failed-write staging files"
    );
}

#[test]
fn error_after_replace_reports_uncertainty_instead_of_false_rollback() {
    let dir = Fixture::new();
    let store = Store::open(&dir.0).unwrap();
    store.save(&session("old")).unwrap();
    let err = store
        .save_with(&session("new"), |stage| {
            if stage == Stage::AfterReplace {
                Err(std::io::Error::other("injected directory sync failure"))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
    assert_eq!(err.code, "save_uncertain");
    let mut recovered = Session::default();
    store.load(&mut recovered).unwrap();
    assert_eq!(recovered.inspect(), session("new").inspect());
}

#[test]
fn corrupt_future_and_oversize_files_do_not_change_live_state() {
    let dir = Fixture::new();
    let store = Store::open(&dir.0).unwrap();
    let mut live = session("keep");
    let before = live.inspect();
    store.save(&session("stored")).unwrap();
    let original = fs::read(dir.0.join("board.json")).unwrap();
    let mut corrupt: serde_json::Value = serde_json::from_slice(&original).unwrap();
    corrupt["data"]["commands"][0]["title"] = serde_json::json!("tampered");
    let mut future: serde_json::Value = serde_json::from_slice(&original).unwrap();
    future["data"]["schema_version"] = serde_json::json!(999);
    let future = forge(&future["data"].to_string());
    for (data, code) in [
        (b"truncated".to_vec(), "invalid_store"),
        (serde_json::to_vec(&corrupt).unwrap(), "corrupt_store"),
        (future, "store_too_new"),
        (vec![b' '; MAX_BYTES + 1], "capacity"),
    ] {
        fs::write(dir.0.join("board.json"), &data).unwrap();
        assert_eq!(store.load(&mut live).unwrap_err().code, code);
        assert_eq!(live.inspect(), before);
    }
    // A newer build with a larger limit may have written it, so save keeps it too.
    assert_eq!(store.save(&live).unwrap_err().code, "capacity");
    assert_eq!(
        fs::read(dir.0.join("board.json")).unwrap(),
        vec![b' '; MAX_BYTES + 1]
    );
}

#[test]
fn another_writer_is_rejected_until_lock_is_released() {
    let dir = Fixture::new();
    let first = Store::open(&dir.0).unwrap();
    assert!(Store::open(&dir.0).is_err());
    drop(first);
    assert!(Store::open(&dir.0).is_ok());
}

#[test]
fn interrupted_writer_child() {
    let Some(root) = std::env::var_os("STARTER_TEST_INTERRUPT_ROOT") else {
        return;
    };
    let store = Store::open(Path::new(&root)).unwrap();
    store
        .save_with(&session("interrupted"), |stage| {
            if stage == Stage::AfterSync {
                std::process::exit(91);
            }
            Ok(())
        })
        .unwrap();
    panic!("fault hook was not reached");
}

#[test]
fn abrupt_process_exit_reopens_last_commit_and_ignores_orphan() {
    let dir = Fixture::new();
    {
        let store = Store::open(&dir.0).unwrap();
        store.save(&session("committed")).unwrap();
    }
    let output = Process::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "store::tests::interrupted_writer_child",
            "--nocapture",
        ])
        .env("STARTER_TEST_INTERRUPT_ROOT", &dir.0)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(91),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let store = Store::open(&dir.0).unwrap();
    let mut recovered = Session::default();
    store.load(&mut recovered).unwrap();
    assert_eq!(recovered.inspect(), session("committed").inspect());
    assert!(
        fs::read_dir(&dir.0).unwrap().any(|e| e
            .unwrap()
            .path()
            .extension()
            .is_some_and(|e| e == "tmp"))
    );
    store.save(&session("next")).unwrap();
    store.load(&mut recovered).unwrap();
    assert_eq!(recovered.inspect(), session("next").inspect());
}
