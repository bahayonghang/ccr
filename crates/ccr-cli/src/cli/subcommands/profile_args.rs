use clap::{Args, ValueEnum};
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct ProfileCreateActionArgs {
    /// Profile name
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long = "base-url")]
    pub base_url: Option<String>,
    #[arg(long = "auth-token", hide = true)]
    pub auth_token: Option<String>,
    /// 从标准输入读取密钥，避免进入 shell 历史
    #[arg(long = "auth-token-stdin", conflicts_with = "auth_token")]
    pub auth_token_stdin: bool,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long = "small-fast-model")]
    pub small_fast_model: Option<String>,
    #[arg(long)]
    pub provider: Option<String>,
    #[arg(long = "provider-type")]
    pub provider_type: Option<String>,
    #[arg(long)]
    pub account: Option<String>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long = "auth-mode")]
    pub auth_mode: Option<String>,
    /// 补充字段赋值，形式为 field=value，可重复
    #[arg(long = "set", value_name = "FIELD=VALUE")]
    pub set: Vec<String>,
    /// 仅当 catalog 仍是这个 revision 时写入
    #[arg(long = "expected-revision")]
    pub expected_revision: Option<String>,
    #[arg(long)]
    pub disabled: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileSetFieldActionArgs {
    /// Profile name
    pub name: String,
    /// Field name in snake_case
    pub field: String,
    /// String value
    #[arg(long, conflicts_with_all = ["value_json", "clear"])]
    pub value: Option<String>,
    /// JSON value, useful for array fields such as tags
    #[arg(long = "value-json", conflicts_with_all = ["value", "clear"])]
    pub value_json: Option<String>,
    /// Clear the field
    #[arg(long, conflicts_with_all = ["value", "value_json"])]
    pub clear: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileNameJsonActionArgs {
    /// Profile name
    pub name: String,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileDisableActionArgs {
    /// Profile name
    pub name: String,
    /// Allow disabling the current profile
    #[arg(long)]
    pub force: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileDeleteActionArgs {
    /// Profile name
    pub name: String,
    /// Allow deleting the currently active profile
    #[arg(long)]
    pub force: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileEditActionArgs {
    /// Profile 名称
    pub name: String,
    /// 改名后的新名称
    #[arg(long)]
    pub rename: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long = "base-url")]
    pub base_url: Option<String>,
    #[arg(long)]
    pub provider: Option<String>,
    /// 整体替换标签，可重复
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long, conflicts_with = "disable")]
    pub enable: bool,
    #[arg(long, conflicts_with = "enable")]
    pub disable: bool,
    /// 补充字段赋值，形式为 field=value，可重复
    #[arg(long = "set", value_name = "FIELD=VALUE")]
    pub set: Vec<String>,
    /// 清空字段，可重复
    #[arg(long = "clear", value_name = "FIELD")]
    pub clear: Vec<String>,
    /// 从标准输入读取密钥，避免进入 shell 历史
    #[arg(long = "auth-token-stdin", conflicts_with = "clear_auth_token")]
    pub auth_token_stdin: bool,
    /// 清除已保存的密钥
    #[arg(long = "clear-auth-token", conflicts_with = "auth_token_stdin")]
    pub clear_auth_token: bool,
    /// 仅当 catalog 仍是这个 revision 时写入
    #[arg(long = "expected-revision")]
    pub expected_revision: Option<String>,
    /// 只修改存储，不重新应用当前生效的运行时
    #[arg(long = "no-apply")]
    pub no_apply: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct CodexProfileEditActionArgs {
    #[command(flatten)]
    pub common: ProfileEditActionArgs,
    /// Codex model_reasoning_effort
    #[arg(long = "reasoning-effort")]
    pub reasoning_effort: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileOffActionArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImportConflictStrategy {
    /// 保留本地同名配置不变，跳过导入项（默认）
    #[default]
    Skip,
    /// 用导入项整条更新本地同名配置（保留本地 usage_count；若命中当前激活项默认自动重应用运行时）
    Overwrite,
    /// 保留本地同名配置，将导入项自动改名为 <name>-imported（或递增序号）新增入库
    Rename,
}

impl ImportConflictStrategy {
    pub const fn to_application_strategy(self) -> crate::application::ConflictStrategy {
        match self {
            Self::Skip => crate::application::ConflictStrategy::Skip,
            Self::Overwrite => crate::application::ConflictStrategy::Overwrite,
            Self::Rename => crate::application::ConflictStrategy::Rename,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct ProfileExportActionArgs {
    /// 仅导出指定名称的 Profile（可传多个；任一名称不存在则报错退出，与 --tag 互斥）
    #[arg(value_name = "NAME", conflicts_with = "tags")]
    pub names: Vec<String>,

    /// 按标签筛选导出（可重复传入，匹配任一标签即导出，与位置参数 NAME 互斥）
    #[arg(
        short = 't',
        long = "tag",
        value_name = "TAG",
        conflicts_with = "names"
    )]
    pub tags: Vec<String>,

    /// 写入指定文件路径 [默认: ./ccr-<platform>-profiles-<YYYYMMDD-HHMMSS>.toml]
    #[arg(
        short = 'o',
        long = "output",
        value_name = "FILE",
        conflicts_with = "stdout"
    )]
    pub output: Option<PathBuf>,

    /// 将导出的 TOML 直接输出到标准输出，不创建本地文件（提示与警告改走 stderr；与 -o、--json 互斥）
    #[arg(long, conflicts_with_all = ["output", "json"])]
    pub stdout: bool,

    /// 剥离所有内联密钥、可疑敏感扩展字段与 URL 内嵌凭据（写出 secrets_mode = "redacted"）
    #[arg(long = "no-secrets")]
    pub no_secrets: bool,

    /// 将导出结果元数据以 JSON 输出到标准输出（仍落盘写入文件，JSON 内不含任何密钥；与 --stdout 互斥）
    #[arg(long, conflicts_with = "stdout")]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileImportActionArgs {
    /// 要导入的 TOML 文件路径；传入 "-" 表示从标准输入读取（与 --stdin 二选一必填）
    #[arg(
        value_name = "FILE",
        required_unless_present = "stdin",
        conflicts_with = "stdin"
    )]
    pub file: Option<PathBuf>,

    /// 从标准输入读取导入 TOML 内容（与位置参数 FILE 互斥）
    #[arg(long, conflicts_with = "file")]
    pub stdin: bool,

    /// 合并模式下遇到同名且内容不同的 Profile 时的处理策略（与 --replace 互斥）
    #[arg(
        long = "on-conflict",
        value_name = "STRATEGY",
        value_enum,
        default_value_t = ImportConflictStrategy::Skip,
        conflicts_with = "replace"
    )]
    pub on_conflict: ImportConflictStrategy,

    /// 合并模式下，若新增或改名候选的归一化配置内容与本地已有异名 Profile 完全相同，则作为重复项跳过（与 --replace 互斥）
    #[arg(long, conflicts_with = "replace")]
    pub dedup: bool,

    /// 整体替换模式：将本地平台目录收敛为导入文件内容，移除未出现在导入文件中的本地 Profile（需交互确认或附加 -y/--yes）
    #[arg(long, conflicts_with_all = ["on_conflict", "dedup"])]
    pub replace: bool,

    /// 仅预览导入计划、冲突与待补密钥明细，不写入数据库、不生成备份、不改动运行时
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// 当更新命中本机当前正在生效的 Profile 时，仅更新目录存储而不重新应用平台运行时文件
    #[arg(long = "no-apply")]
    pub no_apply: bool,

    /// 乐观并发校验：仅当平台目录当前 document.revision 等于 <REV>（首次建库传 ""）时才提交写入
    #[arg(long = "expected-revision", value_name = "REV")]
    pub expected_revision: Option<String>,

    /// 以 JSON 格式输出结构化导入报告（不含密钥值；与 --dry-run 可组合使用）
    #[arg(long)]
    pub json: bool,
}
