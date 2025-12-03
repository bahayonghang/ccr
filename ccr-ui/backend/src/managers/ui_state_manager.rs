// UI State Manager - 管理收藏和历史记录的持久化

use crate::models::ui_state::{AddFavoriteRequest, CommandHistory, FavoriteCommand, UiState};
use chrono::Utc;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

/// UI 状态管理器
pub struct UiStateManager {
    /// 状态文件路径
    state_file: PathBuf,
    /// 内存中的状态缓存
    state: RwLock<UiState>,
}

impl UiStateManager {
    /// 创建新的管理器实例
    pub async fn new() -> Result<Self, String> {
        let state_file = Self::get_state_file_path()?;
        let state = Self::load_state(&state_file).await;

        Ok(Self {
            state_file,
            state: RwLock::new(state),
        })
    }

    /// 获取状态文件路径 (~/.ccr/ui_state.json)
    fn get_state_file_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let ccr_dir = home.join(".ccr");
        Ok(ccr_dir.join("ui_state.json"))
    }

    /// 从文件加载状态
    async fn load_state(path: &PathBuf) -> UiState {
        match fs::read_to_string(path).await {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(state) => {
                    info!("UI 状态已从 {:?} 加载", path);
                    state
                }
                Err(e) => {
                    error!("解析 UI 状态文件失败: {}, 使用默认状态", e);
                    UiState::new()
                }
            },
            Err(_) => {
                info!("UI 状态文件不存在, 使用默认状态");
                UiState::new()
            }
        }
    }

    /// 保存状态到文件
    async fn save_state(&self) -> Result<(), String> {
        let state = self.state.read().await;

        // 确保目录存在
        if let Some(parent) = self.state_file.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let content =
            serde_json::to_string_pretty(&*state).map_err(|e| format!("序列化状态失败: {}", e))?;

        fs::write(&self.state_file, content)
            .await
            .map_err(|e| format!("写入状态文件失败: {}", e))?;

        info!("UI 状态已保存到 {:?}", self.state_file);
        Ok(())
    }

    // ===== 收藏命令操作 =====

    /// 获取所有收藏命令
    pub async fn get_favorites(&self) -> Vec<FavoriteCommand> {
        let state = self.state.read().await;
        state.favorites.clone()
    }

    /// 添加收藏命令
    pub async fn add_favorite(&self, req: AddFavoriteRequest) -> Result<FavoriteCommand, String> {
        // 首先检查是否已存在相同的收藏
        {
            let state = self.state.read().await;
            if let Some(existing) = state
                .favorites
                .iter()
                .find(|f| f.command == req.command && f.args == req.args)
            {
                // 返回已存在的收藏，而不是创建新的
                return Ok(existing.clone());
            }
        }

        // 如果不存在，创建新的收藏
        let favorite = FavoriteCommand {
            id: Uuid::new_v4().to_string(),
            command: req.command,
            args: req.args,
            display_name: req.display_name,
            module: req.module,
            created_at: Utc::now(),
        };

        {
            let mut state = self.state.write().await;
            state.add_favorite(favorite.clone());
        }

        self.save_state().await?;
        Ok(favorite)
    }

    /// 删除收藏命令
    pub async fn remove_favorite(&self, id: &str) -> Result<bool, String> {
        let removed = {
            let mut state = self.state.write().await;
            state.remove_favorite(id)
        };

        if removed {
            self.save_state().await?;
        }

        Ok(removed)
    }

    // ===== 命令历史操作 =====

    /// 获取命令历史
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<CommandHistory> {
        let state = self.state.read().await;
        let limit = limit.unwrap_or(50);
        state.history.iter().take(limit).cloned().collect()
    }

    /// 添加命令历史
    pub async fn add_history(
        &self,
        command: String,
        args: Vec<String>,
        success: bool,
        duration_ms: u64,
    ) -> Result<CommandHistory, String> {
        let full_command = if args.is_empty() {
            format!("ccr {}", command)
        } else {
            format!("ccr {} {}", command, args.join(" "))
        };

        let history = CommandHistory {
            id: Uuid::new_v4().to_string(),
            full_command,
            command,
            args,
            success,
            executed_at: Utc::now(),
            duration_ms,
        };

        {
            let mut state = self.state.write().await;
            state.add_history(history.clone());
        }

        self.save_state().await?;
        Ok(history)
    }

    /// 清空命令历史
    pub async fn clear_history(&self) -> Result<(), String> {
        {
            let mut state = self.state.write().await;
            state.clear_history();
        }

        self.save_state().await
    }
}

// 全局单例
use once_cell::sync::OnceCell;
use std::sync::Arc;

static UI_STATE_MANAGER: OnceCell<Arc<UiStateManager>> = OnceCell::new();

/// 获取全局 UI 状态管理器实例
pub async fn get_ui_state_manager() -> Result<Arc<UiStateManager>, String> {
    if let Some(manager) = UI_STATE_MANAGER.get() {
        return Ok(Arc::clone(manager));
    }

    let manager = UiStateManager::new().await?;
    let manager = Arc::new(manager);

    // 尝试设置全局实例（忽略竞争条件）
    let _ = UI_STATE_MANAGER.set(Arc::clone(&manager));

    Ok(UI_STATE_MANAGER.get().map(Arc::clone).unwrap_or(manager))
}
