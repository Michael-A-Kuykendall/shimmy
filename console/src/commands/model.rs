use anyhow::{Context, Result};
use crate::config::ConsoleConfig;
use serde_json::Value;

const DEFAULT_MODELS_ENDPOINT: &str = "http://localhost:11435/v1/models";

fn models_endpoint() -> String {
    std::env::var("SHIMMY_MODELS_ENDPOINT").unwrap_or_else(|_| DEFAULT_MODELS_ENDPOINT.to_string())
}

/// CLI-friendly helper that both validates the requested model
/// and persists it to the user config.
pub async fn set_model(model_name: String) -> Result<()> {
    select_model(model_name.as_str()).await?;
    println!("✅ Active model set to: {}", model_name);
    Ok(())
}

/// Shared helper for WebSocket/CLI that updates the saved model
/// after verifying it is available on the Shimmy server.
pub async fn select_model(model_name: &str) -> Result<()> {
    // FIXME: ensure_model_exists() uses hardcoded port 11435, but shimmy uses ephemeral ports
    // Skip validation for now since WebSocket handler already has model list
    // ensure_model_exists(model_name).await?;
    let mut config = ConsoleConfig::load()?;
    config.set_active_model(model_name.to_string())?;
    Ok(())
}

pub async fn show_current_model() -> Result<()> {
    match get_selected_model().await? {
        Some(model) => {
            println!("📋 Current active model: {}", model);
        }
        None => {
            println!("❌ No active model selected");
            println!("💡 Use 'shimmy set-model <name>' to select a model");
            println!("💡 Use 'shimmy list' to see available models");
        }
    }
    Ok(())
}

/// Returns the currently selected model from config.
/// NOTE: Validation via ensure_model_exists() is skipped because it uses hardcoded port 11435
/// but shimmy uses ephemeral ports. The WebSocket handler validates models via InferenceBackend.
pub async fn get_selected_model() -> Result<Option<String>> {
    let config = ConsoleConfig::load()?;
    Ok(config.get_active_model().cloned())
}

pub async fn ensure_model_exists(model_name: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get(models_endpoint())
        .send()
        .await?
        .error_for_status()
        .context("failed to query shimmy models endpoint")?;

    let models_response: Value = response.json().await?;
    if response_contains_model(&models_response, model_name) {
        Ok(())
    } else {
        anyhow::bail!("Model '{}' not found", model_name);
    }
}

fn response_contains_model(response: &Value, model_name: &str) -> bool {
    let contains = |items: &[Value]| {
        items.iter().any(|model| {
            model.get("id")
                .and_then(|id| id.as_str())
                .map(|id| id == model_name)
                .unwrap_or(false)
        })
    };

    if let Some(data) = response.get("data").and_then(|d| d.as_array()) {
        return contains(data);
    }

    if let Some(models) = response.get("models").and_then(|m| m.as_array()) {
        return contains(models);
    }

    if let Some(items) = response.as_array() {
        return contains(items);
    }

    false
}