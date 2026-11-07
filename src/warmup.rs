//! Explicit, single-model startup warm-up. Disabled by default.
use crate::engine::{InferenceEngine, ModelSpec};
use crate::model_registry::Registry;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarmupConfig {
    pub enabled: bool,
    pub model: Option<String>,
}

impl WarmupConfig {
    pub fn parse(enabled: Option<&str>, model: Option<&str>) -> Result<Self> {
        let enabled = match enabled.map(str::trim) {
            None => false,
            Some(s) if s == "0" || s.eq_ignore_ascii_case("false") => false,
            Some(s) if s == "1" || s.eq_ignore_ascii_case("true") => true,
            _ => bail!("SHIMMY_WARMUP must be 0, 1, false or true"),
        };
        let model = if enabled {
            model
                .map(|s| {
                    let s = s.trim();
                    if s.is_empty() {
                        bail!("SHIMMY_WARMUP_MODEL must be a nonempty model name");
                    }
                    Ok(s.to_owned())
                })
                .transpose()?
        } else {
            None
        };
        Ok(Self { enabled, model })
    }

    pub fn from_env() -> Result<Self> {
        let value = std::env::var("SHIMMY_WARMUP")
            .map(Some)
            .or_else(|e| match e {
                std::env::VarError::NotPresent => Ok(None),
                _ => Err(anyhow::anyhow!("SHIMMY_WARMUP must be valid Unicode")),
            })?;
        let mut config = Self::parse(value.as_deref(), None)?;
        if config.enabled {
            let model = std::env::var("SHIMMY_WARMUP_MODEL")
                .map(Some)
                .or_else(|e| match e {
                    std::env::VarError::NotPresent => Ok(None),
                    _ => Err(anyhow::anyhow!("SHIMMY_WARMUP_MODEL must be valid Unicode")),
                })?;
            config = Self::parse(value.as_deref(), model.as_deref())?;
        }
        Ok(config)
    }
}

fn lookup(registry: &Registry, name: &str) -> Result<ModelSpec> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| registry.to_spec(name)))
        .map_err(|_| anyhow::anyhow!("Warm-up model metadata is invalid"))?
        .context("Unknown warm-up model")
}

fn canonical(path: &Path) -> Result<std::path::PathBuf> {
    path.canonicalize()
        .map_err(|_| anyhow::anyhow!("Warm-up model file is unavailable"))
}

pub fn resolve_warmup_target(
    registry: &Registry,
    explicit_path: Option<&Path>,
    base_env: bool,
    config: &WarmupConfig,
) -> Result<Option<ModelSpec>> {
    if !config.enabled {
        return Ok(None);
    }
    let selected = config
        .model
        .as_deref()
        .map(|name| lookup(registry, name))
        .transpose()?;
    if let Some(path) = explicit_path {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("direct-model");
        let spec = lookup(registry, name)?;
        let expected = canonical(path)?;
        if canonical(&spec.base_path)? != expected {
            bail!("Explicit warm-up model registration does not match --model-path");
        }
        if let Some(other) = selected {
            if canonical(&other.base_path)? != expected {
                bail!("SHIMMY_WARMUP_MODEL conflicts with --model-path");
            }
        }
        return Ok(Some(spec));
    }
    let spec = if let Some(spec) = selected {
        spec
    } else if base_env {
        lookup(registry, "tinyllama-1.1b")?
    } else {
        let names = registry.list_all_available();
        if names.len() != 1 {
            bail!("Startup warm-up requires one model; set SHIMMY_WARMUP_MODEL when several are available");
        }
        lookup(registry, &names[0])?
    };
    canonical(&spec.base_path)?;
    Ok(Some(spec))
}

pub async fn prepare_warmup(
    engine: &dyn InferenceEngine,
    registry: &Registry,
    explicit_path: Option<&Path>,
    base_env: bool,
    config: &WarmupConfig,
) -> Result<()> {
    let Some(spec) = resolve_warmup_target(registry, explicit_path, base_env, config)? else {
        return Ok(());
    };
    tracing::info!(model = %spec.name, "warmup_start");
    let start = Instant::now();
    let result = async {
        let loaded = engine
            .load(&spec)
            .await
            .context("Startup model load failed")?;
        loaded
            .warmup()
            .await
            .context("Startup model warm-up failed")
    }
    .await;
    if result.is_err() {
        tracing::error!("warmup_failed");
    } else {
        tracing::info!(model = %spec.name, elapsed_ms = start.elapsed().as_millis(), "warmup_complete");
    }
    result
}
