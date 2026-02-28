use crate::core::logging::ColorOutput;

const FAILURE_LOG_LINES: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateFailureKind {
    MissingEmbeddedWebAssets,
    MissingNodeOrNpm,
    CompilationFailed,
}

pub(crate) fn handle_update_failure(repo_url: &str, branch: &str, package: &str, stderr: &str) {
    ColorOutput::error("❌ 更新失败");
    println!();

    let kind = classify_update_failure(stderr);
    print_failure_guidance(kind, repo_url, branch, package);
    print_failure_log_tail(stderr);
    println!();
}

fn classify_update_failure(stderr: &str) -> UpdateFailureKind {
    let stderr_lower = stderr.to_ascii_lowercase();
    if is_missing_web_assets_error(&stderr_lower) {
        return UpdateFailureKind::MissingEmbeddedWebAssets;
    }
    if is_missing_npm_error(&stderr_lower) {
        return UpdateFailureKind::MissingNodeOrNpm;
    }
    UpdateFailureKind::CompilationFailed
}

fn is_missing_web_assets_error(stderr_lower: &str) -> bool {
    stderr_lower.contains("ccr-ui/web/dist")
        || (stderr_lower.contains("couldn't read")
            && stderr_lower.contains("dist/index.html")
            && stderr_lower.contains("dist/style.css")
            && stderr_lower.contains("dist/script.js"))
}

fn is_missing_npm_error(stderr_lower: &str) -> bool {
    stderr_lower.contains("[ccr-build]")
        || stderr_lower.contains("npm: command not found")
        || stderr_lower.contains("'npm' is not recognized")
        || (stderr_lower.contains("npm") && stderr_lower.contains("不是内部或外部命令"))
}

fn print_failure_guidance(kind: UpdateFailureKind, repo_url: &str, branch: &str, package: &str) {
    match kind {
        UpdateFailureKind::MissingEmbeddedWebAssets => {
            ColorOutput::info("已识别原因: 缺少嵌入式 Web 构建产物 (ccr-ui/web/dist/*)");
            ColorOutput::info("解决方案:");
            println!("  1. 在仓库根目录执行: cd ccr-ui/web && npm ci && npm run build");
            println!("  2. 重新执行更新命令: ccr update {}", branch);
            println!(
                "  3. 手动安装: cargo install --git {} {} --branch {} --force",
                repo_url, package, branch
            );
            println!();
        }
        UpdateFailureKind::MissingNodeOrNpm => {
            ColorOutput::info("已识别原因: 当前环境缺少 Node.js/npm，无法构建内嵌 Web 资源");
            ColorOutput::info("解决方案:");
            println!("  1. 检查命令: node --version && npm --version");
            println!("  2. 安装 Node.js 18+（需包含 npm）");
            println!("  3. 重新执行更新命令: ccr update {}", branch);
            println!(
                "  4. 手动安装: cargo install --git {} {} --branch {} --force",
                repo_url, package, branch
            );
            println!();
        }
        UpdateFailureKind::CompilationFailed => {
            ColorOutput::info("已识别原因: Cargo 编译失败（未匹配到特定模式）");
            ColorOutput::info("解决方案:");
            println!("  1. 优先查看上方编译日志中的第一个 error");
            println!("  2. 更新工具链: rustup update");
            println!("  3. 检查网络与 Git: ping github.com && git --version");
            println!(
                "  4. 手动安装复现: cargo install --git {} {} --branch {} --force",
                repo_url, package, branch
            );
            println!();
        }
    }
}

fn print_failure_log_tail(stderr: &str) {
    let tail = tail_lines(stderr, FAILURE_LOG_LINES);
    if tail.is_empty() {
        return;
    }

    ColorOutput::info(&format!("错误摘要（最近 {} 行）:", tail.len()));
    for line in tail {
        println!("  {line}");
    }
}

fn tail_lines(text: &str, max_lines: usize) -> Vec<&str> {
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let start = lines.len().saturating_sub(max_lines);
    lines[start..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_update_failure_missing_dist() {
        let stderr = "error: couldn't read `src/web/../../ccr-ui/web/dist/index.html`";
        assert_eq!(
            classify_update_failure(stderr),
            UpdateFailureKind::MissingEmbeddedWebAssets
        );
    }

    #[test]
    fn test_classify_update_failure_missing_npm() {
        let stderr = "[ccr-build] 无法执行 npm: No such file or directory (os error 2)";
        assert_eq!(
            classify_update_failure(stderr),
            UpdateFailureKind::MissingNodeOrNpm
        );
    }

    #[test]
    fn test_classify_update_failure_fallback() {
        let stderr = "error[E0432]: unresolved import `crate::foo`";
        assert_eq!(
            classify_update_failure(stderr),
            UpdateFailureKind::CompilationFailed
        );
    }

    #[test]
    fn test_tail_lines_returns_last_non_empty_lines() {
        let stderr = "line1\n\nline2\nline3\n";
        let tail = tail_lines(stderr, 2);
        assert_eq!(tail, vec!["line2", "line3"]);
    }
}
