pub mod api_model_config;
pub mod connection_definition;
pub mod connection_model_definition;
pub mod connection_model_schema;
pub mod connection_oauth_definition;
pub mod db_model_config;

use std::{hash::Hash, sync::Arc};

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

use crate::id::Id;

use self::{
    connection_model_definition::ConnectionModelDefinition,
    connection_oauth_definition::ConnectionOAuthDefinition, db_model_config::DbModelConfig,
};

use super::{
    configuration::environment::Environment,
    shared::{ownership::Ownership, record_metadata::RecordMetadata, settings::Settings},
};

fn key_default() -> Arc<str> {
    String::new().into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    #[serde(rename = "_id")]
    pub id: Id,
    pub platform_version: String,
    pub connection_definition_id: Id,
    pub r#type: ConnectionType,
    pub name: String,
    #[serde(default = "key_default")]
    pub key: Arc<str>,
    pub group: String,
    pub environment: Environment,
    pub platform: Arc<str>,
    pub secrets_service_id: String,
    pub event_access_id: Id,
    pub access_key: String,
    pub settings: Settings,
    pub throughput: Throughput,
    pub ownership: Ownership,
    #[serde(default)]
    pub oauth: Option<OAuth>,
    #[serde(flatten, default)]
    pub record_metadata: RecordMetadata,
}

impl Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Connection {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Default)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum OAuth {
    Enabled {
        connection_oauth_definition_id: Id,
        expires_in: Option<i32>,
        #[serde(default)]
        expires_at: Option<i64>,
    },
    #[default]
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, AsRefStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ConnectionType {
    //TODO: Remove
    Api {
        #[serde(rename = "apiModelConfigs")]
        model_configs: Vec<ConnectionModelDefinition>,
        #[serde(default)]
        oauth_configs: Vec<ConnectionOAuthDefinition>,
    },
    //TODO: Remove
    DatabaseSql {
        #[serde(rename = "dbModelConfigs")]
        model_configs: Vec<DbModelConfig>,
    },
    DatabaseNoSql,
    FileSystem,
    Stream,
    Custom,
}

#[derive(
    Debug, Clone, Copy, Serialize, PartialEq, Eq, Deserialize, Display, AsRefStr, EnumString,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Platform {
    RabbitMq,
    Xero,
    PostgreSql,
    MySql,
    MariaDb,
    MsSql,
    Stripe,
    Sage,
    Shopify,
    Snowflake,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Throughput {
    pub key: String,
    pub limit: u64,
}
