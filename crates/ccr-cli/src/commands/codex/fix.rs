//! 🧹 codex fix 命令实现
//!
//! 清理残留 Codex `app-server` 进程，并运行 `codex doctor` 展示实际加载的配置/认证来源。
//! 等价于用户手写的 `codexfix` bash 脚本：修复 SSH / Desktop / VS Code Remote 断开后
//! app-server 仍锁定旧登录态、导致第三方 URL/Key 切换不生效的问题。

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use ccr_codex::{CodexAppServerCleanup, CodexProcessService, TerminationKind};
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use serde_json::Value;

use crate::services::install_detect::which_on_path;

/// `codex doctor` 外部调用（含网络探活）的超时上限。
const DOCTOR_TIMEOUT: Duration = Duration::from_secs(30);

/// codex doctor 报告中优先高亮的字段（键名子串，大小写不敏感）。
const HIGHLIGHT_KEYS: &[&str] = &[
    "codex_home",
    "config",
    "provider",
    "auth",
    "base_url",
    "endpoint",
    "model",
];

pub async fn fix_command(dry_run: bool) -> Result<()> {
    // A. 进程清理（dry-run 时只枚举、不终止）。
    let cleanup = CodexProcessService::new().cleanup(dry_run);
    render_cleanup(&cleanup);

    // B. 校验 codex 存在性；缺失时以退出码 127 结束（对齐脚本 command-not-found 语义）。
    let Some(codex_bin) = which_on_path("codex") else {
        ColorOutput::error("PATH 中找不到 codex 命令，跳过配置诊断");
        ColorOutput::info("请确认 Codex CLI 已安装且在 PATH 中后重试");
        exit_after_flush(127);
    };

    // C. 环境提示（脱敏）。
    render_env_hints();

    // D. codex doctor 诊断。
    let doctor = run_codex_doctor(&codex_bin).await;
    render_doctor(&doctor);

    // E. 复检到 respawn ⇒ 退出码 2（提示关闭远程客户端后重试）。
    if !cleanup.respawned.is_empty() {
        exit_after_flush(2);
    }

    Ok(())
}

/// 刷新 stdout 后以指定退出码结束进程（沿用 `doctor_cmd` 的显式退出码惯例）。
fn exit_after_flush(code: i32) -> ! {
    let _ = io::stdout().flush();
    std::process::exit(code);
}

// ==================== 进程清理渲染 ====================

fn render_cleanup(cleanup: &CodexAppServerCleanup) {
    if cleanup.found.is_empty() {
        ColorOutput::success("未发现残留的 Codex app-server 进程");
        return;
    }

    if cleanup.dry_run {
        ColorOutput::info(&format!(
            "发现 {} 个 app-server 进程（--dry-run，不终止）：",
            cleanup.found.len()
        ));
        for app in &cleanup.found {
            ColorOutput::info(&format!("  PID {} — {}", app.pid, app.cmdline));
        }
        return;
    }

    ColorOutput::info(&format!(
        "发现 {} 个 app-server 进程，正在清理：",
        cleanup.found.len()
    ));
    for app in &cleanup.found {
        ColorOutput::info(&format!("  PID {} — {}", app.pid, app.cmdline));
    }
    for (pid, kind) in &cleanup.terminated {
        ColorOutput::info(&format!("  PID {} → {}", pid, termination_desc(*kind)));
    }

    if cleanup.respawned.is_empty() {
        ColorOutput::success("所有旧 app-server 已清除");
    } else {
        ColorOutput::warning(
            "app-server 已被重新拉起：通常是 Codex Desktop 或 VS Code Remote-SSH 仍保持连接",
        );
        ColorOutput::warning("请关闭相应远程窗口后，再次执行 ccr codex fix");
        for app in &cleanup.respawned {
            ColorOutput::warning(&format!("  PID {} — {}", app.pid, app.cmdline));
        }
    }
}

fn termination_desc(kind: TerminationKind) -> &'static str {
    match kind {
        TerminationKind::Term => "已优雅退出 (SIGTERM)",
        TerminationKind::Kill => "已强制终止",
        TerminationKind::AlreadyGone => "终止前已退出",
    }
}

// ==================== 环境提示（脱敏） ====================

fn render_env_hints() {
    ColorOutput::info("当前进程中可能影响 Codex 的环境变量：");
    print_env_value("CODEX_HOME");
    print_env_value("OPENAI_BASE_URL");
    // API Key 仅显示存在性，绝不回显值。
    match std::env::var("OPENAI_API_KEY") {
        Ok(v) if !v.is_empty() => ColorOutput::info("  OPENAI_API_KEY=<已设置，值已隐藏>"),
        _ => ColorOutput::info("  OPENAI_API_KEY=<未设置>"),
    }
}

fn print_env_value(name: &str) {
    match std::env::var(name) {
        Ok(v) if !v.is_empty() => ColorOutput::info(&format!("  {name}={v}")),
        _ => ColorOutput::info(&format!("  {name}=<未设置>")),
    }
}

// ==================== codex doctor ====================

/// codex doctor 诊断结果（供渲染）。
struct DoctorOutcome {
    /// 完整报告保存路径。
    report_path: Option<PathBuf>,
    /// 从 JSON 报告中提取的关键字段。
    highlights: Vec<(String, String)>,
    /// 降级路径下的原始文本输出。
    raw_text: Option<String>,
    /// 诊断是否失败（超时 / 无法执行）。
    failed: bool,
    /// 附加说明（降级提示或失败原因）。
    note: Option<String>,
}

impl DoctorOutcome {
    fn failed_with(note: &str) -> Self {
        Self {
            report_path: None,
            highlights: Vec::new(),
            raw_text: None,
            failed: true,
            note: Some(note.to_string()),
        }
    }
}

async fn run_codex_doctor(bin: &Path) -> DoctorOutcome {
    // 首选 `codex doctor --json`：以「是否拿到有效 JSON」判断，而非退出码
    // （检查项失败时 doctor 可能返回非 0，但 stdout 仍是有效报告，应照常展示）。
    match capture_doctor(bin, &["doctor", "--json"]).await {
        Ok(stdout) => {
            if let Ok(json) = serde_json::from_slice::<Value>(&stdout) {
                return DoctorOutcome {
                    report_path: save_report(&stdout, "json"),
                    highlights: extract_highlights(&json),
                    raw_text: None,
                    failed: false,
                    note: None,
                };
            }
            // --json 无效（旧版 codex 不支持）→ 回退纯文本。
            fallback_plain_doctor(bin).await
        }
        Err(DoctorError::Timeout) => {
            DoctorOutcome::failed_with("codex doctor 超时（可能卡在网络探活），已跳过诊断")
        }
        Err(DoctorError::Spawn(msg)) => {
            DoctorOutcome::failed_with(&format!("无法执行 codex doctor：{msg}"))
        }
    }
}

async fn fallback_plain_doctor(bin: &Path) -> DoctorOutcome {
    match capture_doctor(bin, &["doctor"]).await {
        Ok(stdout) => {
            let text = String::from_utf8_lossy(&stdout).trim().to_string();
            DoctorOutcome {
                report_path: save_report(&stdout, "txt"),
                highlights: Vec::new(),
                raw_text: (!text.is_empty()).then_some(text),
                failed: false,
                note: Some("当前 codex 版本不支持 doctor --json，已回退文本输出".to_string()),
            }
        }
        Err(_) => DoctorOutcome::failed_with("codex doctor 执行失败"),
    }
}

enum DoctorError {
    Timeout,
    Spawn(String),
}

/// 运行 `codex <args>` 并在超时上限内捕获 stdout。
///
/// 超时时不阻塞本命令；残留的 doctor 子进程会在其自身网络超时后自行退出。
async fn capture_doctor(bin: &Path, args: &[&str]) -> std::result::Result<Vec<u8>, DoctorError> {
    let mut cmd = tokio::process::Command::new(bin);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    match tokio::time::timeout(DOCTOR_TIMEOUT, cmd.output()).await {
        Ok(Ok(output)) => Ok(output.stdout),
        Ok(Err(e)) => Err(DoctorError::Spawn(e.to_string())),
        Err(_) => Err(DoctorError::Timeout),
    }
}

/// 将完整报告写入临时文件，返回其路径（写入失败则返回 None）。
fn save_report(bytes: &[u8], ext: &str) -> Option<PathBuf> {
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let path = std::env::temp_dir().join(format!("codex-doctor-{}-{ts}.{ext}", current_user()));
    std::fs::write(&path, bytes).ok().map(|()| path)
}

fn current_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string())
}

/// 从 doctor JSON 报告中 best-effort 提取关键配置字段。
///
/// 报告结构（schemaVersion 1）：顶层含 `codexVersion` / `overallStatus`，
/// 各检查项在 `checks.<id>.details`（key/value）下携带具体字段，且已由 codex 脱敏
/// （敏感值呈现为 `<redacted>` 或仅存在性）。仅按键名子串匹配感兴趣的标签字段，不触碰 token。
fn extract_highlights(json: &Value) -> Vec<(String, String)> {
    let mut highlights = Vec::new();

    // 顶层定位信息。
    for key in ["codexVersion", "overallStatus"] {
        if let Some(value) = json.get(key) {
            highlights.push((key.to_string(), value_to_display(value)));
        }
    }

    // checks.<id>.details.<key> 中匹配关键标签的字段。
    if let Some(checks) = json.get("checks").and_then(Value::as_object) {
        for check in checks.values() {
            let Some(details) = check.get("details").and_then(Value::as_object) else {
                continue;
            };
            for (key, value) in details {
                let key_lower = key.to_lowercase();
                if HIGHLIGHT_KEYS
                    .iter()
                    .any(|needle| key_lower.contains(needle))
                {
                    highlights.push((key.clone(), value_to_display(value)));
                }
            }
        }
    }

    highlights
}

fn value_to_display(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

fn render_doctor(outcome: &DoctorOutcome) {
    if outcome.failed {
        let note = outcome.note.as_deref().unwrap_or("codex doctor 诊断失败");
        ColorOutput::warning(note);
        return;
    }

    if let Some(note) = &outcome.note {
        ColorOutput::info(note);
    }

    if outcome.highlights.is_empty() {
        if let Some(raw) = &outcome.raw_text {
            ColorOutput::info("codex doctor 输出：");
            println!("{raw}");
        } else {
            ColorOutput::info("codex doctor 已运行（详见完整报告）");
        }
    } else {
        ColorOutput::info("codex doctor 关键配置：");
        for (key, value) in &outcome.highlights {
            ColorOutput::info(&format!("  {key} = {value}"));
        }
    }

    if let Some(path) = &outcome.report_path {
        ColorOutput::info(&format!("完整报告已保存到：{}", path.display()));
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_highlights, value_to_display};
    use serde_json::json;

    #[test]
    fn extracts_labeled_config_fields_from_checks_details() {
        // 对齐 codex doctor --json schemaVersion 1 的真实结构：
        // details 嵌套在 checks.<id> 之下，而非顶层。
        let report = json!({
            "schemaVersion": 1,
            "overallStatus": "fail",
            "codexVersion": "0.144.6",
            "checks": {
                "config.load": {
                    "category": "config",
                    "status": "ok",
                    "details": {
                        "CODEX_HOME": "/home/lyh/.codex",
                        "config.toml": "/home/lyh/.codex/config.toml",
                        "cwd": "/work/repo",
                        "log dir": "/home/lyh/.codex/log"
                    }
                },
                "auth.credentials": {
                    "category": "auth",
                    "status": "ok",
                    "details": {
                        "auth file": "/home/lyh/.codex/auth.json",
                        "stored auth mode": "api_key",
                        "model provider": "thirdparty"
                    }
                }
            }
        });

        let highlights = extract_highlights(&report);

        // 顶层定位信息。
        assert!(
            highlights
                .iter()
                .any(|(k, v)| k == "codexVersion" && v == "0.144.6")
        );
        assert!(
            highlights
                .iter()
                .any(|(k, v)| k == "overallStatus" && v == "fail")
        );
        // checks.*.details 中的关键标签字段命中。
        assert!(
            highlights
                .iter()
                .any(|(k, v)| k == "CODEX_HOME" && v == "/home/lyh/.codex")
        );
        assert!(highlights.iter().any(|(k, _)| k == "config.toml"));
        assert!(highlights.iter().any(|(k, _)| k == "auth file"));
        assert!(highlights.iter().any(|(k, _)| k == "stored auth mode"));
        assert!(highlights.iter().any(|(k, _)| k == "model provider"));
        // 无关字段被忽略。
        assert!(!highlights.iter().any(|(k, _)| k == "cwd"));
        assert!(!highlights.iter().any(|(k, _)| k == "log dir"));
    }

    #[test]
    fn returns_no_check_fields_when_checks_absent() {
        // 无 checks 时仅可能带顶层定位字段，不 panic。
        let highlights = extract_highlights(&json!({ "schemaVersion": 1 }));
        assert!(highlights.is_empty());
        assert!(extract_highlights(&json!("not-an-object")).is_empty());
    }

    #[test]
    fn value_to_display_handles_scalar_kinds() {
        assert_eq!(value_to_display(&json!("text")), "text");
        assert_eq!(value_to_display(&json!(true)), "true");
        assert_eq!(value_to_display(&json!(42)), "42");
        assert_eq!(value_to_display(&json!(null)), "null");
    }
}
