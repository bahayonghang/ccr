use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    register_rerun_paths();
    if env::var_os("CARGO_FEATURE_WEB").is_none() {
        return;
    }

    let web_dir = PathBuf::from("ccr-ui").join("web");
    ensure_npm_available(&web_dir);
    run_npm_command(&web_dir, &["ci"], "安装前端依赖 (npm ci)");
    run_npm_command(
        &web_dir,
        &["run", "build"],
        "构建嵌入式 Web 资源 (npm run build)",
    );
    ensure_dist_artifacts(&web_dir);
}

fn register_rerun_paths() {
    let watched_paths = [
        "ccr-ui/web/src",
        "ccr-ui/web/package.json",
        "ccr-ui/web/package-lock.json",
        "ccr-ui/web/vite.config.js",
        "ccr-ui/web/index.html",
    ];

    for path in watched_paths {
        println!("cargo:rerun-if-changed={path}");
    }
}

fn ensure_npm_available(web_dir: &Path) {
    let output = Command::new(npm_binary())
        .arg("--version")
        .current_dir(web_dir)
        .output();

    match output {
        Ok(result) if result.status.success() => {}
        Ok(result) => fail_build(&format!(
            "[ccr-build] npm 不可用（退出码: {:?}）。\n\
请先安装 Node.js 18+（包含 npm），并确认以下命令可用：\n\
  node --version\n\
  npm --version\n\
npm 输出:\n{}\n{}",
            result.status.code(),
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        )),
        Err(err) => fail_build(&format!(
            "[ccr-build] 无法执行 npm: {err}\n\
请先安装 Node.js 18+（包含 npm），并确认以下命令可用：\n\
  node --version\n\
  npm --version"
        )),
    }
}

fn run_npm_command(web_dir: &Path, args: &[&str], step: &str) {
    let output = Command::new(npm_binary())
        .args(args)
        .current_dir(web_dir)
        .output()
        .unwrap_or_else(|err| {
            fail_build(&format!(
                "[ccr-build] 执行 npm 命令失败（{step}）: {err}\n\
请确认 npm 可用后重试。"
            ))
        });

    if output.status.success() {
        return;
    }

    fail_build(&format!(
        "[ccr-build] {step} 失败（退出码: {:?}）。\n\
请在仓库根目录手动执行以下命令复现并修复：\n\
  cd ccr-ui/web\n\
  npm ci\n\
  npm run build\n\
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
