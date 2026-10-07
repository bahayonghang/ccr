use ccr_core::core::logging::ColorOutput;

const FAILURE_LOG_LINES: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateFailureKind {
    MissingPackageManager,
    CompilationFailed,
}

pub(crate) fn handle_update_failure(repo_url: &str, branch: &str, package: &str, stderr: &str) {
    ColorOutput::error("更新失败");
    println!();

    let kind = classify_update_failure(stderr);
    print_failure_guidance(kind, repo_url, branch, package);
    print_failure_log_tail(stderr);
    println!();
}

fn classify_update_failure(stderr: &str) -> UpdateFailureKind {
    let stderr_lower = stderr.to_ascii_lowercase();
    if is_missing_package_manager_error(&stderr_lower) {
        return UpdateFailureKind::MissingPackageManager;
    }
    UpdateFailureKind::CompilationFailed
}

fn is_missing_package_manager_error(stderr_lower: &str) -> bool {
    stderr_lower.contains("[ccr-build]")
        || stderr_lower.contains("bun: command not found")
        || stderr_lower.contains("'bun' is not recognized")
        || stderr_lower.contains("npm: command not found")
        || stderr_lower.contains("'npm' is not recognized")
        || (stderr_lower.contains("npm") && stderr_lower.contains("不是内部或外部命令"))
        || (stderr_lower.contains("bun") && stderr_lower.contains("不是内部或外部命令"))
}

fn print_failure_guidance(kind: UpdateFailureKind, repo_url: &str, branch: &str, package: &str) {
    ColorOutput::key_value("源码仓库", repo_url, 2);
    ColorOutput::key_value("更新分支", branch, 2);
    ColorOutput::key_value("软件包", package, 2);
    match kind {
        UpdateFailureKind::MissingPackageManager => {
            ColorOutput::info("已识别原因: 当前环境缺少可用的前端包管理器，无法构建相关前端资源");
            ColorOutput::info("安装 Bun 1.3+ 或 Node.js 18+ 与 npm 后，重新执行原更新命令");
            crate::commands::common::print_next_steps(&[
                ("检查 Bun", "bun --version"),
                ("检查 npm", "npm --version"),
            ]);
        }
        UpdateFailureKind::CompilationFailed => {
            ColorOutput::info("已识别原因: Cargo 编译失败（未匹配到特定模式）");
            ColorOutput::info("查看编译日志中的第一个 error，并检查工具链、网络和 Git 后重试");
            crate::commands::common::print_next_steps(&[
                ("检查 Rust 工具链", "rustc --version"),
                ("查看更新选项", "ccr update --help"),
            ]);
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
    fn test_classify_update_failure_missing_package_manager() {
        let stderr = "[ccr-build] 无法执行 npm: No such file or directory (os error 2)";
        assert_eq!(
            classify_update_failure(stderr),
            UpdateFailureKind::MissingPackageManager
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
