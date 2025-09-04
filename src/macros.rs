/// Declarative model configuration macro
/// 
/// # Example
/// ```rust
/// use shimmy::model_config;
/// 
/// let config = model_config! {
///     name: "phi3-mini",
///     backend: LlamaGGUF {
///         base_path: "./models/phi3-mini.gguf",
///         lora_path: Some("./adapters/phi3-lora.gguf"),
///     },
///     template: "ChatML",
///     ctx_len: 4096,
///     device: "cpu",
///     generation: {
///         max_tokens: 1024,
///         temperature: 0.7,
///         top_p: 0.9,
///         top_k: 40,
///     }
/// };
/// ```
#[macro_export]
macro_rules! model_config {
    {
        name: $name:expr,
        backend: LlamaGGUF {
            base_path: $base:expr,
            lora_path: $lora:expr,
        },
        template: $template:expr,
        ctx_len: $ctx_len:expr,
        device: $device:expr,
        generation: {
            max_tokens: $max_tokens:expr,
            temperature: $temp:expr,
            top_p: $top_p:expr,
            top_k: $top_k:expr,
        }
    } => {
        $crate::engine::UniversalModelSpec {
            name: $name.to_string(),
            backend: $crate::engine::ModelBackend::LlamaGGUF {
                base_path: std::path::PathBuf::from($base),
                lora_path: $lora.map(std::path::PathBuf::from),
            },
            template: Some($template.to_string()),
            ctx_len: $ctx_len,
            device: $device.to_string(),
            n_threads: None,
        }
    };
    
    {
        name: $name:expr,
        backend: HuggingFace {
            base_model_id: $model_id:expr,
            peft_path: $peft:expr,
            use_local: $local:expr,
        },
        template: $template:expr,
        ctx_len: $ctx_len:expr,
        device: $device:expr,
    } => {
        $crate::engine::UniversalModelSpec {
            name: $name.to_string(),
            backend: $crate::engine::ModelBackend::HuggingFace {
                base_model_id: $model_id.to_string(),
                peft_path: $peft.map(std::path::PathBuf::from),
                use_local: $local,
            },
            template: Some($template.to_string()),
            ctx_len: $ctx_len,
            device: $device.to_string(),
            n_threads: None,
        }
    };
}

/// Macro for creating generation options with validation
#[macro_export]
macro_rules! gen_options {
    {
        max_tokens: $max:expr,
        temperature: $temp:expr,
        top_p: $top_p:expr,
        top_k: $top_k:expr,
        $(repeat_penalty: $penalty:expr,)?
        $(seed: $seed:expr,)?
        $(stream: $stream:expr,)?
    } => {
        $crate::engine::GenOptions {
            max_tokens: $max,
            temperature: $temp,
            top_p: $top_p,
            top_k: $top_k,
            repeat_penalty: gen_options!(@default_or $($penalty,)? 1.1),
            seed: gen_options!(@option $($seed)?),
            stream: gen_options!(@default_or $($stream,)? false),
        }
    };
    
    (@default_or $value:expr, $default:expr) => { $value };
    (@default_or $default:expr) => { $default };
    (@option $value:expr) => { Some($value) };
    (@option) => { None };
}

/// Template rendering macro with compile-time format checking
#[macro_export]
macro_rules! render_template {
    (ChatML, system: $system:expr, messages: $messages:expr) => {
        $crate::templates::TemplateFamily::ChatML.render(Some($system), $messages, None)
    };
    
    (ChatML, messages: $messages:expr, input: $input:expr) => {
        $crate::templates::TemplateFamily::ChatML.render(None, $messages, Some($input))
    };
    
    (Llama3, system: $system:expr, messages: $messages:expr) => {
        $crate::templates::TemplateFamily::Llama3.render(Some($system), $messages, None)
    };
}
