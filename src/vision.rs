//! Shimmy Vision Module
//!
//! Feature-gated vision capabilities for image and web analysis.
//! Mirrors Seer functionality with structured JSON output.

#[cfg(feature = "vision")]
use base64::{engine::general_purpose, Engine as _};
#[cfg(feature = "vision")]
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
#[cfg(feature = "vision")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "vision")]
use sha2::{Digest, Sha256};
#[cfg(feature = "vision")]
use std::time::Instant;
#[cfg(feature = "vision")]
use tracing::info;

/// Vision response schema
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionResponse {
    pub image_path: Option<String>,
    pub url: Option<String>,
    pub mode: String,
    pub text_blocks: Vec<TextBlock>,
    pub layout: Layout,
    pub visual: Visual,
    pub interaction: Interaction,
    pub dom_map: Option<Vec<DomElement>>,
    pub meta: Meta,
    pub raw_model_output: Option<String>,
}

/// Text block from OCR
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    pub text: String,
    pub confidence: Option<f32>,
}

/// Layout analysis
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub theme: Option<String>,
    pub regions: Vec<Region>,
    pub key_ui_elements: Vec<UIElement>,
}

#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub name: String,
    pub description: String,
}

#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    pub name: String,
    pub element_type: String,
}

/// Visual analysis
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visual {
    pub background: Option<String>,
    pub accent_colors: Vec<String>,
    pub contrast: Option<Contrast>,
    pub description: Option<String>,
}

#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contrast {
    pub ratio: Option<f32>,
    pub compliant: Option<bool>,
    pub issues: Vec<String>,
}

/// Interaction hints
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub description: Option<String>,
}

/// DOM element for web mode
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomElement {
    pub tag: String,
    pub id: Option<String>,
    pub class: Option<String>,
    pub text: Option<String>,
    pub position: Rect,
    pub attributes: std::collections::HashMap<String, String>,
    pub colors: Option<std::collections::HashMap<String, String>>,
}

/// Rectangle for positioning
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Metadata
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub model: String,
    pub backend: String,
    pub duration_ms: u64,
    pub parse_warnings: Option<Vec<String>>,
}

/// Vision request for HTTP API
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionRequest {
    pub image_base64: Option<String>,
    pub url: Option<String>,
    pub mode: String,
    pub model: Option<String>,
    #[allow(dead_code)]
    pub timeout_ms: Option<u64>,
    #[allow(dead_code)]
    pub raw: Option<bool>,
    pub license: Option<String>,
    /// Capture screenshot of URL before analysis
    pub screenshot: Option<bool>,
    /// Viewport dimensions for screenshot
    pub viewport_width: Option<u32>,
    pub viewport_height: Option<u32>,
}

/// Image preprocessing configuration
#[cfg(feature = "vision")]
pub struct PreprocessConfig {
    pub max_long_edge: u32,
    pub max_pixels: u64,
}

/// Get preprocessing config, optionally adjusted for web/screenshot mode
#[cfg(feature = "vision")]
pub fn preprocess_config_for_mode(mode: Option<&str>) -> PreprocessConfig {
    fn env_u32(key: &str) -> Option<u32> {
        std::env::var(key).ok().and_then(|v| v.parse::<u32>().ok())
    }

    fn env_u64(key: &str) -> Option<u64> {
        std::env::var(key).ok().and_then(|v| v.parse::<u64>().ok())
    }

    // For web/screenshot mode, use smaller defaults to reduce image tiles
    // and avoid memory slot exhaustion in the vision model.
    // Web pages often have lots of fine text, so we prioritize fitting
    // in memory over maximum resolution.
    let is_web_mode = mode.map(|m| m == "web").unwrap_or(false);

    let default_long_edge = if is_web_mode { 512 } else { 640 };
    let default_pixels = if is_web_mode { 400_000 } else { 1_500_000 };

    let mut cfg = PreprocessConfig {
        max_long_edge: default_long_edge,
        max_pixels: default_pixels,
    };

    // Environment overrides take precedence
    if let Some(v) = env_u32("SHIMMY_VISION_MAX_LONG_EDGE") {
        cfg.max_long_edge = v;
    }
    if let Some(v) = env_u64("SHIMMY_VISION_MAX_PIXELS") {
        cfg.max_pixels = v;
    }
    cfg
}

/// Preprocessed image payload passed to mtmd/vision backend
#[cfg(feature = "vision")]
pub struct PreprocessedImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Stub implementation - returns feature disabled error
#[cfg(not(feature = "vision"))]
pub fn handle_vision_request(
    _req: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Err("Vision feature not enabled".into())
}

/// Real implementation placeholder
#[cfg(feature = "vision")]
#[allow(dead_code)]
pub fn handle_vision_request(
    _req: VisionRequest,
) -> Result<VisionResponse, Box<dyn std::error::Error>> {
    // TODO: Implement actual vision processing
    Err("Vision processing not yet implemented".into())
}

/// Process vision request with actual model inference
#[cfg(feature = "vision")]
pub async fn process_vision_request(
    req: VisionRequest,
    model_name: &str,
    license_manager: &crate::vision_license::VisionLicenseManager,
    state: &crate::AppState,
) -> Result<VisionResponse, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let trace = std::env::var("SHIMMY_VISION_TRACE").is_ok();

    // Check license first
    license_manager
        .check_vision_access(req.license.as_deref())
        .await?;

    // Record usage
    license_manager.record_usage().await?;

    // Load image data
    let (raw_image_data, captured_dom) = if let Some(base64) = &req.image_base64 {
        // Decode base64 image
        let data = general_purpose::STANDARD
            .decode(base64)
            .map_err(|e| format!("Failed to decode base64 image: {}", e))?;
        (data, None)
    } else if let Some(url) = &req.url {
        // Enable screenshot for web mode or when explicitly requested
        let should_screenshot = req.screenshot.unwrap_or(false) || req.mode == "web";
        if should_screenshot {
            // Try to capture screenshot and extract DOM
            let viewport_width = req.viewport_width.unwrap_or(1280);
            let viewport_height = req.viewport_height.unwrap_or(720);
            match capture_screenshot_and_dom(url, viewport_width, viewport_height).await {
                Ok((screenshot_data, dom_elements)) => (screenshot_data, Some(dom_elements)),
                Err(e) => {
                    tracing::warn!(
                        "Screenshot capture failed: {}. Falling back to URL fetch.",
                        e
                    );
                    // Fall back to fetching URL as image
                    let data = fetch_image_from_url(url).await?;
                    (data, None)
                }
            }
        } else {
            // Fetch image from URL
            let data = fetch_image_from_url(url).await?;
            (data, None)
        }
    } else {
        return Err("Either image_base64 or url must be provided".into());
    };

    if trace {
        info!(
            target: "vision",
            stage = "input",
            bytes = raw_image_data.len(),
            has_base64 = req.image_base64.is_some(),
            has_url = req.url.is_some(),
            mode = %req.mode,
            "vision input loaded"
        );
    }

    // Preprocess image to a safe size/format for the vision backend
    // Web mode uses smaller defaults to reduce tile count for MiniCPM-V
    let preprocess_cfg = preprocess_config_for_mode(Some(req.mode.as_str()));
    tracing::debug!(
        "Preprocess config for mode '{}': max_long_edge={}, max_pixels={}",
        req.mode,
        preprocess_cfg.max_long_edge,
        preprocess_cfg.max_pixels
    );
    tracing::error!("About to preprocess image: {} bytes", raw_image_data.len());
    let preprocessed = preprocess_image(&raw_image_data, &preprocess_cfg)
        .map_err(|e| format!("Failed to preprocess image: {}", e))?;

    if trace {
        info!(
            target: "vision",
            stage = "preprocess",
            width = preprocessed.width,
            height = preprocessed.height,
            encoded_bytes = preprocessed.bytes.len(),
            "vision image preprocessed"
        );
    }

    // Determine model to use (use provided model_name)
    let vision_model = model_name.to_string();
    let vision_model_id = normalize_vision_model_id(&vision_model);

    // Shimmy-native vision bootstrap: no Ollama dependency.
    let (model_spec, resolved_model_name) = if is_builtin_minicpm_v(&vision_model_id) {
        let auto_download = std::env::var("SHIMMY_VISION_AUTO_DOWNLOAD")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        let (model_path, _projector_path) = ensure_minicpm_v_files(auto_download).await?;

        (
            crate::engine::ModelSpec {
                name: "minicpm-v".to_string(),
                base_path: model_path,
                lora_path: None,
                template: Some("chatml".to_string()),
                ctx_len: 32768,
                n_threads: None,
            },
            "minicpm-v".to_string(),
        )
    } else {
        let spec = state
            .registry
            .to_spec(&vision_model_id)
            .ok_or_else(|| {
                format!(
                    "Vision model '{}' not found.\n\nTo use the built-in MiniCPM-V download, set SHIMMY_VISION_MODEL=minicpm-v.",
                    vision_model_id
                )
            })?;
        (spec, vision_model_id.clone())
    };

    let loaded_model = state
        .engine
        .load(&model_spec)
        .await
        .map_err(|e| format!("Failed to load vision model: {}", e))?;

    if trace {
        info!(
            target: "vision",
            stage = "model_load",
            model = %resolved_model_name,
            "vision model loaded"
        );
    }

    // Prepare vision prompt based on mode
    let prompt = prepare_vision_prompt(
        &req.mode,
        preprocessed.width,
        preprocessed.height,
        &vision_model,
    );

    if trace {
        info!(
            target: "vision",
            stage = "prompt",
            chars = prompt.len(),
            "vision prompt prepared"
        );
    }

    // Run inference
    let gen_options = crate::engine::GenOptions {
        max_tokens: 1024,
        temperature: 0.1,
        top_p: 0.9,
        top_k: 40,
        repeat_penalty: 1.0,
        seed: None,
        stream: false,
        stop_tokens: vec!["</s>".to_string(), "<|im_end|>".to_string()],
    };

    // Run inference with timeout to avoid hanging
    let generate_future =
        loaded_model.generate_vision(&preprocessed.bytes, &prompt, gen_options, None);
    let timeout_ms = req.timeout_ms.unwrap_or(60_000);
    if trace {
        info!(
            target: "vision",
            stage = "inference",
            timeout_ms = timeout_ms,
            "vision inference starting"
        );
    }
    let raw_output = match tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        generate_future,
    )
    .await
    {
        Ok(result) => result.map_err(|e| format!("Vision inference failed: {}", e))?,
        Err(_) => return Err(format!("Vision inference timed out after {} ms", timeout_ms).into()),
    };

    if trace {
        info!(
            target: "vision",
            stage = "raw_output",
            chars = raw_output.len(),
            "vision inference completed"
        );
    }

    // Parse model output into structured response
    let response = parse_vision_output(
        &raw_output,
        &req,
        resolved_model_name.as_str(),
        start_time.elapsed().as_millis() as u64,
        captured_dom,
    )?;

    if trace {
        info!(
            target: "vision",
            stage = "parse",
            duration_ms = response.meta.duration_ms,
            warnings = response.meta.parse_warnings.as_ref().map(|w| w.len()).unwrap_or(0),
            "vision output parsed"
        );
    }

    Ok(response)
}

/// Fetch image data from URL
#[cfg(feature = "vision")]
async fn fetch_image_from_url(url: &str) -> Result<Vec<u8>, anyhow::Error> {
    let parsed = validate_remote_url(url).await?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()?;

    let mut response = client.get(parsed).send().await?.error_for_status()?;

    let max_bytes = std::env::var("SHIMMY_VISION_MAX_FETCH_BYTES")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(25 * 1024 * 1024);

    if let Some(len) = response.content_length() {
        if len > max_bytes {
            return Err(anyhow::anyhow!(
                "URL image is too large ({} bytes; max {} bytes)",
                len,
                max_bytes
            ));
        }
    }

    let mut out = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if (out.len() as u64) + (chunk.len() as u64) > max_bytes {
            return Err(anyhow::anyhow!(
                "URL image is too large (exceeded {} bytes)",
                max_bytes
            ));
        }
        out.extend_from_slice(&chunk);
    }

    Ok(out)
}

/// Capture screenshot and extract DOM from URL
#[cfg(feature = "vision")]
async fn capture_screenshot_and_dom(
    url: &str,
    viewport_width: u32,
    viewport_height: u32,
) -> Result<(Vec<u8>, Vec<DomElement>), anyhow::Error> {
    use chromiumoxide::browser::{Browser, BrowserConfig};
    use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
    use futures_util::StreamExt;

    let parsed = validate_remote_url(url).await?;

    // Configure browser for headless operation
    let config = BrowserConfig::builder()
        .no_sandbox()
        .disable_default_args()
        .arg("--headless=new")
        .arg("--disable-gpu")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-software-rasterizer")
        .arg("--disable-background-timer-throttling")
        .arg("--disable-renderer-backgrounding")
        .arg("--disable-features=TranslateUI")
        .arg("--hide-scrollbars")
        .arg("--mute-audio")
        .window_size(viewport_width, viewport_height)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build browser config: {}", e))?;

    // Launch browser - this returns a future
    let (mut browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to launch browser: {}", e))?;

    // Spawn handler to process browser events in background
    let handler_task = tokio::spawn(async move { while (handler.next().await).is_some() {} });

    // Create new page
    let page = browser
        .new_page(parsed.as_str())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create page: {}", e))?;

    // Wait for page to load (networkidle is more reliable than DOMContentLoaded)
    tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        page.wait_for_navigation(),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Timed out waiting for page navigation"))?
    .map_err(|e| anyhow::anyhow!("Failed to wait for navigation: {}", e))?;

    // Small async delay for any remaining dynamic content
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Capture screenshot as PNG
    let screenshot_data = page
        .screenshot(
            chromiumoxide::page::ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .full_page(true)
                .build(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to capture screenshot: {}", e))?;

    tracing::info!("Screenshot captured: {} bytes", screenshot_data.len());

    // Extract DOM elements
    let dom_elements = extract_dom_elements(&page).await?;

    // Clean up
    drop(page);
    browser.close().await.ok();
    handler_task.abort();

    Ok((screenshot_data, dom_elements))
}

#[cfg(feature = "vision")]
async fn validate_remote_url(input: &str) -> Result<reqwest::Url, anyhow::Error> {
    let url = reqwest::Url::parse(input)
        .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", input, e))?;

    match url.scheme() {
        "http" | "https" => {}
        other => return Err(anyhow::anyhow!("Unsupported URL scheme: {}", other)),
    }

    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("URL is missing a host"))?;

    if host.eq_ignore_ascii_case("localhost") {
        return Err(anyhow::anyhow!("Refusing to fetch localhost URL"));
    }

    // If the host is an IP literal, validate it directly.
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if is_private_or_local_ip(ip) {
            return Err(anyhow::anyhow!("Refusing to fetch private/local IP URL"));
        }
        return Ok(url);
    }

    let port = url.port_or_known_default().unwrap_or(80);
    let addrs = tokio::net::lookup_host((host, port)).await?;
    for addr in addrs {
        if is_private_or_local_ip(addr.ip()) {
            return Err(anyhow::anyhow!(
                "Refusing to fetch URL that resolves to private/local IP"
            ));
        }
    }

    Ok(url)
}

#[cfg(feature = "vision")]
fn is_private_or_local_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_unspecified()
                || v4.is_multicast()
        }
        std::net::IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() {
                return true;
            }

            // Unique local addresses: fc00::/7
            let seg0 = v6.segments()[0];
            if (seg0 & 0xfe00) == 0xfc00 {
                return true;
            }

            // Link-local unicast: fe80::/10
            if (seg0 & 0xffc0) == 0xfe80 {
                return true;
            }

            false
        }
    }
}

/// Extract interactive DOM elements from the page
#[cfg(feature = "vision")]
async fn extract_dom_elements(
    page: &chromiumoxide::Page,
) -> Result<Vec<DomElement>, anyhow::Error> {
    // Get all interactive elements via JavaScript
    let script = r#"
        (function getInteractiveElements() {
            const selectors = [
                'button', 'input', 'select', 'textarea', 'a[href]',
                '[role="button"]', '[onclick]', '[role="link"]',
                '[role="textbox"]', '[role="combobox"]', '[role="listbox"]',
                'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'p', 'span', 'div'
            ];

            const elements = [];
            selectors.forEach(selector => {
                document.querySelectorAll(selector).forEach(el => {
                    const rect = el.getBoundingClientRect();
                    if (rect.width > 0 && rect.height > 0) {
                        const computedStyle = window.getComputedStyle(el);
                        elements.push({
                            tag: el.tagName.toLowerCase(),
                            id: el.id || null,
                            class: el.className || null,
                            text: el.textContent?.trim().substring(0, 100) || null,
                            position: {
                                x: rect.left / window.innerWidth,
                                y: rect.top / window.innerHeight,
                                width: rect.width / window.innerWidth,
                                height: rect.height / window.innerHeight
                            },
                            attributes: {
                                href: el.href || null,
                                type: el.type || null,
                                placeholder: el.placeholder || null,
                                role: el.getAttribute('role') || null
                            },
                            colors: {
                                background_color: computedStyle.backgroundColor,
                                color: computedStyle.color,
                                border_color: computedStyle.borderColor,
                                fill: computedStyle.fill,
                                stroke: computedStyle.stroke
                            }
                        });
                    }
                });
            });
            return elements;
        })()
    "#;

    let result: serde_json::Value = page
        .evaluate(script)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to evaluate DOM extraction script: {}", e))?
        .into_value()
        .map_err(|e| anyhow::anyhow!("Failed to get value from evaluation: {}", e))?;

    let elements: Vec<serde_json::Value> = serde_json::from_value(result)?;

    let dom_elements = elements
        .into_iter()
        .filter_map(|el| {
            Some(DomElement {
                tag: el.get("tag")?.as_str()?.to_string(),
                id: el.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                class: el
                    .get("class")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                text: el
                    .get("text")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                position: el.get("position").and_then(|p| {
                    Some(Rect {
                        x: p.get("x")?.as_f64()? as f32,
                        y: p.get("y")?.as_f64()? as f32,
                        width: p.get("width")?.as_f64()? as f32,
                        height: p.get("height")?.as_f64()? as f32,
                    })
                })?,
                attributes: el
                    .get("attributes")
                    .and_then(|a| a.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default(),
                colors: el.get("colors").and_then(|c| c.as_object()).map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                }),
            })
        })
        .collect();

    Ok(dom_elements)
}

/// Decode, downscale, and lossless-PNG-encode an image to a backend-friendly payload.
#[cfg(feature = "vision")]
pub fn preprocess_image(
    data: &[u8],
    cfg: &PreprocessConfig,
) -> Result<PreprocessedImage, Box<dyn std::error::Error>> {
    let img = image::load_from_memory(data)?;
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();

    let mut target_w = w;
    let mut target_h = h;

    // Clamp by long edge first.
    if w.max(h) > cfg.max_long_edge {
        if w >= h {
            target_w = cfg.max_long_edge;
            target_h = ((h as f32 * cfg.max_long_edge as f32 / w as f32)
                .round()
                .max(1.0)) as u32;
        } else {
            target_h = cfg.max_long_edge;
            target_w = ((w as f32 * cfg.max_long_edge as f32 / h as f32)
                .round()
                .max(1.0)) as u32;
        }
    }

    // Enforce total pixel budget.
    let mut target_pixels = target_w as u64 * target_h as u64;
    if target_pixels > cfg.max_pixels {
        let scale = (cfg.max_pixels as f64 / target_pixels as f64).sqrt();
        target_w = ((target_w as f64 * scale).floor().max(1.0)) as u32;
        target_h = ((target_h as f64 * scale).floor().max(1.0)) as u32;
        target_pixels = target_w as u64 * target_h as u64;
    }

    // Resize if needed.
    let resized_rgb: image::RgbImage = if (target_w, target_h) != (w, h) {
        image::imageops::resize(
            &rgb,
            target_w,
            target_h,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        rgb
    };

    // Final guard against unexpected oversize inputs.
    if target_pixels > cfg.max_pixels {
        return Err(format!("image too large after resize ({}x{})", target_w, target_h).into());
    }

    let mut encoded = Vec::new();
    // Lossless encoding improves OCR on low-contrast UI text compared to JPEG artifacts.
    let encoder = PngEncoder::new(&mut encoded);
    encoder.write_image(resized_rgb.as_raw(), target_w, target_h, ColorType::Rgb8)?;

    Ok(PreprocessedImage {
        bytes: encoded,
        width: target_w,
        height: target_h,
    })
}

/// Prepare vision prompt based on analysis mode
#[cfg(feature = "vision")]
pub fn prepare_vision_prompt(mode: &str, width: u32, height: u32, model_name: &str) -> String {
    let base_instruction = format!(
        "Analyze the provided image ({}x{} px). Return ONE valid JSON object only (no markdown). Use null for unknowns and [] for empty lists.",
        width, height
    );

    // Keep this short: long prompts increase token count and can trigger mtmd "memory slot" failures.
    let schema_hint = "Keys: text_blocks([{text,confidence}]), layout({theme,regions,key_ui_elements}), visual({background,accent_colors,contrast,description}), interaction({description}), dom_map(list or null).";

    let analysis_task = match mode {
        "ocr" => "OCR: extract all visible on-screen text exactly as written. Do not add labels or prefixes (no 'A:', 'Q:', 'User:', 'Assistant:', bullet markers). Do not paraphrase, summarize, or correct spelling. Preserve punctuation and casing.",
        "layout" => "Layout: identify major regions and key UI elements.",
        "brief" => "Brief: concise visual description.",
        "web" => "Web screenshot: include dom_map with approximate normalized boxes (x,y,width,height in 0..1) and describe interactions.",
        "full" => "Full: fill text_blocks, layout, visual (accent_colors as #RRGGBB when possible), and interaction.",
        _ => "Full: fill text_blocks, layout, visual (accent_colors as #RRGGBB when possible), and interaction.",
    };

    // Image is provided separately to the backend; keep prompt small to avoid Windows argv limits.
    if model_name.to_lowercase().contains("llava") {
        format!(
            "<s>[INST] {} {} {} [/INST]",
            base_instruction, schema_hint, analysis_task
        )
    } else {
        format!(
            "<|im_start|>user\n{} {} {}<|im_end|>\n<|im_start|>assistant\n",
            base_instruction, schema_hint, analysis_task
        )
    }
}

#[cfg(all(test, feature = "vision"))]
mod tests {
    use super::*;

    #[test]
    fn preprocess_image_downscales_and_pngs() {
        // Construct a large synthetic image and encode as PNG (input format doesn't matter).
        let img = image::RgbImage::from_fn(2000, 1000, |x, y| {
            let r = (x % 256) as u8;
            let g = (y % 256) as u8;
            let b = ((x.wrapping_add(y)) % 256) as u8;
            image::Rgb([r, g, b])
        });

        let dyn_img = image::DynamicImage::ImageRgb8(img);
        let mut png_bytes = Vec::new();
        dyn_img
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .expect("png encode");

        let cfg = PreprocessConfig {
            max_long_edge: 640,
            max_pixels: 1_500_000,
        };

        let out = preprocess_image(&png_bytes, &cfg).expect("preprocess");
        assert!(out.width.max(out.height) <= cfg.max_long_edge);
        assert!((out.width as u64) * (out.height as u64) <= cfg.max_pixels);
        // PNG magic bytes: 89 50 4E 47 0D 0A 1A 0A
        let sig: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(out.bytes.len() >= sig.len());
        assert_eq!(&out.bytes[..sig.len()], &sig);
    }

    #[test]
    fn prepare_vision_prompt_is_compact_and_json_only() {
        let p = prepare_vision_prompt("full", 640, 480, "minicpm-v");
        assert!(p.contains("valid JSON"));
        assert!(!p.contains("```"));
        assert!(p.contains("text_blocks"));
        assert!(p.contains("dom_map"));
    }
}

/// Parse model output into structured vision response
#[cfg(feature = "vision")]
pub fn parse_vision_output(
    raw_output: &str,
    req: &VisionRequest,
    model_name: &str,
    duration_ms: u64,
    captured_dom: Option<Vec<DomElement>>,
) -> Result<VisionResponse, Box<dyn std::error::Error>> {
    let (json_candidate, warnings) = extract_json_candidate(raw_output);

    if let Some(json_str) = json_candidate {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
            return parse_structured_output(
                &parsed,
                req,
                model_name,
                duration_ms,
                raw_output,
                if warnings.is_empty() {
                    None
                } else {
                    Some(warnings)
                },
                captured_dom.clone(),
            );
        }
    }

    // Final fallback: create basic response from raw text
    Ok(VisionResponse {
        image_path: None,
        url: req.url.clone(),
        mode: req.mode.clone(),
        text_blocks: vec![TextBlock {
            text: raw_output.trim().to_string(),
            confidence: Some(0.5),
        }],
        layout: Layout {
            theme: None,
            regions: vec![],
            key_ui_elements: vec![],
        },
        visual: Visual {
            background: None,
            accent_colors: vec![],
            contrast: None,
            description: Some("Analysis completed".to_string()),
        },
        interaction: Interaction { description: None },
        dom_map: captured_dom,
        meta: Meta {
            model: model_name.to_string(),
            backend: "llama.cpp".to_string(),
            duration_ms,
            parse_warnings: Some(vec!["Could not parse structured output".to_string()]),
        },
        raw_model_output: Some(raw_output.to_string()),
    })
}

#[cfg(feature = "vision")]
pub fn extract_json_candidate(raw_output: &str) -> (Option<String>, Vec<String>) {
    let mut warnings = Vec::new();
    let mut s = raw_output.trim().to_string();

    // Strip common markdown code fences.
    if s.starts_with("```") {
        warnings.push("Stripped markdown code fences".to_string());
        // Drop first fence line
        if let Some(nl) = s.find('\n') {
            s = s[nl + 1..].to_string();
        } else {
            s.clear();
        }
        // Drop trailing fence
        if let Some(end) = s.rfind("```") {
            s = s[..end].to_string();
        }
        s = s.trim().to_string();
    }

    // Fast path: whole string is JSON
    if s.starts_with('{') && s.ends_with('}') {
        return (Some(s), warnings);
    }

    // Try to extract a balanced {...} object from within surrounding text.
    if let Some(extracted) = extract_first_json_object(&s) {
        warnings.push("Extracted JSON object from surrounding text".to_string());
        return (Some(extracted), warnings);
    }

    (None, warnings)
}

#[cfg(feature = "vision")]
pub fn extract_first_json_object(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let mut depth = 0i32;
            let mut in_string = false;
            let mut escape = false;
            let start = i;
            let mut j = i;
            while j < bytes.len() {
                let b = bytes[j];
                if in_string {
                    if escape {
                        escape = false;
                    } else if b == b'\\' {
                        escape = true;
                    } else if b == b'"' {
                        in_string = false;
                    }
                } else if b == b'"' {
                    in_string = true;
                } else if b == b'{' {
                    depth += 1;
                } else if b == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        let end = j;
                        return Some(s[start..=end].to_string());
                    }
                }
                j += 1;
            }
        }
        i += 1;
    }
    None
}

/// Parse structured JSON output into VisionResponse
#[cfg(feature = "vision")]
pub fn parse_structured_output(
    parsed: &serde_json::Value,
    req: &VisionRequest,
    model_name: &str,
    duration_ms: u64,
    raw_output: &str,
    parse_warnings: Option<Vec<String>>,
    captured_dom: Option<Vec<DomElement>>,
) -> Result<VisionResponse, Box<dyn std::error::Error>> {
    // Extract text blocks
    let mut text_blocks = parsed
        .get("text_blocks")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(TextBlock {
                        text: item.get("text")?.as_str()?.to_string(),
                        confidence: item
                            .get("confidence")
                            .and_then(|c| c.as_f64())
                            .map(|c| c as f32),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // OCR should be literal text. Some models add conversational labels like "A:".
    // Strip common prefixes for mode=ocr to keep output clean for OCR quality evaluation.
    if req.mode == "ocr" {
        for block in &mut text_blocks {
            let trimmed = block.text.trim_start();
            let cleaned = trimmed
                .strip_prefix("A: ")
                .or_else(|| trimmed.strip_prefix("A:"))
                .or_else(|| trimmed.strip_prefix("Q: "))
                .or_else(|| trimmed.strip_prefix("Q:"))
                .or_else(|| trimmed.strip_prefix("User: "))
                .or_else(|| trimmed.strip_prefix("User:"))
                .or_else(|| trimmed.strip_prefix("Assistant: "))
                .or_else(|| trimmed.strip_prefix("Assistant:"));
            if let Some(rest) = cleaned {
                let rest = rest.trim_start();
                if !rest.is_empty() {
                    block.text = rest.to_string();
                }
            }
        }
    }

    // Extract layout information
    let layout = if let Some(layout_obj) = parsed.get("layout") {
        Layout {
            theme: layout_obj
                .get("theme")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string()),
            regions: layout_obj
                .get("regions")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            Some(Region {
                                name: item.get("name")?.as_str()?.to_string(),
                                description: item.get("description")?.as_str()?.to_string(),
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            key_ui_elements: layout_obj
                .get("key_ui_elements")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            Some(UIElement {
                                name: item.get("name")?.as_str()?.to_string(),
                                element_type: item.get("element_type")?.as_str()?.to_string(),
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        }
    } else {
        Layout {
            theme: None,
            regions: vec![],
            key_ui_elements: vec![],
        }
    };

    // Extract visual information
    let visual = if let Some(visual_obj) = parsed.get("visual") {
        Visual {
            background: visual_obj
                .get("background")
                .and_then(|b| b.as_str())
                .map(|s| s.to_string()),
            accent_colors: visual_obj
                .get("accent_colors")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            contrast: visual_obj.get("contrast").map(|c| Contrast {
                ratio: c.get("ratio").and_then(|r| r.as_f64()).map(|r| r as f32),
                compliant: c.get("compliant").and_then(|c| c.as_bool()),
                issues: c
                    .get("issues")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| item.as_str().map(|s| s.to_string()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
            }),
            description: visual_obj
                .get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
        }
    } else {
        Visual {
            background: None,
            accent_colors: vec![],
            contrast: None,
            description: None,
        }
    };

    // Extract interaction information
    let interaction = Interaction {
        description: parsed
            .get("interaction")
            .and_then(|i| i.get("description"))
            .and_then(|d| d.as_str())
            .map(|s| s.to_string()),
    };

    // Extract DOM map for web mode
    let dom_map = parsed.get("dom_map").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|item| {
                Some(DomElement {
                    tag: item.get("tag")?.as_str()?.to_string(),
                    id: item
                        .get("id")
                        .and_then(|i| i.as_str())
                        .map(|s| s.to_string()),
                    class: item
                        .get("class")
                        .and_then(|c| c.as_str())
                        .map(|s| s.to_string()),
                    text: item
                        .get("text")
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string()),
                    position: item.get("position").and_then(|p| {
                        Some(Rect {
                            x: p.get("x")?.as_f64()? as f32,
                            y: p.get("y")?.as_f64()? as f32,
                            width: p.get("width")?.as_f64()? as f32,
                            height: p.get("height")?.as_f64()? as f32,
                        })
                    })?,
                    attributes: item
                        .get("attributes")
                        .and_then(|a| a.as_object())
                        .map(|obj| {
                            obj.iter()
                                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                .collect()
                        })
                        .unwrap_or_default(),
                    colors: item.get("colors").and_then(|c| c.as_object()).map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    }),
                })
            })
            .collect::<Vec<_>>()
    });

    Ok(VisionResponse {
        image_path: None,
        url: req.url.clone(),
        mode: req.mode.clone(),
        text_blocks,
        layout,
        visual,
        interaction,
        dom_map: captured_dom.or(dom_map),
        meta: Meta {
            model: model_name.to_string(),
            backend: "llama.cpp".to_string(),
            duration_ms,
            parse_warnings,
        },
        raw_model_output: if req.raw.unwrap_or(false) {
            Some(raw_output.to_string())
        } else {
            None
        },
    })
}

#[cfg(feature = "vision")]
fn normalize_vision_model_id(input: &str) -> String {
    let s = input.trim();

    // Back-compat: older default used an Ollama registry URL-like string.
    if let Some(stripped) = s.strip_prefix("registry.ollama.ai/library/") {
        let candidate = stripped
            .trim_end_matches("/latest")
            .trim_end_matches(":latest")
            .trim_matches('/');
        return candidate.to_string();
    }

    // Also accept minicpm-v:latest and normalize to minicpm-v.
    s.trim_end_matches(":latest").to_string()
}

#[cfg(feature = "vision")]
fn is_builtin_minicpm_v(model_id: &str) -> bool {
    let lower = model_id.to_lowercase();
    lower == "minicpm-v" || lower == "minicpm" || lower.contains("minicpm")
}

#[cfg(feature = "vision")]
fn vision_model_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("SHIMMY_VISION_MODEL_DIR") {
        if !dir.trim().is_empty() {
            return std::path::PathBuf::from(dir);
        }
    }

    let base = dirs::data_local_dir()
        .or_else(dirs::cache_dir)
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("shimmy").join("vision").join("models")
}

#[cfg(feature = "vision")]
fn minicpm_bootstrap_mutex() -> &'static tokio::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

#[cfg(feature = "vision")]
async fn ensure_minicpm_v_files(
    auto_download: bool,
) -> Result<(std::path::PathBuf, std::path::PathBuf), Box<dyn std::error::Error>> {
    const MODEL_URL: &str =
        "https://huggingface.co/openbmb/MiniCPM-V-2_6-gguf/resolve/main/ggml-model-Q4_K_M.gguf";
    const MODEL_SHA256_HEX: &str =
        "3a4078d53b46f22989adbf998ce5a3fd090b6541f112d7e936eb4204a04100b1";
    const PROJ_URL: &str =
        "https://huggingface.co/openbmb/MiniCPM-V-2_6-gguf/resolve/main/mmproj-model-f16.gguf";
    const PROJ_SHA256_HEX: &str =
        "4485f68a0f1aa404c391e788ea88ea653c100d8e98fe572698f701e5809711fd";

    let dir = vision_model_dir().join("minicpm-v-2_6");
    tokio::fs::create_dir_all(&dir).await?;

    let model_path = dir.join("ggml-model-Q4_K_M.gguf");
    let proj_path = dir.join("mmproj-model-f16.gguf");

    if !auto_download && (!model_path.exists() || !proj_path.exists()) {
        return Err(format!(
            "MiniCPM-V model files are missing.\n\nExpected:\n  - {}\n  - {}\n\nSet SHIMMY_VISION_AUTO_DOWNLOAD=1 to let Shimmy download them automatically.",
            model_path.display(),
            proj_path.display()
        )
        .into());
    }

    // Prevent duplicate concurrent downloads within the same process.
    let _guard = minicpm_bootstrap_mutex().lock().await;

    ensure_download_and_verify(&model_path, MODEL_URL, MODEL_SHA256_HEX).await?;
    ensure_download_and_verify(&proj_path, PROJ_URL, PROJ_SHA256_HEX).await?;

    Ok((model_path, proj_path))
}

#[cfg(feature = "vision")]
async fn ensure_download_and_verify(
    dest: &std::path::Path,
    url: &str,
    expected_sha256_hex: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if dest.exists() {
        if verify_file_sha256(dest, expected_sha256_hex).await.is_ok() {
            return Ok(());
        }

        // Corrupt or wrong file: remove and re-download.
        let _ = tokio::fs::remove_file(dest).await;
    }

    let tmp = dest.with_extension("partial");
    if tmp.exists() {
        let _ = tokio::fs::remove_file(&tmp).await;
    }

    let timeout_secs = std::env::var("SHIMMY_VISION_DOWNLOAD_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(60 * 60);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()?;
    let mut resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(format!("Failed to download {} (HTTP {})", url, resp.status()).into());
    }

    let mut file = tokio::fs::File::create(&tmp).await?;
    let mut hasher = Sha256::new();

    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = resp.chunk().await? {
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
    }
    file.flush().await?;

    let expected_lower = expected_sha256_hex.to_lowercase();
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected_lower {
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(format!(
            "SHA256 mismatch for {}. Expected {}, got {}",
            dest.display(),
            expected_sha256_hex,
            actual
        )
        .into());
    }

    tokio::fs::rename(&tmp, dest).await?;
    Ok(())
}

#[cfg(feature = "vision")]
async fn verify_file_sha256(
    path: &std::path::Path,
    expected_sha256_hex: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 8 * 1024 * 1024];

    use tokio::io::AsyncReadExt;
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let expected_lower = expected_sha256_hex.to_lowercase();
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected_lower {
        return Err(format!(
            "SHA256 mismatch for {}. Expected {}, got {}",
            path.display(),
            expected_sha256_hex,
            actual
        )
        .into());
    }

    Ok(())
}
