// 64bit Frontend - AGA graphics system
// Premium console experience with authentic low-res styling

use crate::frontend::Frontend;
use anyhow::Result;

pub mod aga_app;
pub mod websocket_client;

/// 64bit Frontend with authentic 32bit styling
pub struct SixtyFourBitFrontend {
    initialized: bool,
}

impl Default for SixtyFourBitFrontend {
    fn default() -> Self {
        Self::new()
    }
}

impl SixtyFourBitFrontend {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Frontend for SixtyFourBitFrontend {
    fn name(&self) -> &'static str {
        "64bit"
    }

    fn initialize(&mut self) -> Result<()> {
        if !self.initialized {
            // Initialize 64bit interface
            println!("Initializing 64bit AGA frontend...");
            println!("Resolution: 320×256 PAL with authentic AGA color palette");
            println!("WebSocket endpoint: ws://localhost:11435/ws/console");
            println!("Metrics endpoint: http://localhost:11435/api/metrics");
            self.initialized = true;
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        if self.initialized {
            println!("Cleaning up 64bit frontend...");
            self.initialized = false;
        }
        Ok(())
    }
}

/// Launch the 64bit frontend
pub async fn launch_64bit_frontend() -> Result<()> {
    println!("Launching Shimmy Console - 64bit AGA Frontend");
    aga_app::run_shimmy_console().await
}
