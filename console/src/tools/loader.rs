use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Error types for manifest loading
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("IO error reading manifest: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Invalid manifest schema: {0}")]
    Schema(String),
}

/// Error types for snap-in tool loading
#[derive(Debug, Error, Clone)]
pub enum ToolLoadError {
    #[error("IO error reading tool definition: {0}")]
    Io(String),
    #[error("JSON parse error: {0}")]
    Parse(String),
    #[error("Invalid tool.json schema: {0}")]
    Schema(String),
    #[error("Unsupported tool type: {0}")]
    UnsupportedType(String),
}

/// Per-tool toggle in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolToggle {
    #[serde(default)]
    pub enabled: Option<bool>,
}

/// Global tools manifest at tools/manifest.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    #[serde(default)]
    pub tools: HashMap<String, ToolToggle>,
}

/// Snap-in tool definition from tools/<name>/tool.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapInDefinition {
    pub name: String,
    #[serde(default)]
    pub version: String,
    pub description: String,
    #[serde(rename = "type")]
    pub tool_type: String, // "builtin", "command", "wasm"
    pub parameters: serde_json::Value, // JSON Schema object
    #[serde(default)]
    pub permissions: SnapInPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnapInPermissions {
    #[serde(default = "default_filesystem")]
    pub filesystem: String, // "read-only", "read-write", "none"
    #[serde(default)]
    pub network: Vec<String>, // allowed domains or HTTP methods
    #[serde(default)]
    pub exec: bool, // allow spawning subprocess
}

fn default_filesystem() -> String {
    "read-write".to_string()
}

/// Load the global manifest from tools/manifest.json
/// Returns:
/// - Ok(None) if file does not exist (no manifest is fine)
/// - Ok(Some(manifest)) if file exists and is valid
/// - Err(ManifestError) if file exists but is malformed
pub fn load_manifest(path: &Path) -> Result<Option<ToolManifest>, ManifestError> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(path)?;
    let manifest: ToolManifest = serde_json::from_str(&contents)?;

    // Basic schema validation
    for (tool_name, toggle) in &manifest.tools {
        if tool_name.is_empty() {
            return Err(ManifestError::Schema(
                "Tool name cannot be empty".to_string(),
            ));
        }
        // enabled field is optional and defaults to None, which is valid
        if let Some(enabled) = toggle.enabled {
            if !enabled && !enabled {
                // This is just a safety check; enabled is bool so always valid
            }
        }
    }

    Ok(Some(manifest))
}

/// Load all snap-in definitions from tools/ directory
/// Scans for tools/*/tool.json files and returns a Vec of Results
/// Each element is either a valid SnapInDefinition or a ToolLoadError
pub fn load_snapins(tools_dir: &Path) -> Vec<Result<SnapInDefinition, ToolLoadError>> {
    let mut results = Vec::new();

    // Check if tools directory exists
    if !tools_dir.exists() || !tools_dir.is_dir() {
        return results; // Empty vec, no tools to load
    }

    // Iterate over subdirectories in tools/
    let entries = match std::fs::read_dir(tools_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("⚠️  Failed to read tools directory: {}", e);
            return results;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue; // Skip files at top level
        }

        let tool_json_path = path.join("tool.json");
        if !tool_json_path.exists() {
            continue; // No tool.json, skip this directory
        }

        // Try to load and parse the tool.json
        match load_single_snapin(&tool_json_path) {
            Ok(def) => results.push(Ok(def)),
            Err(e) => results.push(Err(e)),
        }
    }

    results
}

/// Load a single snap-in definition from a tool.json file
fn load_single_snapin(path: &Path) -> Result<SnapInDefinition, ToolLoadError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| ToolLoadError::Io(e.to_string()))?;

    let def: SnapInDefinition = serde_json::from_str(&contents)
        .map_err(|e| ToolLoadError::Parse(e.to_string()))?;

    // Validate required fields
    if def.name.is_empty() {
        return Err(ToolLoadError::Schema("name field is required and cannot be empty".to_string()));
    }
    if def.description.is_empty() {
        return Err(ToolLoadError::Schema("description field is required and cannot be empty".to_string()));
    }
    if def.tool_type.is_empty() {
        return Err(ToolLoadError::Schema("type field is required and cannot be empty".to_string()));
    }

    // Validate parameters is an object
    if !def.parameters.is_object() {
        return Err(ToolLoadError::Schema("parameters must be a JSON object (JSON Schema)".to_string()));
    }

    Ok(def)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("shimmy_loader_test_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(&temp).unwrap();
        temp
    }

    #[test]
    fn test_load_manifest_not_exists() {
        let temp = temp_dir();
        let manifest_path = temp.join("nonexistent_manifest.json");
        // Ensure the file doesn't exist
        assert!(!manifest_path.exists());
        let result = load_manifest(&manifest_path).unwrap();
        assert!(result.is_none());
        fs::remove_dir_all(&temp).ok();
    }

    #[test]
    fn test_load_manifest_valid() {
        let temp = temp_dir();
        let manifest_path = temp.join("manifest.json");
        fs::write(&manifest_path, r#"{"tools": {"read_image": {"enabled": false}}}"#).unwrap();

        let result = load_manifest(&manifest_path).unwrap();
        assert!(result.is_some());
        let manifest = result.unwrap();
        assert_eq!(manifest.tools.len(), 1);
        assert_eq!(manifest.tools.get("read_image").unwrap().enabled, Some(false));

        fs::remove_dir_all(&temp).ok();
    }

    #[test]
    fn test_load_manifest_malformed() {
        let temp = temp_dir();
        let manifest_path = temp.join("manifest.json");
        fs::write(&manifest_path, r#"{"tools": invalid json}"#).unwrap();

        let result = load_manifest(&manifest_path);
        assert!(result.is_err());

        fs::remove_dir_all(&temp).ok();
    }

    #[test]
    fn test_load_snapins_empty_dir() {
        let temp = temp_dir();
        let tools_dir = temp.join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        let results = load_snapins(&tools_dir);
        assert_eq!(results.len(), 0);

        fs::remove_dir_all(&temp).ok();
    }

    #[test]
    fn test_load_snapins_valid() {
        let temp = temp_dir();
        let tools_dir = temp.join("tools");
        let tool_dir = tools_dir.join("test_tool");
        fs::create_dir_all(&tool_dir).unwrap();

        let tool_json = r#"{
            "name": "test_tool",
            "version": "1.0.0",
            "description": "Test tool",
            "type": "builtin",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        }"#;
        fs::write(tool_dir.join("tool.json"), tool_json).unwrap();

        let results = load_snapins(&tools_dir);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
        let def = results[0].as_ref().unwrap();
        assert_eq!(def.name, "test_tool");

        fs::remove_dir_all(&temp).ok();
    }

    #[test]
    fn test_load_snapins_invalid() {
        let temp = temp_dir();
        let tools_dir = temp.join("tools");
        let tool_dir = tools_dir.join("bad_tool");
        fs::create_dir_all(&tool_dir).unwrap();

        fs::write(tool_dir.join("tool.json"), r#"{"invalid": json}"#).unwrap();

        let results = load_snapins(&tools_dir);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());

        fs::remove_dir_all(&temp).ok();
    }
}
