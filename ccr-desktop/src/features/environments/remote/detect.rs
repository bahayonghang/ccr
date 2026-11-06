use super::*;
use gpui_kit::base::TestSupportExt;
use gpui_kit::{InteractiveElement, StatefulInteractiveElement};

impl RemoteView {
    pub fn clear_cli_on_leave(&mut self, cx: &mut Context<Self>) {
        self.cli = None;
        self.cli_error = None;
        if self.detecting {
            if let Some(cancel) = &self.cancel {
                cancel.request_cancel();
            }
            self.task = None;
            self.cancel = None;
            self.busy = false;
            self.detecting = false;
            self.connected = false;
            self.generation += 1;
        }
        cx.notify();
    }
    pub(super) fn cli_finished(
        &mut self,
        generation: u64,
        id: &str,
        result: &Completion<SshResult, &'static str>,
        cx: &mut Context<Self>,
    ) {
        if self.generation != generation || self.selected.as_ref().is_none_or(|host| host.id != id)
        {
            return;
        }
        self.busy = false;
        self.cancel = None;
        self.detecting = false;
        match result {
            Completion::Success(SshResult::CliDetected(rows)) => self.cli = Some(rows.clone()),
            _ => {
                self.cli = None;
                self.connected = false;
                self.cli_error = Some(match result {
                    Completion::Failed(error) | Completion::CleanupFailed(error) => *error,
                    Completion::Cancelled => "ssh_cancelled",
                    _ => "ssh_failed",
                });
            }
        }
        cx.notify();
    }
    pub(super) fn render_cli(&self, root: gpui_kit::Div, cx: &mut Context<Self>) -> gpui_kit::Div {
        let button = Button::new("ssh-cli-detect")
            .label(if self.detecting {
                text(cx, "Detecting remote CLIs…", "正在检测远端 CLI…")
            } else {
                text(cx, "Detect remote CLIs", "检测远端 CLI")
            })
            .ghost()
            .on_click(cx.listener(|this, _, _, cx| {
                if this.connected
                    && let Some(host) = &this.selected
                {
                    this.start(SshAction::DetectCli(host.id.clone()), cx);
                }
            }));
        let mut section = div()
            .id("ssh-cli")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(semantics::button(button, self.busy || !self.connected, self.detecting, cx))
            .child(text(
                cx,
                "Check remote PATH for Claude, Codex, Antigravity and OpenCode. Versions are not queried.",
                "检查远端 Claude、Codex、Antigravity 和 OpenCode 的 PATH，不查询版本。",
            ));
        if self.detecting {
            section = section.child(semantics::button(
                Button::new("ssh-cli-cancel")
                    .label(text(cx, "Cancel detection", "取消检测"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(cancel) = &this.cancel {
                            cancel.request_cancel();
                        }
                        cx.notify();
                    })),
                false,
                false,
                cx,
            ));
        }
        if let Some(rows) = &self.cli {
            for row in rows {
                let name = match row.name {
                    "claude" => "Claude",
                    "codex" => "Codex",
                    "gemini" => "Antigravity",
                    "opencode" => "OpenCode",
                    _ => row.name,
                };
                let status = if row.installed {
                    text(cx, "Installed", "已安装")
                } else {
                    text(cx, "Not found in remote PATH", "远端 PATH 中未找到")
                };
                let line = match &row.path {
                    Some(path) => format!("{name}: {status} · {path}"),
                    None => format!("{name}: {status}"),
                };
                section = section.child(
                    div()
                        .id(SharedString::from(format!("ssh-cli-{}", row.name)))
                        .test_support()
                        .aria_label(line.clone())
                        .child(line),
                );
            }
        } else {
            let status = if let Some(error) = self.cli_error {
                match error {
                    "ssh_not_connected" => text(
                        cx,
                        "Connect and verify this host before detecting CLIs.",
                        "请先连接并验证此主机，再检测 CLI。",
                    ),
                    "ssh_session_changed" | "ssh_environment_mismatch" => text(
                        cx,
                        "Connection changed. Reconnect and detect again.",
                        "连接已变化，请重新连接后再次检测。",
                    ),
                    "ssh_cli_output_invalid" => text(
                        cx,
                        "Remote CLI output was not a supported single path. No installed status was accepted.",
                        "远端 CLI 输出不是受支持的单个路径，没有接受安装状态。",
                    ),
                    "ssh_fixture_offline" => text(
                        cx,
                        "Live SSH is disabled for fixture and benchmark sources.",
                        "演示与基准来源禁止真实 SSH。",
                    ),
                    _ => message(error, crate::locales::is_chinese(cx)),
                }
            } else if self.detecting {
                text(cx, "Checking the selected connection…", "正在检查所选连接…")
            } else if self.connected {
                text(
                    cx,
                    "Remote CLIs have not been checked.",
                    "尚未检查远端 CLI。",
                )
            } else {
                text(
                    cx,
                    "Connect and verify this host before detecting CLIs.",
                    "请先连接并验证此主机，再检测 CLI。",
                )
            };
            section = section.child(
                div()
                    .id("ssh-cli-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(status)
                    .child(status),
            );
        }
        root.child(section)
    }
}

#[cfg(test)]
mod tests;
