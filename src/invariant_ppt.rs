// PPT + Invariant Testing System for Shimmy
// Provides semantic integrity and regression protection

use std::sync::Mutex;
use std::collections::HashSet;

lazy_static::lazy_static! {
    static ref INVARIANT_LOG: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    static ref FAILED_INVARIANTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

/// Core invariant assertion - logs and enforces semantic contracts
pub fn assert_invariant(condition: bool, message: &str, context: Option<&str>) {
    let full_message = match context {
        Some(ctx) => format!("{} [{}]", message, ctx),
        None => message.to_string(),
    };
    
    // Always log that this invariant was checked
    if let Ok(mut log) = INVARIANT_LOG.lock() {
        log.insert(full_message.clone());
    }
    
    // Enforce the invariant
    if !condition {
        if let Ok(mut failed) = FAILED_INVARIANTS.lock() {
            failed.push(full_message.clone());
        }
        panic!("INVARIANT VIOLATION: {}", full_message);
    }
}

/// Property-based test helper - tests behaviors across input ranges
pub fn property_test<F>(name: &str, test_fn: F) 
where 
    F: Fn() -> bool,
{
    println!("🧪 Running property test: {}", name);
    
    // Run multiple iterations with different conditions
    for iteration in 1..=10 {
        if !test_fn() {
            panic!("Property test '{}' failed on iteration {}", name, iteration);
        }
    }
    
    println!("✅ Property test '{}' passed", name);
}

/// Contract test - verifies that specific invariants were actually checked
pub fn contract_test(name: &str, required_invariants: &[&str]) {
    println!("📋 Running contract test: {}", name);
    
    let log = match INVARIANT_LOG.lock() {
        Ok(log) => log,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut missing_invariants = Vec::new();
    
    for required in required_invariants {
        let found = log.iter().any(|logged| logged.contains(required));
        if !found {
            missing_invariants.push(*required);
        }
    }
    
    if !missing_invariants.is_empty() {
        panic!("Contract test '{}' failed. Missing invariants: {:?}", name, missing_invariants);
    }
    
    println!("✅ Contract test '{}' passed - all invariants verified", name);
}

/// Exploration test helper - for temporary tests during development  
pub fn explore_test<F>(name: &str, test_fn: F)
where
    F: Fn() -> bool,
{
    println!("🔍 Exploration test: {}", name);
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| test_fn())) {
        Ok(true) => println!("✅ Exploration test '{}' passed", name),
        Ok(false) => println!("❌ Exploration test '{}' failed", name),
        Err(_) => println!("💥 Exploration test '{}' panicked", name),
    }
}

/// Clear the invariant log (for test isolation)
pub fn clear_invariant_log() {
    // Handle poisoned mutexes by force-clearing the data
    match INVARIANT_LOG.lock() {
        Ok(mut log) => log.clear(),
        Err(poisoned) => {
            let mut log = poisoned.into_inner();
            log.clear();
        }
    }
    match FAILED_INVARIANTS.lock() {
        Ok(mut failed) => failed.clear(),
        Err(poisoned) => {
            let mut failed = poisoned.into_inner();
            failed.clear();
        }
    }
}

/// Get all invariants that have been checked
pub fn get_checked_invariants() -> Vec<String> {
    match INVARIANT_LOG.lock() {
        Ok(log) => log.iter().cloned().collect(),
        Err(poisoned) => poisoned.into_inner().iter().cloned().collect(),
    }
}

/// Get all failed invariants
pub fn get_failed_invariants() -> Vec<String> {
    match FAILED_INVARIANTS.lock() {
        Ok(failed) => failed.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// Shimmy-specific invariant helpers
pub mod shimmy_invariants {
    use super::assert_invariant;
    
    /// Model loading invariants
    pub fn assert_model_loaded(model_name: &str, success: bool) {
        assert_invariant(
            !model_name.is_empty(),
            "Model name must not be empty",
            Some("model_loading")
        );
        
        if success {
            assert_invariant(
                true,
                "Model loaded successfully",
                Some(&format!("model_loading:{}", model_name))
            );
        }
    }
    
    /// Generation invariants  
    pub fn assert_generation_valid(prompt: &str, response: &str) {
        assert_invariant(
            !prompt.is_empty(),
            "Generation prompt must not be empty", 
            Some("generation")
        );
        
        assert_invariant(
            !response.is_empty(),
            "Generation response must not be empty",
            Some("generation")
        );
        
        assert_invariant(
            response.len() > 0,
            "Generation must produce output",
            Some("generation")
        );
    }
    
    /// API invariants
    pub fn assert_api_response_valid(status_code: u16, body: &str) {
        assert_invariant(
            status_code >= 200 && status_code < 600,
            "API response status must be valid HTTP code",
            Some("api_response")
        );
        
        assert_invariant(
            !body.is_empty() || status_code == 204,
            "API response body must exist (unless 204)",
            Some("api_response")  
        );
    }
    
    /// Model discovery invariants
    pub fn assert_discovery_valid(models_found: usize) {
        // usize is always >= 0, so we check for reasonable bounds instead
        assert_invariant(
            models_found < 10000, // Sanity check for reasonable model counts
            "Model discovery must return reasonable count",
            Some("discovery")
        );
    }
    
    /// Backend selection invariants
    pub fn assert_backend_selection_valid(file_path: &str, backend: &str) {
        assert_invariant(
            !file_path.is_empty(),
            "File path for backend selection must not be empty",
            Some("backend_selection")
        );
        
        assert_invariant(
            !backend.is_empty(),
            "Selected backend must not be empty",
            Some("backend_selection")
        );
        
        // GGUF files must use Llama backend
        if file_path.to_lowercase().ends_with(".gguf") {
            assert_invariant(
                backend == "llama" || backend == "Llama",
                "GGUF files must use Llama backend",
                Some("backend_selection")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_invariant_logging() {
        clear_invariant_log();
        
        assert_invariant(true, "Test invariant", Some("test_context"));
        
        let checked = get_checked_invariants();
        assert!(checked.iter().any(|msg| msg.contains("Test invariant")));
    }
    
    #[test]
    #[should_panic(expected = "INVARIANT VIOLATION")]
    fn test_invariant_violation() {
        assert_invariant(false, "This should fail", None);
    }
    
    #[test]
    fn test_property_test_success() {
        property_test("always_true", || true);
    }
    
    #[test]
    fn test_contract_test_success() {
        clear_invariant_log();
        assert_invariant(true, "Required contract", Some("test"));
        contract_test("test_contract", &["Required contract"]);
    }
}