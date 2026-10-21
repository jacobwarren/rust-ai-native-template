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
