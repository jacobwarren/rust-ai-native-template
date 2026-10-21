//! One writer, bounded reads, versioned journal and same-directory atomic replacement.
use crate::{Command, Error, MAX_BYTES, Result, Session};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    schema_version: u32,
    commands: Vec<Command>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    data: Journal,
    sha256: String,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub struct Store {
    root: PathBuf,
    // The OS releases this lock on normal drop and process termination.
    _lock: File,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    BeforeWrite,
    AfterSync,
    AfterReplace,
}

impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        if !cfg!(unix) {
            return Err(Error::new(
                "unsupported",
                "persistence is supported on Linux/macOS",
            ));
        }
        fs::create_dir_all(root).map_err(io_error)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join("writer.lock"))
            .map_err(io_error)?;
        lock.try_lock().map_err(|e| {
            Error::new(
                "store_unavailable",
                format!("cannot acquire single-writer lock: {e}"),
            )
        })?;
        Ok(Self {
            root: root.to_owned(),
            _lock: lock,
        })
    }

    pub fn save(&self, session: &Session) -> Result<()> {
        self.save_with(session, |_| Ok(()))
    }

    // Hooks are private and supplied only by unit tests; no protocol fault switch exists.
    fn save_with(
        &self,
        session: &Session,
        hook: impl Fn(Stage) -> std::io::Result<()>,
    ) -> Result<()> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let data = Journal {
            schema_version: 1,
            commands: session.journal().to_vec(),
        };
        let sha256 = digest(&serde_json::to_vec(&data).map_err(json_error)?);
        let bytes = serde_json::to_vec(&Envelope { data, sha256 }).map_err(json_error)?;
        if bytes.len() > MAX_BYTES {
            return Err(Error::new("capacity", "saved journal exceeds 256 KiB"));
        }
        let temp = self.root.join(format!(
            ".board-{}-{}.tmp",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = File::create_new(&temp).map_err(io_error)?;
        let mut replaced = false;
        let result = (|| {
            hook(Stage::BeforeWrite)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            hook(Stage::AfterSync)?;
            fs::rename(&temp, self.root.join("board.json"))?;
            replaced = true;
            hook(Stage::AfterReplace)?;
            File::open(&self.root)?.sync_all()
        })();
        drop(file);
        if !replaced {
            let _ = fs::remove_file(&temp);
        }
        result.map_err(|e| {
            if replaced {
                Error::new("save_uncertain", format!("replacement is visible but directory durability is unconfirmed: {e}; inspect/reopen before retry"))
            } else { io_error(e) }
        })
    }

    pub fn load(&self, session: &mut Session) -> Result<()> {
        // Orphan staging files are never candidates for recovery. Last committed board wins.
        let mut bytes = Vec::new();
        File::open(self.root.join("board.json"))
            .map_err(io_error)?
            .take((MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() > MAX_BYTES {
            return Err(Error::new("capacity", "saved journal exceeds 256 KiB"));
        }
        let envelope: Envelope = serde_json::from_slice(&bytes).map_err(json_error)?;
        if envelope.data.schema_version != 1 {
            return Err(Error::new("version", "unsupported saved journal version"));
        }
        let expected = digest(&serde_json::to_vec(&envelope.data).map_err(json_error)?);
        if envelope.sha256 != expected {
            return Err(Error::new("corrupt_store", "journal checksum mismatch"));
        }
        session.replay(envelope.data.commands)
    }
}

fn io_error(e: std::io::Error) -> Error {
    Error::new("storage_io", e.to_string())
}
fn json_error(e: serde_json::Error) -> Error {
    Error::new("invalid_store", e.to_string())
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
