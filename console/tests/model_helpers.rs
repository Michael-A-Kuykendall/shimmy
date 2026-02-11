use mockito::Server;
use serial_test::serial;
use std::env;
use std::ffi::OsString;
use tempfile::TempDir;
use shimmy_console::commands::model;

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let original = env::var_os(key);
        env::set_var(key, value);
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => env::set_var(self.key, value),
            None => env::remove_var(self.key),
        }
    }
}

fn configure_config_dir(temp_dir: &TempDir) -> EnvVarGuard {
    EnvVarGuard::set("SHIMMY_CONSOLE_CONFIG_DIR", temp_dir.path().to_string_lossy().as_ref())
}

fn configure_models_endpoint(endpoint: &str) -> EnvVarGuard {
    EnvVarGuard::set("SHIMMY_MODELS_ENDPOINT", endpoint)
}

#[tokio::test]
#[serial]
async fn select_model_updates_config_when_model_exists() {
    let temp_dir = TempDir::new().expect("tempdir");
    let _config_guard = configure_config_dir(&temp_dir);

    let server = Server::new();
    let mock = server.mock("GET", "/v1/models")
        .with_status(200)
        .with_body(r#"{"data":[{"id":"test-model"}]}"#)
        .create();

    let endpoint = format!("{}/v1/models", server.url());
    let _endpoint_guard = configure_models_endpoint(&endpoint);

    model::select_model("test-model").await.expect("should set model");
    let selected = model::get_selected_model().await.expect("should read model");

    mock.assert();
    assert_eq!(selected.as_deref(), Some("test-model"));

    drop(_endpoint_guard);
    drop(_config_guard);
}

#[tokio::test]
#[serial]
async fn get_selected_model_clears_stale_selection_when_missing() {
    let temp_dir = TempDir::new().expect("tempdir");
    let _config_guard = configure_config_dir(&temp_dir);

    let mut server = Server::new();
    let _initial_mock = server.mock("GET", "/v1/models")
        .with_status(200)
        .with_body(r#"{"data":[{"id":"test-model"}]}"#)
        .create();

    let endpoint = format!("{}/v1/models", server.url());
    let _endpoint_guard = configure_models_endpoint(&endpoint);

    model::select_model("test-model").await.expect("initial set");

    server.reset();
    let _missing_mock = server.mock("GET", "/v1/models")
        .with_status(200)
        .with_body(r#"{"data":[]}"#)
        .create();

    let selected = model::get_selected_model().await.expect("should read after reset");
    assert!(selected.is_none(), "stale model should be cleared");

    drop(_endpoint_guard);
    drop(_config_guard);
}
