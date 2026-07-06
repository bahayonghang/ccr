// 🔐 敏感信息掩码工具
// 统一处理 API 密钥、Token 等敏感信息的显示

/// 🔐 掩码敏感信息
///
/// 将敏感信息(如 API Token)进行部分隐藏显示
///
/// 掩码规则:
/// - 字符数 <= 10: 全部替换为 *
/// - 字符数 > 10: 显示前 4 个和后 4 个字符,中间用 ... 代替
///
/// # Examples
///
/// ```no_run
/// use ccr_core::mask_sensitive;
///
/// assert_eq!(mask_sensitive("sk-ant-1234567890abcdef"), "sk-a...cdef");
/// assert_eq!(mask_sensitive("short"), "*****");
/// ```
pub fn mask_sensitive(value: &str) -> String {
    // 按字符计数/切片: token 来自用户可编辑配置, 字节切片遇到多字节
    // 字符会在 char boundary 上 panic, 这里保证任意 UTF-8 输入安全。
    let char_count = value.chars().count();
    if char_count <= 10 {
        "*".repeat(char_count)
    } else {
        let visible_prefix: String = value.chars().take(4).collect();
        let visible_suffix: String = value.chars().skip(char_count - 4).collect();
        format!("{visible_prefix}...{visible_suffix}")
    }
}

/// 🔐 根据变量名判断是否需要掩码并执行掩码
///
/// 检查变量名是否包含敏感关键词(TOKEN, KEY, SECRET),
/// 如果是则执行掩码,否则返回原值
///
/// # Examples
///
/// ```no_run
/// use ccr_core::mask_if_sensitive;
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
#[allow(clippy::unwrap_used)]
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
    fn test_mask_sensitive_multibyte_no_panic() {
        // 多字节字符落在前 4 / 后 4 边界内不得 panic
        assert_eq!(mask_sensitive("sk-é-secret-key"), "sk-é...-key");
        assert_eq!(mask_sensitive("sk测试1234567890"), "sk测试...7890");
        assert_eq!(mask_sensitive("中文密码"), "****");
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
