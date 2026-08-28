//! `gguf_routing` — the offline "screen test" for template routing.
//!
//! For every GGUF model in the local models dir, read the GGUF header via
//! airframe's BindlessMetadata (metadata only, no GPU) and assert that
//! `prompt_render::decide` routes it to the CORRECT render source:
//!   - GGUF chat_template present  -> GgufJinja (or correct per-arch fallback)
//!   - no template (base/completion)-> Raw (NOT a bogus OpenChat wrap)
//!
//! This is the cheap per-model template check — no GPU, no inference. It
//! validates that every model's template is routed correctly, so the heavy
//! 5-prompt inference battery (oqu.9) only has to confirm generation works.
//!
//! Gated on the `airframe` feature (needs BindlessMetadata). Reads the model
//! dir from SHIMMY_MODELS_DIR (default /home/michael/models); skips (passes)
//! if the dir is absent so CI without models stays green.

#![cfg(feature = "airframe")]

use std::path::PathBuf;

use airframe::backend::bindless::metadata::BindlessMetadata;
use shimmy::prompt_render::{decide, RenderMode, RenderSource};

fn models_dir() -> Option<PathBuf> {
    let dir = std::env::var("SHIMMY_MODELS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/michael/models"));
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

fn walk_ggufs(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walk_ggufs(&p));
            } else if p.extension().map(|s| s == "gguf").unwrap_or(false) {
                out.push(p);
            }
        }
    }
    out
}

#[test]
fn gguf_routing_every_model_correct_source() {
    let Some(dir) = models_dir() else {
        eprintln!("SHIMMY_MODELS_DIR absent — skipping gguf_routing (no models)");
        return;
    };
    let ggu = walk_ggufs(&dir);
    assert!(!ggu.is_empty(), "no .gguf files found under {dir:?}");
    eprintln!("gguf_routing: {} models", ggu.len());

    let mut jinja = 0usize;
    let mut raw = 0usize;
    let mut fallback = 0usize;

    for path in &ggu {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let lower = name.to_lowercase();

        // Read the GGUF header (metadata only, no GPU).
        let f = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("  skip {name}: cannot open ({e})");
                continue;
            }
        };
        let mut r = std::io::BufReader::new(f);
        let meta = BindlessMetadata::new(&mut r);
        let spec = meta.to_model_spec();
        let gguf_tpl = spec.chat_template.as_deref();

        let src = decide(gguf_tpl, None, &name, RenderMode::Auto);

        let is_instruct_arch = lower.contains("gemma")
            || lower.contains("qwen")
            || lower.contains("llama")
            || lower.contains("phi")
            || lower.contains("tinyllama")
            || lower.contains("mistral")
            || lower.contains("deepseek")
            || lower.contains("ministral");

        match src {
            RenderSource::GgufJinja => {
                assert!(
                    gguf_tpl.is_some(),
                    "{name}: decided GgufJinja but no GGUF template"
                );
                jinja += 1;
            }
            RenderSource::Raw => {
                // Raw is correct ONLY for true base/completion models (no template
                // AND not an instruct arch). An instruct arch with a template that
                // failed must NOT be raw.
                if is_instruct_arch && gguf_tpl.is_some() {
                    panic!(
                        "{name}: instruct model with a template routed to RAW — \
                         template render failed silently; must use per-arch fallback"
                    );
                }
                raw += 1;
            }
            RenderSource::FamilyFallback => {
                // Fallback is correct for instruct models whose template fails to
                // render (Gemma-4 tool/macro templates) OR no template but instruct.
                assert!(
                    is_instruct_arch,
                    "{name}: FamilyFallback for a non-instruct model — should be Raw"
                );
                fallback += 1;
            }
        }
        eprintln!(
            "  {name:45} -> {:?} (tpl={})",
            src,
            gguf_tpl.map(|s| s.len()).unwrap_or(0)
        );
    }

    // Sanity: we should have seen Jinja-routed models (the common case).
    assert!(jinja > 0, "expected at least one GGUF-Jinja-routed model");
    eprintln!("gguf_routing result: {jinja} Jinja, {fallback} family-fallback, {raw} raw");
}
