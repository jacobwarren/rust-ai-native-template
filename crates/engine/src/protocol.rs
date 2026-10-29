//! Strict adapter protocol. All adapters dispatch through this driver and Session.
use crate::{
    Command, Error, JOURNAL_FORMAT, MAX_BYTES, MAX_COMMANDS, OLDEST_JOURNAL_FORMAT, Result,
    Session,
    store::{Store, digest},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub id: u64,
    pub action: Action,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Discover {},
    Inspect {},
    Execute { command: Command },
    Replay { commands: Vec<Command> },
    Capture {},
    Save {},
    Load {},
}

/// Discoverable descriptors include strict example payloads and current availability.
/// The typed enums remain authoritative; descriptor compatibility is tested.
pub fn capabilities(session: &Session, persistence: bool) -> Value {
    json!({"protocol_version":1,"boundary":"application_engine",
        "limits":{"request_bytes":MAX_BYTES,"journal_commands":MAX_COMMANDS,"tasks":64,"title_chars":120},
        "methods":["discover","inspect","execute","replay","capture","save","load"],
        "persistence_available":persistence,
        "journal_formats":{"reads":(OLDEST_JOURNAL_FORMAT..=JOURNAL_FORMAT).collect::<Vec<_>>(),"writes":(1..=JOURNAL_FORMAT).collect::<Vec<_>>()},
        "commands":[
            {"id":"add","example":{"command":"add","title":"Ship a change"},"effect":"Add a task; returns its id"},
            {"id":"complete","example":{"command":"complete","id":1},"effect":"Complete an existing open task"},
            {"id":"remove","example":{"command":"remove","id":1},"effect":"Remove an existing task"},
            {"id":"undo","example":{"command":"undo"},"effect":"Undo the last edit","enabled":session.inspect()["can_undo"]}
        ],"errors":"Invalid requests and rejected edits preserve session state; see docs/DRIVER.md"})
}

#[derive(Default)]
pub struct Driver {
    session: Session,
    store: Option<Store>,
}

impl Driver {
    pub fn open(root: Option<&Path>) -> Result<Self> {
        Ok(Self {
            session: Session::default(),
            store: root.map(Store::open).transpose()?,
        })
    }

    pub fn handle(&mut self, bytes: &[u8]) -> Value {
        let parsed: Result<Request> = if bytes.len() > MAX_BYTES {
            Err(Error::new("capacity", "request exceeds 256 KiB"))
        } else {
            serde_json::from_slice(bytes).map_err(|e| Error::new("invalid_request", e.to_string()))
        };
        let id = parsed.as_ref().ok().map(|r| r.id);
        let result = parsed.and_then(|request| {
            if request.schema_version != 1 {
                return Err(Error::new("version", "unsupported protocol version"));
            }
            self.dispatch(request.action)
        });
        match result {
            Ok(result) => json!({"schema_version":1,"id":id,"ok":true,"result":result}),
            Err(error) => json!({"schema_version":1,"id":id,"ok":false,"error":error}),
        }
    }

    fn dispatch(&mut self, action: Action) -> Result<Value> {
        match action {
            Action::Discover {} => Ok(capabilities(&self.session, self.store.is_some())),
            Action::Inspect {} => Ok(self.session.inspect()),
            Action::Execute { command } => self.session.execute(command),
            Action::Replay { commands } => {
                self.session.replay(commands)?;
                Ok(self.session.inspect())
            }
            Action::Capture {} => {
                let snapshot = self.session.inspect();
                let hash = digest(
                    &serde_json::to_vec(&snapshot)
                        .map_err(|e| Error::new("capture", e.to_string()))?,
                );
                Ok(
                    json!({"boundary":"application_engine","snapshot":snapshot,"journal":self.session.journal(),"snapshot_sha256":hash}),
                )
            }
            Action::Save {} => {
                self.store
                    .as_ref()
                    .ok_or_else(no_store)?
                    .save(&self.session)?;
                Ok(json!({"saved":true,"revision":self.session.journal().len()}))
            }
            Action::Load {} => {
                self.store
                    .as_ref()
                    .ok_or_else(no_store)?
                    .load(&mut self.session)?;
                Ok(self.session.inspect())
            }
        }
    }
}

fn no_store() -> Error {
    Error::new(
        "storage_disabled",
        "set APP_DATA_DIR to an isolated directory for save/load",
    )
}
