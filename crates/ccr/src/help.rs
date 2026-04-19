// 帮助输出薄封装
//
// 所有帮助正文都来自增强后的 clap 命令树。
// `ccr help ...` 只是把同一条 `--help` 输出再走一遍。

use clap::error::ErrorKind;

fn render_help_text(path: &[String]) -> Result<String, String> {
    let arg0 = std::env::args_os()
        .next()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ccr".to_string());

    let mut args = vec![arg0];
    args.extend(path.iter().cloned());
    args.push("--help".to_string());

    match crate::cli::build_cli_command().try_get_matches_from(args) {
        Err(err)
            if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            Ok(err.to_string())
        }
        Err(err) => Err(err.to_string()),
        Ok(_) => Err("未能生成帮助输出".to_string()),
    }
}

pub fn print_command_help(path: &[String]) {
    match render_help_text(path) {
        Ok(text) => print!("{text}"),
        Err(err) => eprintln!("{err}"),
    }
}

/// 打印子命令帮助（复用增强后的 clap 输出）
pub fn print_subcommand_help(name: &str) {
    print_command_help(&[name.to_string()]);
}

/// 打印嵌套子命令帮助（如 "codex auth"）
pub fn print_nested_subcommand_help(path: &[&str]) {
    let path = path
        .iter()
        .map(|segment| (*segment).to_string())
        .collect::<Vec<_>>();
    print_command_help(&path);
}
