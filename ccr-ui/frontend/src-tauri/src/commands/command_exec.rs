//! 命令执行模块 — CCR CLI 命令白名单执行。

use serde_json::Value;
use tokio::process::Command;

/// 允许执行的 CCR 子命令白名单
const ALLOWED_COMMANDS: &[&str] = &[
    "list", "switch", "add", "delete", "rename", "duplicate",
    "show", "validate", "export", "import", "history", "version",
    "help", "backup", "restore", "diff", "status",
];

/// 每个白名单命令的简要描述
const COMMAND_DESCRIPTIONS: &[(&str, &str)] = &[
    ("list",      "列出所有配置"),
    ("switch",    "切换到指定配置"),
    ("add",       "添加新配置"),
    ("delete",    "删除配置"),
    ("rename",    "重命名配置"),
    ("duplicate", "复制配置"),
    ("show",      "显示配置内容"),
    ("validate",  "校验配置文件"),
    ("export",    "导出配置"),
    ("import",    "导入配置"),
    ("history",   "查看操作历史"),
    ("version",   "显示版本信息"),
    ("help",      "显示帮助信息"),
    ("backup",    "备份配置"),
    ("restore",   "恢复配置"),
    ("diff",      "比较配置差异"),
    ("status",    "显示当前状态"),
];

/// 校验子命令是否在白名单中
fn validate_command(command: &str) -> Result<(), String> {
    if ALLOWED_COMMANDS.contains(&command) {
        Ok(())
    } else {
        Err(format!(
            "命令 '{}' 不在允许列表中。允许的命令: {}",
            command,
            ALLOWED_COMMANDS.join(", ")
        ))
    }
}

/// 执行白名单内的 CCR CLI 子命令并返回输出
///
/// 返回 `{ success, stdout, stderr, exit_code }`
#[tauri::command]
pub async fn execute_ccr_command(
    command: String,
    args: Option<Vec<String>>,
) -> Result<Value, String> {
    validate_command(&command)?;

    let mut cmd = Command::new("ccr");
    cmd.arg(&command);
    if let Some(extra_args) = args {
        cmd.args(&extra_args);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "CCR 二进制未找到，请确认已安装并在 PATH 中".to_string()
            } else {
                format!("执行失败: {e}")
            }
        })?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(serde_json::json!({
        "success": output.status.success(),
        "stdout": stdout,
        "stderr": stderr,
        "exit_code": exit_code,
    }))
}

/// 返回白名单命令列表及其描述
///
/// 返回 `[{ name, description }, ...]`
#[tauri::command]
pub async fn list_ccr_commands() -> Result<Value, String> {
    let commands: Vec<Value> = COMMAND_DESCRIPTIONS
        .iter()
        .map(|(name, description)| {
            serde_json::json!({
                "name": name,
                "description": description,
            })
        })
        .collect();

    Ok(Value::Array(commands))
}

/// 执行 `ccr help <command>` 并返回帮助文本
#[tauri::command]
pub async fn get_ccr_command_help(command: String) -> Result<Value, String> {
    validate_command(&command)?;

    let output = Command::new("ccr")
        .args(["help", &command])
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "CCR 二进制未找到，请确认已安装并在 PATH 中".to_string()
            } else {
                format!("执行失败: {e}")
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(serde_json::json!({
        "command": command,
        "help": stdout,
        "stderr": stderr,
        "success": output.status.success(),
    }))
}
