use std::fmt::{self, Display, Formatter};

use envconfig::Envconfig;
use serde::{Deserialize, Serialize};

use crate::{IntegrationOSError, InternalError};

#[derive(Serialize, Deserialize, Debug, Clone, Envconfig)]
pub struct ClaudeConfig {
    #[envconfig(from = "CLAUDE_URL", default = "https://api.anthropic.com/v1/complete")]
    pub url: String,
    #[envconfig(from = "CLAUDE_ANTHROPIC_VERSION", default = "2023-06-01")]
    pub anthropic_version: String,
    /// The Claude API key
    #[envconfig(from = "CLAUDE_API_KEY", default = "")]
    pub api_key: String,
    #[envconfig(from = "CLAUDE_MODEL", default = "claude-2.0")]
    pub model: String,
    #[envconfig(from = "CLAUDE_MAX_TOKENS_TO_SAMPLE", default = "10000")]
    pub max_tokens_to_sample: u32,
    #[envconfig(from = "CLAUDE_STOP_SEQUENCES", default = "")]
    pub stop_sequences: String,
    #[envconfig(from = "CLAUDE_TEMPERATURE", default = "0.1")]
    pub temperature: f32,
    #[envconfig(from = "CLAUDE_TOP_P", default = "0.7")]
    pub top_p: f32,
    #[envconfig(from = "CLAUDE_TOP_K", default = "5")]
    pub top_k: u32,
    #[envconfig(from = "CLAUDE_USER_ID", default = "internal-system")]
    pub user_id: String,
}

impl ClaudeConfig {
    pub fn new(temperature: f32) -> Result<Self, IntegrationOSError> {
        let mut config = Self::init_from_env().map_err(|e| {
            InternalError::configuration_error(
                &e.to_string(),
                Some("Failed to initialize ClaudeConfig from environment"),
            )
        })?;
        config.temperature = temperature;
        Ok(config)
    }
}

impl Display for ClaudeConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "CLAUDE_URL: {}", self.url)?;
        writeln!(f, "CLAUDE_ANTHROPIC_VERSION: {}", self.anthropic_version)?;
        writeln!(f, "CLAUDE_API_KEY: ***")?;
        writeln!(f, "CLAUDE_MODEL: {}", self.model)?;
        writeln!(
            f,
            "CLAUDE_MAX_TOKENS_TO_SAMPLE: {}",
            self.max_tokens_to_sample
        )?;
        writeln!(f, "CLAUDE_STOP_SEQUENCES: {}", self.stop_sequences)?;
        writeln!(f, "CLAUDE_TEMPERATURE: {}", self.temperature)?;
        writeln!(f, "CLAUDE_TOP_P: {}", self.top_p)?;
        writeln!(f, "CLAUDE_TOP_K: {}", self.top_k)?;
        writeln!(f, "CLAUDE_USER_ID: {}", self.user_id)
    }
}
