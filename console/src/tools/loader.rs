use std::collections::HashSet;
use std::path::Path;

use super::ToolRegistry;

/// Controls which tools are active.
///
/// Loads from `<tools_dir>/manifest.json` (silently ignored if absent or malformed).
///
/// Format:
/// ```json
/// {
///   "tools": {
///     "shell_command": { "enabled": false },
///     "git_commit":    { "enabled": false }
///   }
/// }
/// ```
///
/// Only `"enabled": false` entries have any effect — tools not mentioned are enabled.
pub struct ToolManifest {
    disabled: HashSet<String>,
}

impl ToolManifest {
    /// Load manifest from `<tools_dir>/manifest.json`.
    /// Returns an empty manifest (all tools enabled) if the file is absent or unparseable.
    pub fn load(tools_dir: &Path) -> Self {
        let manifest_path = tools_dir.join("manifest.json");

        let disabled = (|| -> Option<HashSet<String>> {
            let text = std::fs::read_to_string(&manifest_path).ok()?;
            let root: serde_json::Value = serde_json::from_str(&text).ok()?;
            let tools_map = root.get("tools")?.as_object()?;

            let mut set = HashSet::new();
            for (name, entry) in tools_map {
                // Treat missing "enabled" key as enabled=true
                let enabled = entry
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                if !enabled {
                    set.insert(name.clone());
                }
            }
            Some(set)
        })()
        .unwrap_or_default();

        ToolManifest { disabled }
    }

    /// Create a manifest with an explicit set of disabled tool names.
    /// Useful for testing or programmatic configuration.
    pub fn from_disabled(disabled: impl IntoIterator<Item = impl Into<String>>) -> Self {
        ToolManifest {
            disabled: disabled.into_iter().map(|s| s.into()).collect(),
        }
    }

    /// Returns `true` if the named tool should be active.
    pub fn is_enabled(&self, name: &str) -> bool {
        !self.disabled.contains(name)
    }

    /// Names of all disabled tools.
    pub fn disabled_tools(&self) -> impl Iterator<Item = &str> {
        self.disabled.iter().map(|s| s.as_str())
    }
}

/// Remove every disabled tool from the registry.
///
/// Because `ToolRegistry` uses an interior `Arc<Mutex<HashMap>>`, this mutates
/// the registry in-place and affects all clones sharing the same inner map.
pub fn apply_manifest(registry: &ToolRegistry, manifest: &ToolManifest) {
    // Collect names to remove first to avoid holding the lock while iterating.
    let to_remove: Vec<String> = registry
        .list()
        .into_iter()
        .filter(|name| !manifest.is_enabled(name))
        .collect();

    for name in to_remove {
        registry.remove(&name);
    }
}
