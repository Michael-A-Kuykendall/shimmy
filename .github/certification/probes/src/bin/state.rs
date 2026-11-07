use airframe::{
    control::{ControlDecision, InferenceControl, InferenceEvent},
    execution::admission::ExecutionError,
    runtime::gpu::{GpuRuntime, SamplingParams},
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Weak,
};
struct ForceInvalid;
impl InferenceControl for ForceInvalid {
    fn intervene(&self, _: &InferenceEvent) -> ControlDecision {
        ControlDecision::ForceToken(usize::MAX)
    }
}
struct Cancel;
impl InferenceControl for Cancel {
    fn intervene(&self, _: &InferenceEvent) -> ControlDecision {
        ControlDecision::EarlyExit
    }
}
struct ResetDuringRequest {
    runtime: Weak<GpuRuntime>,
    busy: Arc<AtomicBool>,
}
impl InferenceControl for ResetDuringRequest {
    fn intervene(&self, event: &InferenceEvent) -> ControlDecision {
        assert_eq!(event.step, 0);
        let runtime = self.runtime.upgrade().unwrap();
        let error = runtime.try_reset().unwrap_err();
        assert!(matches!(
            error.downcast_ref::<ExecutionError>(),
            Some(ExecutionError::State(
                airframe::runtime::state::StateError::Busy
            ))
        ));
        #[allow(deprecated)]
        let legacy = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.reset()));
        assert!(
            legacy.is_err(),
            "legacy reset must surface failure instead of silently succeeding"
        );
        self.busy.store(true, Ordering::Release);
        ControlDecision::Allow
    }
}
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);
    let waker = std::task::Waker::noop();
    let mut context = std::task::Context::from_waker(waker);
    loop {
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Ready(value) => return value,
            std::task::Poll::Pending => std::thread::sleep(std::time::Duration::from_millis(1)),
        }
    }
}
fn main() {
    let path = std::path::PathBuf::from(
        std::env::var("AIRFRAME_QWEN_FIXTURE").expect("exact Qwen fixture"),
    );
    let runtime = Arc::new(block_on(GpuRuntime::load_with_context(&path, Some(128))).unwrap());
    let params = SamplingParams {
        max_tokens: 1,
        temperature: 0.0,
        top_p: 1.0,
        repetition_penalty: 1.0,
        seed: 0,
        extra_stop_tokens: vec![],
    };
    let run = || {
        runtime
            .generate_with_metadata("Hello", &params, None, None, None, None)
            .unwrap()
    };
    let expected = run();
    let error = runtime
        .generate_with_metadata(
            "Hello",
            &params,
            None,
            Some(Box::new(ForceInvalid)),
            None,
            None,
        )
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<ExecutionError>(),
        Some(ExecutionError::Request {
            code: "invalid_control_token",
            ..
        })
    ));
    assert_eq!(run().text, expected.text);
    let error = runtime
        .generate_with_metadata(
            "Hello",
            &params,
            None,
            None,
            Some(Box::new(|v| v[0] = f32::NAN)),
            None,
        )
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<ExecutionError>(),
        Some(ExecutionError::Request {
            code: "invalid_logit_mask",
            ..
        })
    ));
    assert_eq!(run().text, expected.text);
    let stopped = runtime
        .generate_with_metadata("Hello", &params, None, Some(Box::new(Cancel)), None, None)
        .unwrap();
    assert_eq!(stopped.completion_tokens, 0);
    assert_eq!(run().text, expected.text);
    let busy = Arc::new(AtomicBool::new(false));
    runtime
        .generate_with_metadata(
            "Hello",
            &params,
            None,
            Some(Box::new(ResetDuringRequest {
                runtime: Arc::downgrade(&runtime),
                busy: busy.clone(),
            })),
            None,
            None,
        )
        .unwrap();
    assert!(busy.load(Ordering::Acquire));
    runtime.try_reset().unwrap();
    assert_eq!(run().text, expected.text);
    let left = runtime.clone();
    let right = runtime.clone();
    let p = params.clone();
    let q = params.clone();
    let l = std::thread::spawn(move || {
        left.generate_with_metadata("Hello", &p, None, None, None, None)
            .unwrap()
            .text
    });
    let r = std::thread::spawn(move || {
        right
            .generate_with_metadata("Hello", &q, None, None, None, None)
            .unwrap()
            .text
    });
    assert_eq!(l.join().unwrap(), expected.text);
    assert_eq!(r.join().unwrap(), expected.text);
    println!("STATE_PROBE_PASS");
}
