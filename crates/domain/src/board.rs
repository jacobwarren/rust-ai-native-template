//! Small deterministic domain. No process, serialization, storage, or clock access.
use std::collections::BTreeMap;

pub const MAX_TASKS: usize = 64;
pub const MAX_TITLE_CHARS: usize = 120;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Title(String);

impl TryFrom<&str> for Title {
    type Error = &'static str;
    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        if raw.len() > 512 {
            return Err("raw title exceeds 512 bytes");
        }
        if raw.chars().any(char::is_control) {
            return Err("title must not contain control characters");
        }
        let text = raw.trim();
        if text.is_empty() || text.chars().count() > MAX_TITLE_CHARS {
            return Err("title must contain 1–120 Unicode scalar values");
        }
        Ok(Self(text.to_owned()))
    }
}

impl Title {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub title: Title,
    pub done: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    next_id: u64,
    tasks: BTreeMap<u64, Task>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            next_id: 1,
            tasks: BTreeMap::new(),
        }
    }
}

impl Board {
    pub fn tasks(&self) -> &BTreeMap<u64, Task> {
        &self.tasks
    }
    pub fn next_id(&self) -> u64 {
        self.next_id
    }
    pub fn add(&mut self, title: Title) -> Result<u64, &'static str> {
        if self.tasks.len() >= MAX_TASKS {
            return Err("board capacity reached");
        }
        let next = self
            .next_id
            .checked_add(1)
            .ok_or("task identifiers exhausted")?;
        let id = self.next_id;
        self.tasks.insert(id, Task { title, done: false });
        self.next_id = next;
        Ok(id)
    }
    pub fn complete(&mut self, id: u64) -> Result<(), &'static str> {
        let task = self.tasks.get_mut(&id).ok_or("task does not exist")?;
        if task.done {
            return Err("task is already complete");
        }
        task.done = true;
        Ok(())
    }
    pub fn remove(&mut self, id: u64) -> Result<(), &'static str> {
        self.tasks.remove(&id).ok_or("task does not exist")?;
        Ok(())
    }
}
