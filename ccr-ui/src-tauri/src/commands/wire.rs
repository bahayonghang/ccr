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
            OpenJsonValueDto::Number(value) => json_number_from_f64(value),
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

/// JS/Tauri 把所有 JSON 数字收成 `f64`。直接 `Number::from_f64` 会得到 Float 变体，
/// `as_u64()` / `as_i64()` 恒为 `None`。能无损还原的整数值必须写成整数 Number。
pub(crate) fn json_number_from_f64(value: f64) -> serde_json::Value {
    if let Some(number) = integer_number_from_f64(value) {
        return serde_json::Value::Number(number);
    }
    serde_json::Number::from_f64(value)
        .map(serde_json::Value::Number)
        .unwrap_or(serde_json::Value::Null)
}

fn integer_number_from_f64(value: f64) -> Option<serde_json::Number> {
    if !value.is_finite() {
        return None;
    }
    if value >= 0.0 {
        let as_u = value as u64;
        (as_u as f64 == value).then(|| as_u.into())
    } else {
        let as_i = value as i64;
        (as_i as f64 == value).then(|| as_i.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{OpenJsonValueDto, json_number_from_f64};

    #[test]
    fn whole_f64_round_trips_as_integer_json_number() {
        let value = json_number_from_f64(500_000.0);
        assert_eq!(value.as_u64(), Some(500_000));
        assert_eq!(value.as_i64(), Some(500_000));

        let dto = OpenJsonValueDto::Number(500_000.0);
        let converted = serde_json::Value::from(dto);
        assert_eq!(converted.as_u64(), Some(500_000));
    }

    #[test]
    fn fractional_and_negative_f64_keep_expected_json_shape() {
        let fraction = json_number_from_f64(1.5);
        assert!(fraction.as_u64().is_none());
        assert!(fraction.as_i64().is_none());
        assert_eq!(fraction.as_f64(), Some(1.5));

        let negative = json_number_from_f64(-8.0);
        assert_eq!(negative.as_i64(), Some(-8));
        assert!(negative.as_u64().is_none());
    }
}
