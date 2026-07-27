//! Shared wire-only DTOs for command payloads with intentionally open JSON schemas.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../src/types/generated/common/")]
pub enum OpenJsonValueDto {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<OpenJsonValueDto>),
    Object(BTreeMap<String, OpenJsonValueDto>),
}

impl TryFrom<serde_json::Value> for OpenJsonValueDto {
    type Error = String;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        match value {
            serde_json::Value::Null => Ok(Self::Null),
            serde_json::Value::Bool(value) => Ok(Self::Bool(value)),
            serde_json::Value::Number(value) => value
                .as_f64()
                .map(Self::Number)
                .ok_or_else(|| "JSON number cannot be represented as f64".to_string()),
            serde_json::Value::String(value) => Ok(Self::String(value)),
            serde_json::Value::Array(values) => values
                .into_iter()
                .map(Self::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map(Self::Array),
            serde_json::Value::Object(values) => values
                .into_iter()
                .map(|(key, value)| Self::try_from(value).map(|value| (key, value)))
                .collect::<Result<BTreeMap<_, _>, _>>()
                .map(Self::Object),
        }
    }
}

impl From<OpenJsonValueDto> for serde_json::Value {
    fn from(value: OpenJsonValueDto) -> Self {
        match value {
            OpenJsonValueDto::Null => Self::Null,
            OpenJsonValueDto::Bool(value) => Self::Bool(value),
            OpenJsonValueDto::Number(value) => serde_json::Number::from_f64(value)
                .map(Self::Number)
                .unwrap_or(Self::Null),
            OpenJsonValueDto::String(value) => Self::String(value),
            OpenJsonValueDto::Array(values) => {
                Self::Array(values.into_iter().map(Self::from).collect())
            }
            OpenJsonValueDto::Object(values) => Self::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, Self::from(value)))
                    .collect(),
            ),
        }
    }
}
