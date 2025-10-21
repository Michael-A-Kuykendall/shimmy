/// Regression test for Issue #113: OpenAI API compatibility for frontends
///
/// GitHub: https://github.com/Michael-A-Kuykendall/shimmy/issues/113
///
/// **Bug**: OpenAI API responses missing fields required by frontend frameworks
/// **Fix**: Enhanced Model structure with permission, root, parent fields
// **This test**: Verifies OpenAI API response structure matches spec
#[cfg(test)]
mod issue_113_tests {
    use shimmy::openai_compat::{Model, ModelsResponse};

    #[test]
    fn test_openai_model_structure_complete() {
        // Test that Model struct has all OpenAI-compatible fields
        let model = Model {
            id: "test-model".to_string(),
            object: "model".to_string(),
            created: 1640995200,
            owned_by: "shimmy".to_string(),
            permission: None,
            root: Some("test-model".to_string()),
            parent: None,
        };

        // Verify all fields can be set
        assert_eq!(model.id, "test-model");
        assert_eq!(model.object, "model");
        assert_eq!(model.created, 1640995200);
        assert_eq!(model.owned_by, "shimmy");
        assert_eq!(model.root, Some("test-model".to_string()));

        println!("✅ Issue #113: Model structure has all OpenAI fields");
    }

    #[test]
    fn test_openai_model_serialization() {
        // Test that Model serializes correctly for frontend compatibility
        let model = Model {
            id: "test-model".to_string(),
            object: "model".to_string(),
            created: 1640995200,
            owned_by: "shimmy".to_string(),
            permission: None,
            root: Some("test-model".to_string()),
            parent: None,
        };

        let json = serde_json::to_value(&model).unwrap();

        // Verify required fields present
        assert_eq!(json["id"], "test-model");
        assert_eq!(json["owned_by"], "shimmy");
        assert_eq!(json["object"], "model");
        assert_eq!(json["created"], 1640995200);
        assert_eq!(json["root"], "test-model");

        // Verify optional fields handled correctly (omitted when None)
        assert!(json.get("permission").is_none());
        assert!(json.get("parent").is_none());

        println!("✅ Issue #113: Model serialization frontend-compatible");
    }

    #[test]
    fn test_openai_models_response_structure() {
        // Test that ModelsResponse matches OpenAI spec
        let model = Model {
            id: "test-model".to_string(),
            object: "model".to_string(),
            created: 1640995200,
            owned_by: "shimmy".to_string(),
            permission: None,
            root: Some("test-model".to_string()),
            parent: None,
        };

        let response = ModelsResponse {
            object: "list".to_string(),
            data: vec![model],
        };

        let json = serde_json::to_value(&response).unwrap();

        // Verify response structure
        assert_eq!(json["object"], "list");
        assert!(json["data"].is_array());
        assert_eq!(json["data"].as_array().unwrap().len(), 1);

        // Verify frontend expectations
        assert!(json.as_object().unwrap().contains_key("object"));
        assert!(json.as_object().unwrap().contains_key("data"));

        println!("✅ Issue #113: ModelsResponse structure OpenAI-compatible");
    }

    #[test]
    fn test_frontend_integration_fields() {
        // Test specific fields that frontends expect
        let model = Model {
            id: "gpt-3.5-turbo".to_string(), // Realistic model name
            object: "model".to_string(),
            created: 1640995200,
            owned_by: "shimmy".to_string(),
            permission: None,
            root: Some("gpt-3.5-turbo".to_string()),
            parent: None,
        };

        let json = serde_json::to_value(&model).unwrap();

        // Frontends expect these exact field names and types
        assert!(json.get("id").is_some());
        assert!(json.get("object").is_some());
        assert!(json.get("created").is_some());
        assert!(json.get("owned_by").is_some());
        assert!(json.get("root").is_some());

        // Types must match OpenAI spec
        assert!(json["id"].is_string());
        assert!(json["object"].is_string());
        assert!(json["created"].is_number());
        assert!(json["owned_by"].is_string());

        println!("✅ Issue #113: Frontend integration fields verified");
    }
}
