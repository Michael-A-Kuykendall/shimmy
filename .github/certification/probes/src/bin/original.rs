use airframe::backend::bindless::pipeline::inference::{
    clear_stack_layer_capture_sink, set_stack_layer_capture_sink, StackLayerSnap, StageSnap,
};
use airframe::control::{ControlDecision, InferenceControl, InferenceEvent};
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
fn stage(s: &StageSnap) -> Value {
    let consumed = s.name != "ffn_norm";
    let producer = match s.name.as_str() {
        "attn_norm" => "main_attn_norm",
        "q" | "k" | "v" => "main_qkv",
        "q_norm" | "k_norm" => "main_qk_norm",
        "attn_ctx" => "main_attn_out",
        "attn_residual" => "main_attn_proj",
        "ffn_gate" | "ffn_up" => "main_ffn_proj",
        "ffn_residual" => "main_ffn_down",
        "ffn_norm" => "NOT_DISPATCHED_IN_DEFAULT_GATED_PATH",
        _ => "DIAGNOSTIC_READBACK",
    };
    json!({"name":s.name,"values":s.values,"count":s.count,"sampled":s.sampled,"nan_count":s.nan_count,"buffer":s.buffer,"offset_elems":s.offset_elems,"consumed":consumed,"producer":producer})
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let a: Vec<_> = std::env::args().collect();
    let cases: Vec<Value> = serde_json::from_slice(&std::fs::read(&a[2])?)?;
    assert_eq!(cases.len(), 1);
    let ids: Vec<u32> = serde_json::from_value(cases[0]["ids"].clone())?;
    assert_eq!(ids.len(), 2032);
    assert_eq!(cases[0]["max_tokens"], 4);
    let runtime = GpuRuntime::load(std::path::Path::new(&a[1])).await?;
    assert_eq!(
        runtime.encode_prompt(cases[0]["prompt"].as_str().unwrap())?,
        ids
    );
    let params = SamplingParams {
        max_tokens: 4,
        temperature: 0.0,
        top_p: 1.0,
        repetition_penalty: 1.0,
        seed: 0,
        extra_stop_tokens: vec![],
    };
    let mut rows = Vec::new();
    let modes: &[&str] = match a.get(5).map(String::as_str) {
        None => &["off", "on", "repeat"],
        Some("on-only") => &["on"],
        Some("off-only") => &["off"],
        Some(_) => {
            return Err(
                "mode must be on-only or off-only; omit for same-runtime off/on/repeat".into(),
            )
        }
    };
    for &mode in modes {
        let mut layers: Vec<StackLayerSnap> = Vec::new();
        if mode != "off" {
            set_stack_layer_capture_sink(&mut layers);
        }
        let logits = Arc::new(Mutex::new(Vec::<Vec<f32>>::new()));
        let sink = logits.clone();
        let events = Arc::new(Mutex::new(Vec::new()));
        let mask = Some(
            Box::new(move |v: &mut [f32]| sink.lock().unwrap().push(v.to_vec()))
                as Box<dyn Fn(&mut [f32]) + Send + Sync>,
        );
        let control =
            Some(Box::new(Capture(events.clone())) as Box<dyn InferenceControl + Send + Sync>);
        let result =
            runtime.generate_token_ids_with_metadata(&ids, &params, None, control, mask, None);
        clear_stack_layer_capture_sink();
        let output = result?;
        let vectors = logits.lock().unwrap().clone();
        assert_eq!(vectors.len(), 4);
        if mode == "off" && !std::path::Path::new(&a[4]).is_dir() {
            let previous: Value = serde_json::from_slice(&std::fs::read(&a[4])?)?;
            let old: Vec<f32> =
                serde_json::from_value(previous["results"][0]["logits"][0].clone())?;
            assert_eq!(old.len(), vectors[0].len());
            let maximum = old
                .iter()
                .zip(&vectors[0])
                .map(|(x, y)| (x - y).abs())
                .fold(0.0_f32, f32::max);
            let rms = (old
                .iter()
                .zip(&vectors[0])
                .map(|(x, y)| f64::from(x - y).powi(2))
                .sum::<f64>()
                / old.len() as f64)
                .sqrt();
            assert!(
                maximum <= 0.1 && rms <= 0.03,
                "original failing computation not reproduced"
            );
        }
        if std::path::Path::new(&a[4]).is_dir() {
            for (step, actual) in vectors.iter().enumerate() {
                let bytes = std::fs::read(
                    std::path::Path::new(&a[4]).join(format!("request-0-step-{step}.f32")),
                )?;
                let expected: Vec<f32> = bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|b| f32::from_le_bytes(*b))
                    .collect();
                assert_eq!(actual.len(), expected.len());
                let maximum = actual
                    .iter()
                    .zip(&expected)
                    .map(|(x, y)| f64::from(*x) - f64::from(*y))
                    .map(f64::abs)
                    .fold(0.0_f64, f64::max);
                let rms = (actual
                    .iter()
                    .zip(&expected)
                    .map(|(x, y)| (f64::from(*x) - f64::from(*y)).powi(2))
                    .sum::<f64>()
                    / actual.len() as f64)
                    .sqrt();
                let argmax = |v: &[f32]| {
                    v.iter()
                        .enumerate()
                        .max_by(|a, b| a.1.total_cmp(b.1))
                        .unwrap()
                        .0
                };
                assert!(
                    maximum <= 0.1 && rms <= 0.03 && argmax(actual) == argmax(&expected),
                    "reference failure mode={mode} step={step} max={maximum} rms={rms}"
                );
                eprintln!("REFERENCE_PASS mode={mode} step={step} max={maximum} rms={rms}");
            }
        }
        if mode != "off" {
            assert_eq!(layers.len(), 28);
            assert!(layers.iter().all(|l| l.position == 2031));
        }
        rows.push(json!({"mode":mode,"logits":vectors,"events":*events.lock().unwrap(),"text":output.text,"prompt_tokens":output.prompt_tokens,"completion_tokens":output.completion_tokens,"finish_reason":format!("{:?}",output.finish_reason),"layers":layers.iter().map(|l|json!({"layer":l.layer_idx,"position":l.position,"residual_in":l.residual_in.as_ref().map(stage),"stages":l.stages.iter().map(stage).collect::<Vec<_>>()})).collect::<Vec<_>>()}));
        std::fs::write(
            &a[3],
            serde_json::to_vec(
                &json!({"complete":false,"model_sha256":runtime.model_identity().model_sha256,"ids":ids,"rows":rows}),
            )?,
        )?;
        eprintln!("REQUEST_COMPLETED {mode}");
    }
    std::fs::write(
        &a[3],
        serde_json::to_vec(
            &json!({"complete":true,"model_sha256":runtime.model_identity().model_sha256,"ids":ids,"rows":rows}),
        )?,
    )?;
    Ok(())
}
