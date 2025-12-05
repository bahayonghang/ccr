// 🎨 UI 服务层
// 负责启动和管理 CCR UI (Web 应用)

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// GitHub 仓库信息
const GITHUB_REPO: &str = "bahayonghang/ccr";
const GITHUB_BRANCH: &str = "main";

/// 🎨 UI 服务
///
/// 提供 CCR UI 的启动和管理功能
pub struct UiService {
    /// CCR-UI 项目路径 (开发模式使用)
    ccr_ui_path: Option<PathBuf>,
    /// UI 资源目录 (~/.ccr/repo/ccr-ui/) - 用于下载的版本
    ui_dir: PathBuf,
    /// 旧版 UI 目录 (~/.ccr/ccr-ui/) - 用于迁移检测
    legacy_ui_dir: PathBuf,
}

impl UiService {
    /// 🏗️ 创建新的 UI 服务
    pub fn new() -> Result<Self> {
        // 获取用户主目录
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".to_string()))?;

        // UI 资源目录 (~/.ccr/repo/ccr-ui/) - 指向完整仓库下的 ccr-ui
        let ui_dir = home.join(".ccr/repo/ccr-ui");
        // 旧版 UI 目录 (~/.ccr/ccr-ui/) - 用于迁移检测
        let legacy_ui_dir = home.join(".ccr/ccr-ui");

        // 检查是否在开发环境中
        let ccr_ui_path = Self::detect_ccr_ui_path();

        Ok(Self {
            ccr_ui_path,
            ui_dir,
            legacy_ui_dir,
        })
    }

    /// 🔍 检测 ccr-ui 项目路径
    ///
    /// 优先级:
    /// 1. 当前目录下的 ccr-ui/
    /// 2. CCR 项目根目录下的 ccr-ui/
    fn detect_ccr_ui_path() -> Option<PathBuf> {
        // 尝试当前目录
        let current_dir_ui = std::env::current_dir().ok().map(|p| p.join("ccr-ui"));

        if let Some(ref path) = current_dir_ui
            && path.exists()
            && path.join("justfile").exists()
        {
            return Some(path.clone());
        }

        // 尝试父目录 (适用于在 ccr/src 等子目录运行的情况)
        let parent_dir_ui = std::env::current_dir()
            .ok()
            .and_then(|p| p.parent().map(|parent| parent.join("ccr-ui")));

        if let Some(ref path) = parent_dir_ui
            && path.exists()
            && path.join("justfile").exists()
        {
            return Some(path.clone());
        }

        None
    }

    /// 🚀 启动 UI (智能选择模式)
    ///
    /// 根据环境自动选择最佳启动方式:
    /// 1. 开发环境: 使用当前目录的 ccr-ui/ 启动源码
    /// 2. 用户目录: 使用 ~/.ccr/ccr-ui/ 启动下载版本
    /// 3. 未找到: 提示从 GitHub 下载
    pub fn start(&self, port: u16, backend_port: u16) -> Result<()> {
        ColorOutput::title("🚀 CCR UI 启动中...");
        println!();

        // 优先级 1: 检查开发环境（当前目录的 ccr-ui/）
        if let Some(ref ccr_ui_path) = self.ccr_ui_path {
            ColorOutput::info(&format!("📁 检测到开发环境: {}", ccr_ui_path.display()));
            return self.start_dev_mode(ccr_ui_path, port, backend_port);
        }

        // 优先级 2: 检查用户目录下载版本（~/.ccr/repo/ccr-ui/）
        if self.ui_dir.exists() && self.ui_dir.join("justfile").exists() {
            ColorOutput::info(&format!("📁 检测到用户目录版本: {}", self.ui_dir.display()));
            return self.start_dev_mode(&self.ui_dir, port, backend_port);
        }

        // 优先级 3: 检查旧版目录并提示迁移
        if self.legacy_ui_dir.exists() && self.legacy_ui_dir.join("justfile").exists() {
            ColorOutput::warning(&format!(
                "⚠️  检测到旧版 CCR UI 目录: {}",
                self.legacy_ui_dir.display()
            ));
            ColorOutput::info("旧版本使用 npm，建议迁移到新版本以使用 bun");
            println!();

            if self.prompt_migrate_legacy()? {
                // 删除旧目录
                fs::remove_dir_all(&self.legacy_ui_dir)
                    .map_err(|e| CcrError::ConfigError(format!("删除旧目录失败: {}", e)))?;
                ColorOutput::success("✅ 已删除旧版本");
                // 继续下载新版本
            } else {
                // 用户选择继续使用旧版本
                ColorOutput::warning("⚠️  继续使用旧版本（使用 npm）");
                return self.start_dev_mode(&self.legacy_ui_dir, port, backend_port);
            }
        }

        // 优先级 4: 未找到，提示下载
        ColorOutput::warning("⚠️  未找到 CCR UI");
        println!();
        ColorOutput::info("CCR UI 可以从以下位置获取：");
        ColorOutput::info("  1. 开发环境: 项目根目录下的 ccr-ui/");
        ColorOutput::info(&format!("  2. 用户目录: {}", self.ui_dir.display()));
        println!();

        // 询问是否下载
        if self.prompt_download()? {
            self.download_from_github()?;
            // 下载完成后启动
            return self.start_dev_mode(&self.ui_dir, port, backend_port);
        }

        Err(CcrError::ConfigError(
            "用户取消下载，无法启动 CCR UI".to_string(),
        ))
    }

    /// 🔧 开发模式启动
    ///
    /// 使用 `just dev` 启动 ccr-ui 开发环境
    fn start_dev_mode(&self, ccr_ui_path: &Path, _port: u16, _backend_port: u16) -> Result<()> {
        ColorOutput::step("启动开发模式");
        println!();

        // 检查 just 命令
        self.check_just_installed()?;

        // 检查依赖是否已安装
        self.check_and_install_deps(ccr_ui_path)?;

        ColorOutput::info("🔧 使用开发模式启动 CCR UI");
        ColorOutput::info("📍 后端: http://localhost:8081");
        ColorOutput::info("📍 前端: http://localhost:3000 (Next.js)");
        println!();

        ColorOutput::warning("💡 提示: 按 Ctrl+C 停止服务");
        println!();

        // 启动开发服务器
        let status = Command::new("just")
            .arg("dev")
            .current_dir(ccr_ui_path)
            .status()
            .map_err(|e| CcrError::ConfigError(format!("启动失败: {}", e)))?;

        if !status.success() {
            return Err(CcrError::ConfigError(
                "开发模式启动失败,请查看上方错误信息".to_string(),
            ));
        }

        Ok(())
    }

    /// ✅ 检查 just 是否已安装
    fn check_just_installed(&self) -> Result<()> {
        ColorOutput::info("🔍 检查 just 工具...");

        match Command::new("just").arg("--version").output() {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    ColorOutput::success(&format!("✅ just 已安装: {}", version.trim()));
                    Ok(())
                } else {
                    self.prompt_install_just()
                }
            }
            Err(_) => self.prompt_install_just(),
        }
    }

    /// 📦 提示安装 just
    fn prompt_install_just(&self) -> Result<()> {
        ColorOutput::error("❌ 未检测到 just 工具");
        println!();
        ColorOutput::info("just 是一个现代化的命令运行器,CCR UI 依赖它来启动");
        ColorOutput::info("请访问: https://just.systems/");
        println!();
        ColorOutput::info("快速安装:");
        ColorOutput::info("  cargo install just");
        println!();

        Err(CcrError::ConfigError(
            "缺少必要工具: just (请安装后重试)".to_string(),
        ))
    }

    /// 📦 检查并安装依赖
    fn check_and_install_deps(&self, ccr_ui_path: &Path) -> Result<()> {
        ColorOutput::info("🔍 检查项目依赖...");

        // 检查前端依赖
        let frontend_node_modules = ccr_ui_path.join("frontend/node_modules");
        let needs_frontend_install = !frontend_node_modules.exists();

        // 检查后端是否构建过
        let backend_target = ccr_ui_path.join("backend/target");
        let needs_backend_build = !backend_target.exists();

        if needs_frontend_install || needs_backend_build {
            ColorOutput::warning("⚠️  检测到未安装的依赖,开始安装...");
            println!();

            // 询问用户是否继续
            if !self.confirm_installation()? {
                return Err(CcrError::ConfigError("用户取消安装".to_string()));
            }

            // 运行 just install
            ColorOutput::info("📦 正在安装依赖 (这可能需要几分钟)...");
            let status = Command::new("just")
                .arg("install")
                .current_dir(ccr_ui_path)
                .status()
                .map_err(|e| CcrError::ConfigError(format!("安装依赖失败: {}", e)))?;

            if !status.success() {
                return Err(CcrError::ConfigError(
                    "依赖安装失败,请查看上方错误信息".to_string(),
                ));
            }

            ColorOutput::success("✅ 依赖安装完成");
            println!();
        } else {
            ColorOutput::success("✅ 依赖已就绪");
        }

        Ok(())
    }

    /// ❓ 确认是否安装依赖
    fn confirm_installation(&self) -> Result<bool> {
        use dialoguer::Confirm;

        let confirmed = Confirm::new()
            .with_prompt("是否立即安装 CCR UI 依赖?")
            .default(true)
            .interact()
            .map_err(|e| CcrError::ConfigError(format!("交互失败: {}", e)))?;

        Ok(confirmed)
    }

    /// ❓ 提示是否迁移旧版本
    fn prompt_migrate_legacy(&self) -> Result<bool> {
        use dialoguer::Confirm;

        ColorOutput::info("新版本使用 bun 作为包管理器，性能更好");
        ColorOutput::info(&format!(
            "迁移将删除旧目录 {} 并下载新版本",
            self.legacy_ui_dir.display()
        ));
        println!();

        let confirmed = Confirm::new()
            .with_prompt("是否迁移到新版本?")
            .default(true)
            .interact()
            .map_err(|e| CcrError::ConfigError(format!("交互失败: {}", e)))?;

        Ok(confirmed)
    }

    /// 🌐 仅启动前端 (用于测试)
    #[allow(dead_code)]
    pub fn start_frontend_only(&self) -> Result<()> {
        if let Some(ref ccr_ui_path) = self.ccr_ui_path {
            ColorOutput::info("🌐 启动前端服务器...");

            let status = Command::new("just")
                .arg("dev-frontend")
                .current_dir(ccr_ui_path)
                .status()
                .map_err(|e| CcrError::ConfigError(format!("启动前端失败: {}", e)))?;

            if !status.success() {
                return Err(CcrError::ConfigError("前端启动失败".to_string()));
            }

            Ok(())
        } else {
            Err(CcrError::ConfigError("未找到 ccr-ui 目录".to_string()))
        }
    }

    /// 🦀 仅启动后端 (用于测试)
    #[allow(dead_code)]
    pub fn start_backend_only(&self) -> Result<()> {
        if let Some(ref ccr_ui_path) = self.ccr_ui_path {
            ColorOutput::info("🦀 启动后端服务器...");

            let status = Command::new("just")
                .arg("dev-backend")
                .current_dir(ccr_ui_path)
                .status()
                .map_err(|e| CcrError::ConfigError(format!("启动后端失败: {}", e)))?;

            if !status.success() {
                return Err(CcrError::ConfigError("后端启动失败".to_string()));
            }

            Ok(())
        } else {
            Err(CcrError::ConfigError("未找到 ccr-ui 目录".to_string()))
        }
    }

    /// 🏗️ 构建生产版本
    #[allow(dead_code)]
    pub fn build_production(&self) -> Result<()> {
        if let Some(ref ccr_ui_path) = self.ccr_ui_path {
            ColorOutput::info("🏗️  构建生产版本...");

            let status = Command::new("just")
                .arg("build")
                .current_dir(ccr_ui_path)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .map_err(|e| CcrError::ConfigError(format!("构建失败: {}", e)))?;

            if !status.success() {
                return Err(CcrError::ConfigError("生产构建失败".to_string()));
            }

            ColorOutput::success("✅ 生产构建完成");
            ColorOutput::info(&format!(
                "📦 后端: {}/backend/target/release/ccr-ui-backend",
                ccr_ui_path.display()
            ));
            ColorOutput::info(&format!(
                "📦 前端: {}/frontend/dist/",
                ccr_ui_path.display()
            ));

            Ok(())
        } else {
            Err(CcrError::ConfigError("未找到 ccr-ui 目录".to_string()))
        }
    }

    // === GitHub 下载功能 ===

    /// ❓ 提示用户是否下载 CCR UI
    fn prompt_download(&self) -> Result<bool> {
        use dialoguer::Confirm;

        ColorOutput::info("💡 提示: CCR UI 是一个完整的 Next.js + Actix Web 应用");
        ColorOutput::info("   可以从 GitHub 下载到用户目录:");
        ColorOutput::info(&format!("   {}", self.ui_dir.display()));
        println!();

        let confirmed = Confirm::new()
            .with_prompt("是否立即从 GitHub 下载 CCR UI?")
            .default(true)
            .interact()
            .map_err(|e| CcrError::ConfigError(format!("交互失败: {}", e)))?;

        Ok(confirmed)
    }

    /// 📥 从 GitHub 下载 ccr-ui 源码
    fn download_from_github(&self) -> Result<()> {
        use std::fs::create_dir_all;
        use tempfile::TempDir;

        ColorOutput::step("从 GitHub 下载 CCR UI");
        println!();

        // 创建目标目录的父目录
        let parent_dir = self
            .ui_dir
            .parent()
            .ok_or_else(|| CcrError::ConfigError("无法获取父目录".to_string()))?;

        if !parent_dir.exists() {
            create_dir_all(parent_dir)
                .map_err(|e| CcrError::ConfigError(format!("创建目录失败: {}", e)))?;
        }

        // 创建临时目录用于克隆
        let temp_dir = TempDir::new()
            .map_err(|e| CcrError::ConfigError(format!("创建临时目录失败: {}", e)))?;

        ColorOutput::info(&format!(
            "📦 克隆仓库: https://github.com/{}.git",
            GITHUB_REPO
        ));
        ColorOutput::info(&format!("📁 临时目录: {}", temp_dir.path().display()));
        println!();

        ColorOutput::warning("⏳ 下载中 (这可能需要几分钟)...");

        // 克隆整个仓库到临时目录
        let status = Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--branch")
            .arg(GITHUB_BRANCH)
            .arg(format!("https://github.com/{}.git", GITHUB_REPO))
            .arg(temp_dir.path())
            .status()
            .map_err(|e| {
                CcrError::ConfigError(format!(
                    "执行 git clone 失败: {}\n\n💡 请确保已安装 git: sudo apt-get install git",
                    e
                ))
            })?;

        if !status.success() {
            return Err(CcrError::ConfigError(
                "下载失败，请检查网络连接和 git 安装".to_string(),
            ));
        }

        // 检查 ccr-ui 子目录是否存在
        let ccr_ui_src = temp_dir.path().join("ccr-ui");
        if !ccr_ui_src.exists() {
            return Err(CcrError::ConfigError(
                "下载的仓库中未找到 ccr-ui 目录".to_string(),
            ));
        }

        // 验证 ccr-ui 目录的完整性
        if !ccr_ui_src.join("justfile").exists() {
            return Err(CcrError::ConfigError(
                "ccr-ui 目录不完整，缺少 justfile".to_string(),
            ));
        }

        ColorOutput::info("📦 正在复制仓库文件...");

        // 目标是 repo 根目录 (~/.ccr/repo)
        // self.ui_dir 是 ~/.ccr/repo/ccr-ui
        // 所以我们要复制到 self.ui_dir.parent()
        let repo_dir = self
            .ui_dir
            .parent()
            .ok_or_else(|| CcrError::ConfigError("无法获取仓库根目录".to_string()))?;

        // 如果 repo 目录已存在，先删除
        if repo_dir.exists() {
            fs::remove_dir_all(repo_dir)
                .map_err(|e| CcrError::ConfigError(format!("删除旧目录失败: {}", e)))?;
        }

        // 复制整个仓库到目标位置
        // temp_dir.path() 是仓库根目录
        // repo_dir 是目标仓库根目录
        self.copy_dir_recursive(temp_dir.path(), repo_dir)?;

        ColorOutput::success("✅ CCR 仓库下载完成");
        ColorOutput::info(&format!("📁 安装位置: {}", self.ui_dir.display()));
        println!();

        // 临时目录会在这里自动清理

        Ok(())
    }

    /// 递归复制目录
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        Self::copy_dir_recursive_impl(src, dst)
    }

    /// 递归复制目录的内部实现
    fn copy_dir_recursive_impl(src: &Path, dst: &Path) -> Result<()> {
        use std::fs;

        if !dst.exists() {
            fs::create_dir_all(dst)
                .map_err(|e| CcrError::ConfigError(format!("创建目录失败: {}", e)))?;
        }

        for entry in
            fs::read_dir(src).map_err(|e| CcrError::ConfigError(format!("读取目录失败: {}", e)))?
        {
            let entry = entry.map_err(|e| CcrError::ConfigError(format!("读取条目失败: {}", e)))?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);

            if path.is_dir() {
                // 跳过 .git 目录
                if file_name == ".git" {
                    continue;
                }
                Self::copy_dir_recursive_impl(&path, &dst_path)?;
            } else {
                fs::copy(&path, &dst_path)
                    .map_err(|e| CcrError::ConfigError(format!("复制文件失败: {}", e)))?;
            }
        }

        Ok(())
    }

    // === 预留接口: 预构建版本管理 ===

    /// 🔍 检查本地预构建版本 (预留)
    #[allow(dead_code)]
    fn has_local_version(&self) -> Result<bool> {
        Ok(self.ui_dir.join("version.txt").exists())
    }

    /// 📥 下载并安装预构建版本 (预留)
    #[allow(dead_code)]
    fn download_and_install(&self) -> Result<()> {
        ColorOutput::info("📥 预构建版本下载功能将在未来版本中实现");
        Err(CcrError::ConfigError("预构建版本功能尚未实现".to_string()))
    }

    /// 🚀 启动本地预构建版本 (预留)
    #[allow(dead_code)]
    fn start_local(&self, _port: u16, _backend_port: u16) -> Result<()> {
        ColorOutput::info("🚀 预构建版本启动功能将在未来版本中实现");
        Err(CcrError::ConfigError("预构建版本功能尚未实现".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_service_creation() {
        let service = UiService::new();
        assert!(service.is_ok());
    }

    #[test]
    fn test_detect_ccr_ui_path() {
        // 这个测试会根据运行环境有不同结果
        let path = UiService::detect_ccr_ui_path();
        // 在 CI 环境中可能找不到 ccr-ui
        println!("检测到的 ccr-ui 路径: {:?}", path);
    }
}
