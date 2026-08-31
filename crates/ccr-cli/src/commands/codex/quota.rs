//! 💰 Codex 配额查询命令
//!
//! 查询 Codex 账号的 API 配额余额（5h 窗口 / 周限额）

use crate::commands::common::new_table;
use crate::services::CodexQuotaService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;

/// 执行配额查询命令
pub async fn quota_command(account: Option<&str>, json_output: bool, refresh: bool) -> Result<()> {
    let service = CodexQuotaService::new()?;

    let quotas = if let Some(name) = account {
        if refresh {
            vec![service.fetch_account_quota_force_refresh(name).await]
        } else {
            vec![service.fetch_account_quota(name).await]
        }
    } else {
        if refresh {
            service.fetch_all_quotas_force_refresh().await
        } else {
            service.fetch_all_quotas().await
        }
    };

    if json_output {
        let json = serde_json::to_string_pretty(&quotas)
            .map_err(|e| CcrError::ConfigError(format!("JSON 序列化失败: {}", e)))?;
        println!("{}", json);
        return Ok(());
    }

    if quotas.is_empty() {
        ColorOutput::warning("未找到已保存的 Codex 账号");
        ColorOutput::info("使用 `ccr codex auth save <name>` 保存账号后再查询配额");
        return Ok(());
    }

    // 表格输出
    use comfy_table::{Cell, Color as CColor, ContentArrangement};

    let mut table = new_table();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("账号"),
        Cell::new("5h限额"),
        Cell::new("周限额"),
        Cell::new("订阅"),
        Cell::new("状态"),
    ]);

    for aq in &quotas {
        if let Some(ref quota) = aq.quota {
            let h_pct = format!("{}%", quota.hourly_percentage);
            let w_pct = format!("{}%", quota.weekly_percentage);
            let plan = quota.plan_type.as_deref().unwrap_or("-");

            let h_color = pct_table_color(quota.hourly_percentage);
            let w_color = pct_table_color(quota.weekly_percentage);

            let name_display = if let Some(ref email) = aq.email {
                format!("{} ({})", aq.account_name, email)
            } else {
                aq.account_name.clone()
            };

            table.add_row(vec![
                Cell::new(name_display),
                Cell::new(h_pct).fg(h_color),
                Cell::new(w_pct).fg(w_color),
                Cell::new(plan),
                Cell::new("✓").fg(CColor::Green),
            ]);
        } else {
            let error_msg = aq.error.as_deref().unwrap_or("未知错误");
            let short_err = if error_msg.len() > 40 {
                format!("{}...", &error_msg[..40])
            } else {
                error_msg.to_string()
            };

            table.add_row(vec![
                Cell::new(&aq.account_name),
                Cell::new("-"),
                Cell::new("-"),
                Cell::new("-"),
                Cell::new(format!("✗ {}", short_err)).fg(CColor::Red),
            ]);
        }
    }

    ColorOutput::success("Codex 账号配额余额");
    println!("{table}");

    // 显示重置时间
    for aq in &quotas {
        if let Some(ref quota) = aq.quota {
            let mut reset_info = Vec::new();
            if let Some(t) = quota.hourly_reset_time {
                reset_info.push(format!(
                    "5h重置: {}",
                    CodexQuotaService::format_reset_duration(t)
                ));
            }
            if let Some(t) = quota.weekly_reset_time {
                reset_info.push(format!(
                    "周重置: {}",
                    CodexQuotaService::format_reset_duration(t)
                ));
            }
            if !reset_info.is_empty() {
                ColorOutput::info(&format!(
                    "  {} → {}",
                    aq.account_name,
                    reset_info.join(" | ")
                ));
            }
        }
    }

    Ok(())
}

/// 百分比对应表格颜色
fn pct_table_color(pct: i32) -> comfy_table::Color {
    if pct >= 60 {
        comfy_table::Color::Green
    } else if pct >= 30 {
        comfy_table::Color::Yellow
    } else {
        comfy_table::Color::Red
    }
}
