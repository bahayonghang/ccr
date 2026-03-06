//! 🛠️ CCR 工具模块
//!
//! 提供通用的工具函数和 trait。
//!
//! ## 模块
//!
//! - [`auto_complete`] - 自动补全支持
//! - [`mask`] - 敏感信息掩码
//! - [`toml_json`] - TOML/JSON 转换
//! - [`validation`] - 验证 trait

pub mod auto_complete;
pub mod mask;
pub mod toml_json;
pub mod validation;

pub use auto_complete::AutoCompletable;
pub use mask::{mask_if_sensitive, mask_sensitive};
pub use validation::Validatable;
