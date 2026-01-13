/// IPC Backend Registration and Leader Election
///
/// Backends connect to the discovery leader as followers, or become leaders themselves
/// if no leader exists. Implements automatic re-election on leader failure.

use crate::discovery::{
    protocol::{BackendMessage, LeaderMessage, ConnectionRole, ErrorCode, BackendCapabilities},
    ipc::{IpcStream, IpcConnection, IpcListener},
    leader::DiscoveryLeader,
    service::ModelInfo,
};
use anyhow::{Result, Context};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use uuid::Uuid;

/// Backend registration and discovery coordinator
pub struct DiscoveryCandidate {
    backend_id: String,
    backend_port: u16,
    capabilities: BackendCapabilities,
    models: Vec<ModelInfo>,
    pid: u32,
}

impl DiscoveryCandidate {
    /// Create a new discovery candidate
    pub fn new(backend_port: u16, capabilities: BackendCapabilities, models: Vec<ModelInfo>) -> Self {
        Self {
            backend_id: Uuid::new_v4().to_string(),
            backend_port,
            capabilities,
            models,
            pid: std::process::id(),
        }
    }
    
    /// Start the discovery candidate (leader election + registration loop)
    pub async fn start(&self) -> Result<()> {
        loop {
            println!("🗳️  Attempting discovery leadership or follower registration");
            
            match self.try_become_leader().await {
                Ok(()) => {
                    println!("👑 Became discovery leader, accepting connections");
                    // If we become leader, we don't return unless there's an error
                    // The leader runs until the process terminates
                }
                Err(_) => {
                    println!("👥 Leader exists, registering as follower");
                    if let Err(e) = self.run_as_follower().await {
                        eprintln!("❌ Follower registration failed: {}", e);
                    }
                }
            }
            
            // If we get here, either leadership failed or follower disconnected
            // Wait with jitter before retrying to prevent thundering herd
            let jitter = Duration::from_millis(20 + (rand::random::<u64>() % 100));
            println!("⏳ Retrying in {}ms", jitter.as_millis());
            sleep(jitter).await;
        }
    }
    
    /// Try to become the discovery leader (bind IPC endpoint)
    async fn try_become_leader(&self) -> Result<()> {
        // Try to bind the discovery endpoint
        match IpcListener::bind().await {
            Ok(_listener) => {
                println!("🔥 Successfully bound discovery endpoint, starting leader");
                
                // We became the leader - start the discovery service
                let leader = DiscoveryLeader::new();
                
                // Register ourselves as the first backend
                tokio::spawn({
                    let backend_id = self.backend_id.clone();
                    let backend_port = self.backend_port;
                    let capabilities = self.capabilities.clone();
                    let models = self.models.clone();
                    
                    async move {
                        // Give the leader a moment to start
                        sleep(Duration::from_millis(100)).await;
                        
                        // Register ourselves as a backend
                        if let Err(e) = Self::self_register_to_leader(
                            backend_id,
                            backend_port,
                            capabilities,
                            models,
                        ).await {
                            eprintln!("⚠️  Failed to self-register to leader: {}", e);
                        }
                    }
                });
                
                // Start the leader (this blocks until shutdown)
                leader.start().await?;
                Ok(())
            }
            Err(_) => {
                Err(anyhow::anyhow!("Failed to bind discovery endpoint (leader exists)"))
            }
        }
    }
    
    /// Run as a follower (connect to existing leader)
    async fn run_as_follower(&self) -> Result<()> {
        println!("🔗 Connecting to discovery leader as follower");
        
        // Connect to leader
        let mut stream = IpcConnection::connect_as(ConnectionRole::Backend).await
            .context("Failed to connect to discovery leader")?;
        
        println!("✅ Connected to discovery leader");
        
        // Send registration message
        let register_msg = BackendMessage::Register {
            id: self.backend_id.clone(),
            pid: self.pid,
            port: self.backend_port,
            capabilities: self.capabilities.clone(), 
            models: self.models.clone(),
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        stream.send_message(&register_msg).await
            .context("Failed to send registration message")?;
        
        // Wait for acknowledgment
        let response: LeaderMessage = stream.recv_message().await
            .context("Failed to receive registration response")?;
        
        match response {
            LeaderMessage::Ack => {
                println!("✅ Registration successful, maintaining connection");
            }
            LeaderMessage::Error { message, error_code } => {
                return Err(anyhow::anyhow!(
                    "Registration rejected ({:?}): {}", 
                    error_code, 
                    message
                ));
            }
            _ => {
                return Err(anyhow::anyhow!("Unexpected response from leader"));
            }
        }
        
        // Keep connection alive (this is our "heartbeat")
        // Send periodic updates if capabilities or models change
        let mut update_interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            tokio::select! {
                // Send periodic update (in case capabilities changed)
                _ = update_interval.tick() => {
                    let update_msg = BackendMessage::Update {
                        id: self.backend_id.clone(),
                        capabilities: self.get_current_capabilities(),
                        models: self.get_current_models(),
                    };
                    
                    if let Err(e) = stream.send_message(&update_msg).await {
                        eprintln!("❌ Failed to send update: {}", e);
                        break;
                    }
                    
                    // Wait for ack (optional, but good for error detection)
                    match stream.recv_message::<LeaderMessage>().await {
                        Ok(LeaderMessage::Ack) => {
                            // Update successful
                        }
                        Ok(LeaderMessage::Error { message, .. }) => {
                            eprintln!("⚠️  Update rejected: {}", message);
                        }
                        Err(_) => {
                            eprintln!("📪 Leader connection lost");
                            break;
                        }
                        _ => {
                            eprintln!("⚠️  Unexpected response to update");
                        }
                    }
                }
                
                // Listen for leader messages (pings, etc.)
                result = stream.recv_message::<LeaderMessage>() => {
                    match result {
                        Ok(LeaderMessage::Ping) => {
                            // Respond to ping (keep-alive)
                            if let Err(e) = stream.send_message(&LeaderMessage::Ack).await {
                                eprintln!("❌ Failed to respond to ping: {}", e);
                                break;
                            }
                        }
                        Ok(msg) => {
                            println!("📨 Received message from leader: {:?}", msg);
                        }
                        Err(e) => {
                            println!("📪 Connection to leader lost: {}", e);
                            break;
                        }
                    }
                }
            }
        }
        
        println!("🔌 Follower connection ended");
        Ok(())
    }
    
    /// Self-register to the leader we just started
    async fn self_register_to_leader(
        backend_id: String,
        backend_port: u16,
        capabilities: BackendCapabilities,
        models: Vec<ModelInfo>,
    ) -> Result<()> {
        println!("🔄 Self-registering to leader we just started");
        
        // Connect as backend
        let mut stream = IpcConnection::connect_as(ConnectionRole::Backend).await
            .context("Failed to connect to our own leader")?;
        
        // Send registration
        let register_msg = BackendMessage::Register {
            id: backend_id.clone(),
            pid: std::process::id(),
            port: backend_port,
            capabilities,
            models,
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        stream.send_message(&register_msg).await
            .context("Failed to send self-registration")?;
        
        // Wait for ack
        let response: LeaderMessage = stream.recv_message().await
            .context("Failed to receive self-registration response")?;
        
        match response {
            LeaderMessage::Ack => {
                println!("✅ Self-registration successful");
                
                // Keep the connection alive in background
                tokio::spawn(async move {
                    loop {
                        // Send periodic ping or just wait for disconnect
                        match stream.recv_message::<LeaderMessage>().await {
                            Ok(_) => {
                                // Leader sent us something
                            }
                            Err(_) => {
                                println!("📪 Self-registration connection lost");
                                break;
                            }
                        }
                    }
                });
                
                Ok(())
            }
            LeaderMessage::Error { message, .. } => {
                Err(anyhow::anyhow!("Self-registration failed: {}", message))
            }
            _ => {
                Err(anyhow::anyhow!("Unexpected self-registration response"))
            }
        }
    }
    
    /// Get current backend capabilities (might change over time)
    fn get_current_capabilities(&self) -> BackendCapabilities {
        // In a real implementation, this might check what features are currently available
        // For now, just return the static capabilities
        self.capabilities.clone()
    }
    
    /// Get current loaded models (might change over time)  
    fn get_current_models(&self) -> Vec<ModelInfo> {
        // In a real implementation, this might query the actual loaded models
        // For now, just return the static model list
        self.models.clone()
    }
    
    /// Send graceful goodbye before shutdown
    pub async fn shutdown(&self) -> Result<()> {
        println!("👋 Sending goodbye to discovery leader");
        
        // Try to connect and send goodbye
        if let Ok(mut stream) = IpcConnection::connect_as(ConnectionRole::Backend).await {
            let goodbye_msg = BackendMessage::Goodbye {
                id: self.backend_id.clone(),
            };
            
            let _ = stream.send_message(&goodbye_msg).await;
            let _ = stream.close().await;
        }
        
        Ok(())
    }
}

/// Convenience function for starting discovery from server.rs
pub async fn start_discovery_candidate(
    backend_port: u16,
    capabilities: BackendCapabilities,
    models: Vec<ModelInfo>,
) -> Result<()> {
    let candidate = DiscoveryCandidate::new(backend_port, capabilities, models);
    candidate.start().await
}

/// Simplified interface for backends that just want to register
pub struct SimpleBackendRegistration {
    candidate: DiscoveryCandidate,
}

impl SimpleBackendRegistration {
    /// Create a simple backend registration
    pub fn new(backend_port: u16) -> Self {
        // Get capabilities from compile-time features
        let capabilities = Self::detect_capabilities();
        
        // Get models from environment or default empty
        let models = Self::detect_models();
        
        let candidate = DiscoveryCandidate::new(backend_port, capabilities, models);
        
        Self { candidate }
    }
    
    /// Start registration (leader election or follower)
    pub async fn start(&self) -> Result<()> {
        self.candidate.start().await
    }
    
    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        self.candidate.shutdown().await
    }
    
    /// Detect capabilities from compile-time features
    fn detect_capabilities() -> BackendCapabilities {
        let mut features = vec![];
        
        #[cfg(feature = "llama")]
        features.push("llama".to_string());
        
        #[cfg(feature = "llama-cuda")]
        features.push("cuda".to_string());
        
        #[cfg(feature = "llama-vulkan")]
        features.push("vulkan".to_string());
        
        #[cfg(feature = "llama-opencl")]
        features.push("opencl".to_string());
        
        #[cfg(feature = "mlx")]
        features.push("mlx".to_string());
        
        #[cfg(feature = "console")]
        features.push("console".to_string());

        let backend_type = if cfg!(feature = "llama") {
            "llama"
        } else if cfg!(feature = "mlx") {
            "mlx"
        } else {
            "stub"
        }.to_string();

        BackendCapabilities {
            backend_type,
            features_compiled: features,
            websocket_working: false, // Will be set by validation
            http_working: false,       // Will be set by validation
            models_loaded: false,      // Will be set by validation
            streaming_supported: true,  // Shimmy always supports streaming
        }
    }
    
    /// Detect models from environment variables or config
    fn detect_models() -> Vec<ModelInfo> {
        // Check for model environment variables
        let mut models = vec![];
        
        if let Ok(base_model) = std::env::var("SHIMMY_BASE_GGUF") {
            let model_name = std::path::Path::new(&base_model)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            models.push(ModelInfo {
                name: model_name,
                backend_type: Self::detect_capabilities().backend_type,
                compiled_support: true,
            });
        }
        
        // Could also check for LoRA adapters, additional models, etc.
        
        models
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_discovery_candidate_creation() {
        let capabilities = BackendCapabilities {
            backend_type: "llama".to_string(),
            features_compiled: vec!["cuda".to_string()],
            ..Default::default()
        };
        
        let models = vec![ModelInfo {
            name: "test-model".to_string(),
            backend_type: "llama".to_string(),
            compiled_support: true,
        }];
        
        let candidate = DiscoveryCandidate::new(8080, capabilities.clone(), models.clone());
        
        assert_eq!(candidate.backend_port, 8080);
        assert_eq!(candidate.capabilities.backend_type, "llama");
        assert_eq!(candidate.models.len(), 1);
        assert_eq!(candidate.pid, std::process::id());
        assert!(!candidate.backend_id.is_empty());
    }
    
    #[test]
    fn test_simple_backend_registration_creation() {
        let registration = SimpleBackendRegistration::new(9090);
        
        assert_eq!(registration.candidate.backend_port, 9090);
        assert!(!registration.candidate.backend_id.is_empty());
        
        // Should detect capabilities from compile-time features
        let caps = &registration.candidate.capabilities;
        
        #[cfg(feature = "llama")]
        {
            assert_eq!(caps.backend_type, "llama");
            assert!(caps.features_compiled.contains(&"llama".to_string()));
        }
        
        #[cfg(not(feature = "llama"))]
        {
            // Should fallback to stub if no backend features
            assert!(caps.backend_type == "mlx" || caps.backend_type == "stub");
        }
    }
    
    #[test]
    fn test_capability_detection() {
        let caps = SimpleBackendRegistration::detect_capabilities();
        
        // Should always have streaming support
        assert!(caps.streaming_supported);
        
        // Backend type should be determined by features
        assert!(!caps.backend_type.is_empty());
        
        // Features should match compile-time flags
        #[cfg(feature = "llama")]
        assert!(caps.features_compiled.contains(&"llama".to_string()));
        
        #[cfg(feature = "llama-cuda")]
        assert!(caps.features_compiled.contains(&"cuda".to_string()));
        
        #[cfg(feature = "llama-vulkan")]
        assert!(caps.features_compiled.contains(&"vulkan".to_string()));
    }
    
    #[test]
    fn test_model_detection_from_env() {
        // Set test environment variable
        std::env::set_var("SHIMMY_BASE_GGUF", "/path/to/model.gguf");
        
        let models = SimpleBackendRegistration::detect_models();
        
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "model");
        assert!(models[0].compiled_support);
        
        // Clean up
        std::env::remove_var("SHIMMY_BASE_GGUF");
    }
    
    #[test]
    fn test_model_detection_no_env() {
        // Ensure no model environment variables
        std::env::remove_var("SHIMMY_BASE_GGUF");
        
        let models = SimpleBackendRegistration::detect_models();
        
        // Should return empty if no models configured
        assert_eq!(models.len(), 0);
    }
    
    #[tokio::test]
    async fn test_candidate_update_capabilities() {
        let candidate = DiscoveryCandidate::new(
            8080,
            BackendCapabilities::default(),
            vec![],
        );
        
        // Test that get_current_capabilities returns the stored capabilities
        let caps = candidate.get_current_capabilities();
        assert_eq!(caps.backend_type, candidate.capabilities.backend_type);
        
        // Test that get_current_models returns the stored models
        let models = candidate.get_current_models();
        assert_eq!(models.len(), candidate.models.len());
    }
    
    #[tokio::test]
    async fn test_candidate_id_uniqueness() {
        let candidate1 = DiscoveryCandidate::new(8080, BackendCapabilities::default(), vec![]);
        let candidate2 = DiscoveryCandidate::new(8081, BackendCapabilities::default(), vec![]);
        
        // Each candidate should have a unique ID
        assert_ne!(candidate1.backend_id, candidate2.backend_id);
    }
    
    #[test]  
    fn test_backend_registration_simplification() {
        // Test that SimpleBackendRegistration provides a clean interface
        let registration = SimpleBackendRegistration::new(7777);
        
        // Should have reasonable defaults
        assert_eq!(registration.candidate.backend_port, 7777);
        assert!(registration.candidate.capabilities.streaming_supported);
        
        // Should handle the complexity internally
        assert!(!registration.candidate.backend_id.is_empty());
        assert_eq!(registration.candidate.pid, std::process::id());
    }
}
