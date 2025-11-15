// 🤖 CCR 自动补全工具模块
// 📦 提供配置字段自动补全功能
//
// 核心职责:
// - 🔍 定义 AutoCompletable trait
// - 🛠️ 提供配置字段补全接口
// - 🔄 支持未来字段扩展

/// 自动补全 trait
///
/// 为配置结构提供自动补全缺失可选字段的能力
///
/// ## 设计目的
/// 当 CCR 添加新的可选字段时，旧的 TOML 配置文件可能缺少这些字段。
/// 实现此 trait 的结构体可以在加载时自动检测并补全缺失的字段，
/// 提供无缝的升级体验。
///
/// ## 实现要求
/// 实现者应该:
/// 1. 检查所有可选字段是否为 `None`
/// 2. 为 `None` 字段设置合理的默认值
/// 3. 如果有任何字段被修改，返回 `true`
/// 4. 添加 debug 级别日志记录补全的字段
///
/// ## 示例
/// ```rust,ignore
/// use ccr::utils::auto_complete::AutoCompletable;
///
/// impl AutoCompletable for MyConfig {
///     fn auto_complete(&mut self) -> bool {
///         let mut modified = false;
///
///         if self.optional_field.is_none() {
///             self.optional_field = Some(default_value);
///             modified = true;
///             log::debug!("Auto-completed optional_field");
///         }
///
///         modified
///     }
/// }
/// ```
pub trait AutoCompletable {
    /// 自动补全缺失的可选字段
    ///
    /// # 返回值
    /// - `true`: 至少有一个字段被补全（配置已修改）
    /// - `false`: 所有字段已存在，无需修改
    ///
    /// # 副作用
    /// 此方法会直接修改 `self`，为缺失的字段设置默认值
    fn auto_complete(&mut self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用的简单配置结构
    #[derive(Debug, Clone, PartialEq)]
    struct TestConfig {
        required_field: String,
        optional_field_1: Option<u32>,
        optional_field_2: Option<bool>,
    }

    impl AutoCompletable for TestConfig {
        fn auto_complete(&mut self) -> bool {
            let mut modified = false;

            if self.optional_field_1.is_none() {
                self.optional_field_1 = Some(0);
                modified = true;
            }

            if self.optional_field_2.is_none() {
                self.optional_field_2 = Some(true);
                modified = true;
            }

            modified
        }
    }

    #[test]
    fn test_auto_complete_all_fields_missing() {
        let mut config = TestConfig {
            required_field: "test".to_string(),
            optional_field_1: None,
            optional_field_2: None,
        };

        let modified = config.auto_complete();

        assert!(modified, "Should return true when fields are completed");
        assert_eq!(config.optional_field_1, Some(0));
        assert_eq!(config.optional_field_2, Some(true));
    }

    #[test]
    fn test_auto_complete_partial_fields_missing() {
        let mut config = TestConfig {
            required_field: "test".to_string(),
            optional_field_1: Some(42),
            optional_field_2: None,
        };

        let modified = config.auto_complete();

        assert!(
            modified,
            "Should return true when some fields are completed"
        );
        assert_eq!(config.optional_field_1, Some(42)); // 保持原值
        assert_eq!(config.optional_field_2, Some(true)); // 被补全
    }

    #[test]
    fn test_auto_complete_no_fields_missing() {
        let mut config = TestConfig {
            required_field: "test".to_string(),
            optional_field_1: Some(42),
            optional_field_2: Some(false),
        };

        let modified = config.auto_complete();

        assert!(
            !modified,
            "Should return false when no fields need completion"
        );
        assert_eq!(config.optional_field_1, Some(42));
        assert_eq!(config.optional_field_2, Some(false));
    }

    #[test]
    fn test_auto_complete_idempotent() {
        let mut config = TestConfig {
            required_field: "test".to_string(),
            optional_field_1: None,
            optional_field_2: None,
        };

        // 第一次补全
        let modified1 = config.auto_complete();
        assert!(modified1);

        // 第二次补全 - 应该不再修改
        let modified2 = config.auto_complete();
        assert!(!modified2, "Second call should not modify anything");

        // 值应该保持不变
        assert_eq!(config.optional_field_1, Some(0));
        assert_eq!(config.optional_field_2, Some(true));
    }
}
