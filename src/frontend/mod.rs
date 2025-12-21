// Minimal frontend trait for modular UI system
use anyhow::Result;
use std::collections::HashMap;

/// Basic frontend trait - start minimal, expand later
pub trait Frontend: Send + Sync {
    fn name(&self) -> &'static str;
    fn initialize(&mut self) -> Result<()>;
    fn cleanup(&mut self) -> Result<()>;
}

/// Placeholder frontend for testing
pub struct NoOpFrontend;

impl Frontend for NoOpFrontend {
    fn name(&self) -> &'static str {
        "noop"
    }

    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Basic frontend manager - test instantiation only
pub struct FrontendManager {
    frontends: HashMap<String, Box<dyn Frontend>>,
    #[allow(dead_code)]
    active_frontend: Option<String>,
}

impl Default for FrontendManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FrontendManager {
    pub fn new() -> Self {
        Self {
            frontends: HashMap::new(),
            active_frontend: None,
        }
    }

    pub fn register_frontend(&mut self, frontend: Box<dyn Frontend>) -> Result<()> {
        let name = frontend.name().to_string();
        self.frontends.insert(name, frontend);
        Ok(())
    }

    pub fn list_frontends(&self) -> Vec<&str> {
        self.frontends.keys().map(|s| s.as_str()).collect()
    }
}

pub mod cyberpunk;

#[cfg(feature = "aga-frontend")]
pub mod sixty_four_bit;

#[cfg(test)]
mod test_basic;
