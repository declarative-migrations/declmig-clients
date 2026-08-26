#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Health {
    pub ok: bool,
    pub service: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ResourceEnvelope {
    pub id: String,
    pub revision: String,
    #[serde(default)]
    pub payload: Value,
}

impl ResourceEnvelope {
    pub const RESOURCE: &'static str = "MigrationPlan";
}

