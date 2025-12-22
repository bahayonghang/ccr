// WebSocket 服务层
// 提供实时日志推送和连接管理

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{error, info};

use crate::models::monitoring::{LogMessage, LogSource, WsMessage};

/// WebSocket 服务状态
pub struct WsState {
    /// 广播通道发送端
    tx: broadcast::Sender<String>,
    /// 日志缓存（最近 500 条）
    log_cache: Arc<RwLock<Vec<LogMessage>>>,
    /// 最大缓存日志数量
    max_logs: usize,
}

impl WsState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            tx,
            log_cache: Arc::new(RwLock::new(Vec::new())),
            max_logs: 500,
        }
    }

    /// 广播消息到所有连接的客户端
    pub fn broadcast(&self, msg: WsMessage) {
        if let Ok(json) = msg.to_json() {
            let _ = self.tx.send(json);
        }
    }

    /// 广播日志消息并缓存
    pub async fn broadcast_log(&self, log: LogMessage) {
        // 添加到缓存
        {
            let mut cache = self.log_cache.write().await;
            cache.push(log.clone());
            // 保持缓存大小限制
            if cache.len() > self.max_logs {
                cache.remove(0);
            }
        }

        // 广播
        self.broadcast(WsMessage::Log(log));
    }

    /// 获取订阅者
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    /// 获取缓存的日志
    pub async fn get_cached_logs(&self) -> Vec<LogMessage> {
        self.log_cache.read().await.clone()
    }

    /// 添加系统日志
    pub async fn log_system(&self, message: impl Into<String>) {
        let log = LogMessage::info(LogSource::System, message);
        self.broadcast_log(log).await;
    }
}

impl Default for WsState {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket 连接处理器
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WsState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// 处理 WebSocket 连接
async fn handle_socket(socket: WebSocket, state: Arc<WsState>) {
    let (mut sender, mut receiver) = socket.split();

    // 订阅广播通道
    let mut rx = state.subscribe();

    // 发送历史日志
    let cached_logs = state.get_cached_logs().await;
    if !cached_logs.is_empty() {
        let batch_msg = WsMessage::LogBatch(cached_logs);
        if let Ok(json) = batch_msg.to_json()
            && sender.send(Message::Text(json.into())).await.is_err()
        {
            return;
        }
    }

    info!("WebSocket client connected");
    state.log_system("New monitoring client connected").await;

    // 使用 select! 同时处理接收和广播
    loop {
        tokio::select! {
            // 处理来自客户端的消息
            Some(result) = receiver.next() => {
                match result {
                    Ok(Message::Text(text)) => {
                        // 处理客户端发来的消息
                        if let Ok(msg) = serde_json::from_str::<WsMessage>(&text)
                            && matches!(msg, WsMessage::Ping)
                        {
                            // 响应 ping
                            let pong = WsMessage::Pong;
                            if let Ok(json) = pong.to_json() {
                                let _ = sender.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket client disconnected");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            // 处理来自广播通道的消息
            Ok(msg) = rx.recv() => {
                if sender.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            // 所有通道关闭
            else => break,
        }
    }

    info!("WebSocket connection closed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ws_state() {
        let state = WsState::new();

        // 添加日志
        state.log_system("Test message").await;

        // 检查缓存
        let logs = state.get_cached_logs().await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Test message");
    }
}
