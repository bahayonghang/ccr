//! Measured process facts shared by desktop adapters.
pub fn current_process_rss_bytes() -> Option<u64> {
    use sysinfo::{Pid, ProcessesToUpdate, System};
    let pid = Pid::from_u32(std::process::id());
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    system.process(pid).map(|process| process.memory())
}

/// Bounded nearest-rank durations. The caller owns the operation cohort and only
/// records real completed observations; this type never initiates work.
#[derive(Default)]
pub struct LatencyWindow {
    samples_ms: std::collections::VecDeque<f64>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LatencySnapshot {
    /// Retained samples, not a lifetime success count.
    pub sample_count: usize,
    pub p95_ms: Option<f64>,
}
impl LatencyWindow {
    pub const CAPACITY: usize = 2048;
    pub fn record(&mut self, duration: std::time::Duration) {
        if self.samples_ms.len() == Self::CAPACITY {
            self.samples_ms.pop_front();
        }
        self.samples_ms.push_back(duration.as_secs_f64() * 1000.0);
    }
    pub fn snapshot(&self) -> LatencySnapshot {
        let mut sorted: Vec<_> = self.samples_ms.iter().copied().collect();
        sorted.sort_by(f64::total_cmp);
        let rank = (sorted.len() * 95).div_ceil(100);
        LatencySnapshot {
            sample_count: sorted.len(),
            p95_ms: rank.checked_sub(1).map(|index| sorted[index]),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn latency_empty_zero_and_nearest_rank_are_distinct() {
        let mut window = LatencyWindow::default();
        assert_eq!(window.snapshot(), LatencySnapshot::default());
        window.record(Duration::ZERO);
        assert_eq!(
            window.snapshot(),
            LatencySnapshot {
                sample_count: 1,
                p95_ms: Some(0.0)
            }
        );
        let mut window = LatencyWindow::default();
        for value in 1..=20 {
            window.record(Duration::from_millis(value));
        }
        assert_eq!(
            window.snapshot(),
            LatencySnapshot {
                sample_count: 20,
                p95_ms: Some(19.0)
            }
        );
    }
    #[test]
    fn latency_window_evicts_oldest_and_counts_only_retained_samples() {
        let mut window = LatencyWindow::default();
        for value in 0..LatencyWindow::CAPACITY + 100 {
            window.record(Duration::from_millis(value as u64));
        }
        assert_eq!(
            window.snapshot(),
            LatencySnapshot {
                sample_count: 2048,
                p95_ms: Some(2045.0)
            }
        );
        let independent = LatencyWindow::default();
        assert_eq!(independent.snapshot().p95_ms, None);
    }
}
