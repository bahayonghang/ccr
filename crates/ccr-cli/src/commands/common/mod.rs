// 🔧 common 命令模块 - 公共工具和模式
// 提供配置模式检测、表格构建、交互式提示等公共功能
//
// 注：这些是设计好的公共 API，当前部分功能尚未被使用，
// 但保留以供将来扩展和外部调用。

mod mode;
mod prompt;
#[expect(dead_code)]
mod table;

// 公共 API 导出
#[allow(unused_imports)]
pub use mode::detect_config_mode;
pub use prompt::{prompt_optional, prompt_required, prompt_tags};
#[allow(unused_imports)]
pub use table::{ConfigTableBuilder, PlatformTableBuilder, TablePreset};
pub(crate) use table::{new_table, new_utf8_table};
