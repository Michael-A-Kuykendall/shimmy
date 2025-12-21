use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ShimmyInstance {
    pub http_port: u16,
    pub websocket_port: u16,
    pub base_url: String,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

pub struct PortDiscovery {
    scan_range: std::ops::Range<u16>,
    discovered_ports: HashMap<String, u16>,
    health_check_interval: Duration,
}

#[derive(Debug)]
pub enum DiscoveryError {
    NoShimmyFound,
    InvalidResponse,
    NetworkError(String),
    MultipleInstances,
}

impl PortDiscovery {
    pub fn new() -> Self {
        Self {
            scan_range: 8000..12000,
            discovered_ports: HashMap::new(),
            health_check_interval: Duration::from_secs(30),
        }
    }

    pub async fn discover_shimmy_instance(&self) -> Result<ShimmyInstance, DiscoveryError> {
        // Use existing shimmy port manager logic
        let common_ports = [11434, 11435, 8080, 8000, 3000, 5000];

        for port in common_ports {
            if let Ok(instance) = self.probe_shimmy_at_port(port).await {
                return Ok(instance);
            }
        }

        // Fallback to shimmy's port allocator logic
        Err(DiscoveryError::NoShimmyFound)
    }

    async fn probe_shimmy_at_port(&self, port: u16) -> Result<ShimmyInstance, DiscoveryError> {
        let health_url = format!("http://localhost:{}/v1/models", port);
        
        match reqwest::get(&health_url).await {
            Ok(response) if response.status().is_success() => {
                // Also verify WebSocket health endpoint exists for console feature
                let ws_health_url = format!("http://localhost:{}/ws/health", port);
                match reqwest::get(&ws_health_url).await {
                    Ok(ws_response) if ws_response.status().is_success() => {
                        Ok(ShimmyInstance {
                            http_port: port,
                            websocket_port: port, // Same port for WebSocket upgrade
                            base_url: format!("http://localhost:{}", port),
                            health_status: HealthStatus::Healthy,
                        })
                    }
                    _ => Err(DiscoveryError::InvalidResponse),
                }
            }
            _ => Err(DiscoveryError::InvalidResponse),
        }
    }

    pub async fn discover_shimmy_websocket(&self) -> Result<String, DiscoveryError> {
        let instance = self.discover_shimmy_instance().await?;
        Ok(format!("ws://localhost:{}/ws/console", instance.websocket_port))
    }
}

// Specification-required functions (lines 332-359)
pub async fn discover_shimmy_instance() -> Result<ShimmyInstance, DiscoveryError> {
    // Use existing shimmy port manager logic
    let common_ports = [11434, 11435, 8080, 8000, 3000, 5000];

    for port in common_ports {
        if let Ok(instance) = probe_shimmy_at_port(port).await {
            return Ok(instance);
        }
    }

    // Fallback to shimmy's port allocator logic
    Err(DiscoveryError::NoShimmyFound)
}

async fn probe_shimmy_at_port(port: u16) -> Result<ShimmyInstance, DiscoveryError> {
    let health_url = format!("http://localhost:{}/v1/models", port);
    let response = reqwest::get(&health_url).await
        .map_err(|_| DiscoveryError::InvalidResponse)?;

    if response.status().is_success() {
        // Also verify WebSocket health endpoint exists for console feature
        let ws_health_url = format!("http://localhost:{}/ws/health", port);
        let ws_response = reqwest::get(&ws_health_url).await
            .map_err(|_| DiscoveryError::InvalidResponse)?;
        
        if ws_response.status().is_success() {
            Ok(ShimmyInstance {
                http_port: port,
                websocket_port: port, // Same port for WebSocket upgrade
                base_url: format!("http://localhost:{}", port),
                health_status: HealthStatus::Healthy,
            })
        } else {
            Err(DiscoveryError::InvalidResponse)
        }
    } else {
        Err(DiscoveryError::InvalidResponse)
    }
}

pub async fn discover_shimmy_websocket() -> Result<String, DiscoveryError> {
    let instance = discover_shimmy_instance().await?;
    Ok(format!("ws://localhost:{}/ws/console", instance.websocket_port))
}