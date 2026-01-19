// UI State Models - 收藏命令和命令历史数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 收藏的命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteCommand {
    /// 唯一标识
    pub id: String,
    /// 命令名称 (如 "platform switch")
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 显示名称 (用户自定义)
    pub display_name: Option<String>,
    /// 所属模块
    pub module: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 命令执行历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    /// 唯一标识
    pub id: String,
    /// 完整命令 (如 "ccr platform switch claude")
    pub full_command: String,
    /// 命令名称
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 执行是否成功
    pub success: bool,
    /// 执行时间
    pub executed_at: DateTime<Utc>,
    /// 执行耗时 (毫秒)
    pub duration_ms: u64,
}

/// UI 状态持久化数据
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiState {
    /// 收藏的命令列表
    #[serde(default)]
    pub favorites: Vec<FavoriteCommand>,
    /// 命令执行历史 (最近 100 条)
    #[serde(default)]
    pub history: Vec<CommandHistory>,
    /// 最后更新时间
    pub last_updated: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
impl UiState {
    /// 创建新的 UI 状态
    pub fn new() -> Self {
        Self {
            favorites: Vec::new(),
            history: Vec::new(),
            last_updated: Some(Utc::now()),
        }
    }

    /// 添加收藏命令
    pub fn add_favorite(&mut self, favorite: FavoriteCommand) {
        // 检查是否已存在相同命令
        if !self
            .favorites
            .iter()
            .any(|f| f.command == favorite.command && f.args == favorite.args)
        {
            self.favorites.push(favorite);
            self.last_updated = Some(Utc::now());
        }
    }

    /// 删除收藏命令
    pub fn remove_favorite(&mut self, id: &str) -> bool {
        let len_before = self.favorites.len();
        self.favorites.retain(|f| f.id != id);
        let removed = self.favorites.len() < len_before;
        if removed {
            self.last_updated = Some(Utc::now());
        }
        removed
    }

    /// 添加命令历史
    pub fn add_history(&mut self, history: CommandHistory) {
        self.history.insert(0, history);
        // 保留最近 100 条记录
        if self.history.len() > 100 {
            self.history.truncate(100);
        }
        self.last_updated = Some(Utc::now());
    }

    /// 清空命令历史
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.last_updated = Some(Utc::now());
    }
}

// ===== API Request/Response Models =====

/// 添加收藏请求
#[derive(Debug, Deserialize)]
pub struct AddFavoriteRequest {
    pub command: String,
    pub args: Vec<String>,
    pub display_name: Option<String>,
    pub module: String,
}

/// 收藏列表响应
#[derive(Debug, Serialize)]
pub struct FavoritesResponse {
    pub favorites: Vec<FavoriteCommand>,
}

/// 历史列表响应
#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub history: Vec<CommandHistory>,
    pub total: usize,
}
