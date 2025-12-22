//! 🔍 Session 解析器
//!
//! 解析不同平台的 JSONL session 文件。

use crate::core::error::{CcrError, Result};
use crate::models::Platform;
use crate::sessions::models::{IndexStats, Session, SessionEvent};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};

/// 🔍 Session 解析器
pub struct SessionParser;

#[allow(dead_code)]
impl SessionParser {
    /// 解析 session 文件
    ///
    /// 自动检测平台格式并解析。
    pub fn parse_file(path: &Path, platform: Platform) -> Result<Session> {
        match platform {
            Platform::Claude => Self::parse_claude(path),
            Platform::Codex => Self::parse_codex(path),
            Platform::Gemini => Self::parse_gemini(path),
            Platform::Qwen | Platform::IFlow => Self::parse_generic(path, platform),
        }
    }

    /// 解析 Claude session 文件
    ///
    /// Claude session 文件格式: JSONL，每行一个事件
    pub fn parse_claude(path: &Path) -> Result<Session> {
        let events = Self::read_jsonl(path)?;

        let session_id = Self::extract_session_id(&events)
            .or_else(|| Self::extract_id_from_path(path))
            .unwrap_or_else(|| {
                let id = uuid::Uuid::new_v4().to_string();
                debug!("无法提取 session ID，生成新 ID: {}", id);
                id
            });

        let cwd = Self::extract_cwd(&events)
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let fallback = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
                if fallback.as_os_str().is_empty() {
                    debug!("无法提取工作目录，使用空路径: {}", path.display());
                }
                fallback
            });

        let title = Self::extract_title(&events);
        let (created_at, updated_at) = Self::extract_timestamps(&events, path)?;
        let (user_count, assistant_count, tool_count) = Self::count_messages(&events);

        let file_hash = Self::compute_file_hash(path)?;

        Ok(Session {
            id: session_id,
            platform: Platform::Claude,
            title,
            cwd,
            file_path: path.to_path_buf(),
            file_hash,
            created_at,
            updated_at,
            message_count: user_count + assistant_count,
            user_message_count: user_count,
            assistant_message_count: assistant_count,
            tool_use_count: tool_count,
            indexed_at: Utc::now(),
        })
    }

    /// 解析 Codex session 文件
    pub fn parse_codex(path: &Path) -> Result<Session> {
        let events = Self::read_jsonl(path)?;

        let session_id = Self::extract_session_id(&events)
            .or_else(|| Self::extract_id_from_path(path))
            .unwrap_or_else(|| {
                let id = uuid::Uuid::new_v4().to_string();
                debug!("无法提取 session ID，生成新 ID: {}", id);
                id
            });

        let cwd = Self::extract_cwd(&events)
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let fallback = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
                if fallback.as_os_str().is_empty() {
                    debug!("无法提取工作目录，使用空路径: {}", path.display());
                }
                fallback
            });

        let title = Self::extract_title(&events);
        let (created_at, updated_at) = Self::extract_timestamps(&events, path)?;
        let (user_count, assistant_count, tool_count) = Self::count_messages(&events);

        let file_hash = Self::compute_file_hash(path)?;

        Ok(Session {
            id: session_id,
            platform: Platform::Codex,
            title,
            cwd,
            file_path: path.to_path_buf(),
            file_hash,
            created_at,
            updated_at,
            message_count: user_count + assistant_count,
            user_message_count: user_count,
            assistant_message_count: assistant_count,
            tool_use_count: tool_count,
            indexed_at: Utc::now(),
        })
    }

    /// 解析 Gemini session 文件
    pub fn parse_gemini(path: &Path) -> Result<Session> {
        // Gemini 使用不同的格式，尝试解析
        let events = Self::read_jsonl(path).unwrap_or_else(|e| {
            debug!(
                "Gemini session 文件解析失败，使用空事件列表: {} - {}",
                path.display(),
                e
            );
            Vec::new()
        });

        let session_id = Self::extract_id_from_path(path).unwrap_or_else(|| {
            let id = uuid::Uuid::new_v4().to_string();
            debug!("无法从路径提取 session ID，生成新 ID: {}", id);
            id
        });

        let cwd = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| {
            debug!("无法获取文件父目录: {}", path.display());
            PathBuf::new()
        });

        let title = Self::extract_title(&events);
        let (created_at, updated_at) = Self::extract_timestamps(&events, path)?;
        let (user_count, assistant_count, tool_count) = Self::count_messages(&events);

        let file_hash = Self::compute_file_hash(path)?;

        Ok(Session {
            id: session_id,
            platform: Platform::Gemini,
            title,
            cwd,
            file_path: path.to_path_buf(),
            file_hash,
            created_at,
            updated_at,
            message_count: user_count + assistant_count,
            user_message_count: user_count,
            assistant_message_count: assistant_count,
            tool_use_count: tool_count,
            indexed_at: Utc::now(),
        })
    }

    /// 解析通用格式（用于 Qwen、iFlow 等）
    fn parse_generic(path: &Path, platform: Platform) -> Result<Session> {
        let events = Self::read_jsonl(path).unwrap_or_else(|e| {
            debug!(
                "{:?} session 文件解析失败，使用空事件列表: {} - {}",
                platform,
                path.display(),
                e
            );
            Vec::new()
        });

        let session_id = Self::extract_id_from_path(path).unwrap_or_else(|| {
            let id = uuid::Uuid::new_v4().to_string();
            debug!("无法从路径提取 session ID，生成新 ID: {}", id);
            id
        });

        let cwd = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| {
            debug!("无法获取文件父目录: {}", path.display());
            PathBuf::new()
        });

        let title = Self::extract_title(&events);
        let (created_at, updated_at) = Self::extract_timestamps(&events, path)?;
        let (user_count, assistant_count, tool_count) = Self::count_messages(&events);

        let file_hash = Self::compute_file_hash(path)?;

        Ok(Session {
            id: session_id,
            platform,
            title,
            cwd,
            file_path: path.to_path_buf(),
            file_hash,
            created_at,
            updated_at,
            message_count: user_count + assistant_count,
            user_message_count: user_count,
            assistant_message_count: assistant_count,
            tool_use_count: tool_count,
            indexed_at: Utc::now(),
        })
    }

    /// 读取 JSONL 文件
    fn read_jsonl(path: &Path) -> Result<Vec<SessionEvent>> {
        let file = File::open(path).map_err(|e| {
            CcrError::ConfigError(format!("无法打开文件 {}: {}", path.display(), e))
        })?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    trace!("读取行 {} 失败: {}", line_num, e);
                    continue;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<SessionEvent>(&line) {
                Ok(mut event) => {
                    event.raw_json = Some(line.clone());
                    events.push(event);
                }
                Err(e) => {
                    trace!(
                        "解析行 {} 失败: {} - {}",
                        line_num,
                        e,
                        &line[..line.len().min(100)]
                    );
                }
            }
        }

        debug!("从 {} 解析了 {} 个事件", path.display(), events.len());
        Ok(events)
    }

    /// 从事件中提取 session ID
    fn extract_session_id(events: &[SessionEvent]) -> Option<String> {
        events.iter().find_map(|e| e.session_id.clone())
    }

    /// 从文件路径提取 ID
    fn extract_id_from_path(path: &Path) -> Option<String> {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    /// 从事件中提取工作目录
    fn extract_cwd(events: &[SessionEvent]) -> Option<String> {
        events.iter().find_map(|e| e.cwd.clone())
    }

    /// 从事件中提取标题
    fn extract_title(events: &[SessionEvent]) -> Option<String> {
        // 尝试从第一条用户消息获取标题
        events
            .iter()
            .find(|e| e.is_user_message())
            .and_then(|e| e.message_text())
            .map(|msg| {
                // 截取前 50 个字符作为标题
                let title = msg.trim();
                let chars: Vec<char> = title.chars().collect();
                if chars.len() > 50 {
                    let s: String = chars.into_iter().take(47).collect();
                    format!("{}...", s)
                } else {
                    title.to_string()
                }
            })
    }

    /// 从事件中提取时间戳
    fn extract_timestamps(
        events: &[SessionEvent],
        path: &Path,
    ) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
        let timestamps: Vec<DateTime<Utc>> = events
            .iter()
            .filter_map(|e| e.timestamp.as_ref())
            .filter_map(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .collect();

        // 使用 first/last 的安全版本，避免 unwrap
        if let (Some(&created), Some(&updated)) = (timestamps.first(), timestamps.last()) {
            return Ok((created, updated));
        }

        // 回退到文件元数据
        let metadata = std::fs::metadata(path).map_err(|e| {
            CcrError::ConfigError(format!("无法读取文件元数据 {}: {}", path.display(), e))
        })?;

        let modified = metadata
            .modified()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(|_| Utc::now());

        let created = metadata
            .created()
            .map(DateTime::<Utc>::from)
            .unwrap_or(modified);

        Ok((created, modified))
    }

    /// 统计消息数量
    fn count_messages(events: &[SessionEvent]) -> (u32, u32, u32) {
        let mut user_count = 0u32;
        let mut assistant_count = 0u32;
        let mut tool_count = 0u32;

        for event in events {
            if event.is_user_message() {
                user_count += 1;
            } else if event.is_assistant_message() {
                assistant_count += 1;
            }
            if event.is_tool_use() {
                tool_count += 1;
            }
        }

        (user_count, assistant_count, tool_count)
    }

    /// 计算文件哈希
    fn compute_file_hash(path: &Path) -> Result<String> {
        let content = std::fs::read(path).map_err(|e| {
            CcrError::ConfigError(format!("无法读取文件 {}: {}", path.display(), e))
        })?;

        let hash = blake3::hash(&content);
        Ok(hash.to_hex().to_string())
    }

    /// 扫描目录查找 session 文件
    pub fn scan_directory(dir: &Path, platform: Platform) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if !dir.exists() {
            debug!("目录不存在: {}", dir.display());
            return Ok(files);
        }

        Self::scan_directory_recursive(dir, &mut files, platform)?;

        debug!(
            "在 {} 中找到 {} 个 session 文件",
            dir.display(),
            files.len()
        );

        Ok(files)
    }

    fn scan_directory_recursive(
        dir: &Path,
        files: &mut Vec<PathBuf>,
        platform: Platform,
    ) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| CcrError::ConfigError(format!("无法读取目录 {}: {}", dir.display(), e)))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // 递归扫描子目录
                let _ = Self::scan_directory_recursive(&path, files, platform);
            } else if Self::is_session_file(&path, &platform) {
                files.push(path);
            }
        }

        Ok(())
    }

    /// 判断是否是 session 文件
    fn is_session_file(path: &Path, platform: &Platform) -> bool {
        let extension = path.extension().and_then(|e| e.to_str());

        match platform {
            Platform::Claude | Platform::Codex => extension == Some("jsonl"),
            Platform::Gemini => {
                // Gemini 可能使用不同的扩展名
                extension == Some("jsonl") || extension == Some("json")
            }
            _ => extension == Some("jsonl"),
        }
    }

    /// 获取平台的默认 session 目录
    pub fn get_platform_session_dir(platform: &Platform) -> Option<PathBuf> {
        let home = dirs::home_dir()?;

        let path = match platform {
            Platform::Claude => home.join(".claude").join("projects"),
            Platform::Codex => home.join(".codex").join("sessions"),
            Platform::Gemini => home.join(".gemini").join("tmp"),
            Platform::Qwen => home.join(".qwen").join("sessions"),
            Platform::IFlow => home.join(".iflow").join("sessions"),
        };

        if path.exists() { Some(path) } else { None }
    }

    /// 批量解析多个文件
    pub fn parse_files(paths: &[PathBuf], platform: Platform) -> (Vec<Session>, IndexStats) {
        let mut sessions = Vec::new();
        let mut stats = IndexStats::default();

        let start = std::time::Instant::now();

        for path in paths {
            stats.files_scanned += 1;

            match Self::parse_file(path, platform) {
                Ok(session) => {
                    sessions.push(session);
                    stats.sessions_added += 1;
                }
                Err(e) => {
                    warn!("解析文件失败 {}: {}", path.display(), e);
                    stats.errors += 1;
                }
            }
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;

        (sessions, stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_jsonl(content: &str) -> PathBuf {
        let dir = tempdir().expect("Failed to create temp directory for test");
        let file_path = dir.path().join("test.jsonl");
        let mut file = File::create(&file_path).expect("Failed to create test JSONL file");
        write!(file, "{}", content).expect("Failed to write test JSONL content");
        std::mem::forget(dir); // 保持目录存活
        file_path
    }

    #[test]
    fn test_parse_simple_session() {
        let content = r#"{"type": "init", "session_id": "test-123", "cwd": "/tmp/test"}
{"type": "user", "role": "user", "message": "Hello, world!"}
{"type": "assistant", "role": "assistant", "message": "Hi there!"}
"#;

        let path = create_test_jsonl(content);
        let session = SessionParser::parse_claude(&path)
            .expect("Failed to parse test session");

        assert_eq!(session.id, "test-123");
        assert_eq!(session.platform, Platform::Claude);
        assert!(session.message_count >= 2);
    }

    #[test]
    fn test_is_session_file() {
        assert!(SessionParser::is_session_file(
            Path::new("/tmp/test.jsonl"),
            &Platform::Claude
        ));
        assert!(!SessionParser::is_session_file(
            Path::new("/tmp/test.txt"),
            &Platform::Claude
        ));
    }
}
