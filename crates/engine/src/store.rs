//! One writer, bounded reads, versioned journal and same-directory atomic replacement.
use crate::{Command, Error, JOURNAL_FORMAT, MAX_BYTES, OLDEST_JOURNAL_FORMAT, Result, Session};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
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
// A readable format is parsed strictly once the peek has verified it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    data: Journal,
    #[serde(rename = "sha256")]
    _sha256: String,
}
#[derive(Serialize)]
struct EnvelopeOut<'a> {
    data: &'a RawValue,
    sha256: String,
}
// Every format keeps data and sha256. The peek ignores anything else, so a reader
// can verify and date a file from a newer build before it understands it.
#[derive(Deserialize)]
struct Peek<'a> {
    #[serde(borrow)]
    data: &'a RawValue,
    sha256: String,
}
#[derive(Deserialize)]
struct PeekData {
    schema_version: u32,
}
impl Peek<'_> {
    fn parse(bytes: &[u8]) -> Result<(Peek<'_>, u32)> {
        let peek: Peek = serde_json::from_slice(bytes).map_err(json_error)?;
        let data: PeekData = serde_json::from_str(peek.data.get()).map_err(json_error)?;
        Ok((peek, data.schema_version))
    }
}

/// The lowest format that can hold these commands, so older builds keep reading
/// boards that never used a newer command.
fn format_for(commands: &[Command]) -> u32 {
    commands
        .iter()
        .map(Command::introduced_in)
        .fold(1, u32::max)
}

fn check_format(found: u32, oldest: u32, newest: u32) -> Result<()> {
    if found < oldest {
        return Err(Error::new(
            "invalid_store",
            format!("journal format {found} is not a saved-board format"),
        ));
    }
    if found > newest {
        return Err(too_new(format!(
            "board.json uses journal format {found}; this build reads {}. Open it with a newer build. The file was not changed.",
            formats(oldest, newest)
        )));
    }
    Ok(())
}

fn formats(oldest: u32, newest: u32) -> String {
    if oldest == newest {
        format!("format {newest}")
    } else {
        format!("formats {oldest} to {newest}")
    }
}

fn too_new(message: String) -> Error {
    Error::new("store_too_new", message)
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
        self.refuse_newer()?;
        let commands = session.journal().to_vec();
        let data = Journal {
            schema_version: format_for(&commands),
            commands,
        };
        // The checksum covers exactly the data bytes written to disk.
        let data = RawValue::from_string(serde_json::to_string(&data).map_err(json_error)?)
            .map_err(json_error)?;
        let sha256 = digest(data.get().as_bytes());
        let bytes = serde_json::to_vec(&EnvelopeOut {
            data: &data,
            sha256,
        })
        .map_err(json_error)?;
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
        let bytes = self.read().map_err(io_error)?;
        Self::load_from(&bytes, JOURNAL_FORMAT, session)
    }

    // Takes the newest readable format so tests can act as a newer build.
    fn load_from(bytes: &[u8], newest: u32, session: &mut Session) -> Result<()> {
        if bytes.len() > MAX_BYTES {
            return Err(Error::new("capacity", "saved journal exceeds 256 KiB"));
        }
        let (peek, format) = Peek::parse(bytes)?;
        if digest(peek.data.get().as_bytes()) != peek.sha256 && !reserialized_v1(&peek, format) {
            return Err(Error::new("corrupt_store", "journal checksum mismatch"));
        }
        check_format(format, OLDEST_JOURNAL_FORMAT, newest)?;
        let envelope: Envelope = serde_json::from_slice(bytes).map_err(json_error)?;
        session.replay(envelope.data.commands)
    }

    /// Refuse to replace a board from a newer build. A missing, unreadable or
    /// unparseable file is replaced as before: saving is how a damaged board recovers.
    /// An oversized file is refused because its format cannot be read, and this build
    /// never writes one: a newer build with a larger limit may have.
    fn refuse_newer(&self) -> Result<()> {
        let bytes = match self.read() {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(io_error(e)),
        };
        if bytes.len() > MAX_BYTES {
            return Err(Error::new(
                "capacity",
                "board.json exceeds 256 KiB, so this build cannot tell which build wrote it. It was not overwritten.",
            ));
        }
        match Peek::parse(&bytes) {
            Ok((_, found)) if found > JOURNAL_FORMAT => Err(too_new(format!(
                "board.json uses journal format {found}, newer than this build writes (up to {JOURNAL_FORMAT}). It was not overwritten."
            ))),
            _ => Ok(()),
        }
    }

    // Reads one byte past the limit so callers can tell an oversized file.
    fn read(&self) -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        File::open(self.root.join("board.json"))?
            .take((MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}

/// Format 1 files were checked against the re-serialized journal, so a v1 board
/// that was reformatted by hand still loads. Later formats accept only the raw digest.
fn reserialized_v1(peek: &Peek, format: u32) -> bool {
    format == 1
        && serde_json::from_str::<Journal>(peek.data.get())
            .and_then(|journal| serde_json::to_vec(&journal))
            .is_ok_and(|bytes| digest(&bytes) == peek.sha256)
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
