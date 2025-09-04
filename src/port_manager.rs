use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPortConfig {
    pub scan_ranges: Vec<PortRange>,
    pub excluded_ports: Vec<u16>,
    pub max_attempts: u32,
    pub preferred_spacing: u16,
}

pub struct PortAllocator {
    config: DynamicPortConfig,
    allocated_ports: Arc<Mutex<HashMap<u16, String>>>,
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new(DynamicPortConfig::default())
    }
}

impl PortAllocator {
    pub fn new(config: DynamicPortConfig) -> Self {
        Self {
            config,
            allocated_ports: Arc::new(Mutex::new(HashMap::new())),
        }
    }


    pub fn is_port_available(&self, port: u16) -> bool {
        if self.config.excluded_ports.contains(&port) {
            return false;
        }

        if let Ok(allocated) = self.allocated_ports.lock() {
            if allocated.contains_key(&port) {
                return false;
            }
        }

        let addr = format!("127.0.0.1:{}", port);
        match addr.parse::<SocketAddr>() {
            Ok(socket_addr) => {
                TcpListener::bind(socket_addr).is_ok()
            }
            Err(_) => false,
        }
    }

    pub fn find_available_port(&self, project_name: &str) -> Result<u16, String> {
        let mut rng = rand::thread_rng();
        
        for _ in 0..self.config.max_attempts {
            let range_index = rng.gen_range(0..self.config.scan_ranges.len());
            let port_range = &self.config.scan_ranges[range_index];
            
            let port_span = port_range.end - port_range.start + 1;
            let base_port = port_range.start + rng.gen_range(0..port_span);
            
            for offset in 0..(self.config.preferred_spacing * 2) {
                let candidate_port = base_port + offset;
                
                if candidate_port > port_range.end {
                    break;
                }
                
                if self.is_port_available(candidate_port) {
                    if let Ok(mut allocated) = self.allocated_ports.lock() {
                        allocated.insert(candidate_port, project_name.to_string());
                        println!("Allocated port {} to project '{}'", candidate_port, project_name);
                        return Ok(candidate_port);
                    }
                }
            }
        }
        
        Err(format!("Could not find available port after {} attempts", self.config.max_attempts))
    }

    /// Release a previously allocated port back to the pool
    #[allow(dead_code)] // Public API for external integrations
    pub fn release_port(&self, port: u16) {
        if let Ok(mut allocated) = self.allocated_ports.lock() {
            if let Some(project_name) = allocated.remove(&port) {
                println!("Released port {} from project '{}'", port, project_name);
            }
        }
    }

    /// Get a snapshot of all currently allocated ports
    #[allow(dead_code)] // Public API for monitoring/debugging
    pub fn get_allocated_ports(&self) -> HashMap<u16, String> {
        self.allocated_ports.lock().unwrap_or_else(|_| {
            panic!("Port allocator mutex poisoned")
        }).clone()
    }

    /// Scan a range of ports to find available ones
    #[allow(dead_code)] // Public API for advanced port management
    pub fn scan_port_range(&self, start: u16, end: u16) -> Vec<u16> {
        let mut available = Vec::new();
        
        for port in start..=end {
            if self.is_port_available(port) {
                available.push(port);
            }
        }
        
        available
    }

    /// Allocate an ephemeral port from the system-defined range
    #[allow(dead_code)] // Public API for high-performance scenarios
    pub fn allocate_ephemeral_port(&self, project_name: &str) -> Result<u16, String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| format!("Failed to bind ephemeral port: {}", e))?;
        let addr = listener.local_addr().map_err(|e| format!("Failed to get local addr: {}", e))?;
        let port = addr.port();
        
        if let Ok(mut allocated) = self.allocated_ports.lock() {
            allocated.insert(port, project_name.to_string());
        }
        
        Ok(port)
    }
}

impl Default for DynamicPortConfig {
    fn default() -> Self {
        Self {
            scan_ranges: vec![
                PortRange {
                    start: 8000,
                    end: 8999,
                    description: "Primary development range".to_string(),
                },
                PortRange {
                    start: 9000,
                    end: 9999,
                    description: "Secondary development range".to_string(),
                },
                PortRange {
                    start: 10000,
                    end: 19999,
                    description: "User application range".to_string(),
                },
                PortRange {
                    start: 20000,
                    end: 29999,
                    description: "Extended user range".to_string(),
                },
            ],
            excluded_ports: vec![
                8080, 8443, 8888, 9090, 3000, 3001, 4200, 5000, 5432, 6379, 11434, 27017,
            ],
            max_attempts: 100,
            preferred_spacing: 10,
        }
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_PORT_ALLOCATOR: PortAllocator = PortAllocator::default();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_port_allocator_creation() {
        let config = DynamicPortConfig::default();
        let allocator = PortAllocator::new(config);
        
        assert!(!allocator.config.scan_ranges.is_empty());
        assert!(!allocator.config.excluded_ports.is_empty());
    }

    #[test]
    fn test_is_port_available_excluded() {
        let allocator = PortAllocator::default();
        
        assert!(!allocator.is_port_available(8080)); // In excluded list
    }

    #[test]
    fn test_ephemeral_port_allocation() {
        let allocator = PortAllocator::default();
        let port = allocator.allocate_ephemeral_port("test-project").unwrap();
        
        assert!(port > 0);
        
        let allocated = allocator.get_allocated_ports();
        assert_eq!(allocated.get(&port), Some(&"test-project".to_string()));
    }

    #[test]
    fn test_port_release() {
        let allocator = PortAllocator::default();
        let port = allocator.allocate_ephemeral_port("test").unwrap();
        
        allocator.release_port(port);
        
        let allocated = allocator.get_allocated_ports();
        assert!(!allocated.contains_key(&port));
    }

    #[test]
    fn test_find_available_port() {
        let allocator = PortAllocator::default();
        let port = allocator.find_available_port("test-project");
        
        assert!(port.is_ok());
        let port = port.unwrap();
        
        let allocated = allocator.get_allocated_ports();
        assert_eq!(allocated.get(&port), Some(&"test-project".to_string()));
    }

    #[test]
    fn test_scan_port_range() {
        let allocator = PortAllocator::default();
        let available = allocator.scan_port_range(25000, 25010);
        
        assert!(!available.is_empty());
        assert!(available.len() <= 11);
    }

    #[test]
    fn test_multiple_port_allocation() {
        let allocator = PortAllocator::default();
        
        let port1 = allocator.find_available_port("project1").unwrap();
        let port2 = allocator.find_available_port("project2").unwrap();
        
        assert_ne!(port1, port2);
        
        let allocated = allocator.get_allocated_ports();
        assert_eq!(allocated.len(), 2);
        assert!(allocated.contains_key(&port1));
        assert!(allocated.contains_key(&port2));
    }

    #[test]
    fn test_port_spacing_preference() {
        let config = DynamicPortConfig {
            scan_ranges: vec![PortRange {
                start: 20000,
                end: 20100,
                description: "Test range".to_string(),
            }],
            excluded_ports: vec![],
            max_attempts: 10,
            preferred_spacing: 5,
        };
        
        let allocator = PortAllocator::new(config);
        let port1 = allocator.find_available_port("test1").unwrap();
        let port2 = allocator.find_available_port("test2").unwrap();
        
        assert!(port1 >= 20000 && port1 <= 20100);
        assert!(port2 >= 20000 && port2 <= 20100);
    }

    #[test]
    fn test_concurrent_port_allocation() {
        use std::thread;
        
        let allocator = Arc::new(PortAllocator::default());
        let mut handles = vec![];
        
        for i in 0..5 {
            let allocator_clone = Arc::clone(&allocator);
            let handle = thread::spawn(move || {
                allocator_clone.find_available_port(&format!("thread-{}", i))
            });
            handles.push(handle);
        }
        
        let mut ports = vec![];
        for handle in handles {
            if let Ok(Ok(port)) = handle.join() {
                ports.push(port);
            }
        }
        
        ports.sort();
        ports.dedup();
        assert_eq!(ports.len(), 5); // All ports should be unique
    }

    #[test]
    fn test_config_serialization() {
        let config = DynamicPortConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: DynamicPortConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.max_attempts, parsed.max_attempts);
        assert_eq!(config.preferred_spacing, parsed.preferred_spacing);
        assert_eq!(config.scan_ranges.len(), parsed.scan_ranges.len());
    }
}