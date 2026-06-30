/// MLX support regression test for Issue #68
///
/// MLX is a legacy-empty feature stub (`mlx = []`) in v2.0+.
/// Primary GPU path is Airframe/wgpu. These tests verify the stub exists.

#[test]
fn test_mlx_feature_availability() {
    #[cfg(feature = "mlx")]
    {
        use shimmy::engine::mlx::MLXEngine;
        let _engine_check = std::marker::PhantomData::<MLXEngine>;
        println!("✅ MLX engine code is available when feature is enabled");
    }

    #[cfg(not(feature = "mlx"))]
    {
        println!("ℹ️ MLX feature not enabled in this test build");
    }
}

#[test]
fn test_mlx_regression_prevention() {
    let cargo_toml =
        std::fs::read_to_string("Cargo.toml").expect("Should be able to read Cargo.toml");

    assert!(
        cargo_toml.contains("apple = [") && cargo_toml.contains("airframe"),
        "Apple feature set should include airframe (wgpu/Metal) in v2.0 Cargo.toml"
    );

    assert!(
        cargo_toml.contains("mlx"),
        "Cargo.toml should declare the mlx feature stub for backwards compatibility"
    );

    println!("MLX regression prevention test passed (v2.0 Airframe default)");
}
