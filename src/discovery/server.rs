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

    /// Bind to port 11430 with fallback to 11431-11439
    pub async fn bind(&mut self) -> anyhow::Result<u16> {
        // Try ports 11430-11439
        for port in 11430..=11439 {
            match TcpListener::bind(format!("127.0.0.1:{}", port)) {
                Ok(_) => {
                    self.port = port;
                    return Ok(port);
                }
                Err(_) => continue,
            }
        }
        
        anyhow::bail!("Could not bind to any port in range 11430-11439")
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
        
        // Spawn stale backend cleanup task
        let service = self.service.clone();
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(5));
            loop {
                cleanup_interval.tick().await;
                let removed = service.remove_stale_backends().await;
                if removed > 0 {
                    eprintln!("🧹 Cleaned up {} stale backend(s)", removed);
                }
            }
        });
        
        // Start server
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        println!("✅ Discovery service ready on http://127.0.0.1:{}", port);
        
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        
        println!("🛑 Discovery service shutting down");
        Ok(())
    }

    /// Get the service instance (for testing)
    pub fn service(&self) -> Arc<DiscoveryService> {
        self.service.clone()
    }
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

    #[tokio::test]
    async fn test_binds_to_port_11430() {
        let mut server = DiscoveryServer::new();
        let port = server.bind().await.unwrap();
        assert_eq!(port, 11430);
    }

    #[tokio::test]
    async fn test_fallback_to_11431_if_11430_taken() {
        // Bind to 11430 first
        let _listener = TcpListener::bind("127.0.0.1:11430").unwrap();
        
        // Try to bind discovery server - should fall back to 11431
        let mut server = DiscoveryServer::new();
        let port = server.bind().await.unwrap();
        assert_eq!(port, 11431);
    }

    #[tokio::test]
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

