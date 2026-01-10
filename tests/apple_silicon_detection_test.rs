/// Apple Silicon GPU Detection Regression Test
///
/// Tests for issue #87: "Cannot get apple gpu info"
/// Ensures Apple Silicon hardware detection works properly regardless of MLX Python package installation
///
/// Issue: User with Apple M3 Pro received "MLX Backend: Not available (requires Apple Silicon)"
/// Root cause: check_mlx_availability() was checking for MLX Python packages instead of just hardware
/// Solution: Separate hardware detection from software installation checks
#[cfg(test)]
mod apple_silicon_tests {
    #[cfg(feature = "mlx")]
    use shimmy::engine::mlx::MLXEngine;

    #[test]
    #[cfg(feature = "mlx")]
    fn test_hardware_detection_independent_of_python_packages() {
        // This test ensures that Apple Silicon hardware detection
        // works regardless of whether MLX Python packages are installed

        let hardware_supported = MLXEngine::is_hardware_supported();
        let python_available = MLXEngine::check_mlx_python_available();

        // On Apple Silicon macOS, hardware should be detected even if Python packages aren't installed
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            // Hardware should always be detected on Apple Silicon macOS
            assert!(
                hardware_supported,
                "Apple Silicon hardware should be detected on aarch64 macOS"
            );

            // Python packages may or may not be installed, that's separate
            // We don't assert anything about python_available since it depends on user setup
            println!("Apple Silicon detected: {}", hardware_supported);
            println!("MLX Python packages available: {}", python_available);
        }

        // On non-Apple Silicon systems, hardware should not be detected
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            assert!(
                !hardware_supported,
                "Apple Silicon hardware should not be detected on non-Apple Silicon systems"
            );

            // Python packages might still be installed on non-Apple Silicon (e.g., for development)
            // but that's not what we're testing here
            println!("Apple Silicon detected: {}", hardware_supported);
            println!("MLX Python packages available: {}", python_available);
        }
    }

    #[test]
    #[cfg(feature = "mlx")]
    fn test_mlx_engine_creation() {
        // Test that MLXEngine can be created without panicking
        let engine = MLXEngine::new();

        // The engine should reflect hardware capabilities
        let is_available = engine.is_available();
        let hardware_supported = MLXEngine::is_hardware_supported();

        // These should be consistent
        assert_eq!(
            is_available, hardware_supported,
            "Engine availability should match hardware support"
        );
    }

    #[test]
    #[cfg(feature = "mlx")]
    fn test_gpu_info_output_regression() {
        // This test ensures that gpu-info command would provide helpful output
        // for Apple Silicon users, even without MLX Python packages

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            let hardware_supported = MLXEngine::is_hardware_supported();
            let python_available = MLXEngine::check_mlx_python_available();

            // Simulate the output logic from main.rs
            if hardware_supported {
                if python_available {
                    println!("🍎 MLX Backend: ✅ Available (Apple Silicon + MLX installed)");
                } else {
                    println!("🍎 MLX Backend: ⚠️  Hardware supported (Apple Silicon detected)");
                    println!("   📦 MLX Python packages not found");
                    println!("   💡 Install with: pip install mlx-lm");
                }
            } else {
                println!("🍎 MLX Backend: ❌ Not supported (requires Apple Silicon macOS)");
            }

            // The key fix: we should never see "Not supported" on Apple Silicon
            assert!(
                hardware_supported,
                "Apple Silicon users should see 'Hardware supported' not 'Not supported'"
            );
        }
    }

    #[test]
    #[cfg(feature = "mlx")]
    fn test_python_detection_graceful_failure() {
        // Ensure MLX Python detection fails gracefully when python3 is not available
        // or when MLX packages are not installed

        let python_available = MLXEngine::check_mlx_python_available();

        // This should not panic regardless of system state
        // Result can be true or false depending on whether MLX is installed
        println!("MLX Python packages detected: {}", python_available);

        // Test should pass whether MLX is installed or not
    }
}
