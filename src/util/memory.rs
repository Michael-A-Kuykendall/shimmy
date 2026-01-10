/// Memory management utilities for Issue #108
///
/// Provides memory estimation and warnings to help users understand
/// system requirements for large language models.
use sysinfo::System;

/// Get total system memory in bytes
#[allow(dead_code)] // Placeholder utility for future use
pub fn get_total_memory() -> u64 {
    let mut system = System::new_all();
    system.refresh_memory();
    system.total_memory()
}

/// Get available system memory in bytes
#[allow(dead_code)] // Placeholder utility for future use
pub fn get_available_memory() -> u64 {
    let mut system = System::new_all();
    system.refresh_memory();
    system.available_memory()
}

/// Estimate memory requirements for a model file
///
/// This provides a rough estimate based on file size and typical
/// memory overhead for quantized models.
#[allow(dead_code)] // Placeholder utility for future use
pub fn estimate_memory_requirements(model_file_size: u64) -> MemoryEstimate {
    let file_size_gb = model_file_size as f64 / 1_024_000_000.0;

    // Rough estimates based on typical quantized model behavior:
    // - File size is the compressed/quantized size
    // - Runtime needs additional memory for:
    //   * Context buffers
    //   * Intermediate activations
    //   * Operating system overhead
    let runtime_multiplier = 1.8; // 80% overhead is typical
    let estimated_runtime_gb = file_size_gb * runtime_multiplier;

    MemoryEstimate {
        file_size_gb,
        estimated_runtime_gb,
        needs_moe_offloading: estimated_runtime_gb > 16.0, // >16GB suggests MoE needed
    }
}

/// Memory requirement estimate
#[derive(Debug)]
#[allow(dead_code)] // Placeholder utility for future use
pub struct MemoryEstimate {
    pub file_size_gb: f64,
    pub estimated_runtime_gb: f64,
    pub needs_moe_offloading: bool,
}

/// Check if system has enough memory for a model
#[allow(dead_code)] // Placeholder utility for future use
pub fn check_memory_availability(required_gb: f64) -> MemoryAvailability {
    let total_gb = get_total_memory() as f64 / 1_024_000_000.0;
    let available_gb = get_available_memory() as f64 / 1_024_000_000.0;

    let status = if available_gb >= required_gb {
        MemoryStatus::Sufficient
    } else if total_gb >= required_gb {
        MemoryStatus::Tight
    } else {
        MemoryStatus::Insufficient
    };

    MemoryAvailability {
        total_gb,
        available_gb,
        required_gb,
        status,
    }
}

/// Memory availability analysis
#[derive(Debug)]
#[allow(dead_code)] // Placeholder utility for future use
pub struct MemoryAvailability {
    pub total_gb: f64,
    pub available_gb: f64,
    pub required_gb: f64,
    pub status: MemoryStatus,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)] // Placeholder utility for future use
pub enum MemoryStatus {
    Sufficient,   // Available memory > required
    Tight,        // Total memory >= required but available < required
    Insufficient, // Total memory < required
}

impl MemoryAvailability {
    /// Get user-friendly recommendations based on memory status
    #[allow(dead_code)] // Placeholder utility for future use
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        match self.status {
            MemoryStatus::Sufficient => {
                recommendations.push("✅ Sufficient memory available".to_string());
            }
            MemoryStatus::Tight => {
                recommendations.push("⚠️  Close other applications to free memory".to_string());
                recommendations.push("💡 Consider restarting to free cached memory".to_string());
            }
            MemoryStatus::Insufficient => {
                recommendations.push("❌ Insufficient system memory".to_string());
                recommendations
                    .push("💡 Use a smaller model (7B instead of 14B parameters)".to_string());
                recommendations.push("💡 Add more RAM to your system".to_string());
                recommendations
                    .push("💡 Use a more quantized model (Q4_K_M instead of Q8_0)".to_string());
            }
        }

        // Always suggest MoE when applicable
        if self.required_gb > 16.0 {
            recommendations.push(
                "🧠 MoE CPU offloading would help (currently disabled - Issue #108)".to_string(),
            );
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_estimation() {
        // Test with a typical 7B model file (~4GB)
        let estimate = estimate_memory_requirements(4_000_000_000);
        assert!(estimate.file_size_gb > 3.0 && estimate.file_size_gb < 5.0);
        assert!(estimate.estimated_runtime_gb > 6.0); // Should be ~7.2GB
        assert!(!estimate.needs_moe_offloading); // 7B shouldn't need MoE
    }

    #[test]
    fn test_large_model_estimation() {
        // Test with a large 14B model file (~8GB)
        let estimate = estimate_memory_requirements(8_000_000_000);
        assert!(estimate.file_size_gb > 7.0 && estimate.file_size_gb < 9.0);
        assert!(estimate.estimated_runtime_gb > 14.0); // Should be ~14.4GB
        assert!(!estimate.needs_moe_offloading); // 14B is borderline
    }

    #[test]
    fn test_huge_model_estimation() {
        // Test with a huge model requiring MoE (~20GB)
        let estimate = estimate_memory_requirements(20_000_000_000);
        assert!(estimate.needs_moe_offloading); // Should definitely need MoE
        assert!(estimate.estimated_runtime_gb > 30.0);
    }

    #[test]
    fn test_memory_availability_logic() {
        let availability = MemoryAvailability {
            total_gb: 16.0,
            available_gb: 12.0,
            required_gb: 10.0,
            status: MemoryStatus::Sufficient,
        };

        let recommendations = availability.get_recommendations();
        assert!(recommendations.iter().any(|r| r.contains("Sufficient")));
    }

    #[test]
    fn test_insufficient_memory_recommendations() {
        let availability = MemoryAvailability {
            total_gb: 8.0,
            available_gb: 6.0,
            required_gb: 12.0,
            status: MemoryStatus::Insufficient,
        };

        let recommendations = availability.get_recommendations();
        assert!(recommendations.iter().any(|r| r.contains("smaller model")));
        assert!(recommendations.iter().any(|r| r.contains("Add more RAM")));
    }
}
