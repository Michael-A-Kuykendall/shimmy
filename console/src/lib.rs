//! Shimmy Console Library
//!
//! This crate provides the core console functionality for Shimmy,
//! including adapters, commands, tools, plugins, and session management.

pub mod adapters;
pub mod commands;
pub mod config;
pub mod context;
pub mod discovery;
pub mod history;
pub mod license;
pub mod plugins;
pub mod session_store;
pub mod tools;
pub mod websocket;

// Re-exports for convenience
pub use config::Config;
pub use context::ContextBuilder;
pub use history::{HistoryMessage, HistoryStorage};
pub use session_store::SessionStore;

// Adapter re-exports
pub use adapters::{HttpInferenceAdapter, MockInferenceAdapter, WsInferenceAdapter};

// Tool re-exports
pub use tools::{Tool, ToolArgs, ToolCall, ToolError, ToolRegistry, ToolResult};
