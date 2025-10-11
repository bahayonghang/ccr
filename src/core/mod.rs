// 🏗️ CCR 核心抽象模块
// 提供通用的核心抽象和基础设施

pub mod atomic_writer;
pub mod file_manager;

// 这些模块为将来扩展准备，暂时允许未使用警告
#[allow(unused_imports)]
pub use atomic_writer::AtomicWriter;
#[allow(unused_imports)]
pub use file_manager::FileManager;
