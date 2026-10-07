use ccr_core::core::logging::{ColorOutput, OutputStatus};
use std::process::{Command, Output};

const BEGIN: &str = "OUTPUT_PROBE_BEGIN\n";
const END: &str = "OUTPUT_PROBE_END\n";

// The test runner is the probe process. No product command or parent env mutation is needed.
#[test]
#[ignore = "invoked by isolated child-process output tests"]
fn output_probe() {
    ColorOutput::configure_cli_output();
    println!("OUTPUT_PROBE_BEGIN");
    match std::env::var("CCR_OUTPUT_PROBE").as_deref() {
        Ok("messages") => {
            ColorOutput::success("saved teacher");
            ColorOutput::info("account details");
            ColorOutput::warning("check account");
            ColorOutput::error("save failed");
            ColorOutput::step("saving");
            ColorOutput::key_value("邮箱", "tea***@example.test", 2);
            ColorOutput::key_value("描述", "first\nsecond", 2);
            ColorOutput::key_value_sensitive("密钥", "sk-test-1234567890abcdef", 2);
        }
        Ok("terminal") => {
            for status in [
                OutputStatus::Success,
                OutputStatus::Warning,
                OutputStatus::Error,
                OutputStatus::Step,
                OutputStatus::Skipped,
            ] {
                println!("{}", ColorOutput::format_status(status, "message", true));
            }
        }
        Ok("format_only") => {
            let _message = ColorOutput::format_status(OutputStatus::Error, "not printed", false);
        }
        Ok("decorations") => {
            ColorOutput::title("Account details");
            ColorOutput::banner("test");
            ColorOutput::separator();
            ColorOutput::config_status("teacher", true, Some("Synthetic account"));
            ColorOutput::env_status("TEST_OUTPUT", None, false);
            assert!(!ColorOutput::ask_confirmation("Continue?", false));
        }
        mode => panic!("unknown output probe mode: {mode:?}"),
    }
    println!("OUTPUT_PROBE_END");
}

fn run_probe(mode: &str, env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(std::env::current_exe().expect("test executable path"));
    command
        .args(["--exact", "output_probe", "--ignored", "--nocapture"])
        .env("CCR_OUTPUT_PROBE", mode)
        .env_remove("NO_COLOR")
        .env_remove("CLICOLOR")
        .env_remove("CLICOLOR_FORCE")
        .env("TERM", "xterm-256color");
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().expect("output probe starts");
    assert!(
        output.status.success(),
        "probe failed: status={} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn probe_stdout(output: &Output) -> &str {
    let stdout = std::str::from_utf8(&output.stdout).expect("UTF-8 stdout");
    stdout
        .split_once(BEGIN)
        .expect("stdout begin marker")
        .1
        .split_once(END)
        .expect("stdout end marker")
        .0
}

#[test]
fn captured_messages_keep_streams_plain_text_fields_and_masking() {
    let output = run_probe("messages", &[]);
    assert_eq!(
        probe_stdout(&output),
        "成功: saved teacher\naccount details\n警告: check account\n进度: saving\n  邮箱: tea***@example.test\n  描述: first\n    second\n  密钥: sk-t...cdef\n"
    );
    assert_eq!(output.stderr, "错误: save failed\n".as_bytes());
    assert!(!output.stdout.contains(&0x1b));
    assert!(!output.stderr.contains(&0x1b));
}

#[test]
fn no_color_keeps_terminal_symbols_for_nonempty_and_empty_values() {
    for value in ["1", ""] {
        let output = run_probe("terminal", &[("NO_COLOR", value)]);
        assert_eq!(
            probe_stdout(&output),
            "✓ message\n! 警告: message\n× 错误: message\n→ message\n- 跳过: message\n"
        );
        assert!(!output.stdout.contains(&0x1b));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn dumb_terminal_uses_words_without_ansi_even_when_color_is_forced() {
    let env = [("TERM", "dumb"), ("CLICOLOR_FORCE", "1")];
    let output = run_probe("terminal", &env);
    assert_eq!(
        probe_stdout(&output),
        "成功: message\n警告: message\n错误: message\n进度: message\n跳过: message\n"
    );
    assert!(!output.stdout.contains(&0x1b));
    assert!(output.stderr.is_empty());

    let messages = run_probe("messages", &env);
    assert!(probe_stdout(&messages).starts_with("成功: saved teacher\n"));
    assert!(probe_stdout(&messages).contains("  邮箱: tea***@example.test\n"));
    assert!(!messages.stdout.contains(&0x1b));
    assert!(!messages.stderr.contains(&0x1b));
}

#[test]
fn forced_color_styles_only_markers_and_field_names_on_captured_streams() {
    let output = run_probe("messages", &[("CLICOLOR_FORCE", "1"), ("NO_COLOR", "1")]);
    let stdout = probe_stdout(&output);
    let stderr = std::str::from_utf8(&output.stderr).expect("UTF-8 stderr");
    assert!(stdout.starts_with("\x1b[1;32m成功:\x1b[0m saved teacher\naccount details\n"));
    assert!(stdout.contains("\x1b[1;33m警告:\x1b[0m check account\n"));
    assert!(stdout.contains("\x1b[1;36m进度:\x1b[0m saving\n"));
    assert!(stdout.contains("  \x1b[1m邮箱\x1b[0m: tea***@example.test\n"));
    assert!(stdout.contains("  \x1b[1m密钥\x1b[0m: sk-t...cdef\n"));
    assert_eq!(stderr, "\x1b[1;31m错误:\x1b[0m save failed\n");
}

#[test]
fn formatting_does_not_write_to_either_stream() {
    let output = run_probe("format_only", &[]);
    assert_eq!(probe_stdout(&output), "");
    assert!(output.stderr.is_empty());
}

#[test]
fn startup_dumb_policy_disables_existing_decorations_even_with_force() {
    let output = run_probe("decorations", &[("TERM", "dumb"), ("CLICOLOR_FORCE", "1")]);
    let stdout = probe_stdout(&output);
    assert!(stdout.contains("Account details\n===============\n"));
    assert!(stdout.contains("* teacher - Synthetic account\n"));
    assert!(stdout.contains("  TEST_OUTPUT: (not set)\n"));
    assert!(stdout.contains("? Continue? [y/N]: "));
    assert!(!output.stdout.contains(&0x1b));
    assert!(output.stderr.is_empty());
}

#[test]
fn startup_regular_policy_preserves_color_environment_precedence() {
    let forced = run_probe("decorations", &[("CLICOLOR_FORCE", "1"), ("NO_COLOR", "1")]);
    assert!(forced.stdout.contains(&0x1b));
    let no_color = run_probe("decorations", &[("NO_COLOR", "1")]);
    assert!(!no_color.stdout.contains(&0x1b));
    assert!(forced.stderr.is_empty());
    assert!(no_color.stderr.is_empty());
}
