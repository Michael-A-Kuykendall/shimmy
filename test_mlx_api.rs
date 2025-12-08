#[cfg(feature = "mlx")]
fn main() {
    // Test what MLX APIs are available
    println!("Testing MLX API availability...");
    
    // Check if we can import mlx-lm
    use mlx_lm::model::Model;
    println!("✅ mlx_lm::model::Model imported successfully");
    
    // Check mlx-rs
    use mlx_rs::array::Array;
    println!("✅ mlx_rs::array::Array imported successfully");
    
    println!("MLX crates are available!");
}

#[cfg(not(feature = "mlx"))]
fn main() {
    println!("MLX feature not enabled");
}
