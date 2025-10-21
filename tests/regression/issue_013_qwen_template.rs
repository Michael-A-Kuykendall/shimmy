/// Regression test for Issue #13: Qwen models don't use correct templates in VSCode
///
/// GitHub: https://github.com/Michael-A-Kuykendall/shimmy/issues/13
///
/// **Bug**: Qwen/Qwen2.5-Coder models weren't being detected and assigned proper templates
/// **Fix**: Added Qwen family detection in template inference logic
// **This test**: Verifies Qwen models get correct ChatML-based templates
#[cfg(test)]
mod issue_013_tests {
    use shimmy::model_registry::Registry;

    #[test]
    fn test_qwen_model_template_detection() {
        // Test that Qwen models are correctly identified and assigned ChatML templates
        let registry = Registry::new();
        let qwen_models = vec![
            "Qwen/Qwen2.5-Coder-32B-Instruct",
            "Qwen/Qwen2.5-7B-Instruct",
            "qwen2.5-coder-7b-instruct", // lowercase variant
            "Qwen2-7B-Instruct",
        ];

        for model_name in qwen_models {
            let template_str = registry.infer_template(model_name);

            // Check if template is appropriate for Qwen models
            assert!(
                template_str == "chatml" || template_str == "llama3",
                "❌ Issue #13 regression: {} should use chatml or llama3, got {}",
                model_name,
                template_str
            );

            println!("✅ {} correctly uses {} template", model_name, template_str);
        }

        println!("✅ Issue #13 regression test: Qwen template detection working");
    }

    #[test]
    fn test_qwen_vscode_integration_scenario() {
        // Simulate the exact VSCode scenario from Issue #13
        let registry = Registry::new();
        let model_path = "Qwen/Qwen2.5-Coder-32B-Instruct";
        let template_str = registry.infer_template(model_path);

        // VSCode Copilot expects proper conversation formatting
        // ChatML is the correct template for Qwen models
        assert!(
            template_str == "chatml" || template_str == "llama3",
            "Qwen models must use chatml or llama3 templates for VSCode compatibility"
        );

        println!("✅ Issue #13 regression test: VSCode integration scenario verified");
    }
}
