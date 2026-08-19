use crate::utils::mask_sensitive;
use serde_json::Value;

const MAX_DEPTH: usize = 4;
const MAX_JSON_BYTES: usize = 8192;
const MAX_ARRAY_LEN: usize = 32;

const SENSITIVE_KEYS: &[&str] = &[
    "token",
    "apikey",
    "authorization",
    "cookie",
    "cookies",
    "password",
    "secret",
    "bearer",
    "accesstoken",
    "refreshtoken",
    "sessiontoken",
    "privatekey",
    "clientsecret",
    "authjson",
    "cookiesjson",
];

pub fn normalize_log_key(key: &str) -> String {
    key.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

pub fn is_sensitive_log_key(key: &str) -> bool {
    let normalized = normalize_log_key(key);
    SENSITIVE_KEYS.contains(&normalized.as_str())
}

pub fn redact_log_text(input: &str) -> String {
    let trimmed = input.trim();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed)
        && matches!(value, Value::Object(_) | Value::Array(_))
    {
        return redact_log_value(&value).to_string();
    }

    redact_free_text(input)
}

pub fn redact_log_value(value: &Value) -> Value {
    redact_value_inner(value, 0, 0, None).0
}

fn redact_value_inner(
    value: &Value,
    depth: usize,
    used_bytes: usize,
    parent_key: Option<&str>,
) -> (Value, usize) {
    if depth > MAX_DEPTH || used_bytes > MAX_JSON_BYTES {
        return (truncated_object(), used_bytes);
    }

    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            let mut bytes = used_bytes;
            for (key, child) in map {
                bytes = bytes.saturating_add(key.len());
                if bytes > MAX_JSON_BYTES {
                    return (truncated_object(), bytes);
                }
                if is_sensitive_log_key(key) {
                    let masked = match child {
                        Value::String(text) => Value::String(mask_sensitive(text)),
                        _ => Value::String(mask_sensitive(&child.to_string())),
                    };
                    bytes = bytes.saturating_add(estimate_value_bytes(&masked));
                    out.insert(key.clone(), masked);
                } else {
                    let (redacted, next_bytes) =
                        redact_value_inner(child, depth + 1, bytes, Some(key));
                    bytes = next_bytes;
                    out.insert(key.clone(), redacted);
                }
            }
            (Value::Object(out), bytes)
        }
        Value::Array(items) => {
            let parent_sensitive = parent_key.is_some_and(is_sensitive_log_key);
            let mut out = Vec::new();
            let mut bytes = used_bytes;
            for (index, item) in items.iter().enumerate() {
                if index >= MAX_ARRAY_LEN {
                    break;
                }
                if parent_sensitive {
                    let masked = match item {
                        Value::String(text) => Value::String(mask_sensitive(text)),
                        _ => Value::String(mask_sensitive(&item.to_string())),
                    };
                    bytes = bytes.saturating_add(estimate_value_bytes(&masked));
                    out.push(masked);
                } else {
                    let (redacted, next_bytes) = redact_value_inner(item, depth + 1, bytes, None);
                    bytes = next_bytes;
                    out.push(redacted);
                }
            }
            (Value::Array(out), bytes)
        }
        Value::String(text) => {
            let redacted = redact_log_text(text);
            let next = used_bytes.saturating_add(redacted.len());
            (Value::String(redacted), next)
        }
        other => {
            let next = used_bytes.saturating_add(estimate_value_bytes(other));
            (other.clone(), next)
        }
    }
}

fn truncated_object() -> Value {
    serde_json::json!({ "truncated": true })
}

fn estimate_value_bytes(value: &Value) -> usize {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

fn redact_free_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if let Some((consumed, replacement)) = match_cookie_header(&chars, index)
            .or_else(|| match_bearer(&chars, index))
            .or_else(|| match_sk_token(&chars, index))
            .or_else(|| match_jwt(&chars, index))
        {
            output.push_str(&replacement);
            index += consumed;
            continue;
        }

        output.push(chars[index]);
        index += 1;
    }

    output
}

fn match_cookie_header(chars: &[char], start: usize) -> Option<(usize, String)> {
    for header in ["cookie:", "set-cookie:"] {
        if starts_with_ignore_ascii_case(chars, start, header) {
            let mut end = start + header.chars().count();
            while end < chars.len() && chars[end] != '\n' && chars[end] != '\r' {
                end += 1;
            }
            let prefix: String = chars[start..start + header.chars().count()]
                .iter()
                .collect();
            let value: String = chars[start + header.chars().count()..end].iter().collect();
            return Some((
                end - start,
                format!("{prefix}{}", mask_sensitive(value.trim())),
            ));
        }
    }
    None
}

fn match_bearer(chars: &[char], start: usize) -> Option<(usize, String)> {
    if !starts_with_ignore_ascii_case(chars, start, "bearer") {
        return None;
    }
    let mut cursor = start + 6;
    if cursor >= chars.len() || !chars[cursor].is_ascii_whitespace() {
        return None;
    }
    while cursor < chars.len() && chars[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    let token_start = cursor;
    while cursor < chars.len() && is_token_char(chars[cursor]) {
        cursor += 1;
    }
    if cursor - token_start < 8 {
        return None;
    }
    let token: String = chars[token_start..cursor].iter().collect();
    Some((cursor - start, format!("Bearer {}", mask_sensitive(&token))))
}

fn match_sk_token(chars: &[char], start: usize) -> Option<(usize, String)> {
    if start + 3 >= chars.len() {
        return None;
    }
    if start > 0 && chars[start - 1].is_ascii_alphanumeric() {
        return None;
    }
    if chars[start] != 's' || chars[start + 1] != 'k' || chars[start + 2] != '-' {
        return None;
    }
    let mut cursor = start + 3;
    while cursor < chars.len() && is_sk_char(chars[cursor]) {
        cursor += 1;
    }
    if cursor - start < 11 {
        return None;
    }
    let token: String = chars[start..cursor].iter().collect();
    Some((cursor - start, mask_sensitive(&token)))
}

fn match_jwt(chars: &[char], start: usize) -> Option<(usize, String)> {
    if start > 0 && chars[start - 1].is_ascii_alphanumeric() {
        return None;
    }
    if !starts_with_literal(chars, start, "eyJ") {
        return None;
    }
    let mut cursor = start;
    if !consume_jwt_part(chars, &mut cursor) {
        return None;
    }
    if cursor >= chars.len() || chars[cursor] != '.' {
        return None;
    }
    cursor += 1;
    if !consume_jwt_part(chars, &mut cursor) {
        return None;
    }
    if cursor >= chars.len() || chars[cursor] != '.' {
        return None;
    }
    cursor += 1;
    if !consume_jwt_part(chars, &mut cursor) {
        return None;
    }
    if cursor - start < 20 {
        return None;
    }
    let token: String = chars[start..cursor].iter().collect();
    Some((cursor - start, mask_sensitive(&token)))
}

fn consume_jwt_part(chars: &[char], cursor: &mut usize) -> bool {
    let start = *cursor;
    while *cursor < chars.len() && is_jwt_char(chars[*cursor]) {
        *cursor += 1;
    }
    *cursor - start >= 8
}

fn starts_with_ignore_ascii_case(chars: &[char], start: usize, needle: &str) -> bool {
    let needle_chars: Vec<char> = needle.chars().collect();
    if start + needle_chars.len() > chars.len() {
        return false;
    }
    chars[start..start + needle_chars.len()]
        .iter()
        .zip(needle_chars)
        .all(|(left, right)| left.eq_ignore_ascii_case(&right))
}

fn starts_with_literal(chars: &[char], start: usize, needle: &str) -> bool {
    let needle_chars: Vec<char> = needle.chars().collect();
    if start + needle_chars.len() > chars.len() {
        return false;
    }
    chars[start..start + needle_chars.len()]
        .iter()
        .zip(needle_chars)
        .all(|(left, right)| *left == right)
}

fn is_token_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '+' | '/' | '=')
}

fn is_sk_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

fn is_jwt_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct VectorCase {
        id: String,
        kind: String,
        input: Value,
        must_not_contain: Vec<String>,
        must_contain: Vec<String>,
    }

    #[test]
    fn normalize_log_key_strips_separators() {
        assert_eq!(normalize_log_key("api_key"), "apikey");
        assert_eq!(normalize_log_key("apiKey"), "apikey");
        assert_eq!(normalize_log_key("API-KEY"), "apikey");
    }

    #[test]
    fn redact_vectors_from_shared_file() {
        let raw = include_str!("../../testdata/log_redaction_vectors.json");
        let cases: Vec<VectorCase> = serde_json::from_str(raw).unwrap();
        assert!(!cases.is_empty());

        for case in cases {
            let rendered = match case.kind.as_str() {
                "text" => redact_log_text(case.input.as_str().unwrap_or(&case.input.to_string())),
                "value" => redact_log_value(&case.input).to_string(),
                other => panic!("unknown vector kind {other}"),
            };

            for fragment in case.must_not_contain {
                assert!(
                    !rendered.contains(&fragment),
                    "{} still contains {fragment:?}: {rendered}",
                    case.id
                );
            }
            for fragment in case.must_contain {
                assert!(
                    rendered.contains(&fragment),
                    "{} missing {fragment:?}: {rendered}",
                    case.id
                );
            }
        }
    }

    #[test]
    fn ordinary_sentence_is_not_fully_masked() {
        assert_eq!(
            redact_log_text("profile applied successfully"),
            "profile applied successfully"
        );
    }
}
