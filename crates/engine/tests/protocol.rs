#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use serde_json::{Value, json};
use starter_engine::{Command, Driver, Session, capabilities};
fn call(d: &mut Driver, action: Value) -> Value {
    d.handle(
        json!({"schema_version":1,"id":7,"action":action})
            .to_string()
            .as_bytes(),
    )
}
#[test]
fn discovery_examples_are_real_commands_and_capture_replays() {
    let mut s = Session::default();
    s.execute(Command::Add {
        title: "seed".into(),
    })
    .unwrap();
    // A second, completed task lets the clear_done example succeed while the
    // complete/remove examples (id 1, open) still do.
    s.execute(Command::Add {
        title: "done".into(),
    })
    .unwrap();
    s.execute(Command::Complete { id: 2 }).unwrap();
    for descriptor in capabilities(&s, false)["commands"].as_array().unwrap() {
        let command: Command = serde_json::from_value(descriptor["example"].clone()).unwrap();
        assert!(s.clone().execute(command).is_ok(), "{descriptor}");
    }
    let mut driver = Driver::default();
    assert_eq!(
        call(
            &mut driver,
            json!({"method":"execute","command":{"command":"add","title":"Zoë"}})
        )["ok"],
        true
    );
    let captured = call(&mut driver, json!({"method":"capture"}))["result"].clone();
    assert_eq!(captured["snapshot_sha256"].as_str().unwrap().len(), 64);
    let restored = call(
        &mut Driver::default(),
        json!({"method":"replay","commands":captured["journal"]}),
    );
    assert_eq!(restored["result"], captured["snapshot"]);
}
#[test]
fn strict_requests_and_nested_commands_fail_without_mutation() {
    let mut d = Driver::default();
    let before = call(&mut d, json!({"method":"capture"}))["result"].clone();
    for command in [
        json!({"command":"add","title":"ok","typo":1}),
        json!({"command":"complete","id":-1}),
        json!({"command":"undo","id":1}),
        json!({"command":"unknown"}),
    ] {
        assert_eq!(
            call(&mut d, json!({"method":"execute","command":command}))["ok"],
            false
        );
    }
    for action in [
        json!({"method":"inspect","typo":true}),
        json!({"method":"save"}),
        json!({"method":"load"}),
    ] {
        assert_eq!(call(&mut d, action)["ok"], false);
    }
    assert_eq!(
        d.handle(br#"{"schema_version":2,"id":1,"action":{"method":"inspect"}}"#)["ok"],
        false
    );
    assert_eq!(
        d.handle(br#"{"schema_version":1,"id":1,"action":{"method":"inspect"},"extra":1}"#)["ok"],
        false
    );
    assert_eq!(call(&mut d, json!({"method":"capture"}))["result"], before);
}

fn exec(d: &mut Driver, command: Value) -> Value {
    call(d, json!({"method":"execute","command":command}))
}
fn add(d: &mut Driver, title: &str) {
    assert_eq!(exec(d, json!({"command":"add","title":title}))["ok"], true);
}
fn complete(d: &mut Driver, id: u64) {
    assert_eq!(exec(d, json!({"command":"complete","id":id}))["ok"], true);
}
fn inspect(d: &mut Driver) -> Value {
    call(d, json!({"method":"inspect"}))["result"].clone()
}

#[test]
fn clear_done_removes_all_done_and_returns_ascending_ids() {
    let mut d = Driver::default();
    for t in ["a", "b", "c", "d"] {
        add(&mut d, t);
    }
    complete(&mut d, 3);
    complete(&mut d, 1);
    let r = exec(&mut d, json!({"command":"clear_done"}));
    assert_eq!(r["ok"], true);
    assert_eq!(r["result"], json!({"removed":[1,3]}));
    let state = inspect(&mut d);
    assert_eq!(
        state["tasks"],
        json!([{"id":2,"title":"b","done":false},{"id":4,"title":"d","done":false}])
    );
    assert_eq!(state["revision"], 7);
}

#[test]
fn clear_done_without_done_fails_nothing_to_clear_and_changes_nothing() {
    let mut d = Driver::default();
    let r = exec(&mut d, json!({"command":"clear_done"}));
    assert_eq!(r["error"]["code"], "nothing_to_clear");
    add(&mut d, "open");
    let before = call(&mut d, json!({"method":"capture"}))["result"].clone();
    let r = exec(&mut d, json!({"command":"clear_done"}));
    assert_eq!(r["ok"], false);
    assert_eq!(r["error"]["code"], "nothing_to_clear");
    assert_eq!(call(&mut d, json!({"method":"capture"}))["result"], before);
}

#[test]
fn clear_done_undo_restores_everything_in_one_step_and_ids_are_not_reused() {
    let mut d = Driver::default();
    for t in ["a", "b", "c"] {
        add(&mut d, t);
    }
    complete(&mut d, 1);
    complete(&mut d, 3);
    let before = inspect(&mut d);
    exec(&mut d, json!({"command":"clear_done"}));
    // Ids of removed tasks are not reused while the clear stands.
    add(&mut d, "e");
    assert_eq!(inspect(&mut d)["tasks"][1]["id"], 4);
    assert_eq!(exec(&mut d, json!({"command":"undo"}))["ok"], true);
    // One undo reverts the add, a second reverts the whole clear.
    assert_eq!(exec(&mut d, json!({"command":"undo"}))["ok"], true);
    let after = inspect(&mut d);
    assert_eq!(after["tasks"], before["tasks"]);
    assert_eq!(after["next_id"], before["next_id"]);
}

#[test]
fn clear_done_is_strict_discoverable_and_replayable() {
    let mut d = Driver::default();
    add(&mut d, "a");
    complete(&mut d, 1);
    for bad in [
        json!({"command":"clear_done","ids":[1]}),
        json!({"command":"clear_done","id":1}),
        json!({"command":"clear_done","extra":null}),
    ] {
        assert_eq!(exec(&mut d, bad)["ok"], false);
    }
    assert_eq!(inspect(&mut d)["tasks"].as_array().unwrap().len(), 1);
    let caps = call(&mut d, json!({"method":"discover"}))["result"].clone();
    let desc = caps["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "clear_done")
        .unwrap();
    assert_eq!(desc["enabled"], true);
    exec(&mut d, json!({"command":"clear_done"}));
    let caps = call(&mut d, json!({"method":"discover"}))["result"].clone();
    let desc = caps["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "clear_done")
        .unwrap();
    assert_eq!(desc["enabled"], false);
    let captured = call(&mut d, json!({"method":"capture"}))["result"].clone();
    let restored = call(
        &mut Driver::default(),
        json!({"method":"replay","commands":captured["journal"]}),
    );
    assert_eq!(restored["result"], captured["snapshot"]);
}

#[test]
fn clear_done_survives_save_and_load() {
    let dir = std::env::temp_dir().join(format!("clear-done-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut d = Driver::open(Some(&dir)).unwrap();
    add(&mut d, "a");
    add(&mut d, "b");
    complete(&mut d, 1);
    exec(&mut d, json!({"command":"clear_done"}));
    let live = inspect(&mut d);
    assert_eq!(call(&mut d, json!({"method":"save"}))["ok"], true);
    drop(d);
    let mut d = Driver::open(Some(&dir)).unwrap();
    assert_eq!(call(&mut d, json!({"method":"load"}))["ok"], true);
    assert_eq!(inspect(&mut d), live);
    assert_eq!(exec(&mut d, json!({"command":"undo"}))["ok"], true);
    assert_eq!(inspect(&mut d)["tasks"].as_array().unwrap().len(), 2);
    drop(d);
    std::fs::remove_dir_all(&dir).unwrap();
}
