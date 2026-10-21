use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub layers: BTreeMap<String, Layer>,
    pub profiles: BTreeMap<String, Vec<Step>>,
    pub max_rust_lines: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub dependencies: Vec<String>,
    pub external: Vec<String>,
}

/// A command admitted at the boundary. Invalid commands cannot be constructed
/// by ordinary callers; wire JSON is deliberately kept separate.
#[derive(Debug, Deserialize)]
#[serde(try_from = "RawStep")]
pub struct Step {
    id: String,
    program: String,
    args: Vec<String>,
    timeout_seconds: std::num::NonZeroU64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStep {
    id: String,
    argv: Vec<String>,
    timeout_seconds: u64,
}

impl TryFrom<RawStep> for Step {
    type Error = String;

    fn try_from(raw: RawStep) -> std::result::Result<Self, Self::Error> {
        Self::new(raw.id, raw.argv, raw.timeout_seconds)
    }
}

impl Step {
    pub fn new(id: String, argv: Vec<String>, seconds: u64) -> std::result::Result<Self, String> {
        if !valid_id(&id) {
            return Err("command id must be 1-80 ASCII letters, digits, '-' or '_'".into());
        }
        let mut argv = argv.into_iter();
        let program = argv
            .next()
            .filter(|s| !s.trim().is_empty())
            .ok_or("command requires a nonempty executable")?;
        let args: Vec<_> = argv.collect();
        if program.contains('\0') || args.iter().any(|s| s.contains('\0')) {
            return Err("command arguments cannot contain NUL bytes".into());
        }
        let timeout_seconds =
            std::num::NonZeroU64::new(seconds).ok_or("command timeout must be positive")?;
        Ok(Self {
            id,
            program,
            args,
            timeout_seconds,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn program(&self) -> &str {
        &self.program
    }
    pub fn args(&self) -> &[String] {
        &self.args
    }
    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds.get()
    }
    pub fn argv(&self) -> Vec<String> {
        std::iter::once(self.program.clone())
            .chain(self.args.iter().cloned())
            .collect()
    }
}

pub fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

pub fn load(root: &Path) -> Result<Config> {
    let config: Config = read(&root.join("harness/config.json"))?;
    if config.schema_version != 1 || config.profiles.is_empty() || config.layers.is_empty() {
        return Err("unsupported or empty harness configuration".into());
    }
    for (name, profile) in &config.profiles {
        if profile.is_empty() {
            return Err(format!("profile {name} has no steps").into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for step in profile {
            if !ids.insert(step.id()) {
                return Err(format!("duplicate step in {name}: {}", step.id()).into());
            }
        }
    }
    for (name, layer) in &config.layers {
        for dependency in &layer.dependencies {
            if !config.layers.contains_key(dependency) {
                return Err(format!("layer {name} references unknown layer {dependency}").into());
            }
        }
    }
    Ok(config)
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn command_boundary_rejects_invalid_states() {
        for raw in [
            json!({"id":"ok","argv":[],"timeout_seconds":1}),
            json!({"id":"../escape","argv":["cargo"],"timeout_seconds":1}),
            json!({"id":"ok","argv":["  "],"timeout_seconds":1}),
            json!({"id":"ok","argv":["cargo"],"timeout_seconds":0}),
            json!({"id":"ok","argv":["cargo","a\u{0}b"],"timeout_seconds":1}),
            json!({"id":"ok","argv":["cargo"],"timeout_seconds":1,"typo":true}),
        ] {
            assert!(serde_json::from_value::<Step>(raw).is_err());
        }
    }

    #[test]
    fn parsed_command_preserves_argv_including_empty_arguments() {
        let step: Step = serde_json::from_value(json!({
            "id":"valid","argv":["cargo","","two words"],"timeout_seconds":3
        }))
        .expect("valid command");
        assert_eq!(step.program(), "cargo");
        assert_eq!(step.args(), ["", "two words"]);
        assert_eq!(step.timeout_seconds(), 3);
    }
}
