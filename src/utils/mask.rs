// 🔐 敏感信息掩码工具
// 统一处理 API 密钥、Token 等敏感信息的显示

/// 🔐 掩码敏感信息
///
/// 将敏感信息(如 API Token)进行部分隐藏显示
///
/// 掩码规则:
/// - 长度 <= 10: 全部替换为 *
/// - 长度 > 10: 显示前 4 位和后 4 位,中间用 ... 代替
///
/// # Examples
///
/// ```
/// use ccr::utils::mask_sensitive;
///
/// assert_eq!(mask_sensitive("sk-ant-1234567890abcdef"), "sk-a...cdef");
/// assert_eq!(mask_sensitive("short"), "*****");
/// ```
pub fn mask_sensitive(value: &str) -> String {
    if value.len() <= 10 {
        "*".repeat(value.len())
    } else {
        let visible_prefix = &value[..4];
        let visible_suffix = &value[value.len() - 4..];
        format!("{}...{}", visible_prefix, visible_suffix)
    }
}

/// 🔐 根据变量名判断是否需要掩码并执行掩码
///
/// 检查变量名是否包含敏感关键词(TOKEN, KEY, SECRET),
/// 如果是则执行掩码,否则返回原值
///
/// # Examples
///
/// ```
/// use ccr::utils::mask_if_sensitive;
///
/// assert_eq!(mask_if_sensitive("ANTHROPIC_AUTH_TOKEN", "sk-ant-123456"), "sk-a...3456");
/// assert_eq!(mask_if_sensitive("ANTHROPIC_BASE_URL", "https://api.com"), "https://api.com");
/// ```
pub fn mask_if_sensitive(var_name: &str, value: &str) -> String {
    if var_name.contains("TOKEN") || var_name.contains("KEY") || var_name.contains("SECRET") {
        mask_sensitive(value)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_sensitive() {
        assert_eq!(mask_sensitive("sk-ant-1234567890abcdef"), "sk-a...cdef");
        assert_eq!(mask_sensitive("short"), "*****");
        assert_eq!(mask_sensitive(""), "");
        assert_eq!(mask_sensitive("1234567890"), "**********");
        assert_eq!(mask_sensitive("12345678901"), "1234...8901");
    }

    #[test]
    fn test_mask_if_sensitive() {
        assert_eq!(
            mask_if_sensitive("ANTHROPIC_AUTH_TOKEN", "sk-ant-1234567890abcdef"),
            "sk-a...cdef"
        );
        assert_eq!(mask_if_sensitive("API_KEY", "key123456789"), "key1...6789");
        assert_eq!(
            mask_if_sensitive("SECRET_KEY", "secret123456789"),
            "secr...6789"
        );
        assert_eq!(
            mask_if_sensitive("ANTHROPIC_BASE_URL", "https://api.anthropic.com"),
            "https://api.anthropic.com"
        );
        assert_eq!(
            mask_if_sensitive("ANTHROPIC_MODEL", "claude-3-opus"),
            "claude-3-opus"
        );
    }
}
