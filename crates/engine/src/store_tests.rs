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
    for data in [
        b"truncated".to_vec(),
        serde_json::to_vec(&corrupt).unwrap(),
        serde_json::to_vec(&future).unwrap(),
        vec![b' '; MAX_BYTES + 1],
    ] {
        fs::write(dir.0.join("board.json"), data).unwrap();
        assert!(store.load(&mut live).is_err());
        assert_eq!(live.inspect(), before);
    }
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
