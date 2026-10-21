use crate::config::{Config, Result};
use crate::repo;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

pub fn architecture_errors(metadata: &Value, config: &Config) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(packages) = metadata["packages"].as_array() else {
        return vec!["cargo metadata has no packages".into()];
    };
    let members: BTreeSet<&str> = metadata["workspace_members"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut layers = BTreeMap::new();
    let mut paths = BTreeMap::new();
    for package in packages
        .iter()
        .filter(|p| members.contains(p["id"].as_str().unwrap_or("")))
    {
        let name = package["name"].as_str().unwrap_or("<unnamed>");
        let layer = package["metadata"]["harness"]["layer"]
            .as_str()
            .unwrap_or("");
        let owner = package["metadata"]["harness"]["owner"]
            .as_str()
            .unwrap_or("");
        if !config.layers.contains_key(layer) || owner.trim().is_empty() {
            errors.push(format!("ARCH001 {name}: set package.metadata.harness.layer and owner; register the layer in harness/config.json"));
        }
        layers.insert(name, layer);
        if let Some(path) = package["manifest_path"]
            .as_str()
            .and_then(|p| Path::new(p).parent())
        {
            paths.insert(
                name,
                path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
            );
        }
    }
    if layers.is_empty() {
        errors.push("ARCH000 no registered workspace members".into());
    }
    for package in packages {
        let name = package["name"].as_str().unwrap_or("");
        let Some(layer) = layers.get(name).and_then(|l| config.layers.get(*l)) else {
            continue;
        };
        for dependency in package["dependencies"].as_array().into_iter().flatten() {
            // Cargo metadata exposes actual package names, including renamed and target-specific dependencies.
            let dep = dependency["name"].as_str().unwrap_or("");
            if let Some(path) = dependency["path"].as_str() {
                let path = Path::new(path)
                    .canonicalize()
                    .unwrap_or_else(|_| Path::new(path).to_path_buf());
                if paths.get(dep) != Some(&path) {
                    errors.push(format!("ARCH003 {name} -> {dep}: local dependency is outside the governed workspace; register it as a member."));
                } else if !layers.get(dep).is_some_and(|target_layer| {
                    layer.dependencies.iter().any(|l| l == target_layer)
                }) {
                    errors.push(format!("ARCH002 {name} -> {dep}: forbidden layer edge (including dev/build/target deps). Move the dependency outward or document and review a policy change."));
                }
            } else if !layer.external.iter().any(|n| n == dep) {
                errors.push(format!("ARCH004 {name} -> {dep}: external dependency not allowed for this layer; justify it in an ADR and update the allowlist."));
            }
        }
    }
    errors
}

pub fn architecture(root: &Path) -> Result<Value> {
    let config = crate::config::load(root)?;
    let mut errors = architecture_errors(&repo::metadata(root)?, &config);
    for path in repo::files(root)? {
        if path.extension().is_some_and(|e| e == "rs") {
            let lines = std::fs::read_to_string(root.join(&path))?.lines().count();
            if lines > config.max_rust_lines {
                errors.push(format!(
                    "SIZE001 {}: {lines} lines exceeds {}; split by responsibility",
                    path.display(),
                    config.max_rust_lines
                ));
            }
        }
    }
    Ok(json!({"ok":errors.is_empty(),"errors":errors}))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub path: String,
    pub owner: String,
    pub status: String,
    pub verified_by: String,
}

pub fn local_links(text: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        // Supported inline code uses single backticks; fenced blocks are skipped above.
        for text in line.split('`').step_by(2) {
            for part in text.split("](").skip(1) {
                if let Some((target, _)) = part.split_once(')')
                    && !target.contains("://")
                    && !target.starts_with("mailto:")
                    && !target.starts_with('#')
                {
                    links.push(target.split('#').next().unwrap_or("").to_owned());
                }
            }
        }
    }
    links
}

pub fn docs(root: &Path) -> Result<Value> {
    let catalog: Vec<Document> = crate::config::read(&root.join("harness/docs.json"))?;
    let mut errors = Vec::new();
    let mut registered = BTreeSet::new();
    for doc in catalog {
        let path = Path::new(&doc.path);
        if path.is_absolute() || path.components().any(|c| c == Component::ParentDir) {
            errors.push(format!("DOC001 invalid catalog path {}", doc.path));
            continue;
        }
        if !registered.insert(doc.path.clone()) || !root.join(path).is_file() {
            errors.push(format!("DOC002 missing or duplicate document {}", doc.path));
        }
        if doc.owner.trim().is_empty()
            || doc.verified_by.trim().is_empty()
            || !["active", "draft", "archived"].contains(&doc.status.as_str())
        {
            errors.push(format!(
                "DOC003 {}: supply owner, status, and verification method",
                doc.path
            ));
        }
    }
    for path in repo::files(root)?
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
    {
        let path_string = path.to_string_lossy().replace('\\', "/");
        if !registered.contains(&path_string) {
            errors.push(format!(
                "DOC004 {path_string}: add ownership and verification to harness/docs.json"
            ));
        }
        let text = std::fs::read_to_string(root.join(path))?;
        if path.file_name().is_some_and(|n| n == "AGENTS.md") && text.lines().count() > 100 {
            errors.push(format!(
                "DOC005 {path_string}: keep AGENTS.md below 100 lines; link to deeper guidance"
            ));
        }
        for link in local_links(&text) {
            let candidate = root
                .join(path)
                .parent()
                .ok_or("missing parent")?
                .join(&link);
            match candidate.canonicalize() {
                Ok(resolved) if resolved.starts_with(root) => {},
                _ => errors.push(format!("DOC006 {path_string}: broken or escaping link {link}; correct the link or add the missing source")),
            }
        }
    }
    Ok(json!({"ok":errors.is_empty(),"errors":errors,
        "scope":"Markdown catalog and relative inline file links; semantic accuracy and heading anchors need review"}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Config {
        serde_json::from_value(
            json!({"schema_version":1,"max_rust_lines":400,"profiles":{},
            "layers":{"domain":{"dependencies":[],"external":[]},
            "runtime":{"dependencies":["domain"],"external":["serde_json"]}}}),
        )
        .expect("fixture")
    }

    #[test]
    fn rejects_target_and_renamed_dependencies_by_actual_package() {
        let metadata = json!({"workspace_members":["d","r"],"packages":[
            {"id":"d","name":"domain","manifest_path":"/fixture/domain/Cargo.toml","metadata":{"harness":{"layer":"domain","owner":"team"}},
             "dependencies":[{"name":"runtime","path":"/fixture/runtime","rename":"innocent","kind":"dev","target":"cfg(windows)"}]},
            {"id":"r","name":"runtime","manifest_path":"/fixture/runtime/Cargo.toml","metadata":{"harness":{"layer":"runtime","owner":"team"}},"dependencies":[]}]});
        let errors = architecture_errors(&metadata, &policy());
        assert!(errors.iter().any(|e| e.starts_with("ARCH002")));
    }

    #[test]
    fn unknown_layer_and_outside_path_are_not_silent() {
        let metadata = json!({"workspace_members":["d","x"],"packages":[
            {"id":"d","name":"domain","manifest_path":"/fixture/domain/Cargo.toml","metadata":{"harness":{"layer":"domain","owner":"team"}},
             "dependencies":[{"name":"escape","path":"/tmp/escape"}]},
            {"id":"x","name":"x","metadata":{},"dependencies":[]}]});
        let errors = architecture_errors(&metadata, &policy());
        assert!(errors.iter().any(|e| e.starts_with("ARCH001")));
        assert!(errors.iter().any(|e| e.starts_with("ARCH003")));
    }

    #[test]
    fn link_scan_ignores_code_and_external_links() {
        assert_eq!(
            local_links(
                "[a](docs/a.md#topic) [b](https://example.com) `[inline](fake)`\n```\n[x](fake)\n```\n"
            ),
            vec!["docs/a.md"]
        );
    }

    #[test]
    fn registry_name_cannot_impersonate_a_workspace_dependency() {
        let metadata = json!({"workspace_members":["d","r"],"packages":[
            {"id":"d","name":"domain","manifest_path":"/fixture/domain/Cargo.toml","metadata":{"harness":{"layer":"domain","owner":"team"}},"dependencies":[]},
            {"id":"r","name":"runtime","manifest_path":"/fixture/runtime/Cargo.toml","metadata":{"harness":{"layer":"runtime","owner":"team"}},
             "dependencies":[{"name":"domain","source":"registry+https://example.invalid"}]}]});
        assert!(
            architecture_errors(&metadata, &policy())
                .iter()
                .any(|e| e.starts_with("ARCH004"))
        );
        let mut allowed = metadata.clone();
        allowed["packages"][1]["dependencies"][0]["path"] = json!("/fixture/domain");
        assert!(architecture_errors(&allowed, &policy()).is_empty());
    }
}
