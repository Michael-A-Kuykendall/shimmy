//! Vision Provider Adapter Trait
//!
//! Abstracts vision processing behind a trait so the implementation
//! can be swapped between:
//! - Private `shimmy-vision` crate (licensed users)
//! - Error stub (unlicensed/public users)

use crate::vision::{VisionRequest, VisionResponse};
#[allow(unused_imports)]
use crate::AppState;
#[allow(unused_imports)]
use std::future::Future;
#[allow(unused_imports)]
use std::pin::Pin;

/// Vision processing provider trait
pub trait VisionProvider: Send + Sync {
    /// Process a vision request
    #[allow(clippy::type_complexity)]
    fn process_vision_request<'a>(
        &'a self,
        req: VisionRequest,
        model_name: String,
        state: &'a crate::AppState,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<VisionResponse, Box<dyn std::error::Error>>>
                + Send
                + 'a,
        >,
    >;
}

/// Private vision provider (requires shimmy-vision crate)
#[cfg(feature = "vision")]
pub struct PrivateVisionProvider;

#[cfg(feature = "vision")]
impl VisionProvider for PrivateVisionProvider {
    fn process_vision_request<'a>(
        &'a self,
        req: VisionRequest,
        model_name: String,
        state: &'a crate::AppState,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<VisionResponse, Box<dyn std::error::Error>>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            // TODO: Call the private shimmy-vision crate for licensed vision processing
            // For now, call existing vision processing but skip license validation
            // since it will be handled by the private crate
            let license_manager = crate::vision_license::VisionLicenseManager::new();
            crate::vision::process_vision_request(req, &model_name, &license_manager, state).await
        })
    }
}

/// Stub vision provider (returns error for unlicensed users)
#[cfg(not(feature = "vision"))]
pub struct StubVisionProvider;

#[cfg(not(feature = "vision"))]
impl VisionProvider for StubVisionProvider {
    fn process_vision_request<'a>(
        &'a self,
        #[allow(unused_variables)] _req: VisionRequest,
        #[allow(unused_variables)] _model_name: String,
        #[allow(unused_variables)] _state: &'a crate::AppState,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<VisionResponse, Box<dyn std::error::Error>>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(
            async move { Err("Vision feature not enabled. This is a licensed feature.".into()) },
        )
    }
}
