//! Layout for contextual CLI command suggestions.

/// Print action names and commands after the current result.
///
/// An empty slice prints no heading or whitespace. Commands are not executed.
pub fn print_next_steps(steps: &[(&str, &str)]) {
    if !steps.is_empty() {
        println!("{}", format_next_steps(steps));
    }
}

fn format_next_steps(steps: &[(&str, &str)]) -> String {
    if steps.is_empty() {
        return String::new();
    }

    let mut output = String::from("\n下一步");
    for (action, command) in steps {
        for line in action.split('\n') {
            output.push_str(&format!("\n  {line}"));
        }
        for line in command.split('\n') {
            output.push_str(&format!("\n    {line}"));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_suggestions_have_no_heading_or_whitespace() {
        assert_eq!(format_next_steps(&[]), "");
    }

    #[test]
    fn suggestions_separate_actions_from_complete_commands() {
        assert_eq!(
            format_next_steps(&[
                ("查看账号", "ccr codex auth list"),
                ("查看帮助", "ccr codex auth save --help"),
            ]),
            "\n下一步\n  查看账号\n    ccr codex auth list\n  查看帮助\n    ccr codex auth save --help"
        );
    }

    #[test]
    fn suggestions_keep_multiline_and_long_text() {
        let command = format!("ccr export {}", "a".repeat(256));
        let output = format_next_steps(&[("操作说明\n续行", &command)]);
        assert_eq!(
            output,
            format!("\n下一步\n  操作说明\n  续行\n    {command}")
        );
    }
}
