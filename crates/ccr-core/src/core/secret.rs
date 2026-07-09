// 🔐 Secret newtype - 凭据的类型级掩码封装
//
// 设计约束（design.md §2/§3）：
// - Debug/Display/默认 Serialize 一律输出掩码（算法唯一归属 utils::mask::mask_sensitive）
// - expose() 是读取原文的唯一出口；不提供 Deref/AsRef/Into<String>
// - Deserialize 透明接受明文（磁盘已有配置无损读入）
// - 落盘原文必须通过 expose_plaintext / expose_plaintext_option 显式注解，
//   `rg 'expose_plaintext'` 即全部明文持久化点位清单

use crate::utils::mask_sensitive;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A credential string whose `Debug`, `Display`, and default `Serialize`
/// output are always masked.
///
/// `expose()` is the only way to read the plaintext. Deserialization accepts
/// plaintext transparently, so existing on-disk configs load losslessly.
/// Persistence paths that must write the plaintext back to disk opt in with
/// `#[serde(serialize_with = "ccr_core::expose_plaintext")]` (or the
/// `_option` variant for `Option<Secret>` fields).
///
/// # Examples
///
/// ```
/// use ccr_core::Secret;
///
/// let token = Secret::new("sk-ant-1234567890abcdef");
/// assert_eq!(format!("{}", token), "sk-a...cdef");
/// assert_eq!(format!("{:?}", token), "Secret(\"sk-a...cdef\")");
/// assert_eq!(token.expose(), "sk-ant-1234567890abcdef");
/// ```
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// Wraps a plaintext credential.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the plaintext. This is the single read channel; keep call
    /// sites limited to legitimate consumption points (env construction,
    /// HTTP auth, persistence serializers).
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Returns `true` if the underlying credential is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for Secret {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Secret {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

// 测试断言便利：与明文比较不需要 expose()。
// 已知残余风险：任何 PartialEq 都是 eq-oracle，此处接受（design.md §2）。
impl PartialEq<str> for Secret {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for Secret {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&mask_sensitive(&self.0))
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 显式携带类型名，日志里可与普通字符串区分
        write!(f, "Secret(\"{}\")", mask_sensitive(&self.0))
    }
}

// 默认序列化 = 掩码：新增的响应/日志/导出结构无需任何注解即安全。
// 漏加 expose 注解的持久化路径会把掩码串写盘 —— 由 round-trip 测试与
// 首次保存显性暴露（对比现状“漏调 mask → 静默泄漏”，失败模式更可见）。
impl Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&mask_sensitive(&self.0))
    }
}

impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Secret)
    }
}

/// Explicit plaintext serializer for persistence fields.
///
/// Use with `#[serde(serialize_with = "ccr_core::expose_plaintext")]` on
/// `Secret` fields that must keep their on-disk plaintext format.
pub fn expose_plaintext<S>(secret: &Secret, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(secret.expose())
}

/// `Option<Secret>` variant of [`expose_plaintext`].
///
/// Combine with `skip_serializing_if = "Option::is_none"` to preserve the
/// existing "absent field" on-disk shape.
pub fn expose_plaintext_option<S>(secret: &Option<Secret>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match secret {
        Some(secret) => serializer.serialize_some(secret.expose()),
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// 模拟持久化结构：expose 注解字段落盘原文
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct PersistModel {
        name: String,
        #[serde(serialize_with = "expose_plaintext")]
        password: Secret,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            serialize_with = "expose_plaintext_option"
        )]
        auth_token: Option<Secret>,
    }

    /// 模拟响应结构：无注解字段默认掩码
    #[derive(Serialize)]
    struct ResponseModel {
        password: Secret,
    }

    #[test]
    fn test_display_and_debug_are_masked() {
        let long = Secret::new("sk-ant-1234567890abcdef");
        assert_eq!(format!("{}", long), "sk-a...cdef");
        assert_eq!(format!("{:?}", long), "Secret(\"sk-a...cdef\")");

        // 短值全遮，空值输出空串（与 mask_sensitive 规则一致）
        assert_eq!(format!("{}", Secret::new("short")), "*****");
        assert_eq!(format!("{}", Secret::new("")), "");

        // 任何格式化路径都不得出现原文
        let debug = format!("{:?}", long);
        assert!(!debug.contains("1234567890"));
    }

    #[test]
    fn test_default_serialize_masks_json_and_toml() {
        let model = ResponseModel {
            password: Secret::new("super-secret-password"),
        };

        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, r#"{"password":"supe...word"}"#);
        assert!(!json.contains("super-secret-password"));

        let toml = toml::to_string(&model).unwrap();
        assert!(toml.contains("supe...word"));
        assert!(!toml.contains("super-secret-password"));
    }

    #[test]
    fn test_expose_plaintext_serializes_original() {
        let model = PersistModel {
            name: "webdav".to_string(),
            password: Secret::new("app-password-123456"),
            auth_token: Some(Secret::new("sk-ant-1234567890abcdef")),
        };

        let toml = toml::to_string(&model).unwrap();
        assert!(toml.contains("app-password-123456"));
        assert!(toml.contains("sk-ant-1234567890abcdef"));

        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("app-password-123456"));
    }

    #[test]
    fn test_deserialize_accepts_plaintext_round_trip() {
        // 旧格式磁盘文件：明文凭据
        let legacy = r#"
name = "webdav"
password = "app-password-123456"
auth_token = "sk-ant-1234567890abcdef"
"#;
        let model: PersistModel = toml::from_str(legacy).unwrap();
        assert_eq!(model.password.expose(), "app-password-123456");
        assert_eq!(
            model.auth_token.as_ref().unwrap().expose(),
            "sk-ant-1234567890abcdef"
        );

        // 读旧 → 存 → 再读：无损
        let saved = toml::to_string(&model).unwrap();
        let reloaded: PersistModel = toml::from_str(&saved).unwrap();
        assert_eq!(model, reloaded);
    }

    #[test]
    fn test_option_none_is_skipped_on_disk() {
        let model = PersistModel {
            name: "n".to_string(),
            password: Secret::new("p-123456789"),
            auth_token: None,
        };
        let toml = toml::to_string(&model).unwrap();
        assert!(!toml.contains("auth_token"));
    }

    #[test]
    fn test_partial_eq_and_from() {
        let secret = Secret::from("plain-value");
        assert_eq!(secret, "plain-value");
        assert_eq!(Secret::from("x".to_string()), "x");
        assert_eq!(Secret::new("a"), Secret::from("a"));
        assert_ne!(Secret::new("a"), "b");
    }

    #[test]
    fn test_is_empty_and_default() {
        assert!(Secret::default().is_empty());
        assert!(Secret::new("").is_empty());
        assert!(!Secret::new("x").is_empty());
    }
}
