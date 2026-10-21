//! Baseline comparison is read-only. Missing/unreadable baselines always fail.
use crate::config::Result;
use serde_json::{Value, json};
use std::path::{Component, Path};

pub fn baseline_path(root: &Path, path: &str) -> Result<std::path::PathBuf> {
    let relative = Path::new(path);
    if !relative.starts_with("harness/goldens")
        || !relative
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
    {
        return Err("golden path must be a plain path under harness/goldens".into());
    }
    let full = root.join(relative);
    // Missing file is handled as a failed comparison, not blessed automatically.
    if full.exists()
        && !full
            .canonicalize()?
            .starts_with(root.canonicalize()?.join("harness/goldens"))
    {
        return Err("golden path escapes baseline directory".into());
    }
    Ok(full)
}

pub fn compare(root: &Path, baseline: &str, actual: &Value, candidate: &Path) -> Result<Value> {
    let path = baseline_path(root, baseline)?;
    let expected = std::fs::read(&path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string()));
    let ok = expected.as_ref().is_ok_and(|expected| expected == actual);
    let mut result = json!({"ok":ok,"baseline":baseline});
    if !ok {
        std::fs::write(candidate, serde_json::to_vec_pretty(actual)?)?;
        result["candidate"] = json!(candidate);
        result["error"] = json!(expected.err().unwrap_or_else(|| "baseline mismatch".into()));
        result["review"] = json!(
            "Inspect the diff and specification; update the baseline only as an explicit reviewed source change."
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_and_changed_baselines_fail_without_writing_source() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("starter-golden-{stamp}-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("harness/goldens")).unwrap();
        let relative = "harness/goldens/state.json";
        let candidate = dir.join("candidate.json");
        let actual = json!({"tasks":[]});
        assert_eq!(
            compare(&dir, relative, &actual, &candidate).unwrap()["ok"],
            false
        );
        assert!(
            !dir.join(relative).exists(),
            "missing baseline must not be blessed"
        );
        std::fs::write(dir.join(relative), r#"{"tasks":["expected"]}"#).unwrap();
        let bytes = std::fs::read(dir.join(relative)).unwrap();
        assert_eq!(
            compare(&dir, relative, &actual, &candidate).unwrap()["ok"],
            false
        );
        assert_eq!(std::fs::read(dir.join(relative)).unwrap(), bytes);
        assert_eq!(
            compare(&dir, relative, &json!({"tasks":["expected"]}), &candidate).unwrap()["ok"],
            true
        );
        assert!(baseline_path(&dir, "harness/goldens/../../source.json").is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
