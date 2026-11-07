use anyhow::Result;
use async_trait::async_trait;
use shimmy::engine::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};
use shimmy::model_registry::{ModelEntry, Registry};
use shimmy::warmup::{prepare_warmup, resolve_warmup_target, WarmupConfig};
use std::path::Path;
use std::sync::{Arc, Mutex};

struct Model {
    events: Arc<Mutex<Vec<&'static str>>>,
    fail: bool,
}
#[async_trait]
impl LoadedModel for Model {
    async fn generate(
        &self,
        _: &str,
        _: GenOptions,
        _: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        Ok("unused".into())
    }
    async fn warmup(&self) -> Result<()> {
        self.events.lock().unwrap().push("warmup");
        anyhow::ensure!(!self.fail, "forward failed");
        Ok(())
    }
}
struct Engine {
    events: Arc<Mutex<Vec<&'static str>>>,
    fail_load: bool,
    fail_warmup: bool,
}
#[async_trait]
impl InferenceEngine for Engine {
    async fn load(&self, _: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        self.events.lock().unwrap().push("load");
        anyhow::ensure!(!self.fail_load, "load failed");
        Ok(Box::new(Model {
            events: self.events.clone(),
            fail: self.fail_warmup,
        }))
    }
}
fn file(dir: &Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(format!("{name}.gguf"));
    let mut bytes = b"GGUF".to_vec();
    bytes.extend(3u32.to_le_bytes());
    bytes.extend(0u64.to_le_bytes());
    bytes.extend(4u64.to_le_bytes());
    for (key, value) in [
        ("llama.embedding_length", 8u32),
        ("llama.block_count", 1),
        ("llama.feed_forward_length", 16),
        ("llama.attention.head_count", 2),
    ] {
        bytes.extend((key.len() as u64).to_le_bytes());
        bytes.extend(key.as_bytes());
        bytes.extend(4u32.to_le_bytes()); // GGUF UINT32
        bytes.extend(value.to_le_bytes());
    }
    std::fs::write(&path, bytes).unwrap();
    path
}
fn register(reg: &mut Registry, name: &str, path: &Path) {
    reg.register(ModelEntry {
        name: name.into(),
        base_path: path.into(),
        lora_path: None,
        template: None,
        ctx_len: None,
        n_threads: None,
    });
}
fn enabled(name: Option<&str>) -> WarmupConfig {
    WarmupConfig::parse(Some("1"), name).unwrap()
}

#[test]
fn booleans_and_disabled_selector() {
    for flag in [None, Some("0"), Some(" false "), Some("FALSE")] {
        assert_eq!(
            WarmupConfig::parse(flag, Some("")).unwrap(),
            WarmupConfig {
                enabled: false,
                model: None
            }
        );
    }
    for flag in ["1", "true", " TRUE "] {
        assert!(WarmupConfig::parse(Some(flag), None).unwrap().enabled);
    }
    for flag in ["", "yes", "2"] {
        assert!(WarmupConfig::parse(Some(flag), None).is_err());
    }
    assert!(WarmupConfig::parse(Some("1"), Some("  ")).is_err());
}

#[test]
fn explicit_selector_and_base_precedence() {
    let dir = tempfile::tempdir().unwrap();
    let a = file(dir.path(), "a");
    let b = file(dir.path(), "b");
    let mut reg = Registry::new();
    register(&mut reg, "a", &a);
    register(&mut reg, "alias", &a);
    register(&mut reg, "b", &b);
    register(&mut reg, "tinyllama-1.1b", &b);
    assert_eq!(
        resolve_warmup_target(&reg, Some(&a), true, &enabled(None))
            .unwrap()
            .unwrap()
            .name,
        "a"
    );
    assert!(resolve_warmup_target(&reg, Some(&a), false, &enabled(Some("alias"))).is_ok());
    assert!(resolve_warmup_target(&reg, Some(&a), false, &enabled(Some("b"))).is_err());
    assert_eq!(
        resolve_warmup_target(&reg, None, true, &enabled(None))
            .unwrap()
            .unwrap()
            .base_path,
        b
    );
    assert_eq!(
        resolve_warmup_target(&reg, None, true, &enabled(Some("a")))
            .unwrap()
            .unwrap()
            .base_path,
        a
    );
    assert!(resolve_warmup_target(&reg, None, false, &enabled(None)).is_err());
    assert!(resolve_warmup_target(&reg, None, false, &enabled(Some("unknown"))).is_err());
}
#[test]
fn sole_model_missing_file_and_disabled() {
    let dir = tempfile::tempdir().unwrap();
    let a = file(dir.path(), "a");
    let mut reg = Registry::new();
    assert!(resolve_warmup_target(&reg, None, false, &enabled(None)).is_err());
    register(&mut reg, "a", &a);
    assert!(resolve_warmup_target(&reg, None, false, &enabled(None))
        .unwrap()
        .is_some());
    std::fs::remove_file(&a).unwrap();
    assert!(resolve_warmup_target(&reg, None, false, &enabled(None)).is_err());
    assert!(resolve_warmup_target(
        &reg,
        None,
        false,
        &WarmupConfig::parse(None, Some("unknown")).unwrap()
    )
    .unwrap()
    .is_none());
}
#[cfg(unix)]
#[test]
fn canonical_symlink_alias_matches() {
    let dir = tempfile::tempdir().unwrap();
    let a = file(dir.path(), "a");
    let link = dir.path().join("link.gguf");
    std::os::unix::fs::symlink(&a, &link).unwrap();
    let mut reg = Registry::new();
    register(&mut reg, "a", &a);
    register(&mut reg, "link", &link);
    assert!(resolve_warmup_target(&reg, Some(&a), false, &enabled(Some("link"))).is_ok());
}
#[tokio::test]
async fn load_warmup_before_caller_can_be_ready() {
    let dir = tempfile::tempdir().unwrap();
    let a = file(dir.path(), "a");
    let mut reg = Registry::new();
    register(&mut reg, "a", &a);
    let events = Arc::new(Mutex::new(vec![]));
    let engine = Engine {
        events: events.clone(),
        fail_load: false,
        fail_warmup: false,
    };
    prepare_warmup(
        &engine,
        &reg,
        None,
        false,
        &WarmupConfig::parse(None, None).unwrap(),
    )
    .await
    .unwrap();
    assert!(events.lock().unwrap().is_empty());
    prepare_warmup(&engine, &reg, None, false, &enabled(None))
        .await
        .unwrap();
    events.lock().unwrap().push("ready");
    assert_eq!(*events.lock().unwrap(), ["load", "warmup", "ready"]);
}
#[tokio::test]
async fn failures_do_not_reach_ready() {
    let dir = tempfile::tempdir().unwrap();
    let a = file(dir.path(), "a");
    let mut reg = Registry::new();
    register(&mut reg, "a", &a);
    for (fail_load, fail_warmup, expected) in [
        (true, false, vec!["load"]),
        (false, true, vec!["load", "warmup"]),
    ] {
        let events = Arc::new(Mutex::new(vec![]));
        let engine = Engine {
            events: events.clone(),
            fail_load,
            fail_warmup,
        };
        assert!(prepare_warmup(&engine, &reg, None, false, &enabled(None))
            .await
            .is_err());
        assert_eq!(*events.lock().unwrap(), expected);
    }
}

#[cfg(feature = "airframe")]
#[cfg(feature = "airframe")]
#[tokio::test]
async fn malformed_metadata_is_a_startup_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.gguf");
    std::fs::write(&path, b"not a model").unwrap();
    let mut reg = Registry::new();
    register(&mut reg, "bad", &path);
    // Target selection is path-only; authoritative validation belongs to engine admission.
    assert!(resolve_warmup_target(&reg, None, false, &enabled(None))
        .unwrap()
        .is_some());
    let engine = shimmy::engine::airframe::AirframeEngine::new();
    let error = prepare_warmup(&engine, &reg, None, false, &enabled(None))
        .await
        .expect_err("malformed models must not reach readiness");
    let admission = error
        .chain()
        .find_map(|cause| {
            cause.downcast_ref::<airframe::execution::admission::ModelAdmissionError>()
        })
        .expect("typed model admission error must survive startup context");
    assert!(admission.client_error());
    assert!(error.to_string().contains("Startup model load failed"));
}
#[cfg(unix)]
#[test]
fn unicode_environment_validation_and_disabled_selector() {
    use std::os::unix::ffi::OsStringExt;
    let old_flag = std::env::var_os("SHIMMY_WARMUP");
    let old_model = std::env::var_os("SHIMMY_WARMUP_MODEL");
    struct Restore(Option<std::ffi::OsString>, Option<std::ffi::OsString>);
    impl Drop for Restore {
        fn drop(&mut self) {
            for (key, value) in [("SHIMMY_WARMUP", &self.0), ("SHIMMY_WARMUP_MODEL", &self.1)] {
                if let Some(v) = value {
                    std::env::set_var(key, v);
                } else {
                    std::env::remove_var(key);
                }
            }
        }
    }
    let _restore = Restore(old_flag, old_model);
    std::env::set_var("SHIMMY_WARMUP", std::ffi::OsString::from_vec(vec![255]));
    assert!(WarmupConfig::from_env().is_err());
    std::env::set_var("SHIMMY_WARMUP", "0");
    std::env::set_var(
        "SHIMMY_WARMUP_MODEL",
        std::ffi::OsString::from_vec(vec![255]),
    );
    assert!(!WarmupConfig::from_env().unwrap().enabled);
    std::env::set_var("SHIMMY_WARMUP", "1");
    assert!(WarmupConfig::from_env().is_err());
}

struct Unsupported;
#[async_trait]
impl LoadedModel for Unsupported {
    async fn generate(
        &self,
        _: &str,
        _: GenOptions,
        _: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        unreachable!()
    }
}
#[tokio::test]
async fn unsupported_backend_is_not_a_success() {
    assert!(Unsupported.warmup().await.is_err());
}
