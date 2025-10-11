// 🛠️ CCR 工具模块
// 提供通用的工具函数和 trait

pub mod mask;
pub mod validation;

pub use mask::{mask_if_sensitive, mask_sensitive};
pub use validation::Validatable;
