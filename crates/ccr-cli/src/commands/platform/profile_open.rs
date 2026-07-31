//! profile open 子命令实现
//!
//! 用系统默认编辑器（或 $VISUAL / $EDITOR）打开 profiles.toml，
//! 文件不存在时先通过模板创建再打开。

use super::profile_init::ensure_profiles_file;
use crate::models::Platform;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use serde::Serialize;

// --- 模板集中 ---

/// 返回指定平台的内嵌 profiles.toml 模板（编译期注入）
pub fn template_for(platform: Platform) -> &'static str {
    match platform {
        Platform::Claude => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/claude/profiles.example.toml"
        )),
        Platform::Codex => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/codex/profiles.toml"
        )),
        Platform::Grok => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/grok/profiles.toml"
        )),
        _ => unreachable!("template_for 只支持三个 auth-profile 平台"),
    }
}

// --- 编辑器解析 ---

enum EditorTarget {
    /// 来自 $VISUAL 或 $EDITOR，阻塞等待进程退出
    Configured {
        program: String,
        args: Vec<String>,
        /// "$VISUAL" 或 "$EDITOR"，用于输出提示
        source: &'static str,
    },
    /// 系统关联程序，非阻塞（由 open crate 处理）
    SystemAssociation,
}

/// 读取环境变量决定编辑器目标
fn resolve_editor() -> EditorTarget {
    resolve_editor_from(
        std::env::var("VISUAL").ok().as_deref(),
        std::env::var("EDITOR").ok().as_deref(),
    )
}

/// 纯函数版本，便于单元测试（不直接读进程环境变量）
fn resolve_editor_from(visual: Option<&str>, editor: Option<&str>) -> EditorTarget {
    for (value, source) in [(visual, "$VISUAL"), (editor, "$EDITOR")] {
        if let Some(s) = value {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                let mut parts = trimmed.split_whitespace();
                let program = parts.next().unwrap_or_default().to_string();
                if program.is_empty() {
                    continue;
                }
                let args: Vec<String> = parts.map(str::to_string).collect();
                return EditorTarget::Configured {
                    program,
                    args,
                    source,
                };
            }
        }
    }
    EditorTarget::SystemAssociation
}

/// 启动编辑器或系统关联程序
///
/// - Configured：阻塞等待进程退出，非零退出码返回 ExternalCommandError
/// - SystemAssociation：调用 open crate（Windows ShellExecuteW / macOS open / Linux xdg-open）
fn spawn_editor(target: &EditorTarget, path: &std::path::Path) -> Result<()> {
    match target {
        EditorTarget::Configured {
            program,
            args,
            source,
        } => {
            let status = std::process::Command::new(program)
                .args(args)
                .arg(path)
                .status()
                .map_err(|e| CcrError::ExternalCommandError(format!("{source} 无法启动: {e}")))?;
            if !status.success() {
                return Err(CcrError::ExternalCommandError(format!(
                    "{source} 退出码: {}",
                    status.code().unwrap_or(-1)
                )));
            }
            Ok(())
        }
        EditorTarget::SystemAssociation => open::that(path)
            .map_err(|e| CcrError::ExternalCommandError(format!("系统关联程序启动失败: {e}"))),
    }
}

// --- 输出结构 ---

#[derive(Serialize)]
struct ProfileOpenOutput<'a> {
    ok: bool,
    platform: &'a str,
    profiles_file: String,
    created: bool,
    registered: bool,
    /// "$VISUAL" | "$EDITOR" | "system"
    editor: &'a str,
}

fn editor_label(target: &EditorTarget) -> &'static str {
    match target {
        EditorTarget::Configured { source, .. } => source,
        EditorTarget::SystemAssociation => "system",
    }
}

// --- 命令入口 ---

/// 用平台对应的 `parse_platform` 解析名称并检查支持性
fn parse_platform_for_open(platform_name: &str) -> Result<Platform> {
    use std::str::FromStr;
    let platform = Platform::from_str(platform_name)
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))?;
    if !Platform::auth_profile_supported().contains(&platform) {
        return Err(CcrError::PlatformNotSupported(platform_name.to_string()));
    }
    Ok(platform)
}

/// 打开指定平台的 profiles.toml。
///
/// 若文件不存在先从内嵌模板创建，然后用 $VISUAL / $EDITOR
/// 或系统关联程序打开。--json 输出在编辑器退出后打印。
pub async fn platform_profile_open_command(platform_name: &str, json: bool) -> Result<()> {
    let platform = parse_platform_for_open(platform_name)?;
    let template = template_for(platform);
    let ensured = ensure_profiles_file(platform, platform_name, template)?;
    let editor = resolve_editor();
    let label = editor_label(&editor);

    // 人类可读输出在编辑器启动前打印（确保用户能看到"正在打开"提示）
    if !json {
        if ensured.created {
            ColorOutput::success(&format!("已创建 profiles 模板: {}", ensured.path.display()));
        }
        if ensured.registered {
            ColorOutput::success(&format!("已注册平台: {platform_name}"));
        }
        let display_label = match &editor {
            EditorTarget::Configured { source, .. } => *source,
            EditorTarget::SystemAssociation => "系统关联程序",
        };
        ColorOutput::info(&format!(
            "正在用 {display_label} 打开: {}",
            ensured.path.display()
        ));
    }

    // 启动编辑器（Configured 分支阻塞，SystemAssociation 立即返回）
    spawn_editor(&editor, &ensured.path)?;

    // JSON 在编辑器退出后打印，不与 blocking 编辑器的 tty 输出交错
    if json {
        let output = ProfileOpenOutput {
            ok: true,
            platform: platform_name,
            profiles_file: ensured.path.display().to_string(),
            created: ensured.created,
            registered: ensured.registered,
            editor: label,
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CcrError::JsonError)?
        );
    }

    Ok(())
}

// --- 单元测试 ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_prefers_visual_over_editor() {
        let target = resolve_editor_from(Some("nano"), Some("vim"));
        match target {
            EditorTarget::Configured {
                source, program, ..
            } => {
                assert_eq!(source, "$VISUAL");
                assert_eq!(program, "nano");
            }
            EditorTarget::SystemAssociation => panic!("expected Configured"),
        }
    }

    #[test]
    fn resolve_falls_back_to_editor_when_visual_empty() {
        let target = resolve_editor_from(Some("  "), Some("vim"));
        match target {
            EditorTarget::Configured {
                source, program, ..
            } => {
                assert_eq!(source, "$EDITOR");
                assert_eq!(program, "vim");
            }
            EditorTarget::SystemAssociation => panic!("expected Configured"),
        }
    }

    #[test]
    fn resolve_splits_editor_args() {
        let target = resolve_editor_from(None, Some("code --wait"));
        match target {
            EditorTarget::Configured {
                program,
                args,
                source,
            } => {
                assert_eq!(source, "$EDITOR");
                assert_eq!(program, "code");
                assert_eq!(args, vec!["--wait"]);
            }
            EditorTarget::SystemAssociation => panic!("expected Configured"),
        }
    }

    #[test]
    fn resolve_system_when_both_none() {
        let target = resolve_editor_from(None, None);
        assert!(matches!(target, EditorTarget::SystemAssociation));
    }

    #[test]
    fn resolve_system_when_both_empty_strings() {
        let target = resolve_editor_from(Some(""), Some(""));
        assert!(matches!(target, EditorTarget::SystemAssociation));
    }

    #[test]
    fn resolve_system_when_both_whitespace() {
        let target = resolve_editor_from(Some("   "), Some("   "));
        assert!(matches!(target, EditorTarget::SystemAssociation));
    }

    #[test]
    fn editor_label_configured_returns_source() {
        let t = EditorTarget::Configured {
            program: "nano".into(),
            args: vec![],
            source: "$VISUAL",
        };
        assert_eq!(editor_label(&t), "$VISUAL");
    }

    #[test]
    fn editor_label_system_returns_system() {
        assert_eq!(editor_label(&EditorTarget::SystemAssociation), "system");
    }
}
