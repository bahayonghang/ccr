use super::*;
#[derive(Default)]
pub(super) struct Lines {
    buffers: [Vec<u8>; 2],
    dropping: [bool; 2],
}
impl Lines {
    pub(super) fn push(&mut self, owner: &CommandJobs, id: &str, stderr: bool, bytes: &[u8]) {
        let index = usize::from(stderr);
        for &byte in bytes {
            if byte == b'\n' {
                self.emit(owner, id, index);
                self.dropping[index] = false;
            } else if !self.dropping[index] {
                if self.buffers[index].len() < MAX_BYTES {
                    self.buffers[index].push(byte);
                } else {
                    // 超长未终止行整行丢弃，不能先截断再丢失凭据标签。
                    self.buffers[index].clear();
                    self.dropping[index] = true;
                    owner.update(id, |entry| {
                        entry.snapshot.truncated = true;
                        entry.snapshot.dropped_lines += 1;
                    });
                }
            }
        }
    }
    fn emit(&mut self, owner: &CommandJobs, id: &str, index: usize) {
        if self.dropping[index] {
            return;
        }
        let bytes = std::mem::take(&mut self.buffers[index]);
        let line = ccr_core::core::log_redact::redact_log_text(
            String::from_utf8_lossy(&bytes).trim_end_matches('\r'),
        );
        let channel = if index == 0 {
            Channel::Stdout
        } else {
            Channel::Stderr
        };
        owner.update(id, |entry| {
            entry.snapshot.push(channel, line.clone());
            publish(entry, channel, vec![line], 0, None);
        });
    }
    pub(super) fn finish(&mut self, owner: &CommandJobs, id: &str) {
        for index in 0..2 {
            if !self.buffers[index].is_empty() {
                self.emit(owner, id, index);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::runtime::{EnvironmentId, Platform};
    #[test]
    fn complete_lines_redact_split_credentials_and_bound_unterminated_output() {
        let owner = CommandJobs::default();
        let admitted = owner
            .admit(
                RequestKey {
                    environment: EnvironmentId::Ssh("fixture".into()),
                    platform: Platform::Claude,
                    generation: 1,
                    request_id: 1,
                },
                CommandExecutionRequest::background("status".into(), Some(vec![]), None),
            )
            .unwrap();
        let id = &admitted.snapshot.job_id;
        let mut lines = Lines::default();
        lines.push(&owner, id, false, b"safe text\nAuthorization: Bea");
        lines.push(&owner, id, false, b"rer synthetic-private\n");
        let snapshot = owner.status(id).unwrap();
        assert!(snapshot.stdout_lines.iter().any(|line| line == "safe text"));
        assert!(
            !snapshot
                .stdout_lines
                .iter()
                .any(|line| line.contains("synthetic-private"))
        );
        lines.push(&owner, id, true, &vec![b'x'; MAX_BYTES + 1]);
        lines.finish(&owner, id);
        let snapshot = owner.status(id).unwrap();
        assert!(snapshot.truncated);
        assert_eq!(snapshot.dropped_lines, 1);
        assert!(snapshot.stderr_lines.is_empty());
    }
}
