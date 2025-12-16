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
    /// UI 安装目录 (~/.ccr/ccr-ui/) - 用于下载/更新的版本
    ui_dir: PathBuf,
    /// 旧版 UI 目录 (~/.ccr/repo/ccr-ui/) - 兼容历史路径迁移
    legacy_ui_dir: PathBuf,
}

impl UiService {
    /// 🏗️ 创建新的 UI 服务
    pub fn new() -> Result<Self> {
        // 获取用户主目录
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".to_string()))?;

        // UI 安装目录 (~/.ccr/ccr-ui/) - 用户侧固定目录
        let ui_dir = home.join(".ccr/ccr-ui");
        // 旧版 UI 目录 (~/.ccr/repo/ccr-ui/) - 兼容历史路径
        let legacy_ui_dir = home.join(".ccr/repo/ccr-ui");

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
    pub fn start(&self, port: u16, backend_port: u16, auto_yes: bool) -> Result<()> {
        ColorOutput::title("🚀 CCR UI 启动中...");
        println!();

        // 优先级 1: 检查开发环境（当前目录的 ccr-ui/）
        if let Some(ref ccr_ui_path) = self.ccr_ui_path {
            ColorOutput::info(&format!("📁 检测到开发环境: {}", ccr_ui_path.display()));
            return self.start_dev_mode(ccr_ui_path, port, backend_port, auto_yes);
        }

        // 优先级 2: 检查用户目录下载版本（~/.ccr/ccr-ui/）
        if self.ui_dir.exists() && self.ui_dir.join("justfile").exists() {
            ColorOutput::info(&format!("📁 检测到用户目录版本: {}", self.ui_dir.display()));
            return self.start_dev_mode(&self.ui_dir, port, backend_port, auto_yes);
        }

        // 优先级 3: 检查旧版目录并提示迁移（~/.ccr/repo/ccr-ui/ -> ~/.ccr/ccr-ui/）
        if self.legacy_ui_dir.exists() && self.legacy_ui_dir.join("justfile").exists() {
            ColorOutput::warning(&format!(
                "⚠️  检测到旧版 CCR UI 目录: {}",
                self.legacy_ui_dir.display()
            ));
            ColorOutput::info(&format!("建议迁移到新目录: {}", self.ui_dir.display()));
            println!();

            if self.prompt_migrate_legacy(auto_yes)? {
                self.migrate_legacy_dir()?;
                return self.start_dev_mode(&self.ui_dir, port, backend_port, auto_yes);
            }

            // 用户拒绝迁移：仍允许使用旧路径启动（尽量不打断使用）
            ColorOutput::warning("⚠️  已跳过迁移，将使用旧目录启动");
            return self.start_dev_mode(&self.legacy_ui_dir, port, backend_port, auto_yes);
        }

        // 优先级 4: 未找到，提示下载
        ColorOutput::warning("⚠️  未找到 CCR UI");
        println!();
        ColorOutput::info("CCR UI 可以从以下位置获取：");
        ColorOutput::info("  1. 开发环境: 项目根目录下的 ccr-ui/");
        ColorOutput::info(&format!("  2. 用户目录: {}", self.ui_dir.display()));
        println!();

        // 询问是否下载
        if self.prompt_download(auto_yes)? {
            self.sync_from_github(auto_yes)?;
            // 下载完成后启动
            return self.start_dev_mode(&self.ui_dir, port, backend_port, auto_yes);
        }

        Err(CcrError::ConfigError(
            "用户取消下载，无法启动 CCR UI".to_string(),
        ))
    }

    /// 🔄 更新/安装用户目录下的 CCR UI 到最新版本
    pub fn update(&self, auto_yes: bool) -> Result<()> {
        ColorOutput::title("🔄 CCR UI 更新中...");
        println!();
        self.sync_from_github(auto_yes)
    }

    /// 🔧 开发模式启动
    ///
    /// 使用 `just dev` 启动 ccr-ui 开发环境
    fn start_dev_mode(
        &self,
        ccr_ui_path: &Path,
        _port: u16,
        _backend_port: u16,
        auto_yes: bool,
    ) -> Result<()> {
        ColorOutput::step("启动开发模式");
        println!();

        // 检查 just 命令
        self.check_just_installed()?;

        // 检查依赖是否已安装
        self.check_and_install_deps(ccr_ui_path, auto_yes)?;

        ColorOutput::info("🔧 使用开发模式启动 CCR UI");
        ColorOutput::info("📍 后端: http://localhost:38081");
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
    fn check_and_install_deps(&self, ccr_ui_path: &Path, auto_yes: bool) -> Result<()> {
        ColorOutput::info("🔍 检查项目依赖...");

        // 检查前端依赖
        let frontend_node_modules = ccr_ui_path.join("frontend/node_modules");
        let needs_frontend_install = !frontend_node_modules.exists();

        // 检查后端是否构建过
        let backend_target = ccr_ui_path.join("backend/target");
        let needs_backend_build = !backend_target.exists();

        if needs_frontend_install || needs_backend_build {
            ColorOutput::warning("⚠️  检测到未安装的依赖,开始安装...");
            if needs_frontend_install {
                ColorOutput::info("  - 缺少前端依赖: frontend/node_modules");
            }
            if needs_backend_build {
                ColorOutput::info("  - 缺少后端构建产物: backend/target");
            }
            println!();

            // 询问用户是否继续
            if !self.confirm_installation(auto_yes)? {
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
    fn confirm_installation(&self, auto_yes: bool) -> Result<bool> {
        use dialoguer::Confirm;

        if auto_yes {
            return Ok(true);
        }

        let confirmed = Confirm::new()
            .with_prompt("是否立即安装 CCR UI 依赖?")
            .default(true)
            .interact()
            .map_err(|e| CcrError::ConfigError(format!("交互失败: {}", e)))?;

        Ok(confirmed)
    }

    /// ❓ 提示是否迁移旧版本
    fn prompt_migrate_legacy(&self, auto_yes: bool) -> Result<bool> {
        use dialoguer::Confirm;

        if auto_yes {
            return Ok(true);
        }

        ColorOutput::info("检测到旧版安装路径，建议迁移以统一目录结构");
        ColorOutput::info(&format!(
            "迁移将把 {} 移动到 {}",
            self.legacy_ui_dir.display(),
            self.ui_dir.display()
        ));
        println!();

        let confirmed = Confirm::new()
            .with_prompt("是否迁移到新目录?")
            .default(true)
            .interact()
            .map_err(|e| CcrError::ConfigError(format!("交互失败: {}", e)))?;

        Ok(confirmed)
    }

    /// 🔁 迁移旧版目录到新目录
    fn migrate_legacy_dir(&self) -> Result<()> {
        if !self.legacy_ui_dir.exists() {
            return Ok(());
        }

        if self.ui_dir.exists() {
            return Err(CcrError::ConfigError(format!(
                "无法迁移：目标目录已存在: {}",
                self.ui_dir.display()
            )));
        }

        let parent_dir = self
            .ui_dir
            .parent()
            .ok_or_else(|| CcrError::ConfigError("无法获取 UI 目录父路径".to_string()))?;

        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)
                .map_err(|e| CcrError::ConfigError(format!("创建目录失败: {}", e)))?;
        }

        // 优先尝试原地移动（同文件系统时为 O(1)）
        match fs::rename(&self.legacy_ui_dir, &self.ui_dir) {
            Ok(_) => {
                ColorOutput::success(&format!("✅ 已迁移到新目录: {}", self.ui_dir.display()));
                Ok(())
            }
            Err(e) => {
                ColorOutput::warning(&format!("⚠️  目录移动失败，将改为复制: {}", e));
                self.copy_dir_recursive(&self.legacy_ui_dir, &self.ui_dir)?;
                ColorOutput::success(&format!("✅ 已复制到新目录: {}", self.ui_dir.display()));
                Ok(())
            }
        }
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
    fn prompt_download(&self, auto_yes: bool) -> Result<bool> {
        use dialoguer::Confirm;

        if auto_yes {
            return Ok(true);
        }

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

    /// 🔄 从 GitHub 同步 CCR UI（安装/更新）
    fn sync_from_github(&self, auto_yes: bool) -> Result<()> {
        use tempfile::TempDir;

        ColorOutput::step("从 GitHub 同步 CCR UI");
        println!();

        let temp_dir = TempDir::new()
            .map_err(|e| CcrError::ConfigError(format!("创建临时目录失败: {}", e)))?;

        ColorOutput::info(&format!(
            "📦 克隆仓库: https://github.com/{}.git",
            GITHUB_REPO
        ));
        ColorOutput::info(&format!("📁 临时目录: {}", temp_dir.path().display()));
        println!();

        ColorOutput::warning("⏳ 下载中 (这可能需要几分钟)...");

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

        let ccr_ui_src = temp_dir.path().join("ccr-ui");
        if !ccr_ui_src.exists() {
            return Err(CcrError::ConfigError(
                "下载的仓库中未找到 ccr-ui 目录".to_string(),
            ));
        }

        if !ccr_ui_src.join("justfile").exists() {
            return Err(CcrError::ConfigError(
                "ccr-ui 目录不完整，缺少 justfile".to_string(),
            ));
        }

        self.install_or_update_ui_from_source(&ccr_ui_src, auto_yes)?;

        Ok(())
    }

    /// 📥 基于源码目录安装/更新 UI 到用户目录（默认保留依赖缓存）
    fn install_or_update_ui_from_source(&self, src_ui_dir: &Path, auto_yes: bool) -> Result<()> {
        use dialoguer::Confirm;
        use tempfile::TempDir;

        let parent_dir = self
            .ui_dir
            .parent()
            .ok_or_else(|| CcrError::ConfigError("无法获取 UI 目录父路径".to_string()))?;

        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)
                .map_err(|e| CcrError::ConfigError(format!("创建目录失败: {}", e)))?;
        }

        // 兼容：如果旧路径存在且新路径不存在，优先引导迁移（保留缓存）
        if !self.ui_dir.exists()
            && self.legacy_ui_dir.exists()
            && self.legacy_ui_dir.join("justfile").exists()
            && self.prompt_migrate_legacy(auto_yes)?
        {
            self.migrate_legacy_dir()?;
        }

        let is_update = self.ui_dir.exists() && self.ui_dir.join("justfile").exists();
        if is_update && !auto_yes {
            println!();
            ColorOutput::warning("⚠️  检测到已安装的 CCR UI，将执行更新并覆盖源码文件");
            ColorOutput::info("默认会尽量保留以下缓存目录以避免重复安装：");
            ColorOutput::info("  - frontend/node_modules");
            ColorOutput::info("  - backend/target");
            println!();

            let confirmed = Confirm::new()
                .with_prompt("是否继续更新?")
                .default(true)
                .interact()
                .map_err(|e| CcrError::ConfigError(format!("交互失败: {}", e)))?;

            if !confirmed {
                return Err(CcrError::ConfigError("用户取消更新".to_string()));
            }
        }

        // 先把新版本复制到同目录的 staging，避免复制失败导致现有安装损坏
        let staging_dir = TempDir::new_in(parent_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建临时目录失败: {}", e)))?;
        self.copy_dir_recursive(src_ui_dir, staging_dir.path())?;

        // 需要保留的缓存目录（相对 ui_dir）
        let preserve_rel_paths = ["frontend/node_modules", "backend/target"];
        let preserve_dir = TempDir::new_in(parent_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建临时目录失败: {}", e)))?;

        let mut preserved: Vec<(PathBuf, PathBuf)> = Vec::new();
        if self.ui_dir.exists() {
            for rel in preserve_rel_paths {
                let from = self.ui_dir.join(rel);
                if !from.exists() {
                    continue;
                }
                let to = preserve_dir.path().join(rel);
                if let Some(parent) = to.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| CcrError::ConfigError(format!("创建目录失败: {}", e)))?;
                }
                fs::rename(&from, &to)
                    .map_err(|e| CcrError::ConfigError(format!("移动缓存目录失败: {}", e)))?;
                preserved.push((to, self.ui_dir.join(rel)));
            }
        }

        // 清空旧安装目录（缓存已暂存）
        if self.ui_dir.exists() {
            fs::remove_dir_all(&self.ui_dir)
                .map_err(|e| CcrError::ConfigError(format!("删除旧目录失败: {}", e)))?;
        }

        // 将 staging 目录原子替换为目标目录
        fs::rename(staging_dir.path(), &self.ui_dir)
            .map_err(|e| CcrError::ConfigError(format!("写入新版本失败: {}", e)))?;

        // 恢复缓存目录
        for (from, to) in preserved {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| CcrError::ConfigError(format!("创建目录失败: {}", e)))?;
            }
            fs::rename(from, to)
                .map_err(|e| CcrError::ConfigError(format!("恢复缓存目录失败: {}", e)))?;
        }

        ColorOutput::success("✅ CCR UI 已同步到最新版本");
        ColorOutput::info(&format!("📁 安装位置: {}", self.ui_dir.display()));
        println!();

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
#[allow(clippy::unwrap_used)]
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
