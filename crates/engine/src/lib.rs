//! Shared application behavior used by the CLI and the JSON-lines driver.
mod protocol;
pub mod store;
pub use protocol::{Driver, Request, capabilities};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use starter_domain::board::{Board, Title};

pub const MAX_COMMANDS: usize = 256;
pub const MAX_BYTES: usize = 262_144;
/// The newest saved-journal format this build reads and writes.
pub const JOURNAL_FORMAT: u32 = 1;
/// The oldest saved-journal format this build still reads.
pub const OLDEST_JOURNAL_FORMAT: u32 = 1;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Error {
    pub code: &'static str,
    pub message: String,
}

impl Error {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// A strict command wire format; domain admission occurs before any edit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Add { title: String },
    Complete { id: u64 },
    Remove { id: u64 },
    Undo {},
}

impl Command {
    /// The journal format that first stored this command. A new variant must name
    /// its format and raise JOURNAL_FORMAT; there is deliberately no wildcard arm.
    pub(crate) const fn introduced_in(&self) -> u32 {
        match self {
            Self::Add { .. } | Self::Complete { .. } | Self::Remove { .. } | Self::Undo {} => 1,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Session {
    board: Board,
    history: Vec<Board>,
    journal: Vec<Command>,
}

impl Session {
    pub fn execute(&mut self, command: Command) -> Result<Value> {
        if self.journal.len() >= MAX_COMMANDS {
            return Err(Error::new(
                "capacity",
                "journal capacity reached (256 commands)",
            ));
        }
        // Prepare a candidate, then commit. Errors leave board/history/journal unchanged.
        let mut next = self.board.clone();
        let result = match &command {
            Command::Add { title } => {
                let title =
                    Title::try_from(title.as_str()).map_err(|e| Error::new("invalid_title", e))?;
                let id = next.add(title).map_err(|e| Error::new("capacity", e))?;
                json!({"id":id})
            }
            Command::Complete { id } => {
                next.complete(*id)
                    .map_err(|e| Error::new("invalid_task", e))?;
                json!({"id":id})
            }
            Command::Remove { id } => {
                next.remove(*id)
                    .map_err(|e| Error::new("invalid_task", e))?;
                json!({"id":id})
            }
            Command::Undo {} => {
                next = self
                    .history
                    .last()
                    .cloned()
                    .ok_or_else(|| Error::new("no_undo", "nothing to undo"))?;
                json!({"undone":true})
            }
        };
        if matches!(command, Command::Undo {}) {
            self.history.pop();
        } else {
            self.history.push(self.board.clone());
        }
        self.board = next;
        self.journal.push(command);
        Ok(result)
    }

    pub fn inspect(&self) -> Value {
        let tasks: Vec<_> = self
            .board
            .tasks()
            .iter()
            .map(|(id, task)| json!({"id":id,"title":task.title.as_str(),"done":task.done}))
            .collect();
        json!({"tasks":tasks,"next_id":self.board.next_id(),"revision":self.journal.len(),"can_undo":!self.history.is_empty()})
    }

    pub fn journal(&self) -> &[Command] {
        &self.journal
    }

    /// Fresh-state replay is all-or-nothing; it does not append to the current session.
    pub fn replay(&mut self, commands: Vec<Command>) -> Result<()> {
        if commands.len() > MAX_COMMANDS {
            return Err(Error::new("capacity", "replay exceeds 256 commands"));
        }
        let mut candidate = Self::default();
        for (index, command) in commands.into_iter().enumerate() {
            candidate
                .execute(command)
                .map_err(|e| Error::new("replay_failed", format!("step {index}: {}", e.message)))?;
        }
        *self = candidate;
        Ok(())
    }
}
