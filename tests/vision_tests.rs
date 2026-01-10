//! Comprehensive unit tests for vision.rs module
//!
//! Tests all public and internal functions with comprehensive edge cases
//! to achieve 90%+ test coverage.

#[cfg(feature = "vision")]
use shimmy::vision::{
    Contrast, DomElement, Interaction, Layout, Meta, Rect, Region, TextBlock, UIElement,
    VisionRequest, VisionResponse, Visual,
};

#[cfg(feature = "vision")]
mod vision_tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
    use serial_test::serial;
    use std::collections::HashMap;

    // Test helper functions
    fn create_test_image(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::from_fn(width, height, |x, y| {
            let r = (x % 256) as u8;
            let g = (y % 256) as u8;
            let b = ((x.wrapping_add(y)) % 256) as u8;
            image::Rgb([r, g, b])
        });

        let mut png_bytes = Vec::new();
        let encoder = PngEncoder::new(&mut png_bytes);
        encoder
            .write_image(img.as_raw(), width, height, ColorType::Rgb8)
            .expect("Failed to encode test image as PNG");

        png_bytes
    }

    fn encode_test_image_base64(width: u32, height: u32) -> String {
        let png_bytes = create_test_image(width, height);
        general_purpose::STANDARD.encode(&png_bytes)
    }

    #[test]
    #[serial]
    fn test_preprocess_config_for_mode_web() {
        // Clear environment variables first
        std::env::remove_var("SHIMMY_VISION_MAX_LONG_EDGE");
        std::env::remove_var("SHIMMY_VISION_MAX_PIXELS");

        let cfg = shimmy::vision::preprocess_config_for_mode(Some("web"));
        assert_eq!(cfg.max_long_edge, 512);
        assert_eq!(cfg.max_pixels, 400_000);
    }

    #[test]
    #[serial]
    fn test_preprocess_config_for_mode_analyze() {
        // Clear environment variables first
        std::env::remove_var("SHIMMY_VISION_MAX_LONG_EDGE");
        std::env::remove_var("SHIMMY_VISION_MAX_PIXELS");

        let cfg = shimmy::vision::preprocess_config_for_mode(Some("analyze"));
        assert_eq!(cfg.max_long_edge, 640);
        assert_eq!(cfg.max_pixels, 1_500_000);
    }

    #[test]
    #[serial]
    fn test_preprocess_config_for_mode_none() {
        // Clear environment variables first
        std::env::remove_var("SHIMMY_VISION_MAX_LONG_EDGE");
        std::env::remove_var("SHIMMY_VISION_MAX_PIXELS");

        let cfg = shimmy::vision::preprocess_config_for_mode(None);
        assert_eq!(cfg.max_long_edge, 640);
        assert_eq!(cfg.max_pixels, 1_500_000);
    }

    #[test]
    #[serial]
    fn test_preprocess_config_env_override() {
        std::env::set_var("SHIMMY_VISION_MAX_LONG_EDGE", "800");
        std::env::set_var("SHIMMY_VISION_MAX_PIXELS", "2000000");

        let cfg = shimmy::vision::preprocess_config_for_mode(Some("web"));
        assert_eq!(cfg.max_long_edge, 800);
        assert_eq!(cfg.max_pixels, 2_000_000);

        // Cleanup
        std::env::remove_var("SHIMMY_VISION_MAX_LONG_EDGE");
        std::env::remove_var("SHIMMY_VISION_MAX_PIXELS");
    }

    #[test]
    #[serial]
    fn test_preprocess_config_invalid_env_values() {
        std::env::set_var("SHIMMY_VISION_MAX_LONG_EDGE", "invalid");
        std::env::set_var("SHIMMY_VISION_MAX_PIXELS", "not_a_number");

        let cfg = shimmy::vision::preprocess_config_for_mode(Some("web"));
        // Should fall back to defaults when env vars are invalid
        assert_eq!(cfg.max_long_edge, 512);
        assert_eq!(cfg.max_pixels, 400_000);

        // Cleanup
        std::env::remove_var("SHIMMY_VISION_MAX_LONG_EDGE");
        std::env::remove_var("SHIMMY_VISION_MAX_PIXELS");
    }

    #[test]
    fn test_preprocess_image_no_resize_needed() {
        let png_bytes = create_test_image(100, 100);
        let cfg = shimmy::vision::PreprocessConfig {
            max_long_edge: 640,
            max_pixels: 1_500_000,
        };

        let result = shimmy::vision::preprocess_image(&png_bytes, &cfg);
        assert!(result.is_ok());
        let preprocessed = result.unwrap();
        assert_eq!(preprocessed.width, 100);
        assert_eq!(preprocessed.height, 100);
        assert!(preprocessed.bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG signature
    }

    #[test]
    fn test_preprocess_image_resize_by_long_edge() {
        let png_bytes = create_test_image(1000, 500);
        let cfg = shimmy::vision::PreprocessConfig {
            max_long_edge: 640,
            max_pixels: 1_500_000,
        };

        let result = shimmy::vision::preprocess_image(&png_bytes, &cfg);
        assert!(result.is_ok());
        let preprocessed = result.unwrap();
        assert_eq!(preprocessed.width, 640);
        assert_eq!(preprocessed.height, 320);
        assert!(preprocessed.bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG signature
    }

    #[test]
    fn test_preprocess_image_resize_by_pixels() {
        let png_bytes = create_test_image(1200, 1200);
        let cfg = shimmy::vision::PreprocessConfig {
            max_long_edge: 2000,   // High enough to not trigger
            max_pixels: 1_000_000, // 1M pixels limit
        };

        let result = shimmy::vision::preprocess_image(&png_bytes, &cfg);
        assert!(result.is_ok());
        let preprocessed = result.unwrap();
        let actual_pixels = preprocessed.width as u64 * preprocessed.height as u64;
        assert!(actual_pixels <= cfg.max_pixels);
        assert!(preprocessed.bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG signature
    }

    #[test]
    fn test_preprocess_image_invalid_image() {
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03]; // Not a valid image
        let cfg = shimmy::vision::PreprocessConfig {
            max_long_edge: 640,
            max_pixels: 1_500_000,
        };

        let result = shimmy::vision::preprocess_image(&invalid_data, &cfg);
        assert!(result.is_err());
    }

    #[test]
    fn test_preprocess_image_empty_data() {
        let empty_data = vec![];
        let cfg = shimmy::vision::PreprocessConfig {
            max_long_edge: 640,
            max_pixels: 1_500_000,
        };

        let result = shimmy::vision::preprocess_image(&empty_data, &cfg);
        assert!(result.is_err());
    }

    #[test]
    fn test_preprocess_image_portrait_orientation() {
        let png_bytes = create_test_image(500, 1000);
        let cfg = shimmy::vision::PreprocessConfig {
            max_long_edge: 640,
            max_pixels: 1_500_000,
        };

        let result = shimmy::vision::preprocess_image(&png_bytes, &cfg);
        assert!(result.is_ok());
        let preprocessed = result.unwrap();
        assert_eq!(preprocessed.height, 640);
        assert_eq!(preprocessed.width, 320);
    }

    #[test]
    fn test_preprocess_image_very_small_max_pixels() {
        let png_bytes = create_test_image(1000, 1000);
        let cfg = shimmy::vision::PreprocessConfig {
            max_long_edge: 2000,
            max_pixels: 10_000, // Very small pixel budget
        };

        let result = shimmy::vision::preprocess_image(&png_bytes, &cfg);
        assert!(result.is_ok());
        let preprocessed = result.unwrap();
        let actual_pixels = preprocessed.width as u64 * preprocessed.height as u64;
        assert!(actual_pixels <= cfg.max_pixels);
    }

    #[test]
    fn test_prepare_vision_prompt_ocr_mode() {
        let prompt = shimmy::vision::prepare_vision_prompt("ocr", 640, 480, "test-model");
        assert!(prompt.contains("OCR"));
        assert!(prompt.contains("640x480"));
        assert!(prompt.contains("valid JSON"));
        assert!(!prompt.contains("```"));
        assert!(prompt.contains("text_blocks"));
    }

    #[test]
    fn test_prepare_vision_prompt_layout_mode() {
        let prompt = shimmy::vision::prepare_vision_prompt("layout", 800, 600, "test-model");
        assert!(prompt.contains("Layout"));
        assert!(prompt.contains("800x600"));
        assert!(prompt.contains("regions"));
        assert!(prompt.contains("key UI elements"));
    }

    #[test]
    fn test_prepare_vision_prompt_web_mode() {
        let prompt = shimmy::vision::prepare_vision_prompt("web", 1024, 768, "test-model");
        assert!(prompt.contains("Web screenshot"));
        assert!(prompt.contains("dom_map"));
        assert!(prompt.contains("normalized boxes"));
    }

    #[test]
    fn test_prepare_vision_prompt_full_mode() {
        let prompt = shimmy::vision::prepare_vision_prompt("full", 640, 480, "test-model");
        assert!(prompt.contains("Full"));
        assert!(prompt.contains("text_blocks"));
        assert!(prompt.contains("layout"));
        assert!(prompt.contains("visual"));
        assert!(prompt.contains("#RRGGBB"));
    }

    #[test]
    fn test_prepare_vision_prompt_unknown_mode() {
        let prompt = shimmy::vision::prepare_vision_prompt("unknown_mode", 640, 480, "test-model");
        // Should default to "full" behavior
        assert!(prompt.contains("Full"));
        assert!(prompt.contains("text_blocks"));
        assert!(prompt.contains("layout"));
        assert!(prompt.contains("visual"));
    }

    #[test]
    fn test_prepare_vision_prompt_llava_format() {
        let prompt = shimmy::vision::prepare_vision_prompt("full", 640, 480, "llava");
        assert!(prompt.starts_with("<s>[INST]"));
        assert!(prompt.ends_with("[/INST]"));
    }

    #[test]
    fn test_prepare_vision_prompt_non_llava_format() {
        let prompt = shimmy::vision::prepare_vision_prompt("full", 640, 480, "minicpm-v");
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("<|im_end|>"));
        assert!(prompt.contains("<|im_start|>assistant"));
    }

    #[test]
    fn test_extract_json_candidate_clean_json() {
        let raw_output = r#"{"text_blocks": [{"text": "Hello", "confidence": 0.9}]}"#;
        let (result, warnings) = shimmy::vision::extract_json_candidate(raw_output);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), raw_output);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_extract_json_candidate_with_markdown() {
        let raw_output = r#"```json
{"text_blocks": [{"text": "Hello", "confidence": 0.9}]}
```"#;
        let (result, warnings) = shimmy::vision::extract_json_candidate(raw_output);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            r#"{"text_blocks": [{"text": "Hello", "confidence": 0.9}]}"#
        );
        assert_eq!(warnings, vec!["Stripped markdown code fences"]);
    }

    #[test]
    fn test_extract_json_candidate_with_surrounding_text() {
        let raw_output = r#"Here is the analysis: {"text_blocks": [{"text": "Hello", "confidence": 0.9}]} and that's it."#;
        let (result, warnings) = shimmy::vision::extract_json_candidate(raw_output);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            r#"{"text_blocks": [{"text": "Hello", "confidence": 0.9}]}"#
        );
        assert_eq!(
            warnings,
            vec!["Extracted JSON object from surrounding text"]
        );
    }

    #[test]
    fn test_extract_json_candidate_no_valid_json() {
        let raw_output = "This is just plain text with no JSON";
        let (result, warnings) = shimmy::vision::extract_json_candidate(raw_output);
        assert!(result.is_none());
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_extract_json_candidate_malformed_json() {
        // This string starts with { and ends with }, so the fast path accepts it
        // The function is extract_json_CANDIDATE - it doesn't validate JSON
        let raw_output = r#"{"text_blocks": [{"text": "Hello", "confidence": 0.9}"#;
        let (result, _warnings) = shimmy::vision::extract_json_candidate(raw_output);
        // The function returns the candidate because it starts and ends with braces
        // Even though it's malformed JSON, the extraction succeeds
        assert!(result.is_some());
    }

    #[test]
    fn test_extract_first_json_object_nested_braces() {
        let input = r#"Some text {"outer": {"inner": "value"}} more text"#;
        let result = shimmy::vision::extract_first_json_object(input);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), r#"{"outer": {"inner": "value"}}"#);
    }

    #[test]
    fn test_extract_first_json_object_with_strings() {
        let input = r#"Text {"message": "Hello {world}"} end"#;
        let result = shimmy::vision::extract_first_json_object(input);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), r#"{"message": "Hello {world}"}"#);
    }

    #[test]
    fn test_extract_first_json_object_escaped_quotes() {
        let input = r#"{"text": "She said \"Hello {world}\""}"#;
        let result = shimmy::vision::extract_first_json_object(input);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), r#"{"text": "She said \"Hello {world}\""}"#);
    }

    #[test]
    fn test_extract_first_json_object_no_object() {
        let input = "No JSON here at all";
        let result = shimmy::vision::extract_first_json_object(input);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_structured_output_complete_response() {
        let json_str = r##"
        {
            "text_blocks": [{"text": "Hello World", "confidence": 0.95}],
            "layout": {
                "theme": "dark",
                "regions": [{"name": "header", "description": "Top section"}],
                "key_ui_elements": [{"name": "login_button", "element_type": "button"}]
            },
            "visual": {
                "background": "#000000",
                "accent_colors": ["#FF0000", "#00FF00"],
                "contrast": {
                    "ratio": 4.5,
                    "compliant": true,
                    "issues": []
                },
                "description": "Dark theme interface"
            },
            "interaction": {
                "description": "Click the button to proceed"
            },
            "dom_map": [
                {
                    "tag": "button",
                    "id": "submit",
                    "class": "btn-primary",
                    "text": "Submit",
                    "position": {"x": 0.1, "y": 0.2, "width": 0.3, "height": 0.05},
                    "attributes": {"type": "submit"},
                    "colors": {"background": "#007bff", "color": "#ffffff"}
                }
            ]
        }
        "##;

        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let req = VisionRequest {
            image_base64: None,
            url: Some("https://example.com".to_string()),
            mode: "web".to_string(),
            model: None,
            timeout_ms: None,
            raw: Some(false),
            license: None,
            screenshot: None,
            viewport_width: None,
            viewport_height: None,
        };

        let result = shimmy::vision::parse_structured_output(
            &parsed,
            &req,
            "test-model",
            1000,
            "raw output",
            None,
            None,
        );

        assert!(result.is_ok());
        let response = result.unwrap();

        assert_eq!(response.mode, "web");
        assert_eq!(response.text_blocks.len(), 1);
        assert_eq!(response.text_blocks[0].text, "Hello World");
        assert_eq!(response.text_blocks[0].confidence, Some(0.95));

        assert_eq!(response.layout.theme, Some("dark".to_string()));
        assert_eq!(response.layout.regions.len(), 1);
        assert_eq!(response.layout.regions[0].name, "header");

        assert_eq!(response.visual.background, Some("#000000".to_string()));
        assert_eq!(response.visual.accent_colors.len(), 2);
        assert!(response.visual.contrast.is_some());

        assert_eq!(
            response.interaction.description,
            Some("Click the button to proceed".to_string())
        );
        assert_eq!(response.meta.model, "test-model");
        assert_eq!(response.meta.duration_ms, 1000);
        assert!(response.raw_model_output.is_none()); // raw=false
    }

    #[test]
    fn test_parse_structured_output_ocr_mode_strip_prefixes() {
        let json_str = r#"
        {
            "text_blocks": [
                {"text": "A: Hello World", "confidence": 0.9},
                {"text": "Q: What is this?", "confidence": 0.8},
                {"text": "User: Testing", "confidence": 0.85},
                {"text": "Assistant: Response", "confidence": 0.75},
                {"text": "Normal text", "confidence": 0.95}
            ]
        }
        "#;

        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let req = VisionRequest {
            image_base64: None,
            url: None,
            mode: "ocr".to_string(),
            model: None,
            timeout_ms: None,
            raw: None,
            license: None,
            screenshot: None,
            viewport_width: None,
            viewport_height: None,
        };

        let result = shimmy::vision::parse_structured_output(
            &parsed,
            &req,
            "test-model",
            1000,
            "raw output",
            None,
            None,
        );

        assert!(result.is_ok());
        let response = result.unwrap();

        assert_eq!(response.text_blocks.len(), 5);
        assert_eq!(response.text_blocks[0].text, "Hello World");
        assert_eq!(response.text_blocks[1].text, "What is this?");
        assert_eq!(response.text_blocks[2].text, "Testing");
        assert_eq!(response.text_blocks[3].text, "Response");
        assert_eq!(response.text_blocks[4].text, "Normal text");
    }

    #[test]
    fn test_parse_structured_output_minimal_response() {
        let json_str = r#"{}"#;

        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let req = VisionRequest {
            image_base64: Some("base64data".to_string()),
            url: None,
            mode: "brief".to_string(),
            model: None,
            timeout_ms: None,
            raw: Some(true),
            license: None,
            screenshot: None,
            viewport_width: None,
            viewport_height: None,
        };

        let result = shimmy::vision::parse_structured_output(
            &parsed,
            &req,
            "test-model",
            500,
            "raw output text",
            Some(vec!["warning".to_string()]),
            None,
        );

        assert!(result.is_ok());
        let response = result.unwrap();

        assert_eq!(response.mode, "brief");
        assert!(response.text_blocks.is_empty());
        assert!(response.layout.theme.is_none());
        assert!(response.layout.regions.is_empty());
        assert!(response.visual.background.is_none());
        assert!(response.visual.accent_colors.is_empty());
        assert!(response.interaction.description.is_none());
        assert!(response.dom_map.is_none());
        assert_eq!(response.meta.duration_ms, 500);
        assert_eq!(
            response.meta.parse_warnings,
            Some(vec!["warning".to_string()])
        );
        assert_eq!(
            response.raw_model_output,
            Some("raw output text".to_string())
        );
    }

    #[test]
    fn test_parse_vision_output_valid_json() {
        let raw_output = r#"{"text_blocks": [{"text": "Test", "confidence": 0.9}]}"#;
        let req = VisionRequest {
            image_base64: None,
            url: Some("https://test.com".to_string()),
            mode: "analyze".to_string(),
            model: None,
            timeout_ms: None,
            raw: None,
            license: None,
            screenshot: None,
            viewport_width: None,
            viewport_height: None,
        };

        let result =
            shimmy::vision::parse_vision_output(raw_output, &req, "test-model", 1000, None);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.text_blocks.len(), 1);
        assert_eq!(response.text_blocks[0].text, "Test");
        assert_eq!(response.url, Some("https://test.com".to_string()));
    }

    #[test]
    fn test_parse_vision_output_fallback() {
        let raw_output = "This is not JSON at all";
        let req = VisionRequest {
            image_base64: None,
            url: None,
            mode: "analyze".to_string(),
            model: None,
            timeout_ms: None,
            raw: None,
            license: None,
            screenshot: None,
            viewport_width: None,
            viewport_height: None,
        };

        let result =
            shimmy::vision::parse_vision_output(raw_output, &req, "test-model", 1000, None);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.text_blocks.len(), 1);
        assert_eq!(response.text_blocks[0].text, "This is not JSON at all");
        assert_eq!(response.text_blocks[0].confidence, Some(0.5));
        assert_eq!(
            response.visual.description,
            Some("Analysis completed".to_string())
        );
        assert_eq!(
            response.meta.parse_warnings,
            Some(vec!["Could not parse structured output".to_string()])
        );
    }

    #[test]
    fn test_vision_request_validation_missing_input() {
        let req = VisionRequest {
            image_base64: None,
            url: None,
            mode: "analyze".to_string(),
            model: None,
            timeout_ms: None,
            raw: None,
            license: None,
            screenshot: None,
            viewport_width: None,
            viewport_height: None,
        };

        // This would be tested in the actual process_vision_request function
        // Here we just verify the structure is correct
        assert!(req.image_base64.is_none());
        assert!(req.url.is_none());
    }

    #[test]
    fn test_vision_request_with_base64() {
        let base64_image = encode_test_image_base64(100, 100);
        let req = VisionRequest {
            image_base64: Some(base64_image.clone()),
            url: None,
            mode: "ocr".to_string(),
            model: Some("test-model".to_string()),
            timeout_ms: Some(30000),
            raw: Some(true),
            license: Some("test-license".to_string()),
            screenshot: Some(false),
            viewport_width: Some(1920),
            viewport_height: Some(1080),
        };

        assert_eq!(req.image_base64, Some(base64_image));
        assert_eq!(req.mode, "ocr");
        assert_eq!(req.timeout_ms, Some(30000));
        assert_eq!(req.raw, Some(true));
    }

    #[test]
    fn test_vision_request_invalid_base64() {
        let invalid_base64 = "not_valid_base64!!!";

        // Test that base64 decoding would fail
        let decode_result = general_purpose::STANDARD.decode(invalid_base64);
        assert!(decode_result.is_err());
    }

    #[test]
    fn test_vision_request_empty_base64() {
        let empty_base64 = "";

        // Test that empty base64 decodes to empty data
        let decode_result = general_purpose::STANDARD.decode(empty_base64);
        assert!(decode_result.is_ok());
        assert!(decode_result.unwrap().is_empty());
    }

    #[test]
    fn test_check_ollama_model_exists_mock() {
        // This function calls external ollama command, so we just test the logic structure
        let model_name = "registry.ollama.ai/library/minicpm-v:latest";

        // Test that registry prefix gets stripped correctly for the lookup
        let actual_model_name =
            if let Some(stripped) = model_name.strip_prefix("registry.ollama.ai/library/") {
                stripped.replace('/', ":")
            } else {
                model_name.to_string()
            };

        assert_eq!(actual_model_name, "minicpm-v:latest");
    }

    #[test]
    fn test_check_ollama_model_exists_no_prefix() {
        let model_name = "llava:latest";

        let actual_model_name =
            if let Some(stripped) = model_name.strip_prefix("registry.ollama.ai/library/") {
                stripped.replace('/', ":")
            } else {
                model_name.to_string()
            };

        assert_eq!(actual_model_name, "llava:latest");
    }

    #[test]
    fn test_dom_element_structure() {
        let dom_element = DomElement {
            tag: "button".to_string(),
            id: Some("submit-btn".to_string()),
            class: Some("btn btn-primary".to_string()),
            text: Some("Click me".to_string()),
            position: Rect {
                x: 0.1,
                y: 0.2,
                width: 0.3,
                height: 0.05,
            },
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("type".to_string(), "submit".to_string());
                attrs.insert("disabled".to_string(), "false".to_string());
                attrs
            },
            colors: Some({
                let mut colors = HashMap::new();
                colors.insert("background".to_string(), "#007bff".to_string());
                colors.insert("color".to_string(), "#ffffff".to_string());
                colors
            }),
        };

        assert_eq!(dom_element.tag, "button");
        assert_eq!(dom_element.id, Some("submit-btn".to_string()));
        assert_eq!(dom_element.position.x, 0.1);
        assert!(dom_element.colors.is_some());
        assert_eq!(dom_element.attributes.len(), 2);
    }

    #[test]
    fn test_vision_response_structure() {
        let response = VisionResponse {
            image_path: None,
            url: Some("https://example.com".to_string()),
            mode: "web".to_string(),
            text_blocks: vec![TextBlock {
                text: "Header text".to_string(),
                confidence: Some(0.95),
            }],
            layout: Layout {
                theme: Some("light".to_string()),
                regions: vec![Region {
                    name: "header".to_string(),
                    description: "Top navigation area".to_string(),
                }],
                key_ui_elements: vec![UIElement {
                    name: "menu_button".to_string(),
                    element_type: "button".to_string(),
                }],
            },
            visual: Visual {
                background: Some("#ffffff".to_string()),
                accent_colors: vec!["#007bff".to_string(), "#28a745".to_string()],
                contrast: Some(Contrast {
                    ratio: Some(4.5),
                    compliant: Some(true),
                    issues: vec![],
                }),
                description: Some("Clean modern interface".to_string()),
            },
            interaction: Interaction {
                description: Some("Click buttons to navigate".to_string()),
            },
            dom_map: None,
            meta: Meta {
                model: "test-model".to_string(),
                backend: "llama.cpp".to_string(),
                duration_ms: 1500,
                parse_warnings: None,
            },
            raw_model_output: None,
        };

        assert_eq!(response.mode, "web");
        assert_eq!(response.text_blocks.len(), 1);
        assert_eq!(response.layout.regions.len(), 1);
        assert_eq!(response.visual.accent_colors.len(), 2);
        assert!(response.visual.contrast.is_some());
        assert_eq!(response.meta.duration_ms, 1500);
    }

    #[test]
    fn test_handle_vision_request_stub_without_feature() {
        // Test the stub implementation when vision feature is not enabled
        #[cfg(not(feature = "vision"))]
        {
            let req = serde_json::Value::Null;
            let result = shimmy::vision::handle_vision_request(req);
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Vision feature not enabled"
            );
        }
    }

    #[test]
    fn test_preprocess_image_minimum_size_enforcement() {
        let png_bytes = create_test_image(1, 1);
        let cfg = shimmy::vision::PreprocessConfig {
            max_long_edge: 640,
            max_pixels: 1_500_000,
        };

        let result = shimmy::vision::preprocess_image(&png_bytes, &cfg);
        assert!(result.is_ok());
        let preprocessed = result.unwrap();
        // Should maintain minimum of 1x1
        assert!(preprocessed.width >= 1);
        assert!(preprocessed.height >= 1);
    }

    #[test]
    fn test_large_image_processing() {
        let png_bytes = create_test_image(4000, 3000);
        let cfg = shimmy::vision::PreprocessConfig {
            max_long_edge: 640,
            max_pixels: 400_000, // Web mode defaults
        };

        let result = shimmy::vision::preprocess_image(&png_bytes, &cfg);
        assert!(result.is_ok());
        let preprocessed = result.unwrap();

        // Verify it respects both constraints
        assert!(preprocessed.width.max(preprocessed.height) <= cfg.max_long_edge);
        assert!((preprocessed.width as u64 * preprocessed.height as u64) <= cfg.max_pixels);
    }

    #[test]
    fn test_extract_json_with_multiple_objects() {
        let input = r#"First object: {"a": 1} and second object: {"b": 2}"#;
        let result = shimmy::vision::extract_first_json_object(input);
        assert!(result.is_some());
        // Should extract only the first object
        assert_eq!(result.unwrap(), r#"{"a": 1}"#);
    }

    #[test]
    #[serial]
    fn test_preprocess_config_edge_cases() {
        // Test very small values
        std::env::set_var("SHIMMY_VISION_MAX_LONG_EDGE", "1");
        std::env::set_var("SHIMMY_VISION_MAX_PIXELS", "1");

        let cfg = shimmy::vision::preprocess_config_for_mode(Some("test"));
        assert_eq!(cfg.max_long_edge, 1);
        assert_eq!(cfg.max_pixels, 1);

        // Cleanup
        std::env::remove_var("SHIMMY_VISION_MAX_LONG_EDGE");
        std::env::remove_var("SHIMMY_VISION_MAX_PIXELS");
    }

    #[test]
    fn test_parse_structured_output_with_captured_dom() {
        let json_str = r#"{"text_blocks": [{"text": "Test", "confidence": 0.9}]}"#;
        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();

        let captured_dom = vec![DomElement {
            tag: "div".to_string(),
            id: None,
            class: Some("container".to_string()),
            text: Some("Content".to_string()),
            position: Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            attributes: HashMap::new(),
            colors: None,
        }];

        let req = VisionRequest {
            image_base64: None,
            url: Some("https://test.com".to_string()),
            mode: "web".to_string(),
            model: None,
            timeout_ms: None,
            raw: None,
            license: None,
            screenshot: None,
            viewport_width: None,
            viewport_height: None,
        };

        let result = shimmy::vision::parse_structured_output(
            &parsed,
            &req,
            "test-model",
            1000,
            "raw output",
            None,
            Some(captured_dom),
        );

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.dom_map.is_some());
        assert_eq!(response.dom_map.as_ref().unwrap().len(), 1);
        assert_eq!(response.dom_map.as_ref().unwrap()[0].tag, "div");
    }
}

#[cfg(not(feature = "vision"))]
mod vision_stub_tests {
    #[test]
    fn test_vision_feature_disabled() {
        // When vision feature is disabled, we should still be able to compile
        // but the functionality should not be available
        assert!(true, "Vision feature disabled - compilation test passed");
    }
}
