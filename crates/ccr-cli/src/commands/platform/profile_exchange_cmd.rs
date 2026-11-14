//! Shared CLI command handlers for `ccr <claude|codex|grok> profile export` and `import`.

#![allow(clippy::unused_async)]

use crate::application::{
    ExportProfilesOutcome, ExportProfilesRequest, ImportModeKind, ImportProfilesRequest,
    ProfileImportReport, commit_import, export_profiles, platform_secret_stdin_flag,
    preview_import,
};
use crate::cli::subcommands::profile_args::{ProfileExportActionArgs, ProfileImportActionArgs};
use crate::commands::common::print_next_steps;
use crate::models::Platform;
use ccr_config::ExchangeSecretsMode;
use ccr_core::core::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::{ColorOutput, OutputStatus};
use chrono::Local;
use std::collections::HashSet;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Component, Path, PathBuf};

fn platform_display_name(platform: Platform) -> &'static str {
    match platform {
        Platform::Claude => "Claude",
        Platform::Codex => "Codex",
        Platform::Grok => "Grok",
        _ => platform.short_name(),
    }
}

fn is_interactive_tty() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal() && io::stderr().is_terminal()
}

fn is_safe_bare_cli_arg(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
}

fn emit_stderr_warning(msg: &str) {
    let formatted =
        ColorOutput::format_status(OutputStatus::Warning, msg, io::stderr().is_terminal());
    eprintln!("{formatted}");
}

fn confirm_on_stderr(prompt: &str) -> Result<bool> {
    eprint!("? {prompt} [y/N]: ");
    io::stderr()
        .flush()
        .map_err(|e| CcrError::FileIoError(format!("刷新确认提示失败: {e}")))?;

    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| CcrError::FileIoError(format!("读取确认输入失败: {e}")))?;
    let trimmed = line.trim();
    Ok(trimmed.eq_ignore_ascii_case("y") || trimmed.eq_ignore_ascii_case("yes"))
}

fn is_symlink_or_reparse_point(meta: &fs::Metadata) -> bool {
    let is_link = meta.file_type().is_symlink();
    #[cfg(windows)]
    let is_link = {
        use std::os::windows::fs::MetadataExt;
        is_link || (meta.file_attributes() & 0x0400) != 0
    };
    is_link
}

fn validate_regular_exchange_path(path: &Path) -> Result<(PathBuf, bool)> {
    let mut raw_prefix = PathBuf::new();
    for comp in path.components() {
        raw_prefix.push(comp);
        if matches!(comp, Component::Prefix(_)) {
            continue;
        }
        match fs::symlink_metadata(&raw_prefix) {
            Ok(meta) => {
                if is_symlink_or_reparse_point(&meta) {
                    return Err(CcrError::FileIoError(format!(
                        "拒绝访问符号链接或重解析点路径 '{}'",
                        path.display()
                    )));
                }
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(CcrError::FileIoError(format!(
                    "检查路径 '{}' 失败: {err}",
                    path.display()
                )));
            }
        }
    }

    let abs_path = std::path::absolute(path)
        .map_err(|e| CcrError::FileIoError(format!("解析路径 '{}' 失败: {e}", path.display())))?;

    for ancestor in abs_path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        match fs::symlink_metadata(ancestor) {
            Ok(meta) => {
                if is_symlink_or_reparse_point(&meta) {
                    return Err(CcrError::FileIoError(format!(
                        "拒绝访问符号链接或重解析点路径 '{}'",
                        path.display()
                    )));
                }
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(CcrError::FileIoError(format!(
                    "检查路径 '{}' 失败: {err}",
                    path.display()
                )));
            }
        }
    }

    match fs::symlink_metadata(&abs_path) {
        Ok(meta) => {
            if !meta.file_type().is_file() {
                return Err(CcrError::FileIoError(format!(
                    "目标路径 '{}' 不是普通文件",
                    path.display()
                )));
            }
            Ok((abs_path, true))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok((abs_path, false)),
        Err(err) => Err(CcrError::FileIoError(format!(
            "检查路径 '{}' 失败: {err}",
            path.display()
        ))),
    }
}

fn read_regular_import_file(path: &Path) -> Result<String> {
    let (abs_path, exists) = validate_regular_exchange_path(path)?;
    if !exists {
        return Err(CcrError::FileIoError(format!(
            "无法读取导入文件 '{}': 文件不存在",
            path.display()
        )));
    }
    let mut file = fs::File::open(&abs_path).map_err(|e| {
        CcrError::FileIoError(format!("无法读取导入文件 '{}': {e}", path.display()))
    })?;
    let meta = file.metadata().map_err(|e| {
        CcrError::FileIoError(format!("检查导入文件 '{}' 失败: {e}", path.display()))
    })?;
    if !meta.file_type().is_file() {
        return Err(CcrError::FileIoError(format!(
            "导入路径 '{}' 不是普通文件",
            path.display()
        )));
    }
    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(|e| {
        CcrError::FileIoError(format!("无法读取导入文件 '{}': {e}", path.display()))
    })?;
    Ok(buf)
}

fn write_secret_export_file(target_path: &Path, content: &[u8]) -> Result<()> {
    let (abs_target, _) = validate_regular_exchange_path(target_path)?;
    AtomicWriter::new(&abs_target)
        .secret(true)
        .write(content)
        .map_err(|err| match err {
            CcrError::IoError(io_err) => CcrError::FileIoError(format!(
                "写入导出文件 '{}' 失败: {io_err}",
                target_path.display()
            )),
            other => other,
        })
}

pub async fn platform_profile_export_command(
    platform_name: &str,
    args: ProfileExportActionArgs,
    auto_yes: bool,
) -> Result<()> {
    let platform: Platform = platform_name.parse()?;
    let outcome = export_profiles(ExportProfilesRequest {
        platform,
        names: args.names,
        tags: args.tags,
        include_secrets: !args.no_secrets,
    })?;

    if args.stdout {
        if outcome.contains_secrets {
            emit_stderr_warning("导出的标准输出流包含明文内联密钥，请注意重定向目标权限与传输安全");
            for warning in outcome.warnings.iter().skip(1) {
                emit_stderr_warning(warning);
            }
        } else {
            for warning in &outcome.warnings {
                emit_stderr_warning(warning);
            }
        }

        let mut stdout = io::stdout().lock();
        stdout
            .write_all(outcome.toml_content.as_bytes())
            .map_err(|e| CcrError::FileIoError(format!("写入标准输出失败: {e}")))?;
        stdout
            .flush()
            .map_err(|e| CcrError::FileIoError(format!("刷新标准输出失败: {e}")))?;
        return Ok(());
    }

    let target_path = match args.output {
        Some(path) => path,
        None => {
            let timestamp = Local::now().format("%Y%m%d-%H%M%S");
            PathBuf::from(format!(
                "./ccr-{}-profiles-{timestamp}.toml",
                platform.short_name()
            ))
        }
    };

    let (_, already_exists) = validate_regular_exchange_path(&target_path)?;
    if already_exists && !auto_yes {
        if args.json || !is_interactive_tty() {
            return Err(CcrError::ResourceAlreadyExists(format!(
                "输出文件 '{}' 已存在；如需覆盖请附加 --yes，或指定其他 -o 路径",
                target_path.display()
            )));
        }

        if !confirm_on_stderr(&format!(
            "输出文件 '{}' 已存在，确认覆盖?",
            target_path.display()
        ))? {
            println!("已取消导出");
            return Ok(());
        }
    }

    write_secret_export_file(&target_path, outcome.toml_content.as_bytes())?;

    let output_path_str = target_path.display().to_string();
    if args.json {
        let report = outcome.to_json_report(output_path_str);
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(CcrError::JsonError)?
        );
        return Ok(());
    }

    render_export_human_output(&outcome, &output_path_str);
    Ok(())
}

fn render_export_human_output(outcome: &ExportProfilesOutcome, output_path: &str) {
    ColorOutput::success(&format!(
        "已导出 {} 个 {} profile",
        outcome.exported_names.len(),
        platform_display_name(outcome.platform)
    ));
    ColorOutput::key_value("输出文件", output_path, 2);
    ColorOutput::key_value("导出条目", &outcome.exported_names.join(", "), 2);

    let mode_desc = match (outcome.secrets_mode, outcome.contains_secrets) {
        (ExchangeSecretsMode::Redacted, _) => "redacted (敏感字段已彻底移除)",
        (ExchangeSecretsMode::Included, true) => "included (文件权限已限制为当前用户可读写)",
        (ExchangeSecretsMode::Included, false) => {
            "included (所选配置不含内联密钥；目标机器需单独完成官方账号登录或设置环境变量)"
        }
    };
    ColorOutput::key_value("密钥模式", mode_desc, 2);

    for warning in &outcome.warnings {
        ColorOutput::warning(warning);
    }

    if is_safe_bare_cli_arg(output_path) {
        let cmd = format!(
            "ccr {} profile import {} --dry-run",
            outcome.platform.short_name(),
            output_path
        );
        print_next_steps(&[("预览导入该交换文件", cmd.as_str())]);
    } else {
        let cmd = format!("ccr {} profile list", outcome.platform.short_name());
        print_next_steps(&[("查看当前平台 Profile 列表", cmd.as_str())]);
    }
}

pub async fn platform_profile_import_command(
    platform_name: &str,
    args: ProfileImportActionArgs,
    auto_yes: bool,
) -> Result<()> {
    let platform: Platform = platform_name.parse()?;
    let from_stdin = args.stdin || args.file.as_deref() == Some(Path::new("-"));
    let input_path_hint = if from_stdin {
        None
    } else {
        args.file.as_ref().map(|p| p.display().to_string())
    };

    let content = if from_stdin {
        if io::stdin().is_terminal() {
            return Err(CcrError::ValidationError(
                "标准输入连接到交互式终端，请通过管道或输入重定向传入 TOML 内容，或改传文件路径"
                    .into(),
            ));
        }
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| CcrError::FileIoError(format!("无法从标准输入读取导入内容: {e}")))?;
        buf
    } else {
        let path = args.file.as_deref().ok_or_else(|| {
            CcrError::ValidationError("请指定要导入的文件路径或使用 --stdin".into())
        })?;
        read_regular_import_file(path)?
    };

    let mode = if args.replace {
        ImportModeKind::Replace
    } else {
        ImportModeKind::Merge
    };

    let mut request = ImportProfilesRequest {
        platform,
        content,
        mode,
        on_conflict: args.on_conflict.to_application_strategy(),
        dedup: args.dedup,
        dry_run: args.dry_run,
        no_apply: args.no_apply,
        expected_revision: args.expected_revision,
    };

    if args.dry_run {
        let report = preview_import(&request)?;
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(CcrError::JsonError)?
            );
            return Ok(());
        }
        render_import_human_output(platform, &report, input_path_hint.as_deref());
        return Ok(());
    }

    if args.replace && !auto_yes {
        // Complete full validation & active-runtime checks BEFORE prompting or gating.
        // Never consult catalog `settings.skip_confirmation`.
        let preview = preview_import(&request)?;

        if from_stdin {
            return Err(CcrError::ValidationError(
                "从标准输入执行 --replace 整体替换无法进行交互式确认；请先附加 --dry-run 预览，确认无误后显式添加 --yes".into(),
            ));
        }

        if args.json || !is_interactive_tty() {
            return Err(CcrError::ValidationError(
                "整体替换模式 (--replace) 属于破坏性操作，非交互环境或 --json 模式必须显式传入 -y/--yes（建议先用 --dry-run 预览）".into(),
            ));
        }

        emit_stderr_warning(&format!(
            "整体替换模式 (--replace) 将以导入文件为准覆盖本地 {} 目录：",
            platform_display_name(platform)
        ));
        if !preview.added_names.is_empty() {
            eprintln!("  新增条目: {}", preview.added_names.join(", "));
        }
        if !preview.updated_names.is_empty() {
            eprintln!("  更新条目: {}", preview.updated_names.join(", "));
        }
        if !preview.removed_names.is_empty() {
            eprintln!("  将被删除: {}", preview.removed_names.join(", "));
        }
        if !preview.base_revision.is_empty() {
            eprintln!("  安全保障: 写入前将自动生成 0600 备份文件");
        }

        if !confirm_on_stderr("确认执行整体替换?")? {
            println!("已取消导入");
            return Ok(());
        }

        if request.expected_revision.is_none() {
            request.expected_revision = Some(preview.base_revision.clone());
        }
    }

    let report = commit_import(request)?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(CcrError::JsonError)?
        );
        return Ok(());
    }

    render_import_human_output(platform, &report, input_path_hint.as_deref());
    Ok(())
}

fn render_import_human_output(
    platform: Platform,
    report: &ProfileImportReport,
    input_path_hint: Option<&str>,
) {
    let display_name = platform_display_name(platform);
    let is_replace = report.mode == ImportModeKind::Replace.as_str();

    if report.dry_run {
        ColorOutput::success("导入预览完成（dry-run，未写入任何更改，未生成备份）");
    } else if !report.changed {
        ColorOutput::success(&format!(
            "导入完成：{display_name} profiles 无新增或变更（未生成备份）"
        ));
    } else if is_replace {
        ColorOutput::success(&format!(
            "已整体替换导入 {display_name} profiles（新增 {}, 更新 {}, 无变化 {}, 移除 {}）",
            report.counts.added,
            report.counts.updated,
            report.counts.unchanged,
            report.counts.removed
        ));
    } else {
        ColorOutput::success(&format!(
            "已合并导入 {display_name} profiles（新增 {}, 更新 {}, 无变化 {}, 改名 {}, 跳过 {}）",
            report.counts.added,
            report.counts.updated,
            report.counts.unchanged,
            report.counts.renamed,
            report.counts.skipped
        ));
    }

    let platform_line = if report.dry_run {
        match report.on_conflict.as_deref() {
            Some(strategy) => format!(
                "{} (来源格式: {}, 模式: {}, 冲突策略: {})",
                report.platform,
                report.source_format.as_str(),
                report.mode,
                strategy
            ),
            None => format!(
                "{} (来源格式: {}, 模式: {})",
                report.platform,
                report.source_format.as_str(),
                report.mode
            ),
        }
    } else {
        report.platform.clone()
    };
    ColorOutput::key_value("目标平台", &platform_line, 2);

    let counts_line = if is_replace {
        let verb = if report.dry_run {
            "将移除本地配置"
        } else {
            "已移除本地配置"
        };
        format!(
            "共 {} 条输入 — 新增 {}, 更新 {}, 无变化 {}, {verb} {}",
            report.counts.total_input,
            report.counts.added,
            report.counts.updated,
            report.counts.unchanged,
            report.counts.removed
        )
    } else {
        format!(
            "共 {} 条输入 — 新增 {}, 更新 {}, 无变化 {}, 改名 {}, 冲突跳过 {}, 重复跳过 {}",
            report.counts.total_input,
            report.counts.added,
            report.counts.updated,
            report.counts.unchanged,
            report.counts.renamed,
            report.counts.skipped_conflict,
            report.counts.skipped_duplicate
        )
    };
    ColorOutput::key_value("变更统计", &counts_line, 2);

    let missing_set: HashSet<&str> = report.missing_secrets.iter().map(String::as_str).collect();
    let preserved_set: HashSet<&str> = report
        .preserved_local_secrets
        .iter()
        .map(String::as_str)
        .collect();

    if !report.added_names.is_empty() {
        let items = report
            .added_names
            .iter()
            .map(|name| {
                if missing_set.contains(name.as_str()) {
                    if report.dry_run {
                        format!("{name} (缺内联密钥，导入后将设为禁用)")
                    } else {
                        format!("{name} (已禁用：待补全密钥)")
                    }
                } else {
                    name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        ColorOutput::key_value("新增条目", &items, 2);
    }

    if !report.updated_names.is_empty() {
        let items = report
            .updated_names
            .iter()
            .map(|name| {
                if preserved_set.contains(name.as_str()) {
                    format!("{name} (保留本地既有密钥)")
                } else {
                    name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        ColorOutput::key_value("更新条目", &items, 2);
    }

    if !report.unchanged_names.is_empty() {
        ColorOutput::key_value("无变化项", &report.unchanged_names.join(", "), 2);
    }

    if !report.renamed_items.is_empty() {
        let items = report
            .renamed_items
            .iter()
            .map(|item| {
                if missing_set.contains(item.to.as_str()) {
                    format!("{} -> {} (已禁用：待补全密钥)", item.from, item.to)
                } else {
                    format!("{} -> {}", item.from, item.to)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        ColorOutput::key_value("改名导入", &items, 2);
    }

    if !report.skipped_conflicts.is_empty() {
        let detail = format!(
            "{} (本地同名配置已保留；可改用 --on-conflict overwrite 或 --on-conflict rename)",
            report.skipped_conflicts.join(", ")
        );
        ColorOutput::key_value("冲突跳过", &detail, 2);
    }

    if !report.skipped_duplicates.is_empty() {
        let detail = report
            .skipped_duplicates
            .iter()
            .map(|dup| format!("{} (与 '{}' 内容完全一致)", dup.name, dup.duplicate_of))
            .collect::<Vec<_>>()
            .join(", ");
        ColorOutput::key_value("重复跳过", &detail, 2);
    }

    if !report.removed_names.is_empty() {
        let label = if report.dry_run {
            "将被移除"
        } else {
            "已移除项"
        };
        ColorOutput::key_value(label, &report.removed_names.join(", "), 2);
    }

    if let Some(reapplied) = report.reapplied_profile.as_deref() {
        let text = if report.dry_run {
            format!("将重新应用当前生效配置 {reapplied}")
        } else {
            format!("已重新应用当前生效配置 {reapplied}")
        };
        ColorOutput::key_value("运行时同步", &text, 2);
    }

    if let Some(backup) = report.backup_path.as_deref() {
        ColorOutput::key_value("自动备份", backup, 2);
    }

    if !report.dry_run && !report.revision.is_empty() {
        ColorOutput::key_value("目录版本", &report.revision, 2);
    }

    for warning in &report.warnings {
        ColorOutput::warning(warning);
    }

    render_import_next_steps(platform, report, input_path_hint, is_replace);
}

fn render_import_next_steps(
    platform: Platform,
    report: &ProfileImportReport,
    input_path_hint: Option<&str>,
    is_replace: bool,
) {
    let short = platform.short_name();
    if report.dry_run {
        if let Some(path) = input_path_hint.filter(|p| is_safe_bare_cli_arg(p)) {
            if is_replace {
                let cmd = format!("ccr {short} profile import {path} --replace --yes");
                print_next_steps(&[("确认移除差异配置并执行整体替换", cmd.as_str())]);
            } else {
                let cmd = format!("ccr {short} profile import {path}");
                print_next_steps(&[("确认无误后执行合并导入", cmd.as_str())]);
            }
        } else {
            let cmd = format!("ccr {short} profile list");
            print_next_steps(&[("查看当前平台 Profile 列表", cmd.as_str())]);
        }
        return;
    }

    if let Some(first_missing) = report.missing_secrets.first() {
        if is_safe_bare_cli_arg(first_missing) {
            let secret_flag = platform_secret_stdin_flag(platform);
            let cmd = format!("ccr {short} profile edit {first_missing} {secret_flag} --enable");
            print_next_steps(&[("为待补全密钥的新增配置录入密钥", cmd.as_str())]);
        } else {
            let cmd = format!("ccr {short} profile list");
            print_next_steps(&[("查看更新后的 Profile 列表", cmd.as_str())]);
        }
        return;
    }

    if report.reapplied_profile.is_some() {
        let cmd = format!("ccr {short} profile current");
        print_next_steps(&[("检查当前生效状态", cmd.as_str())]);
        return;
    }

    let cmd = format!("ccr {short} profile list");
    print_next_steps(&[("查看更新后的 Profile 列表", cmd.as_str())]);
}
