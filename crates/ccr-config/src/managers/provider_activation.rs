// 🎯 Provider 激活时间线记录器
// 📁 在 profile 切换/清除时向 `<CCR_ROOT>/analytics/provider_activation.jsonl`
//    追加事件，供 llmusage 按 (source, event_at ∈ 激活区间) 把用量归因到 provider。
//
// 设计要点:
// - ✍️ append-only JSONL，绝不改写既有行（非破坏性，无需备份）
// - 🔒 复用 LockManager 串行化并发追加
// - 🙈 只记录 base_url 的 host（并丢弃 userinfo），绝不写入 auth_token / api key
// - 🛟 尽力而为: 任何失败仅告警，绝不让 profile 切换回滚
// - 🧹 按平台对上一条事件去重，避免 drift 修复等路径刷屏

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ccr_core::core::LockManager;
use serde::{Deserialize, Serialize};

use crate::models::ProfileConfig;
use crate::platforms::base::load_profiles_from_toml;

/// 激活日志锁资源名（独立于注册表锁，避免嵌套）
const ACTIVATION_LOCK_RESOURCE: &str = "provider_activation_log";
/// 追加日志的锁等待上限
const ACTIVATION_LOCK_TIMEOUT: Duration = Duration::from_secs(5);

/// 🏷️ 事件类型: 激活某 provider / 清除当前 provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivationKind {
    /// 切换到某个 profile（provider 变为激活）
    Activate,
    /// 当前 profile 被清空（该平台进入未归因状态）
    Clear,
}

/// 📝 一条 provider 激活/清除事件
///
/// 即 llmusage `--provider-map` 的一行。所有字段始终序列化，保证跨工具解析稳定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationEvent {
    /// 平台（= llmusage 的 source）: claude / codex / …
    pub platform: String,
    /// 触发切换的 profile 名（clear 时为 None）
    pub profile: Option<String>,
    /// relay provider 名（归因主键；未知/清除时为 None → 未归因）
    pub provider: Option<String>,
    /// provider 类型（official_relay / third_party_model / …）
    pub provider_type: Option<String>,
    /// base_url 的 host（仅主机名/端口，不含 scheme/path/userinfo）
    pub base_url_host: Option<String>,
    /// 账号标识
    pub account: Option<String>,
    /// 激活时刻（UTC RFC3339，与 llmusage event_at 对齐）
    pub activated_at: String,
    /// 事件类型
    pub event: ActivationKind,
}

/// 记录一次 provider 激活（尽力而为，失败仅告警，绝不影响切换）
pub fn record_activation(root: &Path, platform: &str, profile_name: &str) {
    let event = build_activation_event(root, platform, profile_name);
    if let Err(err) = append_deduped(root, &event) {
        tracing::warn!(
            platform,
            profile = profile_name,
            error = %err,
            "记录 provider 激活事件失败（已忽略，不影响切换）"
        );
    }
}

/// 记录一次 provider 清除（当前 profile 被清空）
pub fn record_clear(root: &Path, platform: &str) {
    let event = ActivationEvent {
        platform: platform.to_string(),
        profile: None,
        provider: None,
        provider_type: None,
        base_url_host: None,
        account: None,
        activated_at: now_rfc3339(),
        event: ActivationKind::Clear,
    };
    if let Err(err) = append_deduped(root, &event) {
        tracing::warn!(
            platform,
            error = %err,
            "记录 provider 清除事件失败（已忽略，不影响切换）"
        );
    }
}

/// 🏠 解析 CCR 根目录（优先 `CCR_ROOT`，否则 `~/.ccr`）
///
/// 与 `PlatformConfigManager::with_default` 的根目录解析保持一致。
pub fn default_ccr_root() -> Option<PathBuf> {
    if let Ok(custom_root) = std::env::var("CCR_ROOT") {
        return Some(PathBuf::from(custom_root));
    }
    dirs::home_dir().map(|home| home.join(".ccr"))
}

/// 📁 激活日志路径: `<root>/analytics/provider_activation.jsonl`
pub fn activation_log_path(root: &Path) -> PathBuf {
    root.join("analytics").join("provider_activation.jsonl")
}

/// 从平台 profiles.toml 解析出激活事件（provider 等字段尽力而为）
fn build_activation_event(root: &Path, platform: &str, profile_name: &str) -> ActivationEvent {
    let profile = resolve_profile(root, platform, profile_name);
    let (provider, provider_type, base_url_host, account) = match profile {
        Some(p) => (
            p.provider,
            p.provider_type,
            p.base_url.as_deref().and_then(host_only),
            p.account,
        ),
        None => (None, None, None, None),
    };

    ActivationEvent {
        platform: platform.to_string(),
        profile: Some(profile_name.to_string()),
        provider,
        provider_type,
        base_url_host,
        account,
        activated_at: now_rfc3339(),
        event: ActivationKind::Activate,
    }
}

/// 加载平台 profiles.toml 并取出指定 profile
fn resolve_profile(root: &Path, platform: &str, profile_name: &str) -> Option<ProfileConfig> {
    let profiles_path = root.join("platforms").join(platform).join("profiles.toml");
    let profiles = load_profiles_from_toml(&profiles_path).ok()?;
    profiles.get(profile_name).cloned()
}

/// 去重后追加：若与该平台最近一条事件等价则跳过
fn append_deduped(root: &Path, event: &ActivationEvent) -> Result<(), String> {
    let path = activation_log_path(root);
    if let Some(prev) = last_event_for_platform(&path, &event.platform)
        && is_redundant(&prev, event)
    {
        return Ok(());
    }
    append_line(&path, event)
}

/// 判断两条事件对同一平台是否等价（类型 + provider + profile 都相同）
fn is_redundant(prev: &ActivationEvent, next: &ActivationEvent) -> bool {
    prev.event == next.event && prev.provider == next.provider && prev.profile == next.profile
}

/// 读取该平台在日志中的最近一条事件（尽力而为，解析失败的行跳过）
fn last_event_for_platform(path: &Path, platform: &str) -> Option<ActivationEvent> {
    let content = std::fs::read_to_string(path).ok()?;
    content
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<ActivationEvent>(line).ok())
        .find(|ev| ev.platform == platform)
}

/// 加锁后原子追加一行 JSON
fn append_line(path: &Path, event: &ActivationEvent) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("创建 analytics 目录失败: {e}"))?;
    }

    let mut line = serde_json::to_string(event).map_err(|e| format!("序列化激活事件失败: {e}"))?;
    line.push('\n');

    let lock_manager =
        LockManager::with_default_path().map_err(|e| format!("获取锁管理器失败: {e}"))?;
    let _lock = lock_manager
        .lock_resource(ACTIVATION_LOCK_RESOURCE, ACTIVATION_LOCK_TIMEOUT)
        .map_err(|e| format!("获取激活日志锁失败: {e}"))?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("打开激活日志失败: {e}"))?;
    file.write_all(line.as_bytes())
        .map_err(|e| format!("写入激活日志失败: {e}"))?;
    Ok(())
}

/// 从 base_url 提取 host（去掉 scheme / path / userinfo），无 host 时返回 None
fn host_only(base_url: &str) -> Option<String> {
    let without_scheme = base_url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(base_url);
    let authority = without_scheme.split(['/', '?', '#']).next()?;
    // 丢弃可能的 userinfo（user:pass@host），只保留 host[:port]
    let host_port = authority.rsplit('@').next()?;
    if host_port.is_empty() {
        None
    } else {
        Some(host_port.to_string())
    }
}

/// 当前时刻（UTC RFC3339），与注册表 last_used 及 llmusage event_at 对齐
fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestCcrEnv;
    use std::fs;

    /// 在临时 CCR_ROOT 下写入一个含密钥的 claude profiles.toml
    fn seed_claude_profile(root: &Path, name: &str, provider: &str, base_url: &str) {
        let dir = root.join("platforms").join("claude");
        fs::create_dir_all(&dir).unwrap();
        let content = format!(
            "[{name}]\n\
             description = \"demo\"\n\
             base_url = \"{base_url}\"\n\
             auth_token = \"sk-super-secret-token\"\n\
             provider = \"{provider}\"\n\
             provider_type = \"official_relay\"\n\
             account = \"acc-1\"\n"
        );
        fs::write(dir.join("profiles.toml"), content).unwrap();
    }

    fn read_log(root: &Path) -> String {
        fs::read_to_string(activation_log_path(root)).unwrap_or_default()
    }

    #[test]
    fn host_only_strips_scheme_path_and_userinfo() {
        assert_eq!(
            host_only("https://api.methink.cc/v1"),
            Some("api.methink.cc".into())
        );
        assert_eq!(
            host_only("https://api.methink.cc:8443/v1"),
            Some("api.methink.cc:8443".into())
        );
        assert_eq!(
            host_only("https://user:pass@host.example/v1"),
            Some("host.example".into())
        );
        assert_eq!(host_only("api.methink.cc"), Some("api.methink.cc".into()));
        assert_eq!(host_only(""), None);
    }

    #[test]
    fn activation_records_provider_and_never_leaks_secret() {
        let env = TestCcrEnv::new();
        let root = env.root().to_path_buf();
        seed_claude_profile(&root, "anyrouter3", "anyrouter", "https://anyrouter.top/v1");

        record_activation(&root, "claude", "anyrouter3");

        let log = read_log(&root);
        assert!(log.contains("\"provider\":\"anyrouter\""), "log: {log}");
        assert!(
            log.contains("\"base_url_host\":\"anyrouter.top\""),
            "log: {log}"
        );
        assert!(log.contains("\"event\":\"activate\""), "log: {log}");
        // 🙈 关键: 绝不能出现 auth_token
        assert!(
            !log.contains("sk-super-secret-token"),
            "secret leaked: {log}"
        );
    }

    #[test]
    fn identical_consecutive_activation_is_deduped() {
        let env = TestCcrEnv::new();
        let root = env.root().to_path_buf();
        seed_claude_profile(&root, "glm", "glm", "https://open.bigmodel.cn/api");

        record_activation(&root, "claude", "glm");
        record_activation(&root, "claude", "glm");

        let lines = read_log(&root).lines().count();
        assert_eq!(lines, 1, "identical activation should dedup to one line");
    }

    #[test]
    fn switching_provider_appends_new_line() {
        let env = TestCcrEnv::new();
        let root = env.root().to_path_buf();
        seed_claude_profile(&root, "glm", "glm", "https://open.bigmodel.cn/api");
        record_activation(&root, "claude", "glm");

        // 覆盖同名文件，加入第二个 provider
        let dir = root.join("platforms").join("claude");
        let content = "[glm]\nprovider = \"glm\"\n\n[any]\nprovider = \"anyrouter\"\n";
        fs::write(dir.join("profiles.toml"), content).unwrap();
        record_activation(&root, "claude", "any");

        let lines = read_log(&root).lines().count();
        assert_eq!(lines, 2, "distinct provider should append");
    }

    #[test]
    fn clear_writes_clear_event() {
        let env = TestCcrEnv::new();
        let root = env.root().to_path_buf();
        seed_claude_profile(&root, "glm", "glm", "https://open.bigmodel.cn/api");
        record_activation(&root, "claude", "glm");
        record_clear(&root, "claude");

        let log = read_log(&root);
        assert!(log.contains("\"event\":\"clear\""), "log: {log}");
        assert_eq!(log.lines().count(), 2);
    }

    #[test]
    fn missing_profiles_still_logs_activate_without_provider() {
        let env = TestCcrEnv::new();
        let root = env.root().to_path_buf();
        // 不创建 profiles.toml
        record_activation(&root, "codex", "ghost");

        let log = read_log(&root);
        assert!(log.contains("\"platform\":\"codex\""), "log: {log}");
        assert!(log.contains("\"provider\":null"), "log: {log}");
        assert!(log.contains("\"event\":\"activate\""), "log: {log}");
    }
}
