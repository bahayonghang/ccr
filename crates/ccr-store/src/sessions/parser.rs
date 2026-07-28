//! 🔍 Session 解析器
//!
//! 解析不同平台的 JSONL session 文件。

use crate::sessions::models::{IndexStats, Session, SessionEvent};
use ccr_config::Platform;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::{is_qwen_chat_file, qwen_project_dir_name_from_chat_path, qwen_projects_dir};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};

/// 🔍 Session 解析器
pub struct SessionParser;

impl SessionParser {
    /// 解析 session 文件
    ///
    /// 自动检测平台格式并解析。
    pub fn parse_file(path: &Path, platform: Platform) -> Result<Session> {
        let file_hash = Self::compute_file_hash(path)?;
        Self::parse_file_with_hash(path, platform, file_hash)
    }

    /// 使用已计算的文件哈希解析 session 文件。
    ///
    /// 索引流程会先读取文件计算增量哈希；该入口复用同一个哈希，
    /// 避免解析结束后再次全量读取文件。
    pub fn parse_file_with_hash(
        path: &Path,
        platform: Platform,
        file_hash: String,
    ) -> Result<Session> {
        match platform {
            Platform::Claude => Self::parse_claude_with_hash(path, file_hash),
            Platform::Codex => Self::parse_codex_with_hash(path, file_hash),
            Platform::Gemini => Self::parse_gemini_with_hash(path, file_hash),
            Platform::Qwen => Self::parse_qwen_with_hash(path, file_hash),
            Platform::Droid => Self::parse_generic_with_hash(path, platform, file_hash),
            Platform::Grok => Err(CcrError::PlatformNotSupported(
                "Grok session parsing".into(),
            )),
        }
    }

    /// 解析 Claude session 文件
    ///
    /// Claude session 文件格式: JSONL，每行一个事件
    pub fn parse_claude(path: &Path) -> Result<Session> {
        let file_hash = Self::compute_file_hash(path)?;
        Self::parse_claude_with_hash(path, file_hash)
    }

    fn parse_claude_with_hash(path: &Path, file_hash: String) -> Result<Session> {
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
        let file_hash = Self::compute_file_hash(path)?;
        Self::parse_codex_with_hash(path, file_hash)
    }

    fn parse_codex_with_hash(path: &Path, file_hash: String) -> Result<Session> {
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
        let file_hash = Self::compute_file_hash(path)?;
        Self::parse_gemini_with_hash(path, file_hash)
    }

    fn parse_gemini_with_hash(path: &Path, file_hash: String) -> Result<Session> {
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

    /// 解析通用格式（用于 Droid 等）
    #[allow(dead_code)]
    fn parse_generic(path: &Path, platform: Platform) -> Result<Session> {
        let file_hash = Self::compute_file_hash(path)?;
        Self::parse_generic_with_hash(path, platform, file_hash)
    }

    fn parse_generic_with_hash(
        path: &Path,
        platform: Platform,
        file_hash: String,
    ) -> Result<Session> {
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

    /// 解析 Qwen session 文件
    #[allow(dead_code)]
    fn parse_qwen(path: &Path) -> Result<Session> {
        let file_hash = Self::compute_file_hash(path)?;
        Self::parse_qwen_with_hash(path, file_hash)
    }

    fn parse_qwen_with_hash(path: &Path, file_hash: String) -> Result<Session> {
        let events = Self::read_jsonl(path).unwrap_or_else(|e| {
            debug!(
                "Qwen session 文件解析失败，使用空事件列表: {} - {}",
                path.display(),
                e
            );
            Vec::new()
        });

        let session_id = Self::extract_session_id(&events)
            .or_else(|| Self::extract_id_from_path(path))
            .unwrap_or_else(|| {
                let id = uuid::Uuid::new_v4().to_string();
                debug!("无法从 Qwen 会话中提取 session ID，生成新 ID: {}", id);
                id
            });

        let cwd = Self::extract_cwd(&events)
            .map(PathBuf::from)
            .or_else(|| qwen_project_dir_name_from_chat_path(path).map(PathBuf::from))
            .unwrap_or_else(|| {
                let fallback = path
                    .parent()
                    .map(|parent| parent.to_path_buf())
                    .unwrap_or_default();
                if fallback.as_os_str().is_empty() {
                    debug!("无法提取 Qwen 工作目录，使用空路径: {}", path.display());
                }
                fallback
            });

        let title = Self::extract_title(&events);
        let (created_at, updated_at) = Self::extract_timestamps(&events, path)?;
        let (user_count, assistant_count, tool_count) = Self::count_messages(&events);

        Ok(Session {
            id: session_id,
            platform: Platform::Qwen,
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
        if !dir.exists() {
            debug!("目录不存在: {}", dir.display());
            return Ok(Vec::new());
        }

        let files = Self::scan_directory_recursive(dir, platform)?;

        debug!(
            "在 {} 中找到 {} 个 session 文件",
            dir.display(),
            files.len()
        );

        Ok(files)
    }

    fn scan_directory_recursive(dir: &Path, platform: Platform) -> Result<Vec<PathBuf>> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| CcrError::ConfigError(format!("无法读取目录 {}: {}", dir.display(), e)))?;

        let entry_paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();

        let files = entry_paths
            .par_iter()
            .map(|path| {
                if path.is_dir() {
                    match Self::scan_directory_recursive(path, platform) {
                        Ok(files) => files,
                        Err(e) => {
                            warn!("扫描子目录失败 {}: {}", path.display(), e);
                            Vec::new()
                        }
                    }
                } else if Self::is_session_file(path, &platform) {
                    vec![path.clone()]
                } else {
                    Vec::new()
                }
            })
            .reduce(Vec::new, |mut acc, mut files| {
                acc.append(&mut files);
                acc
            });

        Ok(files)
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
            Platform::Qwen => is_qwen_chat_file(path),
            Platform::Droid => extension == Some("jsonl"),
            Platform::Grok => false,
        }
    }

    /// 获取平台的默认 session 目录
    pub fn get_platform_session_dir(platform: &Platform) -> Option<PathBuf> {
        let path = match platform {
            Platform::Claude => dirs::home_dir()?.join(".claude").join("projects"),
            Platform::Codex => dirs::home_dir()?.join(".codex").join("sessions"),
            Platform::Gemini => dirs::home_dir()?.join(".gemini").join("tmp"),
            Platform::Qwen => qwen_projects_dir()?,
            Platform::Droid => dirs::home_dir()?.join(".factory").join("sessions"),
            Platform::Grok => return None,
        };

        if path.exists() { Some(path) } else { None }
    }

    /// 批量解析多个文件
    #[allow(dead_code)]
    pub fn parse_files(paths: &[PathBuf], platform: Platform) -> (Vec<Session>, IndexStats) {
        let start = std::time::Instant::now();

        let (sessions, mut stats) = paths
            .par_iter()
            .fold(
                || (Vec::new(), IndexStats::default()),
                |(mut sessions, mut stats), path| {
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

                    (sessions, stats)
                },
            )
            .reduce(
                || (Vec::new(), IndexStats::default()),
                |(mut sessions, mut stats), (mut other_sessions, other_stats)| {
                    sessions.append(&mut other_sessions);
                    stats.merge(&other_stats);
                    (sessions, stats)
                },
            );

        stats.duration_ms = start.elapsed().as_millis() as u64;

        (sessions, stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::Instant;
    use tempfile::tempdir;

    fn create_test_jsonl(content: &str) -> PathBuf {
        let dir = tempdir().expect("Failed to create temp directory for test");
        let file_path = dir.path().join("test.jsonl");
        let mut file = File::create(&file_path).expect("Failed to create test JSONL file");
        write!(file, "{}", content).expect("Failed to write test JSONL content");
        std::mem::forget(dir); // 保持目录存活
        file_path
    }

    fn session_content(session_id: &str) -> String {
        format!(
            r#"{{"type": "init", "session_id": "{session_id}", "cwd": "/tmp/test"}}
{{"type": "user", "role": "user", "message": "Hello"}}
{{"type": "assistant", "role": "assistant", "message": "Hi"}}
"#
        )
    }

    fn write_session_file(dir: &Path, name: &str, session_id: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, session_content(session_id)).expect("Failed to write session file");
        path
    }

    #[test]
    fn test_parse_simple_session() {
        let content = r#"{"type": "init", "session_id": "test-123", "cwd": "/tmp/test"}
{"type": "user", "role": "user", "message": "Hello, world!"}
{"type": "assistant", "role": "assistant", "message": "Hi there!"}
"#;

        let path = create_test_jsonl(content);
        let session = SessionParser::parse_claude(&path).expect("Failed to parse test session");

        assert_eq!(session.id, "test-123");
        assert_eq!(session.platform, Platform::Claude);
        assert!(session.message_count >= 2);
    }

    #[test]
    fn test_parse_qwen_prefers_record_cwd() {
        let content = r#"{"type":"session_meta","session_id":"qwen-1","cwd":"D:\\Documents\\Code\\Github\\ccr"}
{"type":"user","role":"user","message":"hello"}
{"type":"assistant","role":"assistant","message":"world"}
"#;

        let path = create_test_jsonl(content);
        let session = SessionParser::parse_qwen(&path).expect("Failed to parse Qwen session");

        assert_eq!(session.id, "qwen-1");
        assert_eq!(session.cwd, PathBuf::from(r"D:\Documents\Code\Github\ccr"));
        assert_eq!(session.platform, Platform::Qwen);
    }

    #[test]
    fn test_parse_qwen_falls_back_to_project_dir_name() {
        let dir = tempdir().expect("Failed to create temp directory for Qwen test");
        let chats_dir = dir
            .path()
            .join(".qwen")
            .join("projects")
            .join("workspace___repo")
            .join("chats");
        std::fs::create_dir_all(&chats_dir).expect("Failed to create Qwen chats dir");
        let path = chats_dir.join("session-1.jsonl");
        std::fs::write(
            &path,
            r#"{"type":"user","role":"user","message":"hello"}
{"type":"assistant","role":"assistant","message":"world"}
"#,
        )
        .expect("Failed to write Qwen session");

        let session = SessionParser::parse_qwen(&path).expect("Failed to parse Qwen session");

        assert_eq!(session.id, "session-1");
        assert_eq!(session.cwd, PathBuf::from("workspace___repo"));
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
        assert!(SessionParser::is_session_file(
            Path::new("/tmp/.qwen/projects/workspace___repo/chats/session-1.jsonl"),
            &Platform::Qwen
        ));
        assert!(!SessionParser::is_session_file(
            Path::new("/tmp/.qwen/projects/workspace___repo/session-1.jsonl"),
            &Platform::Qwen
        ));
    }

    #[test]
    fn test_parse_files_parallel_counts() {
        let dir = tempdir().expect("Failed to create temp directory for test");
        let mut paths = Vec::new();

        for index in 0..3 {
            let filename = format!("session-{}.jsonl", index);
            let session_id = format!("session-{}", index);
            paths.push(write_session_file(dir.path(), &filename, &session_id));
        }

        let (sessions, stats) = SessionParser::parse_files(&paths, Platform::Claude);

        assert_eq!(sessions.len(), 3);
        assert_eq!(stats.files_scanned, 3);
        assert_eq!(stats.errors, 0);
    }

    #[test]
    fn test_scan_directory_recursive_counts() {
        let dir = tempdir().expect("Failed to create temp directory for test");
        let nested = dir.path().join("nested");
        std::fs::create_dir_all(&nested).expect("Failed to create nested dir");

        write_session_file(dir.path(), "session-root.jsonl", "session-root");
        write_session_file(&nested, "session-nested.jsonl", "session-nested");
        std::fs::write(dir.path().join("note.txt"), "not a session")
            .expect("Failed to write noise file");

        let files = SessionParser::scan_directory(dir.path(), Platform::Claude)
            .expect("Failed to scan directory");

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_parse_file_with_hash_dispatches_every_platform() {
        let dir = tempdir().expect("Failed to create temp directory for dispatch test");
        let content = r#"{"type":"init","session_id":"record-id","cwd":"/tmp/project","timestamp":"2026-07-26T10:00:00Z"}
{"type":"user","role":"user","message":"Hello from dispatch","timestamp":"2026-07-26T10:00:01Z"}
{"type":"assistant","role":"assistant","message":"Ready","timestamp":"2026-07-26T10:00:02Z"}
"#;
        let cases = [
            (Platform::Claude, "claude", "record-id"),
            (Platform::Codex, "codex", "record-id"),
            (Platform::Gemini, "gemini", "gemini"),
            (Platform::Qwen, "qwen", "record-id"),
            (Platform::Droid, "droid", "droid"),
        ];

        for (platform, stem, expected_id) in cases {
            let path = dir.path().join(format!("{stem}.jsonl"));
            std::fs::write(&path, content).expect("Failed to write dispatch fixture");
            let expected_hash = format!("provided-{stem}");

            let session =
                SessionParser::parse_file_with_hash(&path, platform, expected_hash.clone())
                    .expect("Failed to parse dispatched session");

            assert_eq!(session.platform, platform);
            assert_eq!(session.id, expected_id);
            assert_eq!(session.file_hash, expected_hash);
            assert_eq!(session.message_count, 2);
            assert_eq!(session.title.as_deref(), Some("Hello from dispatch"));
        }
    }

    #[test]
    fn test_missing_inputs_report_empty_scan_and_parse_errors() {
        let dir = tempdir().expect("Failed to create temp directory for error test");
        let missing = dir.path().join("missing.jsonl");

        assert!(
            SessionParser::scan_directory(&missing, Platform::Codex)
                .expect("Missing scan root should be empty")
                .is_empty()
        );
        assert!(matches!(
            SessionParser::parse_file(&missing, Platform::Claude),
            Err(CcrError::ConfigError(_))
        ));

        let (sessions, stats) = SessionParser::parse_files(&[missing], Platform::Codex);
        assert!(sessions.is_empty());
        assert_eq!(stats.files_scanned, 1);
        assert_eq!(stats.sessions_added, 0);
        assert_eq!(stats.errors, 1);
    }

    #[test]
    #[ignore]
    fn benchmark_parse_files_parallel() {
        let dir = tempdir().expect("Failed to create temp directory for benchmark");
        let mut paths = Vec::new();
        let file_count = 200;

        for index in 0..file_count {
            let filename = format!("session-{}.jsonl", index);
            let session_id = format!("session-{}", index);
            paths.push(write_session_file(dir.path(), &filename, &session_id));
        }

        let start = Instant::now();
        let (sessions, stats) = SessionParser::parse_files(&paths, Platform::Claude);
        let elapsed = start.elapsed();

        assert_eq!(sessions.len(), file_count);
        eprintln!(
            "parse_files: files={}, duration={:?}, stats={{scanned={}, added={}, errors={}}}",
            file_count, elapsed, stats.files_scanned, stats.sessions_added, stats.errors
        );
    }

    #[test]
    #[ignore]
    fn benchmark_scan_directory_parallel() {
        let dir = tempdir().expect("Failed to create temp directory for benchmark");
        let nested = dir.path().join("nested");
        std::fs::create_dir_all(&nested).expect("Failed to create nested dir");
        let file_count = 300;

        for index in 0..file_count {
            let filename = format!("session-{}.jsonl", index);
            let session_id = format!("session-{}", index);
            if index % 2 == 0 {
                write_session_file(dir.path(), &filename, &session_id);
            } else {
                write_session_file(&nested, &filename, &session_id);
            }
        }

        let start = Instant::now();
        let files = SessionParser::scan_directory(dir.path(), Platform::Claude)
            .expect("Failed to scan directory");
        let elapsed = start.elapsed();

        assert_eq!(files.len(), file_count);
        eprintln!(
            "scan_directory: files={}, duration={:?}",
            file_count, elapsed
        );
    }
}
