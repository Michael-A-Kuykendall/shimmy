use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

lazy_static! {
    pub static ref GLOBAL_PORT_ALLOCATOR: PortAllocator = PortAllocator::new();
}

#[derive(Debug)]
pub struct PortAllocator {
    allocated_ports: Arc<Mutex<HashMap<String, u16>>>,
    #[allow(dead_code)] // Currently unused but may be needed for future port range customization
    port_range: (u16, u16),
}

impl PortAllocator {
    pub fn new() -> Self {
        Self {
            allocated_ports: Arc::new(Mutex::new(HashMap::new())),
            port_range: (11435, 11535), // Reasonable range (100 ports) for faster search
        }
    }

    #[allow(dead_code)] // Currently unused but may be needed for future service management
    pub fn find_available_port(&self, service_name: &str) -> Result<u16> {
        let mut allocated = self.allocated_ports.lock();

        // Check if already allocated for this service
        if let Some(&existing_port) = allocated.get(service_name) {
            if self.is_port_available(existing_port) {
                return Ok(existing_port);
            } else {
                // Port no longer available, remove from tracking
                allocated.remove(service_name);
            }
        }

        // Find new available port
        for port in self.port_range.0..=self.port_range.1 {
            if self.is_port_available(port) {
                allocated.insert(service_name.to_string(), port);
                return Ok(port);
            }
        }

        // Fallback to OS ephemeral port if range exhausted
        match self.find_ephemeral_port() {
            Ok(port) => {
                allocated.insert(service_name.to_string(), port);
                Ok(port)
            }
            Err(_) => Err(anyhow!(
                "No available ports in range {}..{} and OS ephemeral allocation failed",
                self.port_range.0,
                self.port_range.1
            )),
        }
    }

    /// Resolve bind address string to SocketAddr with smart defaults
    pub fn resolve_bind_address(&self, bind: &str) -> Result<SocketAddr> {
        match bind {
            "auto" => {
                // Try environment variable first
                if let Ok(env_addr) = std::env::var("SHIMMY_BIND_ADDRESS") {
                    return env_addr
                        .parse()
                        .map_err(|e| anyhow!("Invalid SHIMMY_BIND_ADDRESS '{}': {}", env_addr, e));
                }

                // Try default port first (11435)
                if self.is_port_available(11435) {
                    return Ok(SocketAddr::from(([127, 0, 0, 1], 11435)));
                }

                // Find any available port in range
                let port = self.find_available_port("shimmy-main")?;
                Ok(SocketAddr::from(([127, 0, 0, 1], port)))
            }
            _ => {
                // Parse explicit address
                bind.parse()
                    .map_err(|e| anyhow!("Invalid bind address '{}': {}", bind, e))
            }
        }
    }

    #[allow(dead_code)]
    pub fn allocate_ephemeral_port(&self, service_name: &str) -> Result<u16> {
        let mut allocated = self.allocated_ports.lock();

        // Generate ephemeral port
        let port = self.find_ephemeral_port()?;
        allocated.insert(service_name.to_string(), port);
        Ok(port)
    }

    #[allow(dead_code)]
    pub fn release_port(&self, port: u16) {
        let mut allocated = self.allocated_ports.lock();
        allocated.retain(|_, &mut v| v != port);
    }

    #[allow(dead_code)] // Used by find_available_port which is currently unused
    fn is_port_available(&self, port: u16) -> bool {
        TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).is_ok()
    }

    #[allow(dead_code)]
    pub fn find_ephemeral_port(&self) -> Result<u16> {
        // Use OS ephemeral port allocation
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener); // Release the port
        Ok(port)
    }

    #[allow(dead_code)]
    pub fn allocated_ports(&self) -> HashMap<String, u16> {
        self.allocated_ports.lock().clone()
    }
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_allocation() {
        let allocator = PortAllocator::new();
        let port1 = allocator.allocate_ephemeral_port("test1").unwrap();
        let port2 = allocator.allocate_ephemeral_port("test2").unwrap();

        assert_ne!(port1, port2);

        allocator.release_port(port1);
        allocator.release_port(port2);
    }

    #[test]
    fn test_find_available_port() {
        let allocator = PortAllocator::new();
        let port = allocator.find_available_port("test-service").unwrap();
        assert!(port >= 11435);

        // Second call should return same port
        let port2 = allocator.find_available_port("test-service").unwrap();
        assert_eq!(port, port2);

        allocator.release_port(port);
    }

    #[test]
    fn test_resolve_bind_address_auto() {
        let allocator = PortAllocator::new();

        // Test auto resolution - should return 127.0.0.1 with some port
        let addr = allocator.resolve_bind_address("auto").unwrap();
        assert_eq!(
            addr.ip(),
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))
        );
        assert!(addr.port() >= 11435);
    }

    #[test]
    fn test_resolve_bind_address_explicit() {
        let allocator = PortAllocator::new();

        // Test explicit address parsing
        let addr = allocator
            .resolve_bind_address("192.168.1.100:9000")
            .unwrap();
        assert_eq!(
            addr.ip(),
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 100))
        );
        assert_eq!(addr.port(), 9000);
    }

    #[test]
    fn test_resolve_bind_address_invalid() {
        let allocator = PortAllocator::new();

        // Test invalid address
        let result = allocator.resolve_bind_address("invalid-address");
        assert!(result.is_err());

        // Test empty string
        let result = allocator.resolve_bind_address("");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_bind_address_env_var() {
        let allocator = PortAllocator::new();

        // Set environment variable
        std::env::set_var("SHIMMY_BIND_ADDRESS", "10.0.0.1:8888");

        // Test that environment variable is used for auto
        let addr = allocator.resolve_bind_address("auto").unwrap();
        assert_eq!(
            addr.ip(),
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1))
        );
        assert_eq!(addr.port(), 8888);

        // Clean up
        std::env::remove_var("SHIMMY_BIND_ADDRESS");
    }
}
