// 内置 Prompt 模板管理器
// 提供常用的 AI 编程助手提示词模板
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// 内置 Prompt 模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinPrompt {
    /// 唯一标识
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 简介描述
    pub description: String,
    /// 完整提示词内容
    pub content: String,
    /// 分类标签
    pub category: PromptCategory,
    /// 使用场景标签
    pub tags: Vec<String>,
    /// 支持的变量占位符
    pub variables: Vec<PromptVariable>,
}

/// Prompt 分类
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptCategory {
    /// 代码审查
    CodeReview,
    /// 调试排错
    Debugging,
    /// 代码重构
    Refactoring,
    /// 测试生成
    Testing,
    /// 文档编写
    Documentation,
    /// 安全审计
    Security,
    /// 通用
    General,
}

impl PromptCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CodeReview => "code_review",
            Self::Debugging => "debugging",
            Self::Refactoring => "refactoring",
            Self::Testing => "testing",
            Self::Documentation => "documentation",
            Self::Security => "security",
            Self::General => "general",
        }
    }
}

/// 变量占位符定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVariable {
    /// 变量名（如 PROJECT_NAME）
    pub name: String,
    /// 描述
    pub description: String,
    /// 默认值
    pub default: Option<String>,
    /// 是否必填
    pub required: bool,
}

/// 获取所有内置 Prompt 模板
pub fn get_builtin_prompts() -> Vec<BuiltinPrompt> {
    vec![
        // 代码审查专家
        BuiltinPrompt {
            id: "code_review".into(),
            name: "代码审查专家".into(),
            description: "详细的代码审查，检查质量、性能和最佳实践".into(),
            category: PromptCategory::CodeReview,
            tags: vec!["review".into(), "quality".into(), "best-practices".into()],
            variables: vec![PromptVariable {
                name: "LANGUAGE".into(),
                description: "编程语言".into(),
                default: Some("auto".into()),
                required: false,
            }],
            content: r#"你是一位资深的代码审查专家。请对提交的代码进行全面审查：

## 审查清单

### 1. 代码质量
- [ ] 命名规范：变量、函数、类命名是否清晰易懂
- [ ] 代码结构：逻辑是否清晰，是否遵循单一职责原则
- [ ] 重复代码：是否存在可抽取的公共逻辑
- [ ] 代码注释：关键逻辑是否有适当注释

### 2. 性能考量
- [ ] 算法效率：时间复杂度是否合理
- [ ] 资源使用：内存分配、连接池等是否优化
- [ ] 并发安全：多线程场景是否正确处理

### 3. 安全检查
- [ ] 输入验证：用户输入是否正确校验
- [ ] 敏感信息：是否存在硬编码的密钥或凭证
- [ ] 异常处理：是否有完善的错误处理机制

### 4. 最佳实践
- [ ] 设计模式：是否恰当运用设计模式
- [ ] SOLID原则：是否遵循面向对象设计原则
- [ ] 测试覆盖：是否便于单元测试

请针对每个问题给出具体的代码行号和修改建议。"#
                .into(),
        },
        // 调试专家
        BuiltinPrompt {
            id: "debug_expert".into(),
            name: "调试排错专家".into(),
            description: "系统化的问题诊断和调试方法".into(),
            category: PromptCategory::Debugging,
            tags: vec!["debug".into(), "troubleshoot".into(), "error".into()],
            variables: vec![],
            content: r#"你是一位经验丰富的调试专家。请帮我系统地诊断和解决问题。

## 调试流程

### 1. 问题复现
请描述：
- 预期行为是什么？
- 实际行为是什么？
- 问题是如何触发的？

### 2. 信息收集
需要的信息：
- 错误日志或堆栈跟踪
- 相关代码片段
- 运行环境信息（系统版本、依赖版本等）

### 3. 假设验证
我会：
- 分析可能的原因并排序
- 提出验证每个假设的方法
- 逐步排除直到定位根因

### 4. 解决方案
- 提供针对性的修复代码
- 解释修复的原理
- 建议预防措施

请描述你遇到的问题，我会帮你一步步排查。"#
                .into(),
        },
        // 重构助手
        BuiltinPrompt {
            id: "refactor_assistant".into(),
            name: "重构助手".into(),
            description: "代码重构建议和实施指导".into(),
            category: PromptCategory::Refactoring,
            tags: vec![
                "refactor".into(),
                "clean-code".into(),
                "architecture".into(),
            ],
            variables: vec![],
            content: r#"你是一位代码重构专家。请帮我改进代码结构和质量。

## 重构原则

### 目标
- 提高代码可读性
- 增强可维护性
- 优化性能和资源使用
- 保持功能不变（行为等价）

### 常用重构手法
1. **提取方法** - 将长方法拆分为小的、职责单一的方法
2. **提取类** - 当一个类承担太多职责时
3. **内联** - 移除不必要的抽象层
4. **重命名** - 让命名更准确地表达意图
5. **移动** - 将方法/字段移动到更合适的类
6. **替换算法** - 用更清晰或高效的实现替换

### 安全重构步骤
1. 确保有测试覆盖
2. 小步修改，频繁验证
3. 每次只做一种重构
4. 保持测试通过

请提供需要重构的代码，我会给出具体的重构建议和实施步骤。"#
                .into(),
        },
        // 测试生成器
        BuiltinPrompt {
            id: "test_generator".into(),
            name: "测试生成器".into(),
            description: "自动生成单元测试和集成测试".into(),
            category: PromptCategory::Testing,
            tags: vec!["test".into(), "unit-test".into(), "tdd".into()],
            variables: vec![PromptVariable {
                name: "FRAMEWORK".into(),
                description: "测试框架".into(),
                default: Some("auto".into()),
                required: false,
            }],
            content: r#"你是一位测试专家。请为给定的代码生成全面的测试用例。

## 测试策略

### 1. 单元测试
- 正常路径测试（Happy Path）
- 边界条件测试
- 异常情况测试
- 空值和特殊值测试

### 2. 测试覆盖
- 语句覆盖
- 分支覆盖
- 条件覆盖

### 3. 测试结构
```
Given: 测试前置条件
When: 执行被测试的操作
Then: 验证预期结果
```

### 4. 命名规范
测试方法名应描述：
- 被测试的方法
- 测试场景
- 预期结果

例如: `test_calculate_sum_with_empty_list_returns_zero`

请提供需要测试的代码，我会生成完整的测试用例。"#
                .into(),
        },
        // 文档编写器
        BuiltinPrompt {
            id: "doc_writer".into(),
            name: "文档编写器".into(),
            description: "生成清晰的API文档和使用说明".into(),
            category: PromptCategory::Documentation,
            tags: vec!["docs".into(), "api".into(), "readme".into()],
            variables: vec![],
            content: r#"你是一位技术文档专家。请为代码生成清晰、完整的文档。

## 文档类型

### 1. API 文档
- 接口描述
- 参数说明（类型、是否必填、默认值）
- 返回值说明
- 使用示例
- 错误码说明

### 2. README 文档
- 项目简介
- 快速开始
- 安装步骤
- 配置说明
- 使用示例
- 常见问题

### 3. 代码注释
- 类/模块描述
- 方法签名和用途
- 参数和返回值
- 使用示例
- 注意事项

### 文档风格
- 语言简洁明了
- 包含实际可运行的示例
- 按逻辑顺序组织
- 使用合适的格式（标题、列表、代码块）

请提供需要文档化的代码，我会生成相应的文档。"#
                .into(),
        },
        // 安全审计专家
        BuiltinPrompt {
            id: "security_audit".into(),
            name: "安全审计专家".into(),
            description: "识别安全漏洞和提供修复建议".into(),
            category: PromptCategory::Security,
            tags: vec!["security".into(), "vulnerability".into(), "audit".into()],
            variables: vec![],
            content: r#"你是一位网络安全专家。请对代码进行全面的安全审计。

## 安全检查清单

### 1. 输入验证
- [ ] SQL 注入防护
- [ ] XSS（跨站脚本）防护
- [ ] 命令注入防护
- [ ] 路径遍历攻击防护
- [ ] 参数类型和范围验证

### 2. 认证授权
- [ ] 密码存储安全（哈希 + 盐值）
- [ ] 会话管理安全
- [ ] JWT/Token 处理安全
- [ ] 权限检查完整性
- [ ] CSRF 防护

### 3. 敏感数据
- [ ] 敏感信息加密存储
- [ ] 日志中无敏感信息泄露
- [ ] 配置文件安全
- [ ] API 密钥保护

### 4. 其他安全
- [ ] 依赖项漏洞检查
- [ ] 错误处理不泄露信息
- [ ] 速率限制和防刷
- [ ] HTTPS 强制使用

对于每个发现的问题，我会：
1. 说明风险等级（高/中/低）
2. 解释漏洞原理
3. 提供修复代码示例
4. 建议防护措施

请提供需要审计的代码。"#
                .into(),
        },
    ]
}

/// 根据 ID 获取单个模板
pub fn get_prompt_by_id(id: &str) -> Option<BuiltinPrompt> {
    get_builtin_prompts().into_iter().find(|p| p.id == id)
}

/// 根据分类获取模板
pub fn get_prompts_by_category(category: PromptCategory) -> Vec<BuiltinPrompt> {
    get_builtin_prompts()
        .into_iter()
        .filter(|p| p.category.as_str() == category.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_prompts() {
        let prompts = get_builtin_prompts();
        assert_eq!(prompts.len(), 6);

        let ids: Vec<&str> = prompts.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"code_review"));
        assert!(ids.contains(&"debug_expert"));
        assert!(ids.contains(&"security_audit"));
    }

    #[test]
    fn test_get_prompt_by_id() {
        let prompt = get_prompt_by_id("code_review");
        assert!(prompt.is_some());
        assert_eq!(prompt.unwrap().name, "代码审查专家");
    }
}
