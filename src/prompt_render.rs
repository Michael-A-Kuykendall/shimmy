//! Unified prompt rendering: **shimmyjinja + GGUF chat_template first**,
//! hardcoded TemplateFamily only as fallback.
//!
//! Goal: every chat/instruct path prefers the model's own Jinja from GGUF
//! (`tokenizer.chat_template`) instead of name→ChatML heuristics alone.

use crate::templates::TemplateFamily;
use shimmyjinja::{try_render_chat_template_with_context, ChatMessage, RenderContext};
use std::panic::{catch_unwind, AssertUnwindSafe};

/// How the prompt was produced (for logs / factory diagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSource {
    /// GGUF Jinja via shimmyjinja
    GgufJinja,
    /// Hardcoded TemplateFamily (ChatML / Llama3 / OpenChat)
    FamilyFallback,
    /// Unchanged raw string (explicit --raw / no template available)
    Raw,
}

#[derive(Debug, Clone)]
pub struct RenderedPrompt {
    pub text: String,
    pub source: RenderSource,
}

/// Optional Jinja context knobs (BOS/EOS, thinking policy).
#[derive(Debug, Clone, Default)]
pub struct JinjaExtras {
    pub bos_token: Option<String>,
    pub eos_token: Option<String>,
    /// Model/path name used for Qwen/DeepSeek thinking heuristics.
    pub model_hint: Option<String>,
    /// Override enable_thinking (None = heuristic).
    pub enable_thinking: Option<bool>,
}

impl JinjaExtras {
    pub fn for_model(model_name: &str) -> Self {
        Self {
            model_hint: Some(model_name.to_string()),
            ..Default::default()
        }
    }
}

/// How the renderer should choose a source (the single decision point).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Auto: GGUF Jinja when present, else Raw for base models, else family fallback.
    Auto,
    /// Force raw completion (bypass template + thinking heuristic).
    ForceRaw,
    /// Force template rendering (Jinja, with family fallback if it fails).
    ForceTemplate,
}

/// The single render-decision point. All served/CLI paths call this so there is
/// exactly ONE rule for how a prompt is built. Returns the source to use (the
/// actual rendered text is produced by the render_* functions with that source).
///
/// Priority (Auto):
/// 1. `gguf_chat_template` (real GGUF Jinja) is Some → GgufJinja.
/// 2. Otherwise, if `registry_template` (shimmy's coarse "chatml"/"llama3") or
///    model-name heuristic resolves a non-OpenChat family → FamilyFallback.
/// 3. Otherwise (no template at all, e.g. base/code models) → Raw.
///
/// `registry_template` is the shimmy model-registry coarse string; it is the
/// fallback only — the real GGUF template always wins when present.
#[allow(dead_code)]
pub fn decide(
    gguf_chat_template: Option<&str>,
    registry_template: Option<&str>,
    model_name: &str,
    mode: RenderMode,
) -> RenderSource {
    match mode {
        RenderMode::ForceRaw => RenderSource::Raw,
        RenderMode::ForceTemplate => {
            if let Some(tpl) = gguf_chat_template {
                if !tpl.trim().is_empty() {
                    return RenderSource::GgufJinja;
                }
            }
            RenderSource::FamilyFallback
        }
        RenderMode::Auto => {
            if let Some(tpl) = gguf_chat_template {
                if !tpl.trim().is_empty() {
                    return RenderSource::GgufJinja;
                }
            }
            // Auto with no GGUF template: family fallback or raw.
            match family_from_spec(registry_template, model_name) {
                TemplateFamily::OpenChat => RenderSource::Raw,
                _ => RenderSource::FamilyFallback,
            }
        }
    }
}

/// Build a chat-style prompt.
///
/// Priority:
/// 1. If `gguf_chat_template` is Some and renders successfully → shimmyjinja
/// 2. Else → `family` hardcoded renderer
///
/// `messages` are (role, content). If `user_input` is Some, it is treated as
/// the final user turn (and omitted from history when rendering via family
/// path that uses history+input).
#[allow(dead_code)] // public API / tests; binary uses `_with_extras`
pub fn render_chat_prompt(
    gguf_chat_template: Option<&str>,
    family: TemplateFamily,
    system: Option<&str>,
    messages: &[(String, String)],
    user_input: Option<&str>,
) -> RenderedPrompt {
    render_chat_prompt_with_extras(
        gguf_chat_template,
        family,
        system,
        messages,
        user_input,
        &JinjaExtras::default(),
    )
}

/// Same as [`render_chat_prompt`] with BOS/EOS and thinking-policy knobs.
pub fn render_chat_prompt_with_extras(
    gguf_chat_template: Option<&str>,
    family: TemplateFamily,
    system: Option<&str>,
    messages: &[(String, String)],
    user_input: Option<&str>,
    extras: &JinjaExtras,
) -> RenderedPrompt {
    if let Some(tpl) = gguf_chat_template {
        match render_via_jinja(tpl, system, messages, user_input, extras) {
            Ok(text) => {
                return RenderedPrompt {
                    text,
                    source: RenderSource::GgufJinja,
                };
            }
            Err(e) => {
                tracing::warn!(
                    "GGUF chat_template present but shimmyjinja failed ({}); falling back to {:?}",
                    e,
                    family
                );
            }
        }
    }

    // Family path: TemplateFamily::render takes history + optional last user input
    let history: Vec<(String, String)> = if user_input.is_some() && !messages.is_empty() {
        let last_is_user = messages.last().map(|(r, _)| r == "user").unwrap_or(false);
        if last_is_user {
            messages[..messages.len() - 1].to_vec()
        } else {
            messages.to_vec()
        }
    } else {
        messages.to_vec()
    };

    let text = family.render(system, &history, user_input);
    RenderedPrompt {
        text,
        source: RenderSource::FamilyFallback,
    }
}

/// Wrap a raw completion prompt as a single user message when a GGUF template exists.
/// If `raw` is true or no template, return the prompt unchanged.
#[allow(dead_code)] // public API / tests; binary uses `_with_extras`
pub fn render_completion_prompt(
    raw_prompt: &str,
    gguf_chat_template: Option<&str>,
    family: TemplateFamily,
    force_raw: bool,
) -> RenderedPrompt {
    render_completion_prompt_with_extras(
        raw_prompt,
        gguf_chat_template,
        family,
        force_raw,
        &JinjaExtras::default(),
    )
}

/// Same as [`render_completion_prompt`] with Jinja extras (model hint for thinking).
pub fn render_completion_prompt_with_extras(
    raw_prompt: &str,
    gguf_chat_template: Option<&str>,
    family: TemplateFamily,
    force_raw: bool,
    extras: &JinjaExtras,
) -> RenderedPrompt {
    if force_raw || raw_prompt.is_empty() {
        return RenderedPrompt {
            text: raw_prompt.to_string(),
            source: RenderSource::Raw,
        };
    }

    if gguf_chat_template.is_some() {
        return render_chat_prompt_with_extras(
            gguf_chat_template,
            family,
            None,
            &[],
            Some(raw_prompt),
            extras,
        );
    }

    // No GGUF Jinja: only family-wrap if the model actually has a chat family
    // (ChatML/Llama3). OpenChat = no real chat structure → true raw completion
    // (base/code models like starcoder2, phi-2). This is the behavior change
    // that makes base models render RAW instead of a bogus "role: content" wrap.
    if family == TemplateFamily::OpenChat {
        return RenderedPrompt {
            text: raw_prompt.to_string(),
            source: RenderSource::Raw,
        };
    }
    let text = family.render(None, &[], Some(raw_prompt));
    RenderedPrompt {
        text,
        source: RenderSource::FamilyFallback,
    }
}

fn thinking_policy(extras: &JinjaExtras) -> (Option<bool>, &'static str) {
    if let Some(v) = extras.enable_thinking {
        return (Some(v), "explicit");
    }
    let hint = extras
        .model_hint
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    // Reasoning families often default-open <think>; keep no-think unless overridden.
    if hint.contains("qwen")
        || hint.contains("qwq")
        || hint.contains("deepseek-r1")
        || hint.contains("deepseek_r1")
    {
        (Some(false), "heuristic_no_think")
    } else {
        (None, "template_default")
    }
}

fn render_via_jinja(
    template: &str,
    system: Option<&str>,
    messages: &[(String, String)],
    user_input: Option<&str>,
    extras: &JinjaExtras,
) -> Result<String, String> {
    let (enable_thinking, _policy) = thinking_policy(extras);

    let mut msgs: Vec<ChatMessage> = Vec::new();
    if let Some(sys) = system {
        if !sys.is_empty() {
            msgs.push(ChatMessage {
                role: "system".into(),
                content: sys.into(),
            });
        }
    }
    for (role, content) in messages {
        msgs.push(ChatMessage {
            role: role.clone(),
            content: content.clone(),
        });
    }
    if let Some(inp) = user_input {
        let dup = msgs
            .last()
            .map(|m| m.role == "user" && m.content == inp)
            .unwrap_or(false);
        if !dup {
            if msgs.last().map(|m| m.role.as_str()) == Some("user") {
                if let Some(last) = msgs.last_mut() {
                    last.content = inp.to_string();
                }
            } else {
                msgs.push(ChatMessage {
                    role: "user".into(),
                    content: inp.into(),
                });
            }
        }
    }

    // Match airframe shimmy_server_gpu: append /nothink when thinking disabled.
    if matches!(enable_thinking, Some(false)) {
        for m in &mut msgs {
            if m.role != "assistant" && !m.content.ends_with("/nothink") {
                m.content.push_str("\n/nothink");
            }
        }
    }

    let bos = extras.bos_token.as_deref().unwrap_or("<s>");
    let eos = extras.eos_token.as_deref().unwrap_or("</s>");

    let mut ctx = RenderContext::new();
    ctx.set_var("bos_token", bos);
    ctx.set_var("eos_token", eos);
    ctx.set_flag("add_generation_prompt", true);
    if let Some(t) = enable_thinking {
        ctx.set_flag("enable_thinking", t);
    }

    // Guard against unsupported template constructs that panic inside minijinja.
    let template = template.to_string();
    let result = catch_unwind(AssertUnwindSafe(|| {
        try_render_chat_template_with_context(&template, &msgs, &ctx)
    }));

    match result {
        Ok(Ok(s)) => Ok(s),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("shimmyjinja panicked during render".into()),
    }
}

/// Infer TemplateFamily from registry template name or model name.
pub fn family_from_spec(template: Option<&str>, model_name: &str) -> TemplateFamily {
    match template.map(|s| s.to_lowercase()) {
        Some(ref t) if t == "chatml" => TemplateFamily::ChatML,
        Some(ref t) if t == "llama3" || t == "llama-3" => TemplateFamily::Llama3,
        Some(ref t) if t == "openchat" => TemplateFamily::OpenChat,
        Some(ref t) if t == "gemma" => TemplateFamily::Gemma,
        _ => {
            let n = model_name.to_lowercase();
            if n.contains("llama-3") || n.contains("llama3") || n.contains("meta-llama-3") {
                TemplateFamily::Llama3
            } else if n.contains("qwen") || n.contains("chatglm") || n.contains("phi") {
                TemplateFamily::ChatML
            } else if n.contains("gemma") {
                TemplateFamily::Gemma
            } else {
                TemplateFamily::OpenChat
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_force_skips_template() {
        let r = render_completion_prompt(
            "The capital of France is",
            Some("{{ messages }}"),
            TemplateFamily::ChatML,
            true,
        );
        assert_eq!(r.source, RenderSource::Raw);
        assert_eq!(r.text, "The capital of France is");
    }

    #[test]
    fn family_fallback_chatml_wraps_user() {
        let r = render_completion_prompt("hi", None, TemplateFamily::ChatML, false);
        assert_eq!(r.source, RenderSource::FamilyFallback);
        assert!(r.text.contains("<|im_start|>user"));
        assert!(r.text.contains("hi"));
        assert!(r.text.contains("<|im_start|>assistant"));
    }

    #[test]
    fn jinja_simple_template() {
        let tpl = "{% for message in messages %}{{ message.role }}: {{ message.content }}\n{% endfor %}{% if add_generation_prompt %}assistant: {% endif %}";
        let r = render_chat_prompt(Some(tpl), TemplateFamily::ChatML, None, &[], Some("Hello"));
        assert_eq!(r.source, RenderSource::GgufJinja);
        assert!(r.text.contains("user: Hello"));
        assert!(r.text.contains("assistant:"));
    }

    #[test]
    fn qwen_hint_appends_nothink() {
        let tpl = "{% for message in messages %}{{ message.role }}: {{ message.content }}\n{% endfor %}{% if add_generation_prompt %}assistant: {% endif %}";
        let extras = JinjaExtras::for_model("qwen3-4b-q4-k-m");
        let r = render_completion_prompt_with_extras(
            "The capital of France is",
            Some(tpl),
            TemplateFamily::ChatML,
            false,
            &extras,
        );
        assert_eq!(r.source, RenderSource::GgufJinja);
        assert!(
            r.text.contains("/nothink"),
            "expected /nothink for qwen hint, got: {}",
            r.text
        );
    }

    #[test]
    fn non_qwen_no_nothink() {
        let tpl = "{% for message in messages %}{{ message.role }}: {{ message.content }}\n{% endfor %}{% if add_generation_prompt %}assistant: {% endif %}";
        let extras = JinjaExtras::for_model("tinyllama-1.1b");
        let r = render_completion_prompt_with_extras(
            "hi",
            Some(tpl),
            TemplateFamily::ChatML,
            false,
            &extras,
        );
        assert_eq!(r.source, RenderSource::GgufJinja);
        assert!(!r.text.contains("/nothink"));
    }

    #[test]
    fn family_chat_messages_add_generation_prompt() {
        // API-style: messages only, no user_input — the family renderer emits the
        // user turn but does NOT append an assistant marker (no input to prompt).
        // The correct generation-prompt behavior is driven by input (see
        // render_completion / api paths); this test pins the family path's contract.
        let msgs = vec![("user".into(), "Hello".into())];
        let r = render_chat_prompt(None, TemplateFamily::ChatML, None, &msgs, None);
        assert_eq!(r.source, RenderSource::FamilyFallback);
        assert!(
            r.text.contains("<|im_start|>user\nHello<|im_end|>"),
            "family path must emit the user turn; got: {}",
            r.text
        );
        // With user_input the assistant marker IS appended (generation prompt).
        let r2 = render_chat_prompt(None, TemplateFamily::ChatML, None, &msgs, Some("Hello"));
        assert!(
            r2.text.contains("<|im_start|>assistant"),
            "with input the family path must add the assistant marker; got: {}",
            r2.text
        );
    }

    // ── decide() — the single render-decision point ─────────────────────────

    #[test]
    fn decide_gguf_template_wins() {
        let src = decide(
            Some("{% for m in messages %}{{ m.role }}: {{ m.content }}{% endfor %}"),
            Some("chatml"),
            "qwen3-0.6b",
            RenderMode::Auto,
        );
        assert_eq!(src, RenderSource::GgufJinja);
    }

    #[test]
    fn decide_no_template_base_model_is_raw() {
        // starcoder2/phi-2: no GGUF template, no registry template -> RAW, not OpenChat wrap.
        let src = decide(None, None, "starcoder2-3b", RenderMode::Auto);
        assert_eq!(src, RenderSource::Raw);
    }

    #[test]
    fn decide_registry_family_fallback() {
        // No GGUF template but registry says llama3 -> FamilyFallback (not raw).
        let src = decide(None, Some("llama3"), "llama-3.2-1b", RenderMode::Auto);
        assert_eq!(src, RenderSource::FamilyFallback);
    }

    #[test]
    fn decide_force_raw_overrides_template() {
        let src = decide(
            Some("{% for m in messages %}{{ m.role }}{% endfor %}"),
            Some("chatml"),
            "qwen3-0.6b",
            RenderMode::ForceRaw,
        );
        assert_eq!(src, RenderSource::Raw);
    }

    #[test]
    fn decide_force_template_without_gguf_is_family() {
        let src = decide(
            None,
            Some("chatml"),
            "qwen2-0.5b",
            RenderMode::ForceTemplate,
        );
        assert_eq!(src, RenderSource::FamilyFallback);
    }

    #[test]
    fn decide_empty_template_treated_as_none() {
        let src = decide(Some("   "), None, "base-model", RenderMode::Auto);
        assert_eq!(src, RenderSource::Raw);
    }

    // ── completion no-template -> RAW (base models) ─────────────────────────

    #[test]
    fn completion_openchat_family_is_raw() {
        // Base model: OpenChat family with no GGUF template -> raw, not "role: content" wrap.
        let r = render_completion_prompt("fn main() {}", None, TemplateFamily::OpenChat, false);
        assert_eq!(r.source, RenderSource::Raw);
        assert_eq!(r.text, "fn main() {}");
    }

    #[test]
    fn completion_chatml_family_wraps_when_no_gguf() {
        // Instruct model with ChatML fallback (no GGUF template) -> family wrap.
        let r = render_completion_prompt("Hello", None, TemplateFamily::ChatML, false);
        assert_eq!(r.source, RenderSource::FamilyFallback);
        assert!(r.text.contains("<|im_start|>user"));
    }

    // ── Gemma family fallback ───────────────────────────────────────────────

    #[test]
    fn gemma_family_renders_start_of_turn() {
        // Gemma model with no GGUF template (or template render failure) -> Gemma family wrap.
        let msgs = vec![("user".into(), "Hi".into())];
        let r = render_chat_prompt(None, TemplateFamily::Gemma, None, &msgs, None);
        assert_eq!(r.source, RenderSource::FamilyFallback);
        assert!(
            r.text.contains("<start_of_turn>user\nHi<end_of_turn>"),
            "gemma user turn; got: {}",
            r.text
        );
        assert!(
            r.text.contains("<start_of_turn>model\n"),
            "gemma gen prompt; got: {}",
            r.text
        );
    }

    #[test]
    fn gemma_assistant_rendered_as_model() {
        let msgs = vec![
            ("user".into(), "Hi".into()),
            ("assistant".into(), "Hello".into()),
        ];
        let r = render_chat_prompt(None, TemplateFamily::Gemma, None, &msgs, None);
        assert!(
            r.text.contains("<start_of_turn>model\nHello<end_of_turn>"),
            "assistant role must render as model; got: {}",
            r.text
        );
    }

    #[test]
    fn decide_gemma_no_template_is_family_not_raw() {
        // Gemma is an instruct family; with no GGUF template it should fall back
        // to the Gemma family wrap, NOT raw (and NOT OpenChat).
        let src = decide(None, None, "gemma-2-2b-it", RenderMode::Auto);
        assert_eq!(src, RenderSource::FamilyFallback);
        assert_eq!(family_from_spec(None, "gemma-4-e4b"), TemplateFamily::Gemma);
    }
}
