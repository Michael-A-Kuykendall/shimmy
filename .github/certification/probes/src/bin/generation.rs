//! Serial numerical probe of the production ISF generation loop.
use airframe::backend::bindless::pipeline::inference::{
    clear_stack_layer_capture_sink, set_stack_layer_capture_sink, StageSnap,
};
use airframe::control::{ControlDecision, InferenceControl, InferenceEvent};
use airframe::runtime::gpu::{GpuRuntime, SamplingParams};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
struct Capture {
    events: Arc<Mutex<Vec<Value>>>,
    forced: Vec<usize>,
}
impl InferenceControl for Capture {
    fn intervene(&self, e: &InferenceEvent<'_>) -> ControlDecision {
        self.events.lock().unwrap().push(json!({"step":e.step,"candidate":e.candidate_token,"tokens":e.tokens,"kv_len":e.kv.len}));
        self.forced
            .get(self.events.lock().unwrap().len() - 1)
            .map(|t| ControlDecision::ForceToken(*t))
            .unwrap_or(ControlDecision::Allow)
    }
}
fn stage(s: &StageSnap) -> Value {
    json!({"capture_valid":s.name!="ffn_norm","excluded_reason":if s.name=="ffn_norm"{Some("airframe-ate.16 observation fix is a separate unmerged branch")}else{None},"name":s.name,"values":s.values,"first8":s.first8,"rms":s.rms,"count":s.count,"buffer":s.buffer,"offset_elems":s.offset_elems})
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 4 {
        return Err("usage: generation_differential_probe MODEL CASES.json OUTPUT.json".into());
    }
    let cases: Value = serde_json::from_slice(&std::fs::read(&a[2])?)?;
    let metadata = shimmytok::gguf::load_metadata(&a[1])?;
    let runtime = GpuRuntime::load(std::path::Path::new(&a[1])).await?;
    let mut results = Vec::new();
    let logits_only = std::env::var("AIRFRAME_CERT_LOGITS_ONLY").ok().as_deref() == Some("1");
    for case in cases.as_array().ok_or("cases must be array")? {
        let rendered;
        let prompt = if let Some(prompt) = case["prompt"].as_str() {
            prompt
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
                metadata
                    .chat_template
                    .as_ref()
                    .ok_or("GGUF template missing")?,
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
        let forced: Vec<usize> = case
            .get("forced")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_default();
        let max_tokens = case["max_tokens"].as_u64().unwrap_or(8) as usize;
        let params = SamplingParams {
            max_tokens,
            temperature: 0.0,
            top_p: 1.0,
            repetition_penalty: 1.0,
            seed: 0,
            extra_stop_tokens: vec![],
        };
        // A text call, an injected-ID call and a repeat all share this one loaded runtime.
        let modes: Vec<String> = case
            .get("modes")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_else(|| vec!["text".into(), "injected".into(), "repeat".into()]);
        if modes.is_empty()
            || modes
                .iter()
                .any(|m| !["text", "injected", "repeat"].contains(&m.as_str()))
        {
            return Err("invalid diagnostic request modes".into());
        }
        for mode in modes {
            let logits = Arc::new(Mutex::new(Vec::<Vec<f32>>::new()));
            let log_sink = logits.clone();
            let events = Arc::new(Mutex::new(Vec::new()));
            let mut layers = Vec::new();
            if !logits_only {
                set_stack_layer_capture_sink(&mut layers);
            }
            let mask = Some(
                Box::new(move |v: &mut [f32]| log_sink.lock().unwrap().push(v.to_vec()))
                    as Box<dyn Fn(&mut [f32]) + Send + Sync>,
            );
            let control = Some(Box::new(Capture {
                events: events.clone(),
                forced: forced.clone(),
            }) as Box<dyn InferenceControl + Send + Sync>);
            let result = if mode == "text" {
                runtime.generate_with_metadata(prompt, &params, None, control, mask, None)
            } else {
                runtime
                    .generate_token_ids_with_metadata(&injected, &params, None, control, mask, None)
            };
            if !logits_only {
                clear_stack_layer_capture_sink();
            }
            let result = result?;
            let layer_values:Vec<_>=layers.iter().map(|s|json!({"layer":s.layer_idx,"position":s.position,"residual_rms":s.rms,"residual_first8":s.first8,"residual_in":s.residual_in.as_ref().map(stage),"stages":s.stages.iter().map(stage).collect::<Vec<_>>()})).collect();
            results.push(json!({"case":case["name"],"mode":mode,"prompt":prompt,"encoded_ids":encoded,"injected_ids":injected,"forced":forced,
     "text":result.text,"prompt_tokens":result.prompt_tokens,"completion_tokens":result.completion_tokens,"finish_reason":format!("{:?}",result.finish_reason),
     "events":*events.lock().unwrap(),"logits":*logits.lock().unwrap(),"layers":layer_values}));
            std::fs::write(
                &a[3],
                serde_json::to_vec(
                    &json!({"schema":"airframe.differential.v1","model":a[1],"model_sha256_at_load":runtime.model_identity().model_sha256,"complete":false,"logits_only":logits_only,"results":results}),
                )?,
            )?;
        }
    }
    std::fs::write(
        &a[3],
        serde_json::to_vec(
            &json!({"schema":"airframe.differential.v1","model":a[1],"model_sha256_at_load":runtime.model_identity().model_sha256,"complete":true,"logits_only":logits_only,"results":results}),
        )?,
    )?;
    Ok(())
}
