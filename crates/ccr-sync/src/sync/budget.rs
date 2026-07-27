use ccr_core::core::error::{CcrError, Result};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SyncLimits {
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
    pub max_depth: usize,
    pub max_entries: usize,
    pub max_component_bytes: usize,
    pub deadline: Duration,
}

impl Default for SyncLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 32 * 1024 * 1024,
            max_total_bytes: 128 * 1024 * 1024,
            max_depth: 16,
            max_entries: 10_000,
            max_component_bytes: 255,
            deadline: Duration::from_secs(120),
        }
    }
}

#[derive(Debug)]
pub struct SyncBudget {
    limits: SyncLimits,
    started_at: Instant,
    total_bytes: u64,
    entries: usize,
}

impl SyncBudget {
    pub fn new(limits: SyncLimits) -> Self {
        Self {
            limits,
            started_at: Instant::now(),
            total_bytes: 0,
            entries: 0,
        }
    }

    pub fn limits(&self) -> &SyncLimits {
        &self.limits
    }

    pub fn check_deadline(&self) -> Result<()> {
        if self.started_at.elapsed() > self.limits.deadline {
            return Err(limit_error("deadline", "同步操作超过时间上限"));
        }
        Ok(())
    }

    pub fn check_depth(&self, depth: usize) -> Result<()> {
        self.check_deadline()?;
        if depth > self.limits.max_depth {
            return Err(limit_error("depth", "远端目录深度超过上限"));
        }
        Ok(())
    }

    pub fn record_entry(&mut self, depth: usize) -> Result<()> {
        self.check_depth(depth)?;
        self.entries = self
            .entries
            .checked_add(1)
            .ok_or_else(|| limit_error("entries", "远端条目计数溢出"))?;
        if self.entries > self.limits.max_entries {
            return Err(limit_error("entries", "远端条目数量超过上限"));
        }
        Ok(())
    }

    pub fn record_chunk(&mut self, file_bytes: &mut u64, chunk_len: usize) -> Result<()> {
        self.check_deadline()?;
        let chunk_len =
            u64::try_from(chunk_len).map_err(|_| limit_error("bytes", "同步数据块长度无效"))?;
        *file_bytes = file_bytes
            .checked_add(chunk_len)
            .ok_or_else(|| limit_error("file_bytes", "单文件字节计数溢出"))?;
        self.total_bytes = self
            .total_bytes
            .checked_add(chunk_len)
            .ok_or_else(|| limit_error("total_bytes", "总字节计数溢出"))?;

        if *file_bytes > self.limits.max_file_bytes {
            return Err(limit_error("file_bytes", "单文件大小超过上限"));
        }
        if self.total_bytes > self.limits.max_total_bytes {
            return Err(limit_error("total_bytes", "同步总大小超过上限"));
        }
        Ok(())
    }

    pub fn check_declared_file_size(&self, size: u64) -> Result<()> {
        self.check_deadline()?;
        if size > self.limits.max_file_bytes {
            return Err(limit_error("file_bytes", "远端声明的单文件大小超过上限"));
        }
        if self.total_bytes.saturating_add(size) > self.limits.max_total_bytes {
            return Err(limit_error(
                "total_bytes",
                "远端声明的数据将超过同步总大小上限",
            ));
        }
        Ok(())
    }
}

fn limit_error(code: &str, message: &str) -> CcrError {
    CcrError::SyncError(format!("sync_limit_{code}: {message}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn tiny_limits() -> SyncLimits {
        SyncLimits {
            max_file_bytes: 4,
            max_total_bytes: 6,
            max_depth: 2,
            max_entries: 2,
            max_component_bytes: 16,
            deadline: Duration::from_secs(1),
        }
    }

    #[test]
    fn enforces_file_total_depth_and_entry_limits() {
        let mut budget = SyncBudget::new(tiny_limits());
        budget.record_entry(1).unwrap();
        budget.record_entry(2).unwrap();
        assert!(
            budget
                .record_entry(1)
                .unwrap_err()
                .to_string()
                .contains("entries")
        );

        let mut first = 0;
        budget.record_chunk(&mut first, 4).unwrap();
        assert!(
            budget
                .record_chunk(&mut first, 1)
                .unwrap_err()
                .to_string()
                .contains("file_bytes")
        );

        let mut second = 0;
        assert!(
            budget
                .record_chunk(&mut second, 3)
                .unwrap_err()
                .to_string()
                .contains("total_bytes")
        );
        assert!(
            budget
                .check_depth(3)
                .unwrap_err()
                .to_string()
                .contains("depth")
        );
    }
}
