//! WSL 执行环境 — 通过 `wsl.exe` 命令行与 Windows Subsystem for Linux 交互。
//!
//! 仅在 Windows 目标平台编译。
//! 注意：`#[cfg(target_os = "windows")]` 已在 `mod.rs` 中应用于本模块。

use std::process::Command;

use super::{CliStatus, EnvError, EnvironmentType, ExecutionEnvironment, PlatformInfo};

/// WSL 发行版状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum WslDistroState {
    Running,
    Stopped,
    Unknown,
}

/// WSL 版本
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum WslVersion {
    Wsl1,
    Wsl2,
    Unknown,
}

/// WSL 发行版信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WslDistroInfo {
    /// 发行版名称（如 "Ubuntu-22.04"）
    pub name: String,
    /// 是否为默认发行版
    pub is_default: bool,
    /// WSL 版本
    pub version: WslVersion,
    /// 运行状态
    pub state: WslDistroState,
}

/// 检测所有已安装的 WSL 发行版。
///
/// 运行 `wsl -l -v` 并解析输出，返回发行版列表。
pub fn detect_wsl_distros() -> Result<Vec<WslDistroInfo>, EnvError> {
    let output = Command::new("wsl")
        .args(["-l", "-v"])
        .output()
        .map_err(|e| EnvError::Other(format!("无法运行 wsl.exe: {e}")))?;

    if !output.status.success() && output.stdout.is_empty() {
        // WSL 未安装或未启用
        return Ok(Vec::new());
    }

    // wsl -l -v 输出为 UTF-16LE（Windows 默认），尝试 UTF-16LE 解码
    let text = decode_wsl_output(&output.stdout);

    let mut distros = Vec::new();

    for line in text.lines().skip(1) {
        // 跳过标题行
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // 格式：  * Ubuntu-22.04  Running  2
        // 其中 * 表示默认发行版
        let is_default = line.starts_with('*');
        let line = line.trim_start_matches('*').trim();

        // 按空白分割
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0].to_string();
        if name.is_empty() || name == "NAME" {
            continue;
        }

        let state = if parts.len() >= 2 {
            match parts[1] {
                "Running" => WslDistroState::Running,
                "Stopped" => WslDistroState::Stopped,
                _ => WslDistroState::Unknown,
            }
        } else {
            WslDistroState::Unknown
        };

        let version = if parts.len() >= 3 {
            match parts[2] {
                "1" => WslVersion::Wsl1,
                "2" => WslVersion::Wsl2,
                _ => WslVersion::Unknown,
            }
        } else {
            WslVersion::Unknown
        };

        distros.push(WslDistroInfo {
            name,
            is_default,
            version,
            state,
        });
    }

    Ok(distros)
}

/// 获取指定 WSL 发行版中的用户名。
///
/// 运行 `wsl -d <distro> -- whoami`。
pub fn get_wsl_username(distro: &str) -> Result<String, EnvError> {
    let output = Command::new("wsl")
        .args(["-d", distro, "--", "whoami"])
        .output()
        .map_err(|e| EnvError::Other(format!("无法运行 whoami in {distro}: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(EnvError::Other(format!(
            "获取 WSL 用户名失败 ({distro}): {stderr}"
        )));
    }

    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if username.is_empty() {
        return Err(EnvError::Other(format!("WSL 用户名为空 ({distro})")));
    }

    Ok(username)
}

/// 将 WSL 命令输出从 UTF-16LE 或 UTF-8 解码为 String。
fn decode_wsl_output(bytes: &[u8]) -> String {
    // 检测 BOM（UTF-16LE: FF FE）
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        // UTF-16LE 解码
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&utf16)
    } else {
        // 尝试直接 UTF-8，部分 WSL 版本会输出 UTF-8
        // 若有嵌入 null 字节（UTF-16 无 BOM），则按 UTF-16LE 处理
        if bytes.contains(&0u8) && bytes.len().is_multiple_of(2) {
            let utf16: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&utf16)
        } else {
            String::from_utf8_lossy(bytes).into_owned()
        }
    }
}

/// WSL 执行环境实现。
pub struct WslEnvironment {
    /// 目标发行版信息
    distro: WslDistroInfo,
    /// 发行版中的 Linux 用户名
    username: String,
}

impl WslEnvironment {
    /// 创建新的 WSL 环境实例。
    ///
    /// 会自动查询发行版内的用户名；失败时回退到 "user"。
    pub fn new(distro: WslDistroInfo) -> Self {
        let username = get_wsl_username(&distro.name).unwrap_or_else(|_| "user".to_string());
        Self { distro, username }
    }

    /// 通过 UNC 路径（`\\wsl$\<distro>\...`）读取文件。
    fn read_via_unc(&self, linux_path: &str) -> Result<String, EnvError> {
        let unc_path = self.to_unc_path(linux_path);
        std::fs::read_to_string(&unc_path)
            .map_err(|e| EnvError::Other(format!("UNC 读取失败 ({unc_path}): {e}")))
    }

    /// 通过 `wsl -d <distro> -- cat <path>` 读取文件（UNC 回退方案）。
    #[allow(dead_code)]
    fn read_via_wsl_cat(&self, linux_path: &str) -> Result<String, EnvError> {
        let output = Command::new("wsl")
            .args(["-d", &self.distro.name, "--", "cat", linux_path])
            .output()
            .map_err(|e| EnvError::Other(format!("wsl cat 失败: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EnvError::ConfigNotFound(format!(
                "{linux_path} in {}: {stderr}",
                self.distro.name
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// 通过 UNC 路径写入文件（原子写：先写临时文件再重命名）。
    fn write_via_unc(&self, linux_path: &str, content: &str) -> Result<(), EnvError> {
        let unc_path = self.to_unc_path(linux_path);
        let tmp_path = format!("{unc_path}.tmp");

        // 确保父目录存在
        if let Some(parent) = std::path::Path::new(&unc_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| EnvError::Other(format!("创建目录失败: {e}")))?;
        }

        std::fs::write(&tmp_path, content)
            .map_err(|e| EnvError::Other(format!("写入临时文件失败 ({tmp_path}): {e}")))?;

        std::fs::rename(&tmp_path, &unc_path)
            .map_err(|e| EnvError::Other(format!("原子重命名失败: {e}")))?;

        Ok(())
    }

    /// 通过 `wsl -d <distro> -- tee <path>` 写入文件（UNC 回退方案）。
    #[allow(dead_code)]
    fn write_via_wsl_tee(&self, linux_path: &str, content: &str) -> Result<(), EnvError> {
        use std::io::Write;
        use std::process::Stdio;

        // 先创建父目录
        let parent = std::path::Path::new(linux_path)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();

        if !parent.is_empty() {
            let _ = Command::new("wsl")
                .args(["-d", &self.distro.name, "--", "mkdir", "-p", &parent])
                .output();
        }

        let mut child = Command::new("wsl")
            .args(["-d", &self.distro.name, "--", "tee", linux_path])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| EnvError::Other(format!("wsl tee spawn 失败: {e}")))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(content.as_bytes())
                .map_err(|e| EnvError::Other(format!("写入 stdin 失败: {e}")))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| EnvError::Other(format!("wsl tee wait 失败: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EnvError::Other(format!(
                "wsl tee 写入失败 ({linux_path}): {stderr}"
            )));
        }

        Ok(())
    }

    /// 将 Linux 路径转换为 UNC 路径（`\\wsl$\<distro>\<path>`）。
    fn to_unc_path(&self, linux_path: &str) -> String {
        // linux_path 形如 /home/user/.claude/settings.json
        // UNC 形如 \\wsl$\Ubuntu-22.04\home\user\.claude\settings.json
        let win_path = linux_path.replace('/', "\\");
        format!(
            "\\\\wsl$\\{}\\{}",
            self.distro.name,
            win_path.trim_start_matches('\\')
        )
    }

    /// 将平台名转换为 Linux home 下的配置目录前缀。
    fn platform_config_dir(&self, platform: &str) -> Result<String, EnvError> {
        let home = format!("/home/{}", self.username);
        let dir = match platform {
            "claude" => format!("{home}/.claude"),
            "codex" => format!("{home}/.codex"),
            "gemini" => format!("{home}/.gemini"),
            "qwen" => format!("{home}/.qwen"),
            "iflow" => format!("{home}/.iflow"),
            "droid" => format!("{home}/.droid"),
            "opencode" => format!("{home}/.opencode"),
            _ => return Err(EnvError::PlatformNotSupported(platform.to_string())),
        };
        Ok(dir)
    }
}

#[async_trait::async_trait]
impl ExecutionEnvironment for WslEnvironment {
    fn env_type(&self) -> EnvironmentType {
        EnvironmentType::Wsl
    }

    fn display_name(&self) -> String {
        format!("WSL: {}", self.distro.name)
    }

    fn env_id(&self) -> String {
        format!("wsl:{}", self.distro.name)
    }

    async fn list_platforms(&self) -> Result<Vec<PlatformInfo>, EnvError> {
        let distro = self.distro.name.clone();

        // 在阻塞线程中运行 wsl 命令
        let output = tokio::task::spawn_blocking(move || {
            Command::new("wsl")
                .args([
                    "-d", &distro,
                    "--",
                    "sh", "-c",
                    "which claude 2>/dev/null; which codex 2>/dev/null; which gemini 2>/dev/null; which qwen 2>/dev/null; which iflow 2>/dev/null",
                ])
                .output()
        })
        .await
        .map_err(|e| EnvError::Other(format!("spawn_blocking 失败: {e}")))?
        .map_err(|e| EnvError::Other(format!("wsl which 失败: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let found_paths: Vec<&str> = stdout.lines().map(str::trim).collect();

        let platforms = [
            ("claude", "Claude Code"),
            ("codex", "Codex CLI"),
            ("gemini", "Gemini CLI"),
            ("qwen", "Qwen"),
            ("iflow", "iFlow"),
        ];

        let result = platforms
            .iter()
            .map(|(name, display)| {
                let installed = found_paths.iter().any(|p| p.contains(name));
                PlatformInfo {
                    name: name.to_string(),
                    display_name: display.to_string(),
                    installed,
                    version: None,
                }
            })
            .collect();

        Ok(result)
    }

    async fn read_config(&self, platform: &str, path: &str) -> Result<String, EnvError> {
        let base_dir = self.platform_config_dir(platform)?;
        let linux_path = format!("{base_dir}/{path}");

        // 优先尝试 UNC 路径（更快，无需启动 WSL 进程）
        match self.read_via_unc(&linux_path) {
            Ok(content) => Ok(content),
            Err(_) => {
                // 回退到 wsl cat
                let distro = self.distro.name.clone();
                let lp = linux_path.clone();
                tokio::task::spawn_blocking(move || {
                    let output = Command::new("wsl")
                        .args(["-d", &distro, "--", "cat", &lp])
                        .output()
                        .map_err(|e| EnvError::Other(format!("wsl cat 失败: {e}")))?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(EnvError::ConfigNotFound(format!("{lp}: {stderr}")));
                    }
                    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
                })
                .await
                .map_err(|e| EnvError::Other(format!("spawn_blocking 失败: {e}")))?
            }
        }
    }

    async fn write_config(
        &self,
        platform: &str,
        path: &str,
        content: &str,
    ) -> Result<(), EnvError> {
        let base_dir = self.platform_config_dir(platform)?;
        let linux_path = format!("{base_dir}/{path}");

        // 优先尝试 UNC 原子写
        match self.write_via_unc(&linux_path, content) {
            Ok(()) => Ok(()),
            Err(_) => {
                // 回退到 wsl tee
                let distro = self.distro.name.clone();
                let lp = linux_path.clone();
                let content_owned = content.to_string();

                tokio::task::spawn_blocking(move || {
                    use std::io::Write;
                    use std::process::Stdio;

                    // 先创建父目录
                    let parent = std::path::Path::new(&lp)
                        .parent()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    if !parent.is_empty() {
                        let _ = Command::new("wsl")
                            .args(["-d", &distro, "--", "mkdir", "-p", &parent])
                            .output();
                    }

                    let mut child = Command::new("wsl")
                        .args(["-d", &distro, "--", "tee", &lp])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::null())
                        .stderr(Stdio::piped())
                        .spawn()
                        .map_err(|e| EnvError::Other(format!("wsl tee spawn 失败: {e}")))?;

                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin
                            .write_all(content_owned.as_bytes())
                            .map_err(|e| EnvError::Other(format!("写入 stdin 失败: {e}")))?;
                    }

                    let out = child
                        .wait_with_output()
                        .map_err(|e| EnvError::Other(format!("wsl tee wait 失败: {e}")))?;

                    if !out.status.success() {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        return Err(EnvError::Other(format!(
                            "wsl tee 写入失败 ({lp}): {stderr}"
                        )));
                    }
                    Ok(())
                })
                .await
                .map_err(|e| EnvError::Other(format!("spawn_blocking 失败: {e}")))?
            }
        }
    }

    async fn detect_cli_status(&self) -> Result<Vec<CliStatus>, EnvError> {
        let distro = self.distro.name.clone();
        let tools = ["claude", "codex", "gemini", "qwen", "iflow"];

        let mut statuses = Vec::new();

        for tool in &tools {
            let distro_clone = distro.clone();
            let tool_str = tool.to_string();

            let result = tokio::task::spawn_blocking(move || {
                Command::new("wsl")
                    .args(["-d", &distro_clone, "--", "which", &tool_str])
                    .output()
            })
            .await
            .map_err(|e| EnvError::Other(format!("spawn_blocking 失败: {e}")))?;

            let (installed, path) = match result {
                Ok(output) if output.status.success() => {
                    let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    (true, if p.is_empty() { None } else { Some(p) })
                }
                _ => (false, None),
            };

            statuses.push(CliStatus {
                name: tool.to_string(),
                installed,
                path,
                version: None,
            });
        }

        Ok(statuses)
    }
}

// ── 同步辅助函数（供 Tauri 命令调用） ──────────────────────────────────────

/// 同步方向
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SyncDirection {
    /// 从本地（Windows）推送到 WSL
    LocalToWsl,
    /// 从 WSL 拉取到本地（Windows）
    WslToLocal,
}

/// 在本地和 WSL 之间同步指定平台的配置文件。
///
/// 返回操作摘要字符串。
pub fn sync_config_blocking(
    distro: &str,
    platform: &str,
    direction: &SyncDirection,
) -> Result<String, EnvError> {
    // 获取本地 home 目录
    let local_home =
        dirs::home_dir().ok_or_else(|| EnvError::Other("本地 home 目录未找到".to_string()))?;

    let local_dir = match platform {
        "claude" => local_home.join(".claude"),
        "codex" => local_home.join(".codex"),
        "gemini" => local_home.join(".gemini"),
        "qwen" => local_home.join(".qwen"),
        "iflow" => local_home.join(".iflow"),
        "droid" => local_home.join(".droid"),
        "opencode" => local_home.join(".opencode"),
        _ => return Err(EnvError::PlatformNotSupported(platform.to_string())),
    };

    // 获取 WSL 用户名以构造路径
    let username = get_wsl_username(distro).unwrap_or_else(|_| "user".to_string());
    let wsl_dir = format!("/home/{}/.{}", username, platform);
    let unc_dir = format!(
        "\\\\wsl$\\{}\\{}",
        distro,
        wsl_dir.replace('/', "\\").trim_start_matches('\\')
    );

    match direction {
        SyncDirection::LocalToWsl => {
            // 本地 → WSL：通过 UNC 路径复制
            std::fs::create_dir_all(&unc_dir)
                .map_err(|e| EnvError::Other(format!("创建 WSL 目录失败: {e}")))?;

            let mut count = 0u32;
            if local_dir.exists() {
                for entry in std::fs::read_dir(&local_dir).map_err(EnvError::Io)? {
                    let entry = entry.map_err(EnvError::Io)?;
                    let file_name = entry.file_name();
                    let dest = format!("{}\\{}", unc_dir, file_name.to_string_lossy());
                    std::fs::copy(entry.path(), &dest)
                        .map_err(|e| EnvError::Other(format!("复制文件失败: {e}")))?;
                    count += 1;
                }
            }
            Ok(format!(
                "Local→WSL ({platform}@{distro}): {count} 个文件已同步"
            ))
        }
        SyncDirection::WslToLocal => {
            // WSL → 本地：通过 UNC 路径复制
            std::fs::create_dir_all(&local_dir).map_err(EnvError::Io)?;

            let unc_path = std::path::Path::new(&unc_dir);
            let mut count = 0u32;
            if unc_path.exists() {
                for entry in std::fs::read_dir(unc_path).map_err(EnvError::Io)? {
                    let entry = entry.map_err(EnvError::Io)?;
                    let file_name = entry.file_name();
                    let dest = local_dir.join(&file_name);
                    std::fs::copy(entry.path(), &dest)
                        .map_err(|e| EnvError::Other(format!("复制文件失败: {e}")))?;
                    count += 1;
                }
            }
            Ok(format!(
                "WSL→Local ({platform}@{distro}): {count} 个文件已同步"
            ))
        }
    }
}
