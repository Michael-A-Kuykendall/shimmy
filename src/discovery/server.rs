/// Discovery service server lifecycle management

use axum::{Router, routing::{get, post}};
use std::net::TcpListener;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use super::service::DiscoveryService;
use super::handlers;

/// Discovery server that binds to port 11430 (or fallback ports)
pub struct DiscoveryServer {
    service: Arc<DiscoveryService>,
    port: u16,
}

impl DiscoveryServer {
    /// Create a new discovery server
    pub fn new() -> Self {
        Self {
            service: Arc::new(DiscoveryService::new()),
            port: 0,
        }
    }

    /// Bind to ephemeral port (0) - let OS assign available port
    pub async fn bind(&mut self) -> anyhow::Result<u16> {
        // Preferred ports — attempt primary 11430 then fallback 11431
        let candidates = [11430u16, 11431u16, 0u16];

        for &p in &candidates {
            let addr = format!("127.0.0.1:{}", p);
            match TcpListener::bind(&addr) {
                Ok(listener) => {
                    self.port = listener.local_addr()?.port();
                    return Ok(self.port);
                }
                Err(_) => continue,
            }
        }

        // Should never reach here, but return an ephemeral port as fallback
        match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => {
                self.port = listener.local_addr()?.port();
                Ok(self.port)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Start the discovery service
    pub async fn start(mut self) -> anyhow::Result<()> {
        let port = self.bind().await?;
        
        println!("🔍 Discovery service starting on port {}", port);
        
        // Create router with all endpoints
        let app = Router::new()
            .route("/discover", get(handlers::discover))
            .route("/register", post(handlers::register))
            .route("/heartbeat/:id", post(handlers::heartbeat))
            .route("/backends/:id/validate", get(handlers::validate_backend))
            .with_state(self.service.clone());
        
        // Spawn stale backend cleanup task with shutdown channel
        let service = self.service.clone();
        let (cleanup_shutdown_tx, mut cleanup_shutdown_rx) = tokio::sync::watch::channel(false);
        
        let cleanup_handle = tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(5));
            loop {
                tokio::select! {
                    _ = cleanup_interval.tick() => {
                        let removed = service.remove_stale_backends().await;
                        if removed > 0 {
                            eprintln!("🧹 Cleaned up {} stale backend(s)", removed);
                        }
                    }
                    _ = cleanup_shutdown_rx.changed() => {
                        println!("🛑 Cleanup task received shutdown signal");
                        break;
                    }
                }
            }
        });
        
        // Start server
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        println!("✅ Discovery service ready on http://127.0.0.1:{}", port);
        
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_signal().await;
                // Signal cleanup task to stop
                let _ = cleanup_shutdown_tx.send(true);
                // Wait for cleanup to finish
                let _ = cleanup_handle.await;
            })
            .await?;
        
        println!("🛑 Discovery service shutting down");
        Ok(())
    }

    /// Get the service instance (for testing)
    pub fn service(&self) -> Arc<DiscoveryService> {
        self.service.clone()
    }
}

/// Standalone function to run discovery server (used by CLI)
pub async fn run() -> anyhow::Result<()> {
    let server = DiscoveryServer::new();
    server.start().await
}

impl Default for DiscoveryServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    use tokio::signal;
    
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("📡 Shutdown signal received");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_binds_to_ephemeral_port() {
        let mut server = DiscoveryServer::new();
        let port = server.bind().await.unwrap();
        assert!(port > 0, "Ephemeral port should be assigned by OS");
        // Port can be any available port (typically > 1024)
    }

    #[tokio::test]
    #[serial]
    async fn test_fallback_to_11431_if_11430_taken() {
        // Bind to 11430 first - keep alive for the duration of the test
        let listener = TcpListener::bind("127.0.0.1:11430").unwrap();
        
        // Try to bind discovery server - should fall back to 11431
        let mut server = DiscoveryServer::new();
        let port = server.bind().await.unwrap();
        assert_eq!(port, 11431);
        
        // Drop listener to free port
        drop(listener);
    }

    #[tokio::test]
    #[serial]
    async fn test_stale_cleanup_removes_old_backends() {
        use super::super::service::{BackendRegistration, BackendCapabilities, ValidationStatus, HealthStatus};
        use std::time::Instant;
        
        let server = DiscoveryServer::new();
        let service = server.service();
        
        // Register a backend with old heartbeat
        let mut backend = BackendRegistration {
            id: "test-1".to_string(),
            port: 12345,
            url: "http://localhost:12345".to_string(),
            models: vec![],
            capabilities: BackendCapabilities::default(),
            last_heartbeat: Instant::now(),
            health: HealthStatus::Ok,
            validation: ValidationStatus::default(),
        };
        
        // Manually set old heartbeat (15+ seconds ago)
        backend.last_heartbeat = Instant::now() - std::time::Duration::from_secs(20);
        service.register(backend).await.unwrap();
        
        // Run cleanup
        let removed = service.remove_stale_backends().await;
        assert_eq!(removed, 1);
        
        // Verify removed
        let backend = service.get_backend("test-1").await;
        assert!(backend.is_none());
    }
}
