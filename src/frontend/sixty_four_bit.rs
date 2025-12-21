// Re-export 64bit frontend module
// Module names can't start with digits in Rust, so we use sixty_four_bit

#[path = "64bit/mod.rs"]
mod inner;

pub use inner::*;
