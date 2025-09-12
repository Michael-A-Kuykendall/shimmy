// Generation Performance Benchmarks
// Measures performance of text generation and API processing

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shimmy::invariant_ppt::shimmy_invariants::*;
use shimmy::templates::*;
use std::time::Duration;

fn benchmark_template_rendering(c: &mut Criterion) {
    c.bench_function("chat_template_rendering", |b| {
        b.iter(|| {
            let prompt = black_box("What is the meaning of life?");
            let template = r#"<|user|>
{{prompt}}
<|assistant|>
"#;

            // Simulate template rendering (simplified)
            let rendered = template.replace("{{prompt}}", prompt);
            black_box(rendered)
        })
    });
}

fn benchmark_invariant_checking(c: &mut Criterion) {
    c.bench_function("generation_invariants", |b| {
        b.iter(|| {
            let prompt = black_box("Hello world");
            let response = black_box("Hello! How can I help you today?");

            // This will log invariants but not panic in benchmarks
            std::panic::catch_unwind(|| {
                assert_generation_valid(prompt, response);
            })
            .unwrap_or_default();
        })
    });

    c.bench_function("api_response_invariants", |b| {
        b.iter(|| {
            let status = black_box(200u16);
            let body = black_box(r#"{"response": "Generated text"}"#);

            std::panic::catch_unwind(|| {
                assert_api_response_valid(status, body);
            })
            .unwrap_or_default();
        })
    });
}

fn benchmark_response_processing(c: &mut Criterion) {
    let sample_responses = vec![
        "Short response.",
        "This is a medium length response that contains more detail and explanation.",
        "This is a very long response that simulates the kind of detailed, comprehensive answer that an AI model might generate when asked a complex question. It includes multiple sentences, various concepts, and demonstrates the kind of text processing that would be typical in a real-world scenario.",
    ];

    c.bench_function("response_length_calculation", |b| {
        b.iter(|| {
            for response in &sample_responses {
                let length = black_box(response).len();
                let word_count = black_box(response).split_whitespace().count();
                black_box((length, word_count));
            }
        })
    });
}

fn benchmark_json_processing(c: &mut Criterion) {
    let sample_request = r#"{
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "What is AI?"}
        ],
        "max_tokens": 100,
        "temperature": 0.7
    }"#;

    c.bench_function("json_parsing", |b| {
        b.iter(
            || match serde_json::from_str::<serde_json::Value>(black_box(sample_request)) {
                Ok(parsed) => black_box(parsed),
                Err(_) => serde_json::Value::Null,
            },
        )
    });

    let sample_response = serde_json::json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "AI stands for Artificial Intelligence."
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 9,
            "completion_tokens": 8,
            "total_tokens": 17
        }
    });

    c.bench_function("json_serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&black_box(&sample_response));
            black_box(serialized)
        })
    });
}

criterion_group!(
    benches,
    benchmark_template_rendering,
    benchmark_invariant_checking,
    benchmark_response_processing,
    benchmark_json_processing
);
criterion_main!(benches);
