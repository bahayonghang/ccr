use indexmap::IndexMap;
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

pub fn toml_map_to_json_map(map: &IndexMap<String, TomlValue>) -> IndexMap<String, JsonValue> {
    map.iter()
        .map(|(key, value)| (key.clone(), toml_value_to_json_value(value)))
        .collect()
}

pub fn json_map_to_toml_map(map: &IndexMap<String, JsonValue>) -> IndexMap<String, TomlValue> {
    map.iter()
        .filter_map(|(key, value)| {
            json_value_to_toml_value(value).map(|converted| (key.clone(), converted))
        })
        .collect()
}

fn toml_value_to_json_value(value: &TomlValue) -> JsonValue {
    match value {
        TomlValue::String(s) => JsonValue::String(s.clone()),
        TomlValue::Integer(i) => JsonValue::Number((*i).into()),
        TomlValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        TomlValue::Boolean(b) => JsonValue::Bool(*b),
        TomlValue::Datetime(dt) => JsonValue::String(dt.to_string()),
        TomlValue::Array(arr) => {
            JsonValue::Array(arr.iter().map(toml_value_to_json_value).collect())
        }
        TomlValue::Table(table) => {
            let mut obj = serde_json::Map::new();
            for (key, val) in table {
                obj.insert(key.clone(), toml_value_to_json_value(val));
            }
            JsonValue::Object(obj)
        }
    }
}

fn json_value_to_toml_value(value: &JsonValue) -> Option<TomlValue> {
    match value {
        JsonValue::Null => None,
        JsonValue::Bool(b) => Some(TomlValue::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(TomlValue::Integer(i))
            } else if let Some(u) = n.as_u64() {
                Some(TomlValue::Integer(u as i64))
            } else {
                n.as_f64().map(TomlValue::Float)
            }
        }
        JsonValue::String(s) => Some(TomlValue::String(s.clone())),
        JsonValue::Array(arr) => {
            let converted = arr
                .iter()
                .filter_map(json_value_to_toml_value)
                .collect::<Vec<_>>();
            Some(TomlValue::Array(converted))
        }
        JsonValue::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (key, val) in obj {
                if let Some(converted) = json_value_to_toml_value(val) {
                    table.insert(key.clone(), converted);
                }
            }
            Some(TomlValue::Table(table))
        }
    }
}
