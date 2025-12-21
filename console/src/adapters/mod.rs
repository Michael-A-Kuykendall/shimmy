pub mod http_adapter;
pub mod mock_adapter;
pub mod ws_adapter;

pub use http_adapter::HttpInferenceAdapter;
pub use mock_adapter::MockInferenceAdapter;
pub use ws_adapter::WsInferenceAdapter;
