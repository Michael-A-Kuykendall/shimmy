//! Dense-only numerical/boundary probe; production admission unchanged.
use airframe::control::{ControlDecision, InferenceControl, InferenceEvent};
use airframe::execution::admission::ExecutionError;
use airframe::runtime::gpu::{GpuRuntime, SamplingParams};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
struct Capture(Arc<Mutex<Vec<Value>>>);
impl InferenceControl for Capture {
    fn intervene(&self, e: &InferenceEvent<'_>) -> ControlDecision {
        self.0.lock().unwrap().push(json!({"step":e.step,"candidate":e.candidate_token,"tokens":e.tokens,"kv_len":e.kv.len}));
        ControlDecision::Allow
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 4 {
        return Err("usage: dense_closure_probe MODEL CASES OUTPUT".into());
    }
    let cases: Value = serde_json::from_slice(&std::fs::read(&a[2])?)?;
    let metadata = shimmytok::gguf::load_metadata(&a[1])?;
    let runtime = GpuRuntime::load(std::path::Path::new(&a[1])).await?;
    let mut results = Vec::new();
    for case in cases.as_array().ok_or("cases must be array")? {
        let rendered;
        let prompt = if let Some(p) = case["prompt"].as_str() {
            p
        } else {
            let messages: Vec<shimmyjinja::ChatMessage> = case["messages"]
                .as_array()
                .ok_or("messages missing")?
                .iter()
                .map(|m| {
                    Ok(shimmyjinja::ChatMessage {
                        role: m["role"].as_str().ok_or("role missing")?.to_string(),
                        content: m["content"].as_str().ok_or("content missing")?.to_string(),
                    })
                })
                .collect::<Result<_, &str>>()?;
            let mut ctx = shimmyjinja::RenderContext::new();
            ctx.set_flag("add_generation_prompt", true);
            if let Some(thinking) = case["enable_thinking"].as_bool() {
                ctx.set_flag("enable_thinking", thinking);
            }
            ctx.set_var("bos_token", "");
            ctx.set_var("eos_token", "<|im_end|>");
            rendered = shimmyjinja::try_render_chat_template_with_context(
                metadata.chat_template.as_ref().ok_or("missing template")?,
                &messages,
                &ctx,
            )?;
            &rendered
        };
        let encoded = runtime.tokenizer().encode_with_options(
            prompt,
            &shimmytok::EncodeOptions {
                add_special_tokens: true,
                parse_special: true,
            },
        )?;
        let injected: Vec<u32> = case
            .get("ids")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_else(|| encoded.clone());
        if encoded != injected {
            return Err("independent input IDs differ".into());
        }
        let params = SamplingParams {
            max_tokens: case["max_tokens"].as_u64().unwrap_or(8) as usize,
            temperature: 0.0,
            top_p: 1.0,
            repetition_penalty: 1.0,
            seed: 0,
            extra_stop_tokens: vec![],
        };
        let modes: Vec<String> = case
            .get("modes")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_else(|| vec!["text".into()]);
        for mode in modes {
            if !["text", "injected", "repeat"].contains(&mode.as_str()) {
                return Err("invalid mode".into());
            }
            let logits = Arc::new(Mutex::new(Vec::<Vec<f32>>::new()));
            let sink = logits.clone();
            let events = Arc::new(Mutex::new(Vec::new()));
            let control =
                Some(Box::new(Capture(events.clone())) as Box<dyn InferenceControl + Send + Sync>);
            let mask = Some(
                Box::new(move |v: &mut [f32]| sink.lock().unwrap().push(v.to_vec()))
                    as Box<dyn Fn(&mut [f32]) + Send + Sync>,
            );
            let output = if mode == "text" {
                runtime.generate_with_metadata(prompt, &params, None, control, mask, None)
            } else {
                runtime
                    .generate_token_ids_with_metadata(&injected, &params, None, control, mask, None)
            };
            let row = if let Some(expected) = case["expected_error"].as_str() {
                let error = output.expect_err("over-capacity request was admitted");
                let code = match error.downcast_ref::<ExecutionError>() {
                    Some(ExecutionError::Request { code, .. }) => *code,
                    other => return Err(format!("wrong typed error: {other:?}").into()),
                };
                assert_eq!(code, expected);
                assert!(logits.lock().unwrap().is_empty());
                assert!(events.lock().unwrap().is_empty());
                json!({"case":case["name"],"mode":mode,"error_code":code,"error":error.to_string(),"events":*events.lock().unwrap(),"logits":*logits.lock().unwrap(),"encoded_ids":encoded,"injected_ids":injected})
            } else {
                let output = output?;
                json!({"case":case["name"],"mode":mode,"prompt":prompt,"encoded_ids":encoded,"injected_ids":injected,"text":output.text,"prompt_tokens":output.prompt_tokens,"completion_tokens":output.completion_tokens,"finish_reason":format!("{:?}",output.finish_reason),"events":*events.lock().unwrap(),"logits":*logits.lock().unwrap()})
            };
            results.push(row);
            std::fs::write(
                &a[3],
                serde_json::to_vec(
                    &json!({"schema":"airframe.dense-closure.v1","complete":false,"model":a[1],"model_sha256_at_load":runtime.model_identity().model_sha256,"results":results}),
                )?,
            )?;
        }
    }
    std::fs::write(
        &a[3],
        serde_json::to_vec(
            &json!({"schema":"airframe.dense-closure.v1","complete":true,"model":a[1],"model_sha256_at_load":runtime.model_identity().model_sha256,"results":results}),
        )?,
    )?;
    Ok(())
}
