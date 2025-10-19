// 🎨 UI 服务层
// 负责启动和管理 CCR UI (Web 应用)

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// 🎨 UI 服务
///
/// 提供 CCR UI 的启动和管理功能
pub struct UiService {
    /// CCR-UI 项目路径 (开发模式使用)
    ccr_ui_path: Option<PathBuf>,
    /// UI 资源目录 (~/.ccr/ui/) - 预留用于预构建版本
    #[allow(dead_code)]
    ui_dir: PathBuf,
}

impl UiService {
    /// 🏗️ 创建新的 UI 服务
    pub fn new() -> Result<Self> {
        // 获取用户主目录
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".to_string()))?;

        // UI 资源目录 (预留用于预构建版本)
        let ui_dir = home.join(".ccr/ui");

        // 检查是否在开发环境中
        let ccr_ui_path = Self::detect_ccr_ui_path();

        Ok(Self {
            ccr_ui_path,
            ui_dir,
        })
    }

    /// 🔍 检测 ccr-ui 项目路径
    ///
    /// 优先级:
    /// 1. 当前目录下的 ccr-ui/
    /// 2. CCR 项目根目录下的 ccr-ui/
    fn detect_ccr_ui_path() -> Option<PathBuf> {
        // 尝试当前目录
        let current_dir_ui = std::env::current_dir()
            .ok()
            .map(|p| p.join("ccr-ui"));

        if let Some(ref path) = current_dir_ui {
            if path.exists() && path.join("justfile").exists() {
                return Some(path.clone());
            }
        }

        // 尝试父目录 (适用于在 ccr/src 等子目录运行的情况)
        let parent_dir_ui = std::env::current_dir()
            .ok()
            .and_then(|p| p.parent().map(|parent| parent.join("ccr-ui")));

        if let Some(ref path) = parent_dir_ui {
            if path.exists() && path.join("justfile").exists() {
                return Some(path.clone());
            }
        }

        None
    }

    /// 🚀 启动 UI (智能选择模式)
    ///
    /// 根据环境自动选择最佳启动方式:
    /// - 开发环境: 使用 `just dev` 启动源码
    /// - 生产环境: 启动预构建版本 (未来实现)
    pub fn start(&self, port: u16, backend_port: u16) -> Result<()> {
        ColorOutput::title("🚀 CCR UI 启动中...");
        println!();

        // 检查开发环境
        if let Some(ref ccr_ui_path) = self.ccr_ui_path {
            ColorOutput::info(&format!(
                "📁 检测到开发环境: {}",
                ccr_ui_path.display()
            ));
            return self.start_dev_mode(ccr_ui_path, port, backend_port);
        }

        // 未来: 检查预构建版本
        // if self.has_local_version()? {
        //     return self.start_local(port, backend_port);
        // }

        // 未找到任何可用的 UI
        Err(CcrError::ConfigError(
            "未找到 CCR UI 资源\n\n\
            请确保:\n\
            1. 在 CCR 项目根目录下运行此命令\n\
            2. ccr-ui/ 目录存在\n\n\
            或等待未来版本支持自动下载 UI 资源"
                .to_string(),
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
                    ColorOutput::success(&format!(
                        "✅ just 已安装: {}",
                        version.trim()
                    ));
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
        Err(CcrError::ConfigError(
            "预构建版本功能尚未实现".to_string(),
        ))
    }

    /// 🚀 启动本地预构建版本 (预留)
    #[allow(dead_code)]
    fn start_local(&self, _port: u16, _backend_port: u16) -> Result<()> {
        ColorOutput::info("🚀 预构建版本启动功能将在未来版本中实现");
        Err(CcrError::ConfigError(
            "预构建版本功能尚未实现".to_string(),
        ))
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
