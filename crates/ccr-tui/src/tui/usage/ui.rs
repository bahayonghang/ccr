// 用量数字格式化助手。整页 Usage tab 已下线,这里只保留 profile 详情
// Usage 分组(tui/ui.rs::usage_section_lines)复用的紧凑格式化函数。

pub(crate) fn format_count(value: i64) -> String {
    let abs = value.abs();
    if abs >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if abs >= 1_000 {
        format!("{:.1}K", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}

pub(crate) fn format_cost(value: f64) -> String {
    if value >= 100.0 {
        format!("${value:.0}")
    } else if value >= 1.0 {
        format!("${value:.2}")
    } else {
        format!("${value:.4}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_formatting_compacts_large_values() {
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1_500), "1.5K");
        assert_eq!(format_count(2_000_000), "2.0M");
    }

    #[test]
    fn cost_formatting_scales_precision_by_magnitude() {
        assert_eq!(format_cost(0.0123), "$0.0123");
        assert_eq!(format_cost(2.5), "$2.50");
        assert_eq!(format_cost(24571.0), "$24571");
    }
}
