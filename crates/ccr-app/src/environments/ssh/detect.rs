//! Fixed remote PATH checks bound to one already verified connection.
use super::*;

pub(super) const CLI_TOOLS: [(&str, &str); 4] = [
    ("claude", "claude"),
    ("codex", "codex"),
    ("gemini", "agy"),
    ("opencode", "opencode"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliStatus {
    pub name: &'static str,
    pub installed: bool,
    pub path: Option<String>,
}
/// Opaque authority captured before queueing; a reconnect cannot rebind it.
#[derive(Clone)]
pub struct CliSession {
    environment: String,
    session_id: uuid::Uuid,
}

fn path(bytes: &[u8]) -> Result<Option<String>, Error> {
    if bytes.len() > 4096 {
        return Err("ssh_cli_output_invalid");
    }
    let value = std::str::from_utf8(bytes)
        .map_err(|_| "ssh_cli_output_invalid")?
        .trim();
    if value.is_empty() {
        return Ok(None);
    }
    if !value.starts_with('/') || value.chars().any(char::is_control) {
        return Err("ssh_cli_output_invalid");
    }
    Ok(Some(value.into()))
}
impl Workspace {
    pub fn cli_session(&self, id: &str) -> Result<CliSession, Error> {
        let session = self.current(id)?;
        Ok(CliSession {
            environment: session.id,
            session_id: session.session_id,
        })
    }
    pub async fn detect_cli(
        &self,
        authority: &CliSession,
        cancel: CancellationToken,
    ) -> Result<Vec<CliStatus>, Error> {
        let _guard = self.serial.lock().await;
        if cancel.is_cancelled() {
            return Err("ssh_cancelled");
        }
        let session = self.current(&authority.environment)?;
        if session.session_id != authority.session_id {
            return Err("ssh_session_changed");
        }
        let result = async {
            let mut rows = Vec::new();
            for (name, tool) in CLI_TOOLS {
                if cancel.is_cancelled() {
                    return Err("ssh_cancelled");
                }
                let mut command = Self::target(&session.host)?
                    .ssh_command(&self.known_hosts, 10)
                    .map_err(|_| "ssh_invalid_target")?;
                command.arg(format!("command -v {} || true", posix_single_quote(tool)));
                let output = self
                    .transport
                    .run(command, ProcessDescriptor::openssh(), None, cancel.clone())
                    .await?;
                if cancel.is_cancelled() {
                    return Err("ssh_cancelled");
                }
                if !output.success {
                    return Err(
                        classify_ssh_failure(&String::from_utf8_lossy(&output.stderr)).code(),
                    );
                }
                let path = path(&output.stdout)?;
                rows.push(CliStatus {
                    name,
                    installed: path.is_some(),
                    path,
                });
            }
            Ok(rows)
        }
        .await;
        if result.is_err() {
            self.clear();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_are_bounded_single_line_facts_not_versions_or_untrusted_banners() {
        assert_eq!(
            path(b" /usr/local/bin/claude\n"),
            Ok(Some("/usr/local/bin/claude".into()))
        );
        assert_eq!(path(b"\n"), Ok(None));
        for bytes in [
            b"banner\n/usr/bin/codex".as_slice(),
            b"claude 1.0",
            b"/bin/cli\nextra",
            b"/bin/\0cli",
            &[b'a'; 4097],
        ] {
            assert_eq!(path(bytes), Err("ssh_cli_output_invalid"));
        }
    }
}
