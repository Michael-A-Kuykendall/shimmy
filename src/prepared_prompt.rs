use crate::prompt_render::RenderSource;
use sha2::{Digest, Sha256};
use std::sync::{atomic::AtomicBool, Arc};
#[derive(Debug, thiserror::Error)]
#[cfg_attr(not(feature = "airframe"), allow(dead_code))]
pub enum PromptError {
    #[error("GGUF chat template cannot be rendered: {0}")]
    Template(String),
    #[error("model does not declare a chat template")]
    MissingTemplate,
    #[error("prepared prompt belongs to a different model/plan")]
    Identity,
    #[error("prepared prompt has no validated token IDs")]
    MissingIds,
    #[error("stop policy changed after preparing prompt")]
    StopPolicy,
    #[error("request cancelled")]
    Cancelled,
}
impl PromptError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Template(_) => "unsupported_chat_template",
            Self::MissingTemplate => "missing_chat_template",
            Self::Identity => "prepared_prompt_model_mismatch",
            Self::MissingIds => "unprepared_prompt",
            Self::StopPolicy => "prepared_stop_policy_mismatch",
            Self::Cancelled => "request_cancelled",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptIdentity {
    pub model_sha256: String,
    pub plan_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenizationPolicy {
    ModelOwned {
        add_special_tokens: bool,
        parse_special: bool,
    },
    BackendOwned,
}
#[derive(Clone, Debug)]
pub struct PreparedPrompt {
    text: String,
    source: RenderSource,
    ids: Option<Vec<u32>>,
    identity: Option<PromptIdentity>,
    template_sha256: Option<String>,
    stops: Vec<String>,
    policy: TokenizationPolicy,
}
impl PreparedPrompt {
    #[cfg_attr(not(feature = "airframe"), allow(dead_code))]
    pub(crate) fn model_owned(
        text: &str,
        source: RenderSource,
        template: Option<&str>,
        stops: &[String],
        ids: Vec<u32>,
        identity: PromptIdentity,
    ) -> Self {
        Self {
            text: text.into(),
            source,
            ids: Some(ids),
            identity: Some(identity),
            template_sha256: template.map(|s| format!("{:x}", Sha256::digest(s.as_bytes()))),
            stops: stops.to_vec(),
            policy: TokenizationPolicy::ModelOwned {
                add_special_tokens: true,
                parse_special: true,
            },
        }
    }
    pub(crate) fn backend_owned(
        text: &str,
        source: RenderSource,
        template: Option<&str>,
        stops: &[String],
    ) -> Self {
        Self {
            text: text.into(),
            source,
            ids: None,
            identity: None,
            template_sha256: template.map(|s| format!("{:x}", Sha256::digest(s.as_bytes()))),
            stops: stops.to_vec(),
            policy: TokenizationPolicy::BackendOwned,
        }
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn source(&self) -> &RenderSource {
        &self.source
    }
    pub fn token_ids(&self) -> Option<&[u32]> {
        self.ids.as_deref()
    }
    pub fn identity(&self) -> Option<&PromptIdentity> {
        self.identity.as_ref()
    }
    pub fn stops(&self) -> &[String] {
        &self.stops
    }
    pub fn template_sha256(&self) -> Option<&str> {
        self.template_sha256.as_deref()
    }
    pub fn tokenization_policy(&self) -> &TokenizationPolicy {
        &self.policy
    }
}
pub type Cancellation = Arc<AtomicBool>;
