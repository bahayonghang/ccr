// ✅ 验证 trait 定义
// 为所有需要验证的数据结构提供统一接口

use crate::error::Result;

/// ✅ 可验证 trait
///
/// 为数据结构提供统一的验证接口
/// 所有需要验证完整性和正确性的数据类型都应实现此 trait
///
/// # Examples
///
/// ```
/// use ccr::utils::Validatable;
/// use ccr::error::Result;
///
/// struct MyConfig {
///     name: String,
///     value: i32,
/// }
///
/// impl Validatable for MyConfig {
///     fn validate(&self) -> Result<()> {
///         if self.name.is_empty() {
///             return Err(CcrError::ValidationError("name cannot be empty".into()));
///         }
///         if self.value < 0 {
///             return Err(CcrError::ValidationError("value must be positive".into()));
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait Validatable {
    /// 验证数据结构的完整性和正确性
    ///
    /// # Returns
    /// - `Ok(())` - 验证通过
    /// - `Err(CcrError)` - 验证失败，包含错误详情
    fn validate(&self) -> Result<()>;
}
