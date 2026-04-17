//! LCS-based line diff — ported from skill-hub `server/versioning/store.ts`.
//!
//! ## 为什么不用 `similar` crate
//! 零新增依赖原则；skill-hub 的 Myers-简化 LCS 已在生产验证过，
//! 行数截断上限 2000 行使 DP 表规模 ≤ 4 百万 cells（~32 MB），可接受。

use serde::{Deserialize, Serialize};

/// LCS DP 表被截断的行数上限，防止超长 SKILL.md 阻塞 UI 线程。
pub const MAX_LINES: usize = 2000;

/// 每一行的变更类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffLineKind {
    Same,
    Add,
    Remove,
}

/// 单行 diff 结果。`old_line` / `new_line` 为 1-based，方便前端直接展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub content: String,
}

/// 截断信息：当输入超过 [`MAX_LINES`] 时暴露给 UI，避免用户误以为 diff 完整。
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TruncationInfo {
    pub truncated: bool,
    /// 原始 old 文本的总行数（未截断前）
    pub total_old_lines: usize,
    /// 原始 new 文本的总行数（未截断前）
    pub total_new_lines: usize,
    /// 实际参与 diff 的行数上限（= MAX_LINES）
    pub limit: usize,
}

/// 按行计算 `old_text` → `new_text` 的 diff，连同截断信息一起返回。
/// 输入超过 [`MAX_LINES`] 行会截断以保证可预测性能。
pub fn diff_with_truncation(old_text: &str, new_text: &str) -> (Vec<DiffLine>, TruncationInfo) {
    let old_all: Vec<&str> = old_text.split('\n').collect();
    let new_all: Vec<&str> = new_text.split('\n').collect();
    let truncated = old_all.len() > MAX_LINES || new_all.len() > MAX_LINES;
    let info = TruncationInfo {
        truncated,
        total_old_lines: old_all.len(),
        total_new_lines: new_all.len(),
        limit: MAX_LINES,
    };
    (diff_inner(&old_all, &new_all), info)
}

/// 向后兼容的便捷 API — 不需要截断信息时用。
pub fn diff(old_text: &str, new_text: &str) -> Vec<DiffLine> {
    diff_with_truncation(old_text, new_text).0
}

fn diff_inner(old_all: &[&str], new_all: &[&str]) -> Vec<DiffLine> {
    let old_bounded: &[&str] = &old_all[..old_all.len().min(MAX_LINES)];
    let new_bounded: &[&str] = &new_all[..new_all.len().min(MAX_LINES)];

    let lcs_seq = longest_common_subsequence(old_bounded, new_bounded);

    let mut out = Vec::with_capacity(old_bounded.len() + new_bounded.len());
    let mut oi = 0usize;
    let mut ni = 0usize;
    let mut li = 0usize;

    while oi < old_bounded.len() || ni < new_bounded.len() {
        if li < lcs_seq.len()
            && oi < old_bounded.len()
            && ni < new_bounded.len()
            && old_bounded[oi] == lcs_seq[li]
            && new_bounded[ni] == lcs_seq[li]
        {
            out.push(DiffLine {
                kind: DiffLineKind::Same,
                old_line: Some(oi + 1),
                new_line: Some(ni + 1),
                content: old_bounded[oi].to_string(),
            });
            oi += 1;
            ni += 1;
            li += 1;
        } else if oi < old_bounded.len() && (li >= lcs_seq.len() || old_bounded[oi] != lcs_seq[li])
        {
            out.push(DiffLine {
                kind: DiffLineKind::Remove,
                old_line: Some(oi + 1),
                new_line: None,
                content: old_bounded[oi].to_string(),
            });
            oi += 1;
        } else if ni < new_bounded.len() && (li >= lcs_seq.len() || new_bounded[ni] != lcs_seq[li])
        {
            out.push(DiffLine {
                kind: DiffLineKind::Add,
                old_line: None,
                new_line: Some(ni + 1),
                content: new_bounded[ni].to_string(),
            });
            ni += 1;
        } else {
            // 理论上 LCS 正确时不应到这里；防御性退出。
            break;
        }
    }

    out
}

/// 经典 O(m·n) DP 求 LCS。输入已在调用侧被裁剪到 `MAX_LINES`。
fn longest_common_subsequence<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut seq = Vec::with_capacity(dp[m][n]);
    let mut i = m;
    let mut j = n;
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            seq.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    seq.reverse();
    seq
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_inputs_are_all_same() {
        let text = "a\nb\nc";
        let d = diff(text, text);
        assert!(d.iter().all(|l| l.kind == DiffLineKind::Same));
        assert_eq!(d.len(), 3);
    }

    #[test]
    fn empty_inputs_produce_empty_diff() {
        let d = diff("", "");
        // "" split('\n') -> vec![""]; same line, len=1
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].kind, DiffLineKind::Same);
    }

    #[test]
    fn line_numbers_are_one_based() {
        let d = diff("a", "a");
        assert_eq!(d[0].old_line, Some(1));
        assert_eq!(d[0].new_line, Some(1));
    }
}
