//! Local execution environment, WSL probe, and SSH host names.
use crate::{
    locales::{is_chinese, text},
    runtime::{self, Services},
    shell::semantics,
    theme,
};
use ccr_app::environments::{
    CapabilityState, EnvironmentReport, RunningEnvironment, ShellCapabilities, SshNames,
    WslExecProbe, WslProbe,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, SystemNotification, Task, Window, div, px,
};

const NOTIFICATION_TAG: &str = "ccr-gpui-environment";

pub struct EnvironmentView {
    services: Services,
    report: Option<EnvironmentReport>,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    loading: bool,
    failed: bool,
    notification_requested: bool,
}

impl EnvironmentView {
    pub fn new(services: Services, cx: &mut Context<Self>) -> Self {
        cx.set_app_identity("io.github.ccr.gpui", "CCR Desktop (GPUI)");
        Self {
            services,
            report: None,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            loading: false,
            failed: false,
            notification_requested: false,
        }
    }

    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.loading = false;
        cx.notify();
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.failed = false;
        if self.services.is_fixture() {
            self.loading = false;
            self.report = Some(ccr_app::environments::inspect(
                ccr_app::environments::InspectOptions::fixture(),
                &|| false,
            ));
            cx.notify();
            return;
        }
        let key = RequestKey {
            environment: EnvironmentId::Local("local".into()),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.loading = true;
        match runtime::environments::load(&self.services, key.clone()) {
            Ok(mut subscription) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.loading = false;
                                match result.as_ref() {
                                    Completion::Success(report) => {
                                        this.report = Some(report.clone());
                                        this.failed = false;
                                    }
                                    Completion::Cancelled => {
                                        this.failed = false;
                                    }
                                    Completion::Failed(())
                                    | Completion::CleanupFailed(())
                                    | Completion::WorkerFailed => {
                                        this.failed = true;
                                    }
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(error) => {
                self.loading = false;
                self.failed = true;
                let _ = error;
            }
        }
        cx.notify();
    }

    fn post_notification(&mut self, cx: &mut Context<Self>) {
        let title = text(cx, "CCR Desktop", "CCR 桌面");
        let body = text(
            cx,
            "CCR Desktop requested a system notification.",
            "CCR 桌面提交了一条系统通知。",
        );
        cx.show_system_notification(SystemNotification {
            tag: NOTIFICATION_TAG.into(),
            title: title.into(),
            body: body.into(),
            actions: Vec::new(),
        });
        self.notification_requested = true;
        cx.notify();
    }
}

pub(crate) fn local_line(chinese: bool, os: &str) -> String {
    if chinese {
        format!("此桌面正在 {os} 上的本地环境中运行。")
    } else {
        format!("This desktop is running in the local environment on {os}.")
    }
}

pub(crate) fn wsl_line(chinese: bool, probe: &WslProbe) -> String {
    match probe {
        WslProbe::Unsupported => {
            if chinese {
                "此操作系统没有 WSL。".to_string()
            } else {
                "WSL is not available on this operating system.".to_string()
            }
        }
        WslProbe::NotQueried => {
            if chinese {
                "演示数据未查询 WSL。".to_string()
            } else {
                "WSL was not queried for demo data.".to_string()
            }
        }
        WslProbe::Unavailable => {
            if chinese {
                "无法查询 WSL。".to_string()
            } else {
                "WSL could not be queried.".to_string()
            }
        }
        WslProbe::None => {
            if chinese {
                "WSL 没有报告发行版。".to_string()
            } else {
                "WSL reported no distributions.".to_string()
            }
        }
        WslProbe::Cancelled => {
            if chinese {
                "WSL 检查已取消。".to_string()
            } else {
                "The WSL check was cancelled.".to_string()
            }
        }
        WslProbe::Listed { names, exec } => {
            let names = names.join(", ");
            match exec {
                WslExecProbe::Completed => {
                    if chinese {
                        format!("WSL 列出了 {names}。默认发行版中的命令已完成。")
                    } else {
                        format!(
                            "WSL listed {names}. A command in the default distribution completed."
                        )
                    }
                }
                WslExecProbe::PathNotFound => {
                    if chinese {
                        format!(
                            "WSL 列出了 {names}。默认发行版中的命令因路径不存在而失败。没有可用发行版接入。"
                        )
                    } else {
                        format!(
                            "WSL listed {names}. A command in the default distribution failed because a path was not found. No usable distribution is connected."
                        )
                    }
                }
                WslExecProbe::Failed => {
                    if chinese {
                        format!("WSL 列出了 {names}。默认发行版中的命令失败。没有可用发行版接入。")
                    } else {
                        format!(
                            "WSL listed {names}. A command in the default distribution failed. No usable distribution is connected."
                        )
                    }
                }
                WslExecProbe::TimedOut => {
                    if chinese {
                        format!("WSL 列出了 {names}。默认发行版中的命令超时。没有可用发行版接入。")
                    } else {
                        format!(
                            "WSL listed {names}. A command in the default distribution timed out. No usable distribution is connected."
                        )
                    }
                }
                WslExecProbe::Cancelled => {
                    if chinese {
                        "WSL 检查已取消。".to_string()
                    } else {
                        "The WSL check was cancelled.".to_string()
                    }
                }
            }
        }
    }
}

pub(crate) fn ssh_line(chinese: bool, names: &SshNames) -> String {
    match names {
        SshNames::NotConnected => {
            if chinese {
                "此桌面壳未接入 SSH 管理。".to_string()
            } else {
                "SSH management is not connected in this desktop shell.".to_string()
            }
        }
        SshNames::Unreadable => {
            if chinese {
                "无法读取已保存的 SSH 主机名。".to_string()
            } else {
                "Saved SSH host names could not be read.".to_string()
            }
        }
        SshNames::Empty => {
            if chinese {
                "没有已保存的 SSH 主机名。".to_string()
            } else {
                "No saved SSH host names.".to_string()
            }
        }
        SshNames::Names(names) => {
            let names = names.join(", ");
            if chinese {
                format!("已保存的 SSH 主机名：{names}。")
            } else {
                format!("Saved SSH host names: {names}.")
            }
        }
    }
}

pub(crate) fn shell_lines(chinese: bool, shell: ShellCapabilities) -> [String; 3] {
    let tray = match shell.tray {
        CapabilityState::Unsupported => {
            if chinese {
                "托盘不受支持。此窗口栈没有托盘。"
            } else {
                "The tray is unsupported. This window stack has no tray."
            }
        }
        CapabilityState::NotRun => {
            if chinese {
                "托盘尚未在此操作系统上运行。"
            } else {
                "The tray was not run on this operating system."
            }
        }
        CapabilityState::NotConnected => {
            if chinese {
                "此壳未接入托盘。"
            } else {
                "The tray is not connected in this shell."
            }
        }
        CapabilityState::Available => {
            if chinese {
                "托盘可用。"
            } else {
                "The tray is available."
            }
        }
    };
    let hide = match shell.hide_restore {
        CapabilityState::Unsupported => {
            if chinese {
                "在 Windows 上不支持隐藏和恢复。"
            } else {
                "Hide and restore are unsupported on Windows."
            }
        }
        CapabilityState::NotRun => {
            if chinese {
                "此操作系统上尚未运行隐藏和恢复。"
            } else {
                "Hide and restore were not run on this operating system."
            }
        }
        CapabilityState::NotConnected => {
            if chinese {
                "此壳未接入隐藏和恢复。"
            } else {
                "Hide and restore are not connected in this shell."
            }
        }
        CapabilityState::Available => {
            if chinese {
                "隐藏和恢复可用。"
            } else {
                "Hide and restore are available."
            }
        }
    };
    let notifications = match shell.notifications {
        CapabilityState::Available => {
            if chinese {
                "可以提交系统通知。系统是否显示尚未确认。"
            } else {
                "A system notification can be posted. Delivery is not confirmed."
            }
        }
        CapabilityState::Unsupported => {
            if chinese {
                "系统通知不受支持。"
            } else {
                "System notifications are unsupported."
            }
        }
        CapabilityState::NotRun => {
            if chinese {
                "系统通知尚未在此操作系统上运行。"
            } else {
                "System notifications were not run on this operating system."
            }
        }
        CapabilityState::NotConnected => {
            if chinese {
                "此壳未接入系统通知。"
            } else {
                "System notifications are not connected in this shell."
            }
        }
    };
    [
        tray.to_string(),
        hide.to_string(),
        notifications.to_string(),
    ]
}

fn line(id: &'static str, value: String) -> impl IntoElement {
    div()
        .id(id)
        .test_support()
        .aria_label(SharedString::from(value.clone()))
        .child(value)
}

impl Render for EnvironmentView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chinese = is_chinese(cx);
        let os = self
            .report
            .as_ref()
            .map(|report| report.os)
            .unwrap_or_else(ccr_app::environments::host_os_name);
        let running = self
            .report
            .as_ref()
            .map(|report| report.running)
            .unwrap_or(RunningEnvironment::Local);
        let local = match running {
            RunningEnvironment::Local => local_line(chinese, os),
        };
        let mut column = div()
            .id("env-section")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .pt(px(theme::SECTION_GAP))
            .child(
                div()
                    .id("env-heading")
                    .test_support()
                    .text_size(px(theme::TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Execution environment", "执行环境")),
            )
            .child(line("env-local", local));
        if self.loading {
            column = column.child(line(
                "env-wsl",
                text(cx, "Checking WSL.", "正在检查 WSL。").to_string(),
            ));
            column = column.child(line(
                "env-ssh",
                text(
                    cx,
                    "Checking saved SSH host names.",
                    "正在检查已保存的 SSH 主机名。",
                )
                .to_string(),
            ));
        } else if self.failed {
            column = column.child(line(
                "env-wsl",
                text(cx, "The environment check failed.", "环境检查失败。").to_string(),
            ));
        } else if let Some(report) = &self.report {
            column = column.child(line("env-wsl", wsl_line(chinese, &report.wsl)));
            column = column.child(line("env-ssh", ssh_line(chinese, &report.ssh)));
            let shell = shell_lines(chinese, report.shell);
            column = column.child(line("env-tray-status", shell[0].clone()));
            column = column.child(line("env-hide-status", shell[1].clone()));
            column = column.child(line("env-notify-status", shell[2].clone()));
        }
        if self.notification_requested {
            column = column.child(line(
                "env-notify-posted",
                text(
                    cx,
                    "Notification request posted. Delivery is not confirmed.",
                    "已提交系统通知请求。未能确认系统是否显示。",
                )
                .to_string(),
            ));
        }
        column.child(semantics::button(
            Button::new("env-notify")
                .label(text(cx, "Post a system notification", "发送系统通知"))
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| this.post_notification(cx))),
            false,
            false,
            cx,
        ))
    }
}

#[cfg(test)]
mod tests;
