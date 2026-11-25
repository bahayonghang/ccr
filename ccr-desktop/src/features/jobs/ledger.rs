//! Run ledger model: grouped log lines, clipping, copy text and readouts.
//!
//! 无 GPUI 依赖。账本视图只消费这里的结果。

use ccr_app::jobs::commands::{Channel, Snapshot};
use std::borrow::Cow;

/// Longest log line shown in full; a longer line is cut and ends with「…」.
pub const MAX_LINE_BYTES: usize = 8 * 1024;
/// Fixed width of the `sys / out / err` channel gutter.
pub const GUTTER_WIDTH: f32 = 40.;

/// Mono line height as a multiple of the mono font size.
const LINE_HEIGHT: f32 = 1.5;
/// Unknown readout value.
const UNKNOWN: &str = "–";
const JOB_PREFIX: &str = "ccr-command-";
/// Display order. Lines are grouped by channel and never posed as a timeline.
const ORDER: [Channel; 3] = [Channel::System, Channel::Stdout, Channel::Stderr];

fn lines(snapshot: &Snapshot, channel: Channel) -> &std::collections::VecDeque<String> {
    match channel {
        Channel::System => &snapshot.system_lines,
        Channel::Stdout => &snapshot.stdout_lines,
        Channel::Stderr => &snapshot.stderr_lines,
    }
}

/// Retained lines across all channels.
pub fn line_count(snapshot: &Snapshot) -> usize {
    ORDER
        .iter()
        .map(|channel| lines(snapshot, *channel).len())
        .sum()
}

/// The `index`th row in system → stdout → stderr order, clipped for display.
pub fn line(snapshot: &Snapshot, index: usize) -> Option<(Channel, Cow<'_, str>)> {
    let mut index = index;
    for channel in ORDER {
        let group = lines(snapshot, channel);
        if index < group.len() {
            return group.get(index).map(|text| (channel, clip(text)));
        }
        index -= group.len();
    }
    None
}

/// Cuts a line longer than [`MAX_LINE_BYTES`] on a char boundary and adds「…」.
pub fn clip(line: &str) -> Cow<'_, str> {
    if line.len() <= MAX_LINE_BYTES {
        return Cow::Borrowed(line);
    }
    let mut end = MAX_LINE_BYTES;
    while !line.is_char_boundary(end) {
        end -= 1;
    }
    Cow::Owned(format!("{}…", &line[..end]))
}

/// The short gutter label.
pub fn gutter(channel: Channel) -> &'static str {
    match channel {
        Channel::System => "sys",
        Channel::Stdout => "out",
        Channel::Stderr => "err",
    }
}

/// The full channel name used by copied output and row labels.
pub fn channel_name(channel: Channel) -> &'static str {
    match channel {
        Channel::System => "system",
        Channel::Stdout => "stdout",
        Channel::Stderr => "stderr",
    }
}

/// Retained output as `[channel] line` rows in display order.
pub fn copy_text(snapshot: &Snapshot) -> String {
    ORDER
        .iter()
        .flat_map(|channel| {
            lines(snapshot, *channel)
                .iter()
                .map(move |text| format!("[{}] {text}", channel_name(*channel)))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The job id without its `ccr-command-` prefix, first eight characters.
pub fn short_job_id(id: &str) -> String {
    id.strip_prefix(JOB_PREFIX)
        .unwrap_or(id)
        .chars()
        .take(8)
        .collect()
}

fn command_line(command: &str, args: &[String]) -> String {
    let mut line = format!("ccr {command}");
    for arg in args {
        line.push(' ');
        if arg.is_empty() || arg.chars().any(char::is_whitespace) {
            line.push('"');
            line.push_str(arg);
            line.push('"');
        } else {
            line.push_str(arg);
        }
    }
    line
}

/// `ccr <command> <args> · job <id> · grouped by channel`.
pub fn subtitle(snapshot: &Snapshot, zh: bool) -> String {
    let (job, grouped) = if zh {
        ("任务", "按通道分组")
    } else {
        ("job", "grouped by channel")
    };
    format!(
        "{} · {job} {} · {grouped}",
        command_line(&snapshot.command, &snapshot.args),
        short_job_id(&snapshot.job_id)
    )
}

/// Elapsed seconds with one decimal, or「–」while unknown.
pub fn duration(ms: Option<u64>, zh: bool) -> String {
    match ms {
        Some(ms) => format!("{:.1} {}", ms as f64 / 1000., if zh { "秒" } else { "s" }),
        None => UNKNOWN.to_string(),
    }
}

pub fn exit_code(code: Option<i32>) -> String {
    code.map_or_else(|| UNKNOWN.to_string(), |code| code.to_string())
}

/// Lines the owner dropped, when output was truncated or lines were dropped.
pub fn dropped(snapshot: &Snapshot) -> Option<usize> {
    (snapshot.truncated || snapshot.dropped_lines > 0).then_some(snapshot.dropped_lines)
}

/// Row height and virtual-list stride for one log line.
pub fn row_height(mono_size: f32) -> f32 {
    (mono_size * LINE_HEIGHT).ceil()
}

/// Whether the log sits at its bottom. Offsets are GPUI's: `offset_y` runs
/// from 0 down to `-max_offset_y`. A log that cannot scroll is pinned.
pub fn pinned(offset_y: f32, max_offset_y: f32) -> bool {
    max_offset_y <= 0. || -offset_y >= max_offset_y - 1.
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_app::jobs::commands::Status;
    use std::collections::VecDeque;

    fn snapshot(system: &[&str], stdout: &[&str], stderr: &[&str]) -> Snapshot {
        let lines = |items: &[&str]| items.iter().map(|line| line.to_string()).collect();
        Snapshot {
            job_id: "ccr-command-1234abcd-5678-90ef".into(),
            command: "status".into(),
            args: vec!["--json".into()],
            status: Status::Running,
            cancel_requested: false,
            started_at: "2026-11-25T00:00:00Z".into(),
            finished_at: None,
            duration_ms: None,
            exit_code: None,
            stdout_lines: lines(stdout),
            stderr_lines: lines(stderr),
            system_lines: lines(system),
            truncated: false,
            dropped_lines: 0,
            error: None,
        }
    }

    #[test]
    fn log_lines_group_system_then_stdout_then_stderr() {
        let snapshot = snapshot(&["Job queued"], &["alpha", "beta"], &["warn"]);
        assert_eq!(line_count(&snapshot), 4);
        let rows: Vec<_> = (0..5)
            .map(|index| line(&snapshot, index).map(|(channel, text)| (channel, text.into_owned())))
            .collect();
        assert_eq!(
            rows,
            [
                Some((Channel::System, "Job queued".to_string())),
                Some((Channel::Stdout, "alpha".to_string())),
                Some((Channel::Stdout, "beta".to_string())),
                Some((Channel::Stderr, "warn".to_string())),
                None,
            ]
        );
    }

    #[test]
    fn gutter_names_channels_sys_out_err_and_copy_uses_full_channel_names() {
        assert_eq!(gutter(Channel::System), "sys");
        assert_eq!(gutter(Channel::Stdout), "out");
        assert_eq!(gutter(Channel::Stderr), "err");
        let snapshot = snapshot(&["Job queued"], &["alpha", "beta"], &["warn"]);
        assert_eq!(
            copy_text(&snapshot),
            "[system] Job queued\n[stdout] alpha\n[stdout] beta\n[stderr] warn"
        );
        assert_eq!(copy_text(&self::snapshot(&[], &[], &[])), "");
    }

    #[test]
    fn lines_over_eight_kib_are_cut_on_a_char_boundary_with_an_ellipsis() {
        let exact = "a".repeat(8192);
        assert_eq!(clip(&exact), exact.as_str());
        let long = "a".repeat(9000);
        assert_eq!(clip(&long), format!("{}…", "a".repeat(8192)));
        // 中 is three bytes: 2730 of them fill 8190 bytes, the 2731st would cross 8192.
        let wide = "中".repeat(3000);
        assert_eq!(clip(&wide), format!("{}…", "中".repeat(2730)));
        let snapshot = snapshot(&[], &[long.as_str()], &[]);
        let (_, shown) = line(&snapshot, 0).unwrap_or_else(|| panic!("clipped row"));
        assert!(shown.ends_with('…'));
        assert_eq!(shown.len(), 8192 + '…'.len_utf8());
    }

    #[test]
    fn job_id_drops_the_ccr_command_prefix_then_keeps_eight_characters() {
        assert_eq!(short_job_id("ccr-command-1234abcd-5678-90ef"), "1234abcd");
        assert_eq!(short_job_id("plainjob-identifier"), "plainjob");
        assert_eq!(short_job_id("short"), "short");
    }

    #[test]
    fn subtitle_states_the_command_line_job_and_channel_grouping() {
        let mut snapshot = snapshot(&[], &[], &[]);
        assert_eq!(
            subtitle(&snapshot, false),
            "ccr status --json · job 1234abcd · grouped by channel"
        );
        assert_eq!(
            subtitle(&snapshot, true),
            "ccr status --json · 任务 1234abcd · 按通道分组"
        );
        snapshot.command = "history".into();
        snapshot.args = vec!["-t".into(), "two words".into()];
        assert_eq!(
            subtitle(&snapshot, false),
            "ccr history -t \"two words\" · job 1234abcd · grouped by channel"
        );
    }

    #[test]
    fn readouts_write_seconds_exit_codes_and_a_dash_when_unknown() {
        assert_eq!(duration(None, false), "–");
        assert_eq!(duration(Some(1234), false), "1.2 s");
        assert_eq!(duration(Some(61_500), true), "61.5 秒");
        assert_eq!(duration(Some(0), false), "0.0 s");
        assert_eq!(exit_code(None), "–");
        assert_eq!(exit_code(Some(0)), "0");
        assert_eq!(exit_code(Some(-1)), "-1");
    }

    #[test]
    fn dropped_lines_are_reported_when_output_was_truncated_or_dropped() {
        let mut snapshot = snapshot(&[], &["alpha"], &[]);
        assert_eq!(dropped(&snapshot), None);
        snapshot.truncated = true;
        assert_eq!(dropped(&snapshot), Some(0));
        snapshot.dropped_lines = 3;
        assert_eq!(dropped(&snapshot), Some(3));
        snapshot.truncated = false;
        assert_eq!(dropped(&snapshot), Some(3));
        snapshot.stdout_lines = VecDeque::new();
        assert_eq!(line_count(&snapshot), 0);
    }

    #[test]
    fn log_rows_follow_the_mono_line_height() {
        assert_eq!(row_height(12.), 18.);
        assert_eq!(row_height(13.), 20.);
        assert_eq!(row_height(14.), 21.);
    }

    #[test]
    fn the_log_is_pinned_at_the_bottom_or_when_it_cannot_scroll() {
        assert!(pinned(0., 0.), "a log that does not overflow follows");
        assert!(pinned(-300., 300.));
        assert!(
            pinned(-299.5, 300.),
            "sub-pixel layout still counts as bottom"
        );
        assert!(!pinned(-200., 300.), "scrolled up");
        assert!(!pinned(0., 300.), "at the top of a long log");
    }
}
