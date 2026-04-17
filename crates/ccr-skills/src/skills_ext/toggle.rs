//! Enable / Disable skill toggle via `~/.claude/settings.json` `permissions.deny[]`。
//!
//! 对应 skill-hub `server/routes/manage.ts` 的 `toggle` 路由。关键行为：
//! - 仅操作 `Skill(<name>)` 形式的 deny 条目；Bash/MCP/Read 等规则**绝不触碰**
//! - `set_enabled(true)` 删除该条；`set_enabled(false)` 追加（幂等：已存在时不重复加）
//! - Skill 名称必须匹配 `^[A-Za-z0-9_-]+$`，防 `Skill(foo) bar")` 注入
//! - 并发安全：进程内 `SETTINGS_MUTEX` 串行化读-改-写；文件本身原子写 (temp+rename)
//! - 磁盘不存在 / 空文件 / `permissions` 缺失 都自动初始化，不抛错

use serde_json::{Value, json};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// 错误类型
// ============================================================================

#[derive(Debug)]
pub enum ToggleError {
    Io(io::Error),
    Json(serde_json::Error),
    NoHomeDir,
    InvalidSkillName(String),
    InvalidSettingsShape(String),
}

impl std::fmt::Display for ToggleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::NoHomeDir => write!(f, "Cannot locate home directory"),
            Self::InvalidSkillName(name) => write!(
                f,
                "Invalid skill name '{name}': must match ^[A-Za-z0-9_-]+$"
            ),
            Self::InvalidSettingsShape(reason) => {
                write!(f, "Invalid settings.json shape: {reason}")
            }
        }
    }
}

impl std::error::Error for ToggleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ToggleError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for ToggleError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

pub type ToggleResult<T> = Result<T, ToggleError>;

// ============================================================================
// 全局串行化锁
// ============================================================================

/// 进程内串行化 settings.json 的 read-modify-write，防止同进程并发 toggle 丢更新。
/// **不保护外部进程**（如 Claude Code 自身）写入。
static SETTINGS_MUTEX: Mutex<()> = Mutex::new(());

/// 临时文件命名用的单调计数器。
static WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

// ============================================================================
// 公共路径
// ============================================================================

/// 默认 `~/.claude/settings.json` 路径。
pub fn default_settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
}

// ============================================================================
// ToggleStore
// ============================================================================

pub struct ToggleStore {
    settings_path: PathBuf,
}

impl ToggleStore {
    /// 默认：读写 `~/.claude/settings.json`。
    pub fn open() -> ToggleResult<Self> {
        let path = default_settings_path().ok_or(ToggleError::NoHomeDir)?;
        Ok(Self::with_path(path))
    }

    /// 测试友好：指定 settings.json 完整路径。
    pub fn with_path(settings_path: PathBuf) -> Self {
        Self { settings_path }
    }

    /// 查询 skill 是否启用 (= deny 列表里没有 `Skill(<name>)`)。
    pub fn is_enabled(&self, skill_name: &str) -> ToggleResult<bool> {
        validate_skill_name(skill_name)?;
        let _guard = lock_mutex();
        let settings = self.load()?;
        let deny = self.deny_slice(&settings)?;
        let rule = format!("Skill({skill_name})");
        Ok(!deny.iter().any(|v| v.as_str() == Some(&rule)))
    }

    /// 设置 skill 启用/禁用状态。幂等。
    /// P1-4 强化：在进程内 mutex 保护下做 read-modify-write；
    /// 若写前内容 hash 与读时不一致（如 Claude Code 外部进程插入了 deny 条目），
    /// 则重新 load → 再算目标状态 → 重写，避免覆盖丢失外部改动。
    pub fn set_enabled(&self, skill_name: &str, enabled: bool) -> ToggleResult<()> {
        validate_skill_name(skill_name)?;
        let _guard = lock_mutex();

        // 最多重试 3 次以防被外部进程频繁抢写
        for _attempt in 0..3 {
            let (mut settings, raw_hash) = self.load_with_hash()?;
            let deny = self.deny_slice_mut(&mut settings)?;
            let rule = format!("Skill({skill_name})");
            let idx = deny.iter().position(|v| v.as_str() == Some(&rule));

            match (enabled, idx) {
                (true, Some(i)) => {
                    deny.remove(i);
                }
                (false, None) => {
                    deny.push(json!(rule));
                }
                _ => return Ok(()), // 幂等 no-op
            }

            // 写前再读一次磁盘 hash；若未变则落盘
            let (_, current_hash) = self.load_with_hash()?;
            if current_hash == raw_hash {
                return self.save(&settings);
            }
            // 外部进程改动了文件 → 重试 load
        }
        Err(ToggleError::InvalidSettingsShape(
            "settings.json changed concurrently 3x, giving up to avoid overwrite".into(),
        ))
    }

    /// 列出所有当前禁用的 skill 名（剥离 `Skill(...)` 外壳）。
    /// Bash/MCP 等其他 deny 规则自动忽略。
    pub fn list_disabled(&self) -> ToggleResult<Vec<String>> {
        let _guard = lock_mutex();
        let settings = self.load()?;
        let deny = self.deny_slice(&settings)?;
        let mut out = Vec::new();
        for v in deny.iter() {
            let Some(s) = v.as_str() else { continue };
            if let Some(name) = extract_skill_rule(s) {
                out.push(name.to_string());
            }
        }
        Ok(out)
    }

    // ------------------------------------------------------------------
    // 内部：加载 / 保存 / 访问 deny 数组
    // ------------------------------------------------------------------

    fn load(&self) -> ToggleResult<Value> {
        match fs::read_to_string(&self.settings_path) {
            Ok(s) if s.trim().is_empty() => Ok(json!({})),
            Ok(s) => Ok(serde_json::from_str(&s)?),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(json!({})),
            Err(e) => Err(ToggleError::Io(e)),
        }
    }

    /// P1-4：读 settings 连同原始字节的 blake3 hash，用于写前冲突检测。
    /// 文件不存在或空文件时 hash 为空字符串。
    fn load_with_hash(&self) -> ToggleResult<(Value, String)> {
        match fs::read(&self.settings_path) {
            Ok(bytes) => {
                let hash = blake3::hash(&bytes).to_hex().as_str()[..16].to_string();
                let text = std::str::from_utf8(&bytes)
                    .map_err(|e| ToggleError::Io(io::Error::new(io::ErrorKind::InvalidData, e)))?;
                let value = if text.trim().is_empty() {
                    json!({})
                } else {
                    serde_json::from_str(text)?
                };
                Ok((value, hash))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok((json!({}), String::new())),
            Err(e) => Err(ToggleError::Io(e)),
        }
    }

    fn save(&self, settings: &Value) -> ToggleResult<()> {
        let json_bytes = serde_json::to_vec_pretty(settings)?;
        atomic_write(&self.settings_path, &json_bytes)
    }

    /// 只读访问 deny 数组；若结构不存在则返回空切片。
    fn deny_slice<'a>(&self, settings: &'a Value) -> ToggleResult<&'a [Value]> {
        let Some(root) = settings.as_object() else {
            return Err(ToggleError::InvalidSettingsShape(
                "root is not a JSON object".into(),
            ));
        };
        let Some(perms) = root.get("permissions") else {
            return Ok(&[]);
        };
        let Some(perms_obj) = perms.as_object() else {
            return Err(ToggleError::InvalidSettingsShape(
                "permissions must be an object".into(),
            ));
        };
        let Some(deny) = perms_obj.get("deny") else {
            return Ok(&[]);
        };
        deny.as_array().map(|a| a.as_slice()).ok_or_else(|| {
            ToggleError::InvalidSettingsShape("permissions.deny must be an array".into())
        })
    }

    /// 可变访问 deny 数组，若结构缺失则按需创建。
    fn deny_slice_mut<'a>(&self, settings: &'a mut Value) -> ToggleResult<&'a mut Vec<Value>> {
        let root = settings
            .as_object_mut()
            .ok_or_else(|| ToggleError::InvalidSettingsShape("root is not a JSON object".into()))?;
        let perms = root
            .entry("permissions".to_string())
            .or_insert_with(|| json!({}));
        let perms_obj = perms.as_object_mut().ok_or_else(|| {
            ToggleError::InvalidSettingsShape("permissions must be an object".into())
        })?;
        let deny = perms_obj
            .entry("deny".to_string())
            .or_insert_with(|| json!([]));
        deny.as_array_mut().ok_or_else(|| {
            ToggleError::InvalidSettingsShape("permissions.deny must be an array".into())
        })
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

fn lock_mutex() -> std::sync::MutexGuard<'static, ()> {
    SETTINGS_MUTEX
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn validate_skill_name(name: &str) -> ToggleResult<()> {
    if name.is_empty() {
        return Err(ToggleError::InvalidSkillName(name.to_string()));
    }
    let ok = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if ok {
        Ok(())
    } else {
        Err(ToggleError::InvalidSkillName(name.to_string()))
    }
}

/// 从 `Skill(foo)` 字符串提取 `foo`。不匹配返回 `None`。
fn extract_skill_rule(raw: &str) -> Option<&str> {
    let inner = raw.strip_prefix("Skill(")?.strip_suffix(')')?;
    if inner.is_empty() {
        return None;
    }
    let valid = inner
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if valid { Some(inner) } else { None }
}

fn atomic_write(final_path: &Path, data: &[u8]) -> ToggleResult<()> {
    let parent = final_path.parent().ok_or_else(|| {
        ToggleError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "settings path has no parent",
        ))
    })?;
    fs::create_dir_all(parent)?;

    let pid = std::process::id();
    let seq = WRITE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_path = parent.join(format!(".ccr-settings-tmp-{pid:x}-{seq:x}"));

    fs::write(&tmp_path, data)?;
    if let Err(e) = fs::rename(&tmp_path, final_path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(ToggleError::Io(e));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_skill_name_accepts_letters_digits_underscore_dash() {
        assert!(validate_skill_name("foo").is_ok());
        assert!(validate_skill_name("foo-bar_42").is_ok());
        assert!(validate_skill_name("A-B_3").is_ok());
    }

    #[test]
    fn validate_skill_name_rejects_bad_chars() {
        assert!(validate_skill_name("").is_err());
        assert!(validate_skill_name("has space").is_err());
        assert!(validate_skill_name("has(paren)").is_err());
        assert!(validate_skill_name("中文").is_err());
        assert!(validate_skill_name("a/b").is_err());
    }

    #[test]
    fn extract_skill_rule_parses_canonical_form() {
        assert_eq!(extract_skill_rule("Skill(foo)"), Some("foo"));
        assert_eq!(extract_skill_rule("Skill(foo-bar_1)"), Some("foo-bar_1"));
        assert_eq!(extract_skill_rule("Skill()"), None);
        assert_eq!(extract_skill_rule("Skill(bad name)"), None);
        assert_eq!(extract_skill_rule("Bash(rm)"), None);
        assert_eq!(extract_skill_rule("Skill(inject)extra"), None);
    }
}
