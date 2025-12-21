# Shimmy Discovery System API Reference

## Overview

The Shimmy Discovery System provides a zero-configuration backend discovery mechanism using IPC (Inter-Process Communication) via Unix Domain Sockets on Unix/Linux/macOS and Named Pipes on Windows. This system eliminates the need for manual port configuration and provides instant cleanup when backends disconnect.

## Architecture

### Core Components

- **Discovery Leader**: The first process to bind the IPC endpoint becomes the leader, responsible for maintaining the backend registry
- **Discovery Followers (Backends)**: Backend processes that register with the leader via persistent IPC connections
- **Frontend Clients**: Short-lived connections from CLI tools and applications to query available backends

### IPC Endpoints

- **Unix/Linux/macOS**: `/tmp/shimmy.discovery.sock`
- **Windows**: `\\.\pipe\shimmy.discovery`

## Public API

### Core Types

#### `ConnectionRole`

Identifies the type of connection during handshake.

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionRole {
    Backend,  // Long-lived connection from backend process
    Frontend, // Short-lived connection from CLI/tooling
}
```

**Methods:**
- `to_byte(self) -> u8`: Convert to wire format byte (`b'B'` or `b'F'`)
- `from_byte(byte: u8) -> Option<Self>`: Parse from wire format byte

**Example:**
```rust
use shimmy::discovery::ConnectionRole;

let role = ConnectionRole::Backend;
assert_eq!(role.to_byte(), b'B');
assert_eq!(ConnectionRole::from_byte(b'F'), Some(ConnectionRole::Frontend));
```

#### `BackendMessage`

Messages sent from backend processes to the discovery leader.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BackendMessage {
    Register {
        id: String,
        pid: u32,
        port: u16,
        capabilities: BackendCapabilities,
        models: Vec<String>,
        started_at: u64,
    },
    Update {
        id: String,
        capabilities: BackendCapabilities,
        models: Vec<String>,
    },
    Goodbye {
        id: String,
    },
}
```

**Variants:**

- **`Register`**: Initial backend registration
  - `id`: Unique backend identifier (UUID v4 recommended)
  - `pid`: Process ID for diagnostics
  - `port`: HTTP server port number
  - `capabilities`: Backend capabilities structure
  - `models`: List of loaded model names
  - `started_at`: Unix timestamp when backend started

- **`Update`**: Update backend information
  - `id`: Backend identifier
  - `capabilities`: Updated capabilities
  - `models`: Updated model list

- **`Goodbye`**: Graceful shutdown notification
  - `id`: Backend identifier

**Example:**
```rust
use shimmy::discovery::{BackendMessage, BackendCapabilities};

let register_msg = BackendMessage::Register {
    id: "backend-123".to_string(),
    pid: std::process::id(),
    port: 8080,
    capabilities: BackendCapabilities {
        backend_type: "llama".to_string(),
        features: vec!["streaming".to_string(), "embeddings".to_string()],
        max_context: 4096,
        supports_system: true,
    },
    models: vec!["phi3-mini".to_string()],
    started_at: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs(),
};
```

#### `FrontendMessage`

Messages sent from frontend clients to query the discovery leader.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FrontendMessage {
    List,
    GetBackend { id: String },
}
```

**Variants:**

- **`List`**: Request list of all registered backends
- **`GetBackend`**: Request specific backend information
  - `id`: Backend identifier to query

**Example:**
```rust
use shimmy::discovery::FrontendMessage;

let list_msg = FrontendMessage::List;
let get_msg = FrontendMessage::GetBackend {
    id: "backend-123".to_string(),
};
```

#### `LeaderMessage`

Response messages sent from the discovery leader to clients.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LeaderMessage {
    Ack,
    Error { message: String },
    BackendList { backends: Vec<BackendInfo> },
    BackendInfo { backend: Option<BackendInfo> },
}
```

**Variants:**

- **`Ack`**: Successful operation acknowledgment
- **`Error`**: Error response with message
- **`BackendList`**: List of backend information
- **`BackendInfo`**: Single backend information (may be None if not found)

**Example:**
```rust
use shimmy::discovery::{LeaderMessage, BackendInfo};

let response = LeaderMessage::BackendList {
    backends: vec![
        BackendInfo {
            id: "backend-123".to_string(),
            port: 8080,
            capabilities: /* ... */,
            models: vec!["phi3-mini".to_string()],
            started_at: 1635724800,
            last_update: 1635724900,
            validation_status: ValidationStatus::Valid,
        },
    ],
};
```

#### `BackendInfo`

Comprehensive information about a registered backend.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInfo {
    pub id: String,
    pub port: u16,
    pub capabilities: BackendCapabilities,
    pub models: Vec<String>,
    pub started_at: u64,
    pub last_update: u64,
    pub validation_status: ValidationStatus,
}
```

**Fields:**
- `id`: Unique backend identifier
- `port`: HTTP server port
- `capabilities`: Backend capabilities
- `models`: Available models
- `started_at`: Backend start timestamp
- `last_update`: Last update timestamp
- `validation_status`: Current validation state

#### `ErrorCode`

Standardized error codes for protocol violations and system errors.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorCode {
    InvalidMessage,
    ValidationFailed,
    BackendNotFound,
    InternalError,
    Timeout,
    RateLimited,
    ResourceExhausted,
}
```

### IPC Communication

#### `IpcConnection`

Core IPC connection abstraction for cross-platform communication.

```rust
pub struct IpcConnection { /* private fields */ }
```

**Methods:**

```rust
impl IpcConnection {
    /// Send a message over the IPC connection
    pub async fn send_message<T: Serialize>(&mut self, message: &T) -> Result<()>;
    
    /// Receive a message from the IPC connection
    pub async fn receive_message<T: DeserializeOwned>(&mut self) -> Result<T>;
    
    /// Check if connection is still alive
    pub fn is_connected(&self) -> bool;
    
    /// Close the connection
    pub async fn close(&mut self) -> Result<()>;
}
```

**Example:**
```rust
use shimmy::discovery::{IpcConnection, BackendMessage, LeaderMessage};

async fn register_backend(mut conn: IpcConnection) -> Result<()> {
    let register_msg = BackendMessage::Register {
        // ... fill in fields
    };
    
    // Send registration
    conn.send_message(&register_msg).await?;
    
    // Wait for acknowledgment
    let response: LeaderMessage = conn.receive_message().await?;
    match response {
        LeaderMessage::Ack => println!("Successfully registered"),
        LeaderMessage::Error { message } => eprintln!("Registration failed: {}", message),
        _ => eprintln!("Unexpected response"),
    }
    
    Ok(())
}
```

#### `IpcListener`

Server-side IPC listener for accepting connections.

```rust
pub struct IpcListener { /* private fields */ }
```

**Methods:**

```rust
impl IpcListener {
    /// Bind to the discovery endpoint (leader election)
    pub async fn bind() -> Result<IpcListener>;
    
    /// Accept incoming connection
    pub async fn accept(&self) -> Result<(IpcConnection, ConnectionInfo)>;
    
    /// Close the listener
    pub async fn close(&mut self) -> Result<()>;
}
```

**Example:**
```rust
use shimmy::discovery::{IpcListener, ConnectionRole};

async fn run_discovery_leader() -> Result<()> {
    let listener = IpcListener::bind().await?;
    println!("Became discovery leader");
    
    loop {
        let (mut conn, info) = listener.accept().await?;
        
        tokio::spawn(async move {
            if let Err(e) = handle_client_connection(conn).await {
                eprintln!("Client connection error: {}", e);
            }
        });
    }
}
```

#### `IpcStream`

Low-level IPC stream for client connections.

```rust
pub struct IpcStream { /* private fields */ }
```

**Methods:**

```rust
impl IpcStream {
    /// Connect to the discovery endpoint
    pub async fn connect() -> Result<IpcStream>;
    
    /// Perform role handshake
    pub async fn handshake(&mut self, role: ConnectionRole) -> Result<()>;
    
    /// Send raw bytes
    pub async fn send_bytes(&mut self, data: &[u8]) -> Result<()>;
    
    /// Receive raw bytes
    pub async fn receive_bytes(&mut self, buffer: &mut [u8]) -> Result<usize>;
}
```

### High-Level Frontend API

#### `cmd_discover()`

Command-line interface for backend discovery.

```rust
pub async fn cmd_discover() -> Result<()>
```

Queries the discovery leader and prints a formatted table of available backends.

**Example:**
```bash
$ shimmy discover
┌─────────────┬──────┬─────────┬──────────────┬──────────────────┐
│ Backend ID  │ Port │ Type    │ Models       │ Status           │
├─────────────┼──────┼─────────┼──────────────┼──────────────────┤
│ backend-123 │ 8080 │ llama   │ phi3-mini    │ Valid            │
│ backend-456 │ 8081 │ mlx     │ llama2-7b    │ Valid            │
└─────────────┴──────┴─────────┴──────────────┴──────────────────┘
```

#### `query_backends_via_ipc()`

Programmatic interface for querying backends.

```rust
pub async fn query_backends_via_ipc() -> Result<Vec<BackendInfo>>
```

Returns a list of all registered backends.

**Example:**
```rust
use shimmy::discovery::query_backends_via_ipc;

async fn find_backend_with_model(model_name: &str) -> Result<Option<BackendInfo>> {
    let backends = query_backends_via_ipc().await?;
    
    for backend in backends {
        if backend.models.contains(&model_name.to_string()) {
            return Ok(Some(backend));
        }
    }
    
    Ok(None)
}
```

#### `create_discovery_proxy_router()`

HTTP proxy router for editor plugins and web frontends.

```rust
pub fn create_discovery_proxy_router() -> Router
```

Creates an Axum router with discovery endpoints for HTTP clients.

**Endpoints:**
- `GET /api/discovery/backends` - List all backends
- `GET /api/discovery/backends/:id` - Get specific backend

**Example:**
```rust
use shimmy::discovery::create_discovery_proxy_router;
use axum::Router;

let app = Router::new()
    .merge(create_discovery_proxy_router())
    .route("/", get(root_handler));
```

### Discovery Leader API

#### `DiscoveryLeader`

Main discovery leader implementation.

```rust
pub struct DiscoveryLeader { /* private fields */ }
```

**Methods:**

```rust
impl DiscoveryLeader {
    /// Create new discovery leader
    pub fn new() -> Self;
    
    /// Start the discovery leader service
    pub async fn run(&mut self) -> Result<()>;
    
    /// Register a new backend
    pub async fn register_backend(&mut self, msg: BackendMessage) -> Result<()>;
    
    /// Update existing backend
    pub async fn update_backend(&mut self, msg: BackendMessage) -> Result<()>;
    
    /// Remove backend (called on disconnect)
    pub fn remove_backend(&mut self, backend_id: &str);
    
    /// Get list of all backends
    pub fn list_backends(&self) -> Vec<BackendInfo>;
    
    /// Get specific backend
    pub fn get_backend(&self, id: &str) -> Option<BackendInfo>;
    
    /// Get system statistics
    pub fn get_stats(&self) -> DiscoveryStats;
}
```

**Example:**
```rust
use shimmy::discovery::DiscoveryLeader;

#[tokio::main]
async fn main() -> Result<()> {
    let mut leader = DiscoveryLeader::new();
    
    // This will run until the process is terminated
    leader.run().await?;
    
    Ok(())
}
```

## Wire Protocol

### Message Framing

All messages use a 4-byte little-endian length prefix followed by JSON payload:

```
[4 bytes: payload length][JSON message]
```

### Connection Handshake

1. Client connects to IPC endpoint
2. Client sends role byte: `b'B'` (Backend) or `b'F'` (Frontend)
3. Leader responds with `b'A'` (Ack) or `b'E'` (Error + disconnect)

### Error Handling

All operations return structured errors with appropriate error codes. Network-level errors are distinguished from protocol-level errors.

**Common Error Patterns:**

```rust
use shimmy::discovery::{ErrorCode, DiscoveryError};

match result {
    Ok(response) => { /* handle success */ },
    Err(DiscoveryError::Protocol { code: ErrorCode::BackendNotFound, .. }) => {
        // Handle backend not found
    },
    Err(DiscoveryError::Network { .. }) => {
        // Handle network connectivity issues
    },
    Err(DiscoveryError::Timeout { .. }) => {
        // Handle timeout scenarios
    },
}
```

## Constants

```rust
/// IPC endpoint path
pub const IPC_ENDPOINT: &str = if cfg!(windows) {
    r"\\.\pipe\shimmy.discovery"
} else {
    "/tmp/shimmy.discovery.sock"
};

/// Handshake timeout in milliseconds
pub const HANDSHAKE_TIMEOUT_MS: u64 = 5000;

/// Message timeout in milliseconds
pub const MESSAGE_TIMEOUT_MS: u64 = 10000;

/// Maximum message size in bytes
pub const MAX_MESSAGE_SIZE: usize = 1_048_576; // 1MB

/// Maximum backends per leader
pub const MAX_BACKENDS: usize = 1000;

/// Connection backlog for listener
pub const CONNECTION_BACKLOG: u32 = 128;
```

## Thread Safety

All public APIs are thread-safe and can be used from multiple threads simultaneously. The discovery leader uses internal locking to ensure consistency.

## Performance Characteristics

- **Registration Latency**: < 10ms typical, < 50ms under load
- **Cleanup Time**: < 100ms (instant on connection close)
- **Memory Overhead**: < 5MB for discovery leader with 100 backends
- **Throughput**: > 1000 operations per second
- **Concurrent Connections**: Limited by OS file descriptor limits

## Security Considerations

- Input validation on all message fields
- Resource limits prevent DoS attacks
- Connection rate limiting
- Structured error responses prevent information leakage
- Security event logging for audit trails

## Platform-Specific Notes

### Windows
- Uses Named Pipes (`\\.\pipe\shimmy.discovery`)
- Default security descriptor (current user + administrators)
- Antivirus software may interfere with pipe creation

### Unix/Linux/macOS
- Uses Unix Domain Sockets (`/tmp/shimmy.discovery.sock`)
- Socket permissions: 0o666 (world readable/writable)
- Automatic cleanup of stale socket files

## Examples

See the `examples/` directory for complete working examples:
- `basic_backend.rs` - Simple backend registration
- `discovery_client.rs` - Frontend query example
- `leader_election.rs` - Multi-process leader election
- `failover_test.rs` - Leader failover demonstration