//! 📥 codex auth import 命令实现
//!
//! 从 JSON 文件导入账号 (交互式选择导入文件)。
//! 自动检测加密/明文格式，加密文件会提示输入密码解密。

#![allow(clippy::unused_async)]

use crate::commands::common::print_next_steps;
use crate::models::ImportMode;
use crate::services::CodexAuthService;
use ccr_codex::models::codex_auth::ImportFormat;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// 获取跨平台的 Downloads 目录
fn get_downloads_dir() -> Result<PathBuf> {
    dirs::download_dir()
        .ok_or_else(|| CcrError::ConfigError("无法获取 Downloads 目录路径".to_string()))
}

/// 扫描 Downloads 目录中的导出文件
fn scan_downloads_for_exports() -> Result<Vec<PathBuf>> {
    let downloads = get_downloads_dir()?;

    let mut files: Vec<(PathBuf, std::time::SystemTime)> = fs::read_dir(&downloads)
        .map_err(|e| CcrError::FileIoError(format!("读取 Downloads 目录失败: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            name_str.starts_with("codex-auth-export-") && name_str.ends_with(".json")
        })
        .filter_map(|entry| {
            let path = entry.path();
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((path, modified))
        })
        .collect();

    files.sort_by_key(|entry| std::cmp::Reverse(entry.1));
    Ok(files.into_iter().map(|(path, _)| path).collect())
}

/// 读取用户输入的路径
fn read_user_path() -> Option<String> {
    print!("  → ");
    io::stdout().flush().ok()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 处理加密文件的导入
///
/// 显示信封头信息，循环提示密码直到解密成功或用户取消 (Ctrl+C)。
fn import_encrypted(
    service: &CodexAuthService,
    content: &str,
    mode: ImportMode,
    force: bool,
) -> Result<()> {
    // 解析信封头以显示预览信息
    if let Ok(envelope) = serde_json::from_str::<serde_json::Value>(content) {
        let account_count = envelope
            .get("account_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let exported_at = envelope
            .get("exported_at")
            .and_then(|v| v.as_str())
            .unwrap_or("未知");

        ColorOutput::info("检测到加密文件");
        ColorOutput::key_value("账号数量", &account_count.to_string(), 2);
        ColorOutput::key_value("导出时间", exported_at, 2);
    }

    // 密码重试循环（无限次，用户可 Ctrl+C 取消）
    loop {
        let password = rpassword::prompt_password("请输入密码: ")
            .map_err(|e| CcrError::FileIoError(format!("读取密码失败: {}", e)))?;

        match service.import_accounts_encrypted(content, &password, mode, force) {
            Ok(result) => {
                print_import_result(&result, mode, force);
                return Ok(());
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("密码错误或文件已损坏") {
                    println!();
                    ColorOutput::error("密码错误或文件已损坏，请重试 (Ctrl+C 取消)");
                    println!();
                    continue;
                }
                // 非密码错误，直接返回
                return Err(e);
            }
        }
    }
}

/// 处理明文文件的导入
fn import_plaintext(
    service: &CodexAuthService,
    content: &str,
    mode: ImportMode,
    force: bool,
) -> Result<()> {
    println!();
    ColorOutput::warning("检测到未加密的导出文件");

    match service.import_accounts(content, mode, force) {
        Ok(result) => {
            print_import_result(&result, mode, force);
            Ok(())
        }
        Err(e) => {
            ColorOutput::error(&format!("导入失败: {}", e));
            let err_msg = e.to_string();
            if err_msg.contains("解析") {
                ColorOutput::info("请确保文件是有效的 JSON 格式");
            }
            Err(e)
        }
    }
}

/// 打印导入结果
fn print_import_result(
    result: &ccr_codex::models::codex_auth::ImportResult,
    mode: ImportMode,
    force: bool,
) {
    if result.added + result.updated > 0 {
        ColorOutput::success("已导入账号");
    } else {
        ColorOutput::info("没有账号被新增或更新");
    }

    if result.added > 0 {
        ColorOutput::key_value("新增账号", &result.added.to_string(), 2);
    }
    if result.updated > 0 {
        ColorOutput::key_value("更新账号", &result.updated.to_string(), 2);
    }
    if result.skipped > 0 {
        ColorOutput::key_value("跳过账号", &result.skipped.to_string(), 2);
    }
    if !result.overwritten.is_empty() {
        ColorOutput::warning(&format!("覆盖账号: {}", result.overwritten.len()));
        for name in &result.overwritten {
            println!("  • {name}");
        }
    }

    match mode {
        ImportMode::Merge => {
            if force {
                ColorOutput::key_value("模式", "合并 (强制覆盖已存在的账号)", 2);
            } else {
                ColorOutput::key_value("模式", "合并 (跳过已存在的账号)", 2);
            }
        }
        ImportMode::Replace => {
            ColorOutput::key_value("模式", "替换 (覆盖同名账号)", 2);
        }
    }

    print_next_steps(&[("查看账号", "ccr codex auth list")]);
}

/// 📥 从 JSON 文件导入账号
pub async fn import_command(replace: bool, force: bool) -> Result<()> {
    let service = CodexAuthService::new()?;

    let exports = scan_downloads_for_exports()?;
    let downloads_dir = get_downloads_dir()?;
    let default_file = exports.first().cloned();

    if let Some(ref file) = default_file {
        ColorOutput::key_value("默认导入文件", &file.display().to_string(), 2);
        if exports.len() > 1 {
            ColorOutput::info(&format!(
                "(在 Downloads 中找到 {} 个导出文件，已选择最新的)",
                exports.len()
            ));
        }
    } else {
        ColorOutput::key_value("默认导入目录", &downloads_dir.display().to_string(), 2);
        ColorOutput::warning("未在 Downloads 中找到导出文件");
    }

    let default_file_for_task = default_file.clone();
    let import_path = tokio::task::spawn_blocking(move || -> Result<PathBuf> {
        print!("是否修改导入路径? [y/N]: ");
        io::stdout()
            .flush()
            .map_err(|e| CcrError::FileIoError(e.to_string()))?;

        let mut confirm = String::new();
        io::stdin()
            .read_line(&mut confirm)
            .map_err(|e| CcrError::FileIoError(e.to_string()))?;

        if confirm.trim().eq_ignore_ascii_case("y") || confirm.trim().eq_ignore_ascii_case("yes") {
            println!("请输入导入文件路径 (JSON 文件):");
            match read_user_path() {
                Some(custom_path) => Ok(PathBuf::from(custom_path)),
                None => {
                    if let Some(file) = default_file_for_task {
                        ColorOutput::info("使用默认文件");
                        Ok(file)
                    } else {
                        ColorOutput::error("未指定文件且无默认文件可用");
                        Ok(PathBuf::new())
                    }
                }
            }
        } else {
            match default_file_for_task {
                Some(file) => Ok(file),
                None => {
                    ColorOutput::error("在 Downloads 目录中未找到导出文件");
                    ColorOutput::info("重新运行导入时输入 'y' 可手动指定文件路径");
                    print_next_steps(&[("导出账号", "ccr codex auth export")]);
                    Ok(PathBuf::new())
                }
            }
        }
    })
    .await
    .map_err(|e| CcrError::FileIoError(format!("读取导入路径失败: {}", e)))??;

    if import_path.as_os_str().is_empty() {
        return Ok(());
    }

    if !import_path.exists() {
        ColorOutput::error(&format!("文件不存在: {}", import_path.display()));
        return Ok(());
    }

    if import_path.extension().is_some_and(|ext| ext != "json") {
        ColorOutput::warning("文件扩展名不是 .json，继续尝试导入...");
    }

    let content = fs::read_to_string(&import_path)
        .map_err(|e| CcrError::FileIoError(format!("读取文件失败: {}", e)))?;

    let mode = if replace {
        ImportMode::Replace
    } else {
        ImportMode::Merge
    };

    // 自动检测文件格式
    match CodexAuthService::detect_import_format(&content) {
        ImportFormat::EncryptedV2 => {
            import_encrypted(&service, &content, mode, force)?;
        }
        ImportFormat::PlaintextV1 => {
            import_plaintext(&service, &content, mode, force)?;
        }
        ImportFormat::Unknown => {
            ColorOutput::error("无法识别的文件格式");
            ColorOutput::info("请确保文件是由 ccr codex auth export 导出的 JSON 文件");
            print_next_steps(&[("查看导出帮助", "ccr codex auth export --help")]);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn p6_plaintext_import_returns_service_rejection() {
        let home = crate::test_support::TestHome::new();
        fs::write(
            home.codex_dir().join("config.toml"),
            "cli_auth_credentials_store = \"file\"\n",
        )
        .unwrap();
        let service = CodexAuthService::from_dirs(
            home.root().join("platforms/codex"),
            home.codex_dir().to_path_buf(),
        );
        let error =
            import_plaintext(&service, "{ invalid json }", ImportMode::Merge, false).unwrap_err();
        assert_ne!(error.exit_code(), 0);
    }

    #[test]
    fn import_result_distinguishes_partial_skip_only_overwrite_and_empty() {
        use ccr_codex::models::codex_auth::ImportResult;
        use std::process::Command;

        const PROBE: &str = "CCR_C2_IMPORT_RESULT_PROBE";
        if let Ok(case) = std::env::var(PROBE) {
            let (result, force) = match case.as_str() {
                "partial" => (
                    ImportResult {
                        added: 1,
                        skipped: 1,
                        ..Default::default()
                    },
                    false,
                ),
                "empty" => (ImportResult::default(), false),
                "skip" => (
                    ImportResult {
                        skipped: 1,
                        ..Default::default()
                    },
                    false,
                ),
                "overwrite" => (
                    ImportResult {
                        updated: 1,
                        overwritten: vec!["teacher".into()],
                        ..Default::default()
                    },
                    true,
                ),
                _ => panic!("unknown fixture case"),
            };
            print_import_result(&result, ImportMode::Merge, force);
            return;
        }
        for case in ["partial", "skip", "overwrite", "empty"] {
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "commands::codex::auth::import::tests::import_result_distinguishes_partial_skip_only_overwrite_and_empty", "--nocapture"])
                .env(PROBE, case).env("NO_COLOR", "1").env_remove("CLICOLOR_FORCE")
                .output().unwrap();
            assert!(output.status.success());
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(!text.contains("[OK]"));
            assert!(!text.contains('\u{1b}'));
            assert_eq!(text.matches("下一步").count(), 1);
            match case {
                "partial" => {
                    assert!(text.contains("成功: 已导入账号\n  新增账号: 1\n  跳过账号: 1"));
                    assert!(text.contains("模式: 合并 (跳过已存在的账号)"));
                }
                "skip" => {
                    assert!(text.contains("没有账号被新增或更新\n  跳过账号: 1"));
                    assert!(!text.contains("成功:"));
                }
                "empty" => {
                    assert!(text.contains("没有账号被新增或更新"));
                    assert!(!text.contains("成功:"));
                    assert!(!text.contains("警告:"));
                }
                _ => {
                    assert!(text.contains("警告: 覆盖账号: 1\n  • teacher"));
                    assert!(text.contains("模式: 合并 (强制覆盖已存在的账号)"));
                }
            }
        }
    }

    #[test]
    fn import_partial_merge_preserves_existing_account_and_adds_metadata() {
        let home = crate::test_support::TestHome::new();
        fs::write(
            home.codex_dir().join("config.toml"),
            "cli_auth_credentials_store = \"file\"\n",
        )
        .unwrap();
        let service = CodexAuthService::from_dirs(
            home.root().join("platforms/codex"),
            home.codex_dir().to_path_buf(),
        );
        let bundle = |accounts| {
            serde_json::json!({"version":"1.0", "exported_at":"2026-01-01T00:00:00Z", "accounts":accounts}).to_string()
        };
        let account =
            serde_json::json!({"account_id":"existing-id", "saved_at":"2026-01-01T00:00:00Z"});
        service
            .import_accounts(
                &bundle(serde_json::json!({"teacher":account})),
                ImportMode::Merge,
                false,
            )
            .unwrap();
        let registry_path = home.root().join("platforms/codex/auth_registry.toml");
        let before = fs::read(&registry_path).unwrap();
        let skipped = service
            .import_accounts(
                &bundle(serde_json::json!({"teacher":account})),
                ImportMode::Merge,
                false,
            )
            .unwrap();
        assert_eq!((skipped.added, skipped.updated, skipped.skipped), (0, 0, 1));
        assert_eq!(fs::read(&registry_path).unwrap(), before);
        let result = service.import_accounts(&bundle(serde_json::json!({"teacher":account,"new-account":{"account_id":"new-id", "saved_at":"2026-01-01T00:00:00Z"}})), ImportMode::Merge, false).unwrap();
        assert_eq!((result.added, result.updated, result.skipped), (1, 0, 1));
        assert!(result.overwritten.is_empty());
        let registry = service.load_registry().unwrap();
        assert_eq!(registry.accounts["teacher"].account_id, "existing-id");
        assert_eq!(registry.accounts["new-account"].account_id, "new-id");
        assert!(!home.codex_dir().join("auth.json").exists());
    }
}
