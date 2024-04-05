use std::fmt::{self, Display, Formatter};

use envconfig::Envconfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Envconfig)]
pub struct OpenAiConfig {
    #[envconfig(from = "OPENAI_API_KEY")]
    pub api_key: String,
}

impl Display for OpenAiConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "OPENAI_API_KEY: ***")
    }
}
