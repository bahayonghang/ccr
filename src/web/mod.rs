// 🌐 CCR Web 服务器模块
// 🖥️ 提供配置管理的 Web 界面和 RESTful API

pub mod error_utils;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod server;
pub mod system_info_cache;

pub use server::web_command;

use crate::core::error::Result;

/// Web 命令入口
#[allow(dead_code)]
pub async fn start_web_server(port: Option<u16>) -> Result<()> {
    web_command(None, port, false).await
}
