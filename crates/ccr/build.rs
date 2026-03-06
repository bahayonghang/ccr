use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    register_rerun_paths();
    if env::var_os("CARGO_FEATURE_WEB").is_none() {
        return;
    }

    let web_dir = workspace_root().join("ccr-ui").join("web");
    let package_manager = detect_package_manager(&web_dir);

    let install_args = package_manager.install_args();
    run_package_manager_command(
        &web_dir,
        &package_manager,
        &install_args,
        &format!(
            "安装前端依赖 ({})",
            package_manager.command_preview(&install_args)
        ),
    );

    let build_args = package_manager.build_args();
    run_package_manager_command(
        &web_dir,
        &package_manager,
        &build_args,
        &format!(
            "构建嵌入式 Web 资源 ({})",
            package_manager.command_preview(&build_args)
        ),
    );

    ensure_dist_artifacts(&web_dir);
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be available"),
    );

    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("crates/ccr should always have a workspace root parent")
        .to_path_buf()
}

fn register_rerun_paths() {
    let workspace_root = workspace_root();
    let watched_paths = [
        workspace_root.join("ccr-ui/web/src"),
        workspace_root.join("ccr-ui/web/bun.lock"),
        workspace_root.join("ccr-ui/web/bun.lockb"),
        workspace_root.join("ccr-ui/web/package.json"),
        workspace_root.join("ccr-ui/web/package-lock.json"),
        workspace_root.join("ccr-ui/web/vite.config.js"),
        workspace_root.join("ccr-ui/web/index.html"),
    ];

    for path in watched_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

enum PackageManager {
    Bun { frozen_lockfile: bool },
    Npm,
}

impl PackageManager {
    fn binary(&self) -> &'static str {
        match self {
            Self::Bun { .. } => "bun",
            Self::Npm => npm_binary(),
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Bun { .. } => "bun",
            Self::Npm => "npm",
        }
    }

    fn install_args(&self) -> Vec<&'static str> {
        match self {
            Self::Bun {
                frozen_lockfile: true,
            } => vec!["install", "--frozen-lockfile"],
            Self::Bun {
                frozen_lockfile: false,
            } => vec!["install", "--no-save"],
            Self::Npm => vec!["ci"],
        }
    }

    fn build_args(&self) -> Vec<&'static str> {
        vec!["run", "build"]
    }

    fn command_preview(&self, args: &[&str]) -> String {
        if args.is_empty() {
            self.binary().to_string()
        } else {
            format!("{} {}", self.binary(), args.join(" "))
        }
    }
}

fn detect_package_manager(web_dir: &Path) -> PackageManager {
    if command_succeeds(web_dir, "bun", &["--version"]) {
        let frozen_lockfile =
            web_dir.join("bun.lock").exists() || web_dir.join("bun.lockb").exists();
        return PackageManager::Bun { frozen_lockfile };
    }

    if command_succeeds(web_dir, npm_binary(), &["--version"]) {
        return PackageManager::Npm;
    }

    fail_build(
        "[ccr-build] 未检测到可用的前端包管理器。\n\
请安装 Bun（推荐）或 Node.js 18+（包含 npm），并确认以下命令至少有一组可用：\n\
  bun --version\n\
  npm --version",
    );
}

fn command_succeeds(web_dir: &Path, binary: &str, args: &[&str]) -> bool {
    Command::new(binary)
        .args(args)
        .current_dir(web_dir)
        .output()
        .map(|result| result.status.success())
        .unwrap_or(false)
}

fn run_package_manager_command(
    web_dir: &Path,
    package_manager: &PackageManager,
    args: &[&str],
    step: &str,
) {
    let output = Command::new(package_manager.binary())
        .args(args)
        .current_dir(web_dir)
        .output()
        .unwrap_or_else(|err| {
            fail_build(&format!(
                "[ccr-build] 执行 {} 命令失败（{step}）: {err}\n\
请确认 {} 可用后重试。",
                package_manager.display_name(),
                package_manager.display_name()
            ))
        });

    if output.status.success() {
        return;
    }

    let install_command = package_manager.command_preview(&package_manager.install_args());
    let build_command = package_manager.command_preview(&package_manager.build_args());

    fail_build(&format!(
        "[ccr-build] {step} 失败（退出码: {:?}）。\n\
请在仓库根目录手动执行以下命令复现并修复：\n\
  cd ccr-ui/web\n\
  {install_command}\n\
  {build_command}\n\
stdout:\n{}\n\
stderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ));
}

fn ensure_dist_artifacts(web_dir: &Path) {
    let dist_dir = web_dir.join("dist");
    let required_files = ["index.html", "style.css", "script.js"];
    let missing_files: Vec<String> = required_files
        .iter()
        .filter_map(|name| {
            let path = dist_dir.join(name);
            if path.exists() {
                None
            } else {
                Some(path.display().to_string())
            }
        })
        .collect();

    if missing_files.is_empty() {
        return;
    }

    fail_build(&format!(
        "[ccr-build] Web 构建完成但缺少嵌入式资源文件:\n{}\n\
请检查 ccr-ui/web/vite.config.js 的输出文件名是否为 index.html/style.css/script.js。",
        missing_files.join("\n")
    ));
}

fn npm_binary() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

fn fail_build(message: &str) -> ! {
    panic!("\n{message}\n");
}
