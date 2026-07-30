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
        let last_is_user = messages
            .last()
            .map(|(r, _)| r == "user")
            .unwrap_or(false);
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

    // No GGUF Jinja: still apply family chat wrap so instruct models get structure
    // unless force_raw.
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
        _ => {
            let n = model_name.to_lowercase();
            if n.contains("llama-3") || n.contains("llama3") || n.contains("meta-llama-3") {
                TemplateFamily::Llama3
            } else if n.contains("qwen") || n.contains("chatglm") || n.contains("phi") {
                TemplateFamily::ChatML
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
        let r = render_chat_prompt(
            Some(tpl),
            TemplateFamily::ChatML,
            None,
            &[],
            Some("Hello"),
        );
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
        // API-style: messages only, no user_input — must end with assistant turn marker
        let msgs = vec![("user".into(), "Hello".into())];
        let r = render_chat_prompt(None, TemplateFamily::ChatML, None, &msgs, None);
        assert_eq!(r.source, RenderSource::FamilyFallback);
        assert!(
            r.text.contains("<|im_start|>assistant"),
            "family path must add generation prompt; got: {}",
            r.text
        );
    }
}
