//! 🧹 codex fix 命令实现
//!
//! 清理残留 Codex `app-server` 进程，并对 CCR profile / Codex runtime 做本地诊断。
//! 默认不调用上游 `codex doctor`；需要补充证据时传入 `--doctor`。
//! 用于修复 SSH / Desktop / VS Code Remote 断开后 app-server 仍锁定旧登录态、
//! 导致第三方 URL/Key 切换不生效的问题。

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use ccr_codex::{
    CodexAppServerCleanupReport, CodexPlatform, CodexProcessService, CodexRuntimeDiagnostic,
    CodexRuntimeIssue, RuntimeMatchStatus, TerminationKind,
};
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use ccr_core::core::process_gateway::{ManagedProcess, read_bounded_line};
use serde_json::Value;
use tokio::io::BufReader;

use crate::services::install_detect::which_on_path;

/// `codex doctor` 外部调用（含网络探活）的超时上限。
const DOCTOR_TIMEOUT: Duration = Duration::from_secs(30);
/// 超时后回收进程树的宽限。
const DOCTOR_TERMINATE_GRACE: Duration = Duration::from_secs(1);
const DOCTOR_MAX_LINE_BYTES: usize = 1024 * 1024;
const DOCTOR_MAX_TOTAL_BYTES: usize = 2 * 1024 * 1024;

/// 本地 profile/runtime 漂移（未修复或修复后仍不一致）。
const LOCAL_DRIFT_EXIT_CODE: i32 = 3;

pub async fn fix_command(dry_run: bool, repair_runtime: bool, doctor: bool) -> Result<()> {
    // A. 进程清理（dry-run 时只枚举、不终止）。
    let cleanup_started = Instant::now();
    let cleanup_report = CodexProcessService::new().cleanup_report(dry_run);
    ColorOutput::step(&format!(
        "进程清理（{} ms）",
        cleanup_started.elapsed().as_millis()
    ));
    render_cleanup(&cleanup_report);

    // B. 在调用任何会 reconcile pointer 的路径前，采集只读 profile/runtime 快照。
    let mut runtime_failed = false;
    let platform = match CodexPlatform::new() {
        Ok(platform) => Some(platform),
        Err(_) => {
            runtime_failed = true;
            render_runtime_unavailable("CCR Codex runtime 初始化失败");
            None
        }
    };
    let inspect_started = Instant::now();
    let mut final_diagnostic = platform.as_ref().and_then(|platform| {
        let before = match platform.inspect_runtime() {
            Ok(diagnostic) => diagnostic,
            Err(_) => {
                runtime_failed = true;
                render_runtime_unavailable("CCR runtime inspection 失败");
                return None;
            }
        };
        render_runtime_diagnostic(&before, "Codex runtime 本地诊断");

        // C. 仅在显式授权时重放当前 profile；dry-run 只展示动作。
        let final_state = match decide_runtime_repair(dry_run, repair_runtime, &before) {
            RuntimeRepairAction::None => before,
            RuntimeRepairAction::Preview => {
                ColorOutput::info(&format!(
                    "--dry-run：将重放 profile {}，不会写入 config.toml / auth.json",
                    before.resolved_profile.as_deref().unwrap_or("<unknown>")
                ));
                before
            }
            RuntimeRepairAction::Apply => {
                let profile = before.resolved_profile.as_deref().unwrap_or("<unknown>");
                ColorOutput::info(&format!("正在重放当前 Codex profile：{profile}"));
                if platform.repair_runtime(&before).is_err() {
                    runtime_failed = true;
                    render_runtime_unavailable("CCR runtime repair 失败");
                    before
                } else {
                    match platform.inspect_runtime() {
                        Ok(after) => {
                            render_repair_result(&after);
                            after
                        }
                        Err(_) => {
                            runtime_failed = true;
                            render_runtime_unavailable("Runtime 修复后的 inspection 失败");
                            before
                        }
                    }
                }
            }
            RuntimeRepairAction::Blocked => {
                ColorOutput::warning(
                    "当前漂移不能通过重放 profile 安全修复；未修改 config.toml / auth.json",
                );
                before
            }
        };
        Some(final_state)
    });
    ColorOutput::step(&format!(
        "本地诊断（{} ms）",
        inspect_started.elapsed().as_millis()
    ));

    // D. 环境提示（脱敏，只展开已设置的变量）。
    render_env_hints(final_diagnostic.as_ref());

    // E/F. 仅 `--doctor` 才查找 PATH 并运行上游 doctor。
    let mut snapshot_changed = false;
    if doctor {
        let Some(codex_bin) = which_on_path("codex") else {
            ColorOutput::error("PATH 中找不到 codex 命令，跳过配置诊断");
            ColorOutput::info("请确认 Codex CLI 已安装且在 PATH 中后重试");
            exit_after_flush(127);
        };

        let doctor_profile = final_diagnostic
            .as_ref()
            .and_then(|diagnostic| diagnostic.resolved_profile.clone());
        let doctor_started = Instant::now();
        let doctor_outcome = run_codex_doctor(&codex_bin, !dry_run, DOCTOR_TIMEOUT).await;
        ColorOutput::step(&format!(
            "上游 doctor（{} ms）",
            doctor_started.elapsed().as_millis()
        ));
        if let (Some(platform), Some(before_doctor)) =
            (platform.as_ref(), final_diagnostic.as_ref())
        {
            match platform.inspect_runtime() {
                Ok(after_doctor) => {
                    snapshot_changed = after_doctor != *before_doctor;
                    if snapshot_changed {
                        ColorOutput::warning(
                            "codex doctor 运行期间 profile/runtime 状态发生变化，doctor 输出不能归属于原快照",
                        );
                        render_runtime_diagnostic(&after_doctor, "Doctor 后的最新本地诊断");
                        final_diagnostic = Some(after_doctor);
                    }
                }
                Err(_) => {
                    runtime_failed = true;
                    render_runtime_unavailable("Doctor 后的 runtime inspection 失败");
                }
            }
        }
        render_doctor(&doctor_outcome, doctor_profile.as_deref(), snapshot_changed);
    } else {
        ColorOutput::step("上游 doctor（skipped）");
        ColorOutput::info("doctor = skipped");
        ColorOutput::info("需要上游健康检查时运行 ccr codex fix --doctor");
    }

    // G. 固定优先级：127（仅 --doctor 且 PATH 缺失，已提前返回）> process(2) > runtime failure(1) > local drift(3)。
    if let Some(code) = diagnostic_exit_code(
        &cleanup_report,
        final_diagnostic.as_ref(),
        runtime_failed,
        snapshot_changed,
    ) {
        exit_after_flush(code);
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeRepairAction {
    None,
    Preview,
    Apply,
    Blocked,
}

fn decide_runtime_repair(
    dry_run: bool,
    repair_runtime: bool,
    diagnostic: &CodexRuntimeDiagnostic,
) -> RuntimeRepairAction {
    if !repair_runtime || !diagnostic.has_local_drift() {
        return RuntimeRepairAction::None;
    }
    if !diagnostic.repairable {
        return RuntimeRepairAction::Blocked;
    }
    if dry_run {
        RuntimeRepairAction::Preview
    } else {
        RuntimeRepairAction::Apply
    }
}

fn diagnostic_exit_code(
    report: &CodexAppServerCleanupReport,
    diagnostic: Option<&CodexRuntimeDiagnostic>,
    runtime_failed: bool,
    snapshot_changed: bool,
) -> Option<i32> {
    if report.discovery_issue.is_some() || !report.cleanup.respawned.is_empty() {
        Some(2)
    } else if runtime_failed {
        Some(1)
    } else if snapshot_changed || diagnostic.is_some_and(CodexRuntimeDiagnostic::has_local_drift) {
        Some(LOCAL_DRIFT_EXIT_CODE)
    } else {
        None
    }
}

/// 刷新 stdout 后以指定退出码结束进程（沿用 `doctor_cmd` 的显式退出码惯例）。
fn exit_after_flush(code: i32) -> ! {
    let _ = io::stdout().flush();
    std::process::exit(code);
}

// ==================== 进程清理渲染 ====================

fn render_cleanup(report: &CodexAppServerCleanupReport) {
    let cleanup = &report.cleanup;
    ColorOutput::info(&format!(
        "process_state = {}",
        cleanup_process_state(report)
    ));
    if let Some(issue) = report.discovery_issue {
        ColorOutput::warning(&format!(
            "无法安全完成当前用户的 app-server 发现/清理（{}）",
            issue.as_str()
        ));
        ColorOutput::warning("为避免误杀，进程阶段已 fail closed；请修复进程可见性后重试");
        render_process_candidates(&cleanup.found, "中断前已发现的 app-server：");
        render_signal_failures(report);
        return;
    }
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
    render_process_candidates(
        &report.discovered_during_cleanup,
        "清理窗口内发现的新 app-server：",
    );
    render_signal_failures(report);

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

fn cleanup_process_state(report: &CodexAppServerCleanupReport) -> &'static str {
    if report.discovery_issue.is_some() {
        return "unavailable";
    }
    let cleanup = &report.cleanup;
    if !cleanup.respawned.is_empty() {
        "respawned"
    } else if cleanup.dry_run && !cleanup.found.is_empty() {
        "dry_run_found"
    } else if cleanup.found.is_empty() {
        "clean"
    } else {
        "cleaned"
    }
}

fn render_process_candidates(processes: &[ccr_codex::CodexAppServer], title: &str) {
    if processes.is_empty() {
        return;
    }
    ColorOutput::info(title);
    for process in processes {
        ColorOutput::info(&format!("  PID {} — {}", process.pid, process.cmdline));
    }
}

fn render_signal_failures(report: &CodexAppServerCleanupReport) {
    for failure in &report.signal_failures {
        ColorOutput::warning(&format!(
            "  PID {} 的 {} 信号发送失败",
            failure.pid,
            failure.stage.as_str()
        ));
    }
}

fn termination_desc(kind: TerminationKind) -> &'static str {
    match kind {
        TerminationKind::Term => "已优雅退出 (SIGTERM)",
        TerminationKind::Kill => "已强制终止",
        TerminationKind::AlreadyGone => "终止前已退出",
    }
}

// ==================== Runtime 诊断与环境提示（脱敏） ====================

fn render_runtime_diagnostic(diagnostic: &CodexRuntimeDiagnostic, title: &str) {
    ColorOutput::info(title);
    for line in runtime_diagnostic_lines(diagnostic) {
        ColorOutput::info(&format!("  {line}"));
    }

    for issue in &diagnostic.issues {
        match issue {
            CodexRuntimeIssue::RegistryPointerMissing => {
                ColorOutput::warning("CCR registry 缺少 Codex current profile pointer")
            }
            CodexRuntimeIssue::ProfilesPointerMissing => {
                ColorOutput::warning("profiles.toml 缺少 current_config pointer")
            }
            CodexRuntimeIssue::ProfilePointerMismatch => ColorOutput::warning(
                "CCR registry 与 profiles.toml 指向不同 profile，拒绝猜测修复目标",
            ),
            CodexRuntimeIssue::ProfileNotFound { profile } => {
                ColorOutput::warning(&format!("pointer 指向不存在的 profile：{profile}"))
            }
            CodexRuntimeIssue::RouteMismatch => {
                ColorOutput::warning("当前 Codex route 与 CCR profile 不一致")
            }
            CodexRuntimeIssue::CredentialMissing => {
                ColorOutput::warning("当前 profile 需要的本地凭据来源缺失")
            }
            CodexRuntimeIssue::CredentialMismatch => {
                ColorOutput::warning("CCR profile secret 与 Codex runtime 凭据不一致")
            }
            CodexRuntimeIssue::CredentialUnsupported => {
                ColorOutput::warning("当前凭据存储或环境覆盖无法由 CCR 只读验证")
            }
            CodexRuntimeIssue::EnvironmentOverride { variable } => ColorOutput::warning(&format!(
                "检测到 {variable}，实际请求可能覆盖本地 runtime 配置（值已隐藏）"
            )),
            CodexRuntimeIssue::CodexHomeMismatch => {
                ColorOutput::warning("CODEX_HOME 与 CCR 当前检查的 Codex runtime 路径不一致")
            }
        }
    }

    match diagnostic.runtime_consistency() {
        RuntimeMatchStatus::Match => {
            ColorOutput::info("本地 profile/runtime 一致；这不表示第三方 Provider 已接受当前 key")
        }
        RuntimeMatchStatus::NotApplicable => {
            ColorOutput::info("当前没有可解析的 CCR profile，仅报告 Codex runtime 状态")
        }
        RuntimeMatchStatus::Unsupported => {
            ColorOutput::warning("本地状态包含不可读凭据或环境覆盖，CCR 无法宣称完全一致")
        }
        RuntimeMatchStatus::Missing | RuntimeMatchStatus::Mismatch => {
            if diagnostic.repairable {
                ColorOutput::info("可运行 ccr codex fix --repair-runtime 显式修复本地漂移");
            } else {
                ColorOutput::info(
                    "请先修正 profile pointer、保存的 secret 或当前 shell 环境后重试",
                );
            }
        }
    }
}

fn render_repair_result(diagnostic: &CodexRuntimeDiagnostic) {
    if diagnostic.runtime_consistency() == RuntimeMatchStatus::Match {
        ColorOutput::success("Codex runtime 本地漂移已修复");
    } else {
        ColorOutput::warning("重放 profile 后本地 runtime 仍不一致");
    }
    ColorOutput::info(&format!(
        "runtime_consistency = {}",
        diagnostic.runtime_consistency().as_str()
    ));
}

fn render_runtime_unavailable(stage: &str) {
    ColorOutput::warning("runtime_consistency = unavailable");
    ColorOutput::warning(&format!(
        "{stage}；未猜测或写入 config.toml / auth.json，继续执行可用的独立诊断阶段"
    ));
}

fn runtime_diagnostic_lines(diagnostic: &CodexRuntimeDiagnostic) -> Vec<String> {
    vec![
        format!("registry = {}", diagnostic.registry_path.display()),
        format!("profiles = {}", diagnostic.profiles_path.display()),
        format!("config.toml = {}", diagnostic.config_path.display()),
        format!("auth.json = {}", diagnostic.auth_path.display()),
        format!(
            "registry profile = {}",
            optional_label(diagnostic.registry_profile.as_deref())
        ),
        format!(
            "profiles.toml profile = {}",
            optional_label(diagnostic.profiles_file_profile.as_deref())
        ),
        format!(
            "resolved profile = {}",
            optional_label(diagnostic.resolved_profile.as_deref())
        ),
        format!(
            "runtime provider id = {}",
            optional_label(diagnostic.runtime_provider_id.as_deref())
        ),
        format!(
            "runtime provider name = {}",
            optional_label(diagnostic.runtime_provider_name.as_deref())
        ),
        format!(
            "base URL = {}",
            optional_label(diagnostic.base_url.as_deref())
        ),
        format!(
            "wire API = {}",
            optional_label(diagnostic.wire_api.as_deref())
        ),
        format!(
            "credential store = {}",
            diagnostic.credential_store.as_str()
        ),
        format!("auth source = {}", diagnostic.auth_source.label()),
        format!("profile pointer = {}", diagnostic.profile_status.as_str()),
        format!("route consistency = {}", diagnostic.route_status.as_str()),
        format!(
            "credential consistency = {}",
            diagnostic.credential_status.as_str()
        ),
        format!(
            "runtime_consistency = {}",
            diagnostic.runtime_consistency().as_str()
        ),
        format!(
            "provider_auth_validity = {}",
            diagnostic.provider_auth_validity.as_str()
        ),
    ]
}

fn optional_label(value: Option<&str>) -> &str {
    value.unwrap_or("<none>")
}

fn render_env_hints(diagnostic: Option<&CodexRuntimeDiagnostic>) {
    let mut lines = Vec::new();
    if let Ok(value) = std::env::var("CODEX_HOME")
        && !value.is_empty()
    {
        lines.push(format!("  CODEX_HOME={value}"));
    }
    if let Ok(value) = std::env::var("CCR_CODEX_DIR")
        && !value.is_empty()
    {
        lines.push(format!("  CCR_CODEX_DIR={value}"));
    }
    if let Ok(value) = std::env::var("OPENAI_BASE_URL")
        && !value.trim().is_empty()
    {
        lines.push("  OPENAI_BASE_URL=<已设置，值已隐藏>".to_string());
    }
    if let Some(diagnostic) = diagnostic {
        for presence in &diagnostic.environment {
            if presence.is_set {
                lines.push(format!("  {}=<已设置，值已隐藏>", presence.variable));
            }
        }
    }
    if lines.is_empty() {
        return;
    }
    ColorOutput::info("当前进程中可能影响 Codex 的环境变量：");
    for line in lines {
        ColorOutput::info(&line);
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

async fn run_codex_doctor(bin: &Path, persist_report: bool, timeout: Duration) -> DoctorOutcome {
    // 首选 `codex doctor --json`：以「是否拿到有效 JSON」判断，而非退出码
    // （检查项失败时 doctor 可能返回非 0，但 stdout 仍是有效报告，应照常展示）。
    match capture_doctor(bin, &["doctor", "--json"], timeout).await {
        Ok(stdout) => {
            if let Ok(json) = serde_json::from_slice::<Value>(&stdout) {
                let sanitized = sanitize_doctor_json(&json);
                let report =
                    serde_json::to_vec_pretty(&sanitized).unwrap_or_else(|_| b"{}".to_vec());
                return DoctorOutcome {
                    report_path: persist_report
                        .then(|| save_report(&report, "json"))
                        .flatten(),
                    highlights: extract_highlights(&sanitized),
                    raw_text: None,
                    failed: false,
                    note: None,
                };
            }
            // --json 已返回非 JSON 文本：脱敏后当文本渲染，不再二次 spawn。
            let text = sanitize_doctor_text(String::from_utf8_lossy(&stdout).trim());
            DoctorOutcome {
                report_path: persist_report
                    .then(|| save_report(text.as_bytes(), "txt"))
                    .flatten(),
                highlights: Vec::new(),
                raw_text: (!text.is_empty()).then_some(text),
                failed: false,
                note: Some("codex doctor --json 未返回有效 JSON，已按文本展示".to_string()),
            }
        }
        Err(DoctorError::Timeout) => {
            DoctorOutcome::failed_with("codex doctor 超时（可能卡在网络探活），已跳过诊断")
        }
        Err(DoctorError::Spawn(msg)) => {
            DoctorOutcome::failed_with(&format!("无法执行 codex doctor：{msg}"))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum DoctorError {
    Timeout,
    Spawn(String),
}

/// 运行 `codex <args>`，在 `timeout` 内捕获受限 stdout。
///
/// 通过 `ManagedProcess` 托管整棵进程树：正常路径 `wait`，超时路径
/// `terminate_tree` 并 await reap。不得把 Drop 当作正常超时回收。
async fn capture_doctor(
    bin: &Path,
    args: &[&str],
    timeout: Duration,
) -> std::result::Result<Vec<u8>, DoctorError> {
    let mut cmd = tokio::process::Command::new(bin);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child =
        ManagedProcess::spawn(cmd).map_err(|error| DoctorError::Spawn(error.to_string()))?;
    let stdout = child.take_stdout();
    let stderr = child.take_stderr();
    let stdout_task = tokio::spawn(drain_bounded_pipe(stdout));
    let stderr_task = tokio::spawn(drain_bounded_pipe(stderr));

    let wait_result = tokio::time::timeout(timeout, child.wait()).await;
    match wait_result {
        Ok(Ok(_status)) => {
            let _ = stderr_task.await;
            match stdout_task.await {
                Ok(bytes) => Ok(bytes),
                Err(error) => Err(DoctorError::Spawn(error.to_string())),
            }
        }
        Ok(Err(error)) => {
            let _ = stdout_task.await;
            let _ = stderr_task.await;
            Err(DoctorError::Spawn(error.to_string()))
        }
        Err(_) => {
            if let Err(error) = child.terminate_tree(DOCTOR_TERMINATE_GRACE).await {
                tracing::warn!(%error, "codex doctor 进程树回收失败");
            }
            let _ = stdout_task.await;
            let _ = stderr_task.await;
            Err(DoctorError::Timeout)
        }
    }
}

async fn drain_bounded_pipe<R>(pipe: Option<R>) -> Vec<u8>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let Some(pipe) = pipe else {
        return Vec::new();
    };
    let mut reader = BufReader::new(pipe);
    let mut out = Vec::new();
    while let Ok(Some(line)) = read_bounded_line(&mut reader, DOCTOR_MAX_LINE_BYTES).await {
        if out.len() >= DOCTOR_MAX_TOTAL_BYTES {
            break;
        }
        let remaining = DOCTOR_MAX_TOTAL_BYTES - out.len();
        let bytes = line.text.as_bytes();
        let take = bytes.len().min(remaining);
        out.extend_from_slice(&bytes[..take]);
        if out.len() < DOCTOR_MAX_TOTAL_BYTES {
            out.push(b'\n');
        }
    }
    out
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

fn sanitize_doctor_json(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| {
                    let value = if is_sensitive_doctor_label(key) {
                        Value::String("<redacted>".to_string())
                    } else if is_doctor_url_label(key) {
                        value
                            .as_str()
                            .map(safe_url_for_display)
                            .map(Value::String)
                            .unwrap_or_else(|| sanitize_doctor_json(value))
                    } else {
                        sanitize_doctor_json(value)
                    };
                    (key.clone(), value)
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(sanitize_doctor_json).collect()),
        Value::String(value) if looks_like_url(value) => Value::String(safe_url_for_display(value)),
        other => other.clone(),
    }
}

fn sanitize_doctor_text(text: &str) -> String {
    text.lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
                return safe_url_for_display(trimmed);
            }
            let separator = first_label_separator(line);
            if let Some(index) = separator {
                let label = &line[..index];
                let value = line[index + 1..].trim();
                if is_sensitive_doctor_label(label) {
                    return format!("{} <redacted>", &line[..=index]);
                }
                if is_doctor_url_label(label) || looks_like_url(value) {
                    return format!("{} {}", &line[..=index], safe_url_for_display(value));
                }
                return line.to_string();
            }
            if is_sensitive_doctor_label(line) {
                return "<redacted>".to_string();
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn first_label_separator(line: &str) -> Option<usize> {
    match (line.find('='), line.find(':')) {
        (Some(equals), Some(colon)) => Some(equals.min(colon)),
        (Some(index), None) | (None, Some(index)) => Some(index),
        (None, None) => None,
    }
}

fn is_sensitive_doctor_label(label: &str) -> bool {
    let normalized = label.to_ascii_lowercase();
    [
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
        "id_token",
        "token",
        "authorization",
        "password",
        "secret",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn is_doctor_url_label(label: &str) -> bool {
    let normalized = label.to_ascii_lowercase();
    normalized.contains("base_url") || normalized.contains("endpoint")
}

fn looks_like_url(value: &str) -> bool {
    value.contains("://")
}

fn safe_url_for_display(value: &str) -> String {
    let without_fragment = value.split('#').next().unwrap_or(value);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment);
    let Some((scheme, remainder)) = without_query.split_once("://") else {
        return without_query.to_string();
    };
    let host_and_path = remainder
        .rsplit_once('@')
        .map(|(_, host_and_path)| host_and_path)
        .unwrap_or(remainder);
    format!("{scheme}://{host_and_path}")
}

/// 从 doctor JSON 报告中提取定位字段与非 ok 检查。
///
/// 报告结构（schemaVersion 1）：顶层含 `codexVersion` / `overallStatus`，
/// 各检查项在 `checks.<id>` 下带 `status`。只展示顶层字段和 `status != "ok"` 的检查 id。
fn extract_highlights(json: &Value) -> Vec<(String, String)> {
    let mut highlights = Vec::new();

    for key in ["codexVersion", "overallStatus"] {
        if let Some(value) = json.get(key) {
            highlights.push((key.to_string(), value_to_display(value)));
        }
    }

    if let Some(checks) = json.get("checks").and_then(Value::as_object) {
        for (id, check) in checks {
            let status = check
                .get("status")
                .map(value_to_display)
                .unwrap_or_else(|| "unknown".to_string());
            if status != "ok" {
                highlights.push((id.clone(), status));
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

fn render_doctor(outcome: &DoctorOutcome, profile: Option<&str>, snapshot_changed: bool) {
    if outcome.failed {
        let note = outcome.note.as_deref().unwrap_or("codex doctor 诊断失败");
        ColorOutput::warning(note);
        return;
    }

    if let Some(note) = &outcome.note {
        ColorOutput::info(note);
    }

    let snapshot_profile = if snapshot_changed {
        "<changed_during_doctor>"
    } else {
        optional_label(profile)
    };

    if outcome.highlights.is_empty() {
        if let Some(raw) = &outcome.raw_text {
            ColorOutput::info(&format!(
                "codex doctor 输出（本次快照 profile={snapshot_profile}）："
            ));
            println!("{raw}");
        } else {
            ColorOutput::info(&format!(
                "codex doctor 已运行（本次快照 profile={snapshot_profile}，详见完整报告）"
            ));
        }
    } else {
        ColorOutput::info(&format!(
            "codex doctor 关键配置（本次快照 profile={}）：",
            snapshot_profile
        ));
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
    use super::{
        DoctorError, LOCAL_DRIFT_EXIT_CODE, RuntimeRepairAction, capture_doctor,
        cleanup_process_state, decide_runtime_repair, diagnostic_exit_code, extract_highlights,
        run_codex_doctor, runtime_diagnostic_lines, sanitize_doctor_json, sanitize_doctor_text,
        value_to_display,
    };
    use ccr_codex::{
        CodexAppServer, CodexAppServerCleanupReport, CodexProcessDiscoveryIssue,
        CodexRuntimeAuthSource, CodexRuntimeDiagnostic, CredentialStoreKind, ProviderAuthValidity,
        RuntimeMatchStatus,
    };
    use serde_json::json;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn extracts_version_status_and_non_ok_check_ids() {
        let report = json!({
            "schemaVersion": 1,
            "overallStatus": "warning",
            "codexVersion": "0.144.6",
            "checks": {
                "config.load": {
                    "category": "config",
                    "status": "ok",
                    "details": {
                        "CODEX_HOME": "/home/lyh/.codex",
                        "config.toml": "/home/lyh/.codex/config.toml",
                        "model provider": "openai"
                    }
                },
                "state.rollout_db_parity": {
                    "category": "state",
                    "status": "warning",
                    "details": {
                        "search provider": "none",
                        "configured servers": []
                    }
                },
                "auth.credentials": {
                    "category": "auth",
                    "status": "ok",
                    "details": {
                        "auth file": "/home/lyh/.codex/auth.json"
                    }
                }
            }
        });

        let highlights = extract_highlights(&report);

        assert!(
            highlights
                .iter()
                .any(|(k, v)| k == "codexVersion" && v == "0.144.6")
        );
        assert!(
            highlights
                .iter()
                .any(|(k, v)| k == "overallStatus" && v == "warning")
        );
        assert!(
            highlights
                .iter()
                .any(|(k, v)| k == "state.rollout_db_parity" && v == "warning")
        );
        assert!(!highlights.iter().any(|(k, _)| k == "CODEX_HOME"));
        assert!(!highlights.iter().any(|(k, _)| k == "config.toml"));
        assert!(!highlights.iter().any(|(k, _)| k == "model provider"));
        assert!(!highlights.iter().any(|(k, _)| k == "search provider"));
        assert!(!highlights.iter().any(|(k, _)| k == "configured servers"));
        assert!(!highlights.iter().any(|(k, _)| k == "auth file"));
        assert!(!highlights.iter().any(|(k, _)| k == "config.load"));
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

    #[test]
    fn doctor_reports_redact_sensitive_values_before_render_or_save() {
        const SENTINEL: &str = "doctor-secret-must-not-leak";
        let report = json!({
            "checks": {
                "auth.credentials": {
                    "details": {
                        "auth file": "/tmp/auth.json",
                        "stored auth mode": "api_key",
                        "base_url": format!("https://user:{SENTINEL}@example.com/v1?key={SENTINEL}"),
                        "proxy": format!("https://user:{SENTINEL}@proxy.example/v1?key={SENTINEL}"),
                        "OPENAI_API_KEY": SENTINEL,
                        "tokens": {
                            "refresh_token": SENTINEL
                        }
                    }
                }
            }
        });

        let sanitized = sanitize_doctor_json(&report);
        let serialized = serde_json::to_string(&sanitized).unwrap_or_default();
        assert!(!serialized.contains(SENTINEL));
        assert!(serialized.contains("<redacted>"));
        assert!(serialized.contains("https://example.com/v1"));

        let text = sanitize_doctor_text(&format!(
            "auth file: /tmp/auth.json\nbase_url=https://user:{SENTINEL}@example.com/v1?key={SENTINEL}\nproxy: https://user:{SENTINEL}@proxy.example/v1?key={SENTINEL}\nhttps://user:{SENTINEL}@standalone.example/v1?key={SENTINEL}\nOPENAI_API_KEY={SENTINEL}\nrefresh_token: {SENTINEL}"
        ));
        assert!(!text.contains(SENTINEL));
        assert!(text.contains("auth file: /tmp/auth.json"));
        assert!(text.contains("base_url= https://example.com/v1"));
        assert!(text.contains("proxy: https://proxy.example/v1"));
    }

    #[test]
    fn runtime_repair_requires_explicit_flag_and_dry_run_only_previews() {
        let mut diagnostic = test_diagnostic(RuntimeMatchStatus::Mismatch, true);

        assert_eq!(
            decide_runtime_repair(false, false, &diagnostic),
            RuntimeRepairAction::None
        );
        assert_eq!(
            decide_runtime_repair(true, true, &diagnostic),
            RuntimeRepairAction::Preview
        );
        assert_eq!(
            decide_runtime_repair(false, true, &diagnostic),
            RuntimeRepairAction::Apply
        );

        diagnostic.repairable = false;
        assert_eq!(
            decide_runtime_repair(false, true, &diagnostic),
            RuntimeRepairAction::Blocked
        );
    }

    #[test]
    fn runtime_diagnostic_lines_state_provider_validity_was_not_checked() {
        let diagnostic = test_diagnostic(RuntimeMatchStatus::Match, false);
        let lines = runtime_diagnostic_lines(&diagnostic);
        assert!(
            lines
                .iter()
                .any(|line| line == "provider_auth_validity = not_checked")
        );
        assert!(lines.iter().any(|line| line == "resolved profile = future"));
    }

    #[test]
    fn respawn_exit_code_takes_priority_over_local_drift() {
        let diagnostic = test_diagnostic(RuntimeMatchStatus::Mismatch, true);
        let mut report = CodexAppServerCleanupReport::default();
        assert_eq!(
            diagnostic_exit_code(&report, Some(&diagnostic), false, false),
            Some(LOCAL_DRIFT_EXIT_CODE)
        );
        let consistent = test_diagnostic(RuntimeMatchStatus::Match, false);
        assert_eq!(
            diagnostic_exit_code(&report, Some(&consistent), false, true),
            Some(LOCAL_DRIFT_EXIT_CODE)
        );

        report.cleanup.respawned.push(CodexAppServer {
            pid: 42,
            cmdline: "codex app-server".to_string(),
        });
        assert_eq!(
            diagnostic_exit_code(&report, Some(&diagnostic), true, true),
            Some(2)
        );
        assert_eq!(cleanup_process_state(&report), "respawned");
    }

    #[test]
    fn runtime_failure_and_process_unavailable_have_stable_priority() {
        let diagnostic = test_diagnostic(RuntimeMatchStatus::Mismatch, true);
        let mut report = CodexAppServerCleanupReport::default();

        assert_eq!(
            diagnostic_exit_code(&report, Some(&diagnostic), true, false),
            Some(1)
        );

        report.discovery_issue = Some(CodexProcessDiscoveryIssue::CurrentOwnerUnavailable);
        assert_eq!(
            diagnostic_exit_code(&report, Some(&diagnostic), true, false),
            Some(2)
        );
        assert_eq!(cleanup_process_state(&report), "unavailable");
    }

    #[tokio::test]
    async fn non_json_doctor_stdout_does_not_spawn_a_second_doctor() {
        let fixture = FakeDoctorScript::write("not-json-doctor-output\n");
        let outcome = run_codex_doctor(&fixture.bin, false, Duration::from_secs(5)).await;
        assert!(!outcome.failed);
        assert!(
            outcome
                .raw_text
                .as_deref()
                .is_some_and(|text| text.contains("not-json-doctor-output"))
        );
        assert!(outcome.highlights.is_empty());
        assert_eq!(fixture.spawn_count(), 1);
    }

    #[tokio::test]
    async fn json_doctor_stdout_extracts_version_and_status() {
        let fixture = FakeDoctorScript::write(
            r#"{"schemaVersion":1,"overallStatus":"ok","codexVersion":"fixture-doctor","checks":{}}"#,
        );
        let outcome = run_codex_doctor(&fixture.bin, false, Duration::from_secs(5)).await;
        assert!(!outcome.failed);
        assert!(
            outcome
                .highlights
                .iter()
                .any(|(key, value)| key == "codexVersion" && value == "fixture-doctor")
        );
        assert!(
            outcome
                .highlights
                .iter()
                .any(|(key, value)| key == "overallStatus" && value == "ok")
        );
        assert!(outcome.report_path.is_none());
        assert_eq!(fixture.spawn_count(), 1);
    }

    #[tokio::test]
    async fn dry_run_doctor_does_not_persist_report() {
        let fixture = FakeDoctorScript::write(
            r#"{"schemaVersion":1,"overallStatus":"ok","codexVersion":"fixture-doctor","checks":{}}"#,
        );
        let outcome = run_codex_doctor(&fixture.bin, false, Duration::from_secs(5)).await;
        assert!(outcome.report_path.is_none());
    }

    #[tokio::test]
    async fn non_dry_run_doctor_persists_sanitized_report() {
        const SENTINEL: &str = "doctor-secret-must-not-leak";
        let payload = format!(
            r#"{{"schemaVersion":1,"overallStatus":"ok","codexVersion":"fixture-doctor","checks":{{"auth.credentials":{{"status":"ok","details":{{"OPENAI_API_KEY":"{SENTINEL}"}}}}}}}}"#
        );
        let fixture = FakeDoctorScript::write(&payload);
        let outcome = run_codex_doctor(&fixture.bin, true, Duration::from_secs(5)).await;
        let path = outcome.report_path.expect("report should be persisted");
        let saved = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(!saved.contains(SENTINEL));
        assert!(saved.contains("<redacted>"));
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn doctor_timeout_terminates_parent_and_grandchild() {
        let temp = tempfile::tempdir().expect("tempdir");
        let parent_pid = temp.path().join("parent.pid");
        let child_pid = temp.path().join("grandchild.pid");
        let (bin, args) = hanging_doctor_command(&parent_pid, &child_pid);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let timeout = if cfg!(windows) {
            Duration::from_secs(12)
        } else {
            Duration::from_secs(2)
        };

        // 在超时杀树的同时等 pid 文件，避免 Windows 并行负载下 PowerShell 还没写完就被杀掉。
        let (result, parent, grandchild) = tokio::join!(
            capture_doctor(&bin, &arg_refs, timeout),
            wait_for_pid_file(&parent_pid),
            wait_for_pid_file(&child_pid),
        );
        assert_eq!(result, Err(DoctorError::Timeout));
        wait_until_process_gone(parent).await;
        wait_until_process_gone(grandchild).await;
    }

    struct FakeDoctorScript {
        _temp: tempfile::TempDir,
        bin: PathBuf,
        count_path: PathBuf,
    }

    impl FakeDoctorScript {
        fn write(payload: &str) -> Self {
            let temp = tempfile::tempdir().expect("tempdir");
            let count_path = temp.path().join("spawn-count.txt");
            let payload_path = temp.path().join("payload.txt");
            std::fs::write(&payload_path, payload).expect("write payload");
            let bin = write_fake_doctor_bin(temp.path(), &count_path, &payload_path);
            Self {
                _temp: temp,
                bin,
                count_path,
            }
        }

        fn spawn_count(&self) -> usize {
            std::fs::read_to_string(&self.count_path)
                .map(|text| text.lines().filter(|line| !line.is_empty()).count())
                .unwrap_or(0)
        }
    }

    #[cfg(windows)]
    fn write_fake_doctor_bin(
        dir: &std::path::Path,
        count_path: &std::path::Path,
        payload_path: &std::path::Path,
    ) -> PathBuf {
        let bin = dir.join("codex.cmd");
        let script = format!(
            "@echo off\r\n>>{count} echo 1\r\ntype {payload}\r\n",
            count = cmd_quote(count_path),
            payload = cmd_quote(payload_path),
        );
        std::fs::write(&bin, script).expect("write fake doctor");
        bin
    }

    #[cfg(unix)]
    fn write_fake_doctor_bin(
        dir: &std::path::Path,
        count_path: &std::path::Path,
        payload_path: &std::path::Path,
    ) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let bin = dir.join("codex");
        let script = format!(
            "#!/bin/sh\nprintf '1\\n' >> {count}\n/bin/cat {payload}\n",
            count = sh_single_quote(count_path),
            payload = sh_single_quote(payload_path),
        );
        std::fs::write(&bin, script).expect("write fake doctor");
        let mut permissions = std::fs::metadata(&bin)
            .expect("stat fake doctor")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&bin, permissions).expect("chmod fake doctor");
        bin
    }

    #[cfg(windows)]
    fn hanging_doctor_command(
        parent_pid: &std::path::Path,
        child_pid: &std::path::Path,
    ) -> (PathBuf, Vec<String>) {
        let script = format!(
            "$ErrorActionPreference='Stop'; \
             [IO.File]::WriteAllText('{}', [string]$PID); \
             $child=Start-Process -FilePath 'cmd.exe' -ArgumentList '/C','ping -n 30 127.0.0.1 >NUL' -PassThru; \
             [IO.File]::WriteAllText('{}', [string]$child.Id); \
             Start-Sleep -Seconds 30",
            parent_pid.to_string_lossy().replace('\'', "''"),
            child_pid.to_string_lossy().replace('\'', "''"),
        );
        (
            PathBuf::from("powershell.exe"),
            vec![
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                script,
            ],
        )
    }

    #[cfg(unix)]
    fn hanging_doctor_command(
        parent_pid: &std::path::Path,
        child_pid: &std::path::Path,
    ) -> (PathBuf, Vec<String>) {
        use std::os::unix::fs::PermissionsExt;
        let dir = parent_pid.parent().expect("pid file parent");
        let bin = dir.join("hanging-doctor");
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$$\" > {parent}\n/bin/sleep 30 &\nprintf '%s\\n' \"$!\" > {child}\nwait\n",
            parent = sh_single_quote(parent_pid),
            child = sh_single_quote(child_pid),
        );
        std::fs::write(&bin, script).expect("write hanging doctor");
        let mut permissions = std::fs::metadata(&bin)
            .expect("stat hanging doctor")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&bin, permissions).expect("chmod hanging doctor");
        (bin, vec!["doctor".to_string(), "--json".to_string()])
    }

    #[cfg(windows)]
    fn cmd_quote(path: &std::path::Path) -> String {
        format!("\"{}\"", path.display())
    }

    #[cfg(unix)]
    fn sh_single_quote(path: &std::path::Path) -> String {
        format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
    }

    async fn wait_for_pid_file(path: &std::path::Path) -> u32 {
        for _ in 0..200 {
            if let Ok(raw) = std::fs::read_to_string(path)
                && let Ok(pid) = raw.trim().parse()
            {
                return pid;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("pid file was not written: {}", path.display());
    }

    async fn wait_until_process_gone(pid: u32) {
        for _ in 0..200 {
            if !test_process_is_running(pid) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("process {pid} is still running");
    }

    #[cfg(windows)]
    fn test_process_is_running(pid: u32) -> bool {
        const SYNCHRONIZE: u32 = 0x0010_0000;
        const WAIT_TIMEOUT: u32 = 258;
        unsafe extern "system" {
            fn OpenProcess(
                desired_access: u32,
                inherit_handle: i32,
                process_id: u32,
            ) -> *mut std::ffi::c_void;
            fn WaitForSingleObject(handle: *mut std::ffi::c_void, milliseconds: u32) -> u32;
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }

        // SAFETY: 打开的同步句柄在返回前关闭。
        unsafe {
            let handle = OpenProcess(SYNCHRONIZE, 0, pid);
            if handle.is_null() {
                return false;
            }
            let running = WaitForSingleObject(handle, 0) == WAIT_TIMEOUT;
            CloseHandle(handle);
            running
        }
    }

    #[cfg(unix)]
    fn test_process_is_running(pid: u32) -> bool {
        unsafe extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }
        // SAFETY: signal 0 只检查进程是否存在，不发送信号。
        unsafe { kill(pid as i32, 0) == 0 }
    }

    fn test_diagnostic(
        route_status: RuntimeMatchStatus,
        repairable: bool,
    ) -> CodexRuntimeDiagnostic {
        CodexRuntimeDiagnostic {
            registry_path: PathBuf::from("registry.toml"),
            profiles_path: PathBuf::from("profiles.toml"),
            config_path: PathBuf::from("config.toml"),
            auth_path: PathBuf::from("auth.json"),
            registry_profile: Some("future".to_string()),
            profiles_file_profile: Some("future".to_string()),
            resolved_profile: Some("future".to_string()),
            runtime_provider_id: Some("custom".to_string()),
            runtime_provider_name: Some("Future Provider".to_string()),
            base_url: Some("https://www.futureapi.cc/v1".to_string()),
            wire_api: Some("responses".to_string()),
            credential_store: CredentialStoreKind::File,
            auth_source: CodexRuntimeAuthSource::AuthJsonOpenAiApiKey,
            profile_status: RuntimeMatchStatus::Match,
            route_status,
            credential_status: RuntimeMatchStatus::Match,
            provider_auth_validity: ProviderAuthValidity::NotChecked,
            environment: Vec::new(),
            issues: Vec::new(),
            repairable,
        }
    }
}
