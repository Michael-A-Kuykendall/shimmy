use shimmy::inference_error::classify;
#[test]
fn prompt_and_residency_errors_keep_their_identity_through_context() {
    let e = anyhow::Error::new(shimmy::prepared_prompt::PromptError::Identity).context("server");
    assert_eq!(classify(&e).status.as_u16(), 409);
    assert_eq!(classify(&e).code, "prepared_prompt_model_mismatch");
    let e = anyhow::Error::new(shimmy::engine::ResidentModelConflict).context("load");
    assert_eq!(classify(&e).status.as_u16(), 409);
}
#[cfg(feature = "airframe")]
#[test]
fn admission_and_runtime_errors_do_not_collapse_into_bad_gateway() {
    use airframe::execution::admission::{ExecutionError, ModelAdmissionError};
    let e = anyhow::Error::new(ModelAdmissionError::Configuration {
        code: "unsupported_architecture",
        detail: "moe".into(),
    })
    .context("load");
    assert_eq!(classify(&e).status.as_u16(), 422);
    assert_eq!(classify(&e).code, "unsupported_architecture");
    let e = anyhow::Error::new(ModelAdmissionError::Device {
        code: "gpu_out_of_memory",
        detail: "allocation".into(),
    });
    assert_eq!(classify(&e).status.as_u16(), 503);
    assert!(!classify(&e).message.contains("allocation"));
    let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(ExecutionError::Request {
        code: "context_length_exceeded",
        detail: "too long".into(),
    });
    let e = shimmy::inference_error::from_boxed(boxed);
    assert_eq!(classify(&e).status.as_u16(), 400);
    assert_eq!(classify(&e).code, "context_length_exceeded");
    let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(ExecutionError::Scheduler {
        code: "isf_budget_exhausted",
        detail: "private graph counters".into(),
    });
    let e = shimmy::inference_error::from_boxed(boxed).context("generation");
    assert_eq!(classify(&e).status.as_u16(), 503);
    assert_eq!(classify(&e).code, "isf_budget_exhausted");
    assert!(!classify(&e).message.contains("private graph counters"));
}
#[test]
fn strict_chat_never_falls_back_to_a_family_template() {
    let extras = Default::default();
    assert!(
        shimmy::prompt_render::render_chat_prompt_strict(None, None, &[], None, &extras).is_err()
    );
    assert!(shimmy::prompt_render::render_chat_prompt_strict(
        Some("{{ missing.call() }}"),
        None,
        &[],
        None,
        &extras
    )
    .is_err());
}
