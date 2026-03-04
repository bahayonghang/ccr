//! # ccr-db
//!
//! CCR Desktop 数据库层 — 提供 SQLite 统一存储、CheckIn 服务和数据模型。
//!
//! 本 crate 从 `ccr-ui-backend` 提取，封装所有与 SQLite 数据库相关的功能，
//! 使 Tauri 应用和未来的其他消费者可以共享数据访问逻辑。
//!
//! ## 模块结构
//!
//! - `database` — 连接池、Schema 定义、迁移、Repository (DAO)
//! - `models` — 数据模型 (CheckIn、UI State、Usage、Monitoring)
//! - `managers` — 业务管理器 (CheckIn 账号/提供商/记录/余额)
//! - `services` — 高层业务服务 (签到执行、用量导入、日志持久化、格式转换)
//! - `core` — 基础设施 (错误类型、加密、命令执行器)

pub mod core;
pub mod database;
pub mod managers;
pub mod models;
pub mod services;
