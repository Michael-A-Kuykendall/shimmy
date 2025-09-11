use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GLOBAL_PORT_ALLOCATOR: PortAllocator = PortAllocator::new();
}

#[derive(Debug)]
pub struct PortAllocator {
    allocated_ports: Arc<Mutex<HashMap<String, u16>>>,
    port_range: (u16, u16),
}

impl PortAllocator {
    pub fn new() -> Self {
        Self {
            allocated_ports: Arc::new(Mutex::new(HashMap::new())),
            port_range: (11435, 65535), // Start from shimmy default port
        }
    }

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

        Err(anyhow!("No available ports in range {}..{}", self.port_range.0, self.port_range.1))
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

    fn is_port_available(&self, port: u16) -> bool {
        match TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[allow(dead_code)]
    fn find_ephemeral_port(&self) -> Result<u16> {
        // Use OS ephemeral port allocation
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener); // Release the port
        Ok(port)
    }

    #[allow(dead_code)]
    pub fn get_allocated_ports(&self) -> HashMap<String, u16> {
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
}