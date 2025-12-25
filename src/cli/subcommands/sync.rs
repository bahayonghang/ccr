// 同步操作子命令

/// ☁️ 同步操作子命令
#[derive(clap::Subcommand)]
pub enum SyncAction {
    /// 管理同步文件夹注册
    ///
    /// 管理可同步的文件夹列表
    /// 示例: ccr sync folder list
    /// 示例: ccr sync folder add claude ~/.claude
    Folder {
        #[command(subcommand)]
        action: FolderAction,
    },

    /// 批量操作所有启用的文件夹
    ///
    /// 对所有已启用的文件夹执行同步操作
    /// 示例: ccr sync all push
    /// 示例: ccr sync all status
    All {
        #[command(subcommand)]
        action: AllSyncAction,
    },

    /// 配置 WebDAV 同步
    ///
    /// 交互式配置 WebDAV 服务器连接信息
    /// 示例: ccr sync config
    Config,

    /// 显示同步状态
    ///
    /// 查看当前同步配置和所有文件夹状态
    /// 示例: ccr sync status
    Status,

    /// 上传配置到云端 (兼容旧命令)
    ///
    /// 将本地配置文件上传到 WebDAV 服务器
    /// 示例: ccr sync push --force
    /// 示例: ccr sync push --interactive  # 交互式选择内容
    Push {
        /// 强制覆盖远程配置，不提示确认
        #[arg(short, long)]
        force: bool,

        /// 交互式选择要同步的内容类型
        #[arg(short = 'i', long)]
        interactive: bool,
    },

    /// 从云端下载配置 (兼容旧命令)
    ///
    /// 从 WebDAV 服务器下载配置文件到本地
    /// 示例: ccr sync pull --force
    Pull {
        /// 强制覆盖本地配置，不提示确认
        #[arg(short, long)]
        force: bool,
    },

    /// 同步特定文件夹 (动态子命令)
    ///
    /// 对指定文件夹执行同步操作
    /// 示例: ccr sync claude push
    /// 示例: ccr sync gemini pull
    /// 示例: ccr sync conf status
    #[command(external_subcommand)]
    #[allow(dead_code)]
    FolderSync(Vec<String>),
}

/// 📁 文件夹管理操作
#[derive(clap::Subcommand)]
pub enum FolderAction {
    /// 列出所有注册的同步文件夹
    ///
    /// 显示文件夹名称、状态、路径等信息
    /// 示例: ccr sync folder list
    List,

    /// 添加新的同步文件夹
    ///
    /// 注册一个新文件夹用于同步
    /// 示例: ccr sync folder add claude ~/.claude
    Add {
        /// 文件夹名称（唯一标识）
        name: String,

        /// 本地路径（支持 ~ 扩展）
        local_path: String,

        /// 远程路径（可选，默认为 /ccr-sync/<name>）
        #[arg(short = 'r', long)]
        remote_path: Option<String>,

        /// 描述信息
        #[arg(short = 'd', long)]
        description: Option<String>,
    },

    /// 删除同步文件夹注册
    ///
    /// 从注册列表中移除文件夹（不删除本地文件）
    /// 示例: ccr sync folder remove claude
    Remove {
        /// 文件夹名称
        name: String,
    },

    /// 显示文件夹详细信息
    ///
    /// 查看文件夹的完整配置
    /// 示例: ccr sync folder info claude
    Info {
        /// 文件夹名称
        name: String,
    },

    /// 启用文件夹同步
    ///
    /// 启用文件夹的同步功能
    /// 示例: ccr sync folder enable claude
    Enable {
        /// 文件夹名称
        name: String,
    },

    /// 禁用文件夹同步
    ///
    /// 禁用文件夹的同步功能（保留配置）
    /// 示例: ccr sync folder disable codex
    Disable {
        /// 文件夹名称
        name: String,
    },
}

/// 🔄 批量同步操作
#[derive(clap::Subcommand)]
pub enum AllSyncAction {
    /// 上传所有启用的文件夹
    ///
    /// 示例: ccr sync all push
    Push {
        /// 强制覆盖，不提示确认
        #[arg(short, long)]
        force: bool,
    },

    /// 下载所有启用的文件夹
    ///
    /// 示例: ccr sync all pull
    Pull {
        /// 强制覆盖，不提示确认
        #[arg(short, long)]
        force: bool,
    },

    /// 显示所有文件夹的状态
    ///
    /// 示例: ccr sync all status
    Status,
}
