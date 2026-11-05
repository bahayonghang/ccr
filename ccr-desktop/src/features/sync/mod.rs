//! Canonical local sync config. Remote access is never claimed from this page.
use crate::{
    locales::{is_chinese, service_error, text},
    runtime::{
        Services,
        sync::{SyncError, SyncFolderRow, SyncLocal, SyncSnapshot, SyncUrl},
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::{
    App, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Task, Window, div, px,
};

pub struct SyncView {
    services: Services,
    snapshot: Option<SyncSnapshot>,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    busy: bool,
    message: Option<&'static str>,
    focus: FocusHandle,
}

impl SyncView {
    pub fn new(services: Services, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            services,
            snapshot: None,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            busy: false,
            message: None,
            focus: cx.focus_handle().tab_stop(false),
        }
    }

    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }

    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.busy = false;
        cx.notify();
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.generation += 1;
        let key = RequestKey {
            environment: EnvironmentId::Local(
                if self.services.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.busy = true;
        self.message = None;
        self.snapshot = None;
        match self.services.sync_request(key.clone()) {
            Ok(mut subscription) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(snapshot) => {
                                        this.snapshot = Some(snapshot.clone());
                                    }
                                    Completion::Failed(error) | Completion::CleanupFailed(error) => {
                                        this.message = Some(error.message());
                                    }
                                    Completion::Cancelled => {
                                        this.message = Some(
                                            "The sync config read did not finish. / 同步配置读取没有完成",
                                        );
                                    }
                                    Completion::WorkerFailed => {
                                        this.message = Some(
                                            "The sync config reader stopped without a result. / 同步配置读取没有返回结果",
                                        );
                                    }
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => {
                self.busy = false;
                self.message = Some(SyncError::Storage.message());
            }
        }
        cx.notify();
    }
}

fn url_line(cx: &App, url: &SyncUrl) -> String {
    match url {
        SyncUrl::Missing => text(cx, "No WebDAV URL is stored.", "未保存 WebDAV URL。").to_string(),
        SyncUrl::Stored => text(cx, "A WebDAV URL is stored.", "已保存 WebDAV URL。").to_string(),
        SyncUrl::Host(host) => {
            if is_chinese(cx) {
                format!("WebDAV 主机：{host}")
            } else {
                format!("WebDAV host: {host}")
            }
        }
    }
}

fn presence_line(
    cx: &App,
    present: bool,
    en_yes: &'static str,
    en_no: &'static str,
    zh_yes: &'static str,
    zh_no: &'static str,
) -> String {
    text(
        cx,
        if present { en_yes } else { en_no },
        if present { zh_yes } else { zh_no },
    )
    .to_string()
}

fn local_lines(cx: &App, local: &SyncLocal) -> Vec<String> {
    let mut lines = vec![
        if local.webdav_enabled {
            text(
                cx,
                "WebDAV is enabled in the local file.",
                "本地文件里启用了 WebDAV。",
            )
            .to_string()
        } else {
            text(
                cx,
                "WebDAV is turned off in the local file.",
                "本地文件里关闭了 WebDAV。",
            )
            .to_string()
        },
        url_line(cx, &local.url),
        presence_line(
            cx,
            local.username_present,
            "A WebDAV username is stored.",
            "No WebDAV username is stored.",
            "已保存 WebDAV 用户名。",
            "未保存 WebDAV 用户名。",
        ),
        presence_line(
            cx,
            local.password_present,
            "A WebDAV password is stored in the local file.",
            "No WebDAV password is stored.",
            "本地文件里保存了 WebDAV 密码。",
            "未保存 WebDAV 密码。",
        ),
        text(
            cx,
            "Remote access was not tested. This page does not sync or save a password.",
            "未测试远程访问。此页不会执行同步，也不会保存密码。",
        )
        .to_string(),
    ];
    if local.folders.is_empty() {
        lines.push(
            text(
                cx,
                "The canonical file has no folders.",
                "规范配置文件里没有文件夹。",
            )
            .to_string(),
        );
    }
    lines
}

fn folder_label(cx: &App, folder: &SyncFolderRow) -> String {
    let state = if folder.enabled {
        text(cx, "enabled", "已启用")
    } else {
        text(cx, "disabled", "已停用")
    };
    if is_chinese(cx) {
        format!(
            "{} · {state} · 本地 {} · 远程 {}",
            folder.name, folder.local_path, folder.remote_path
        )
    } else {
        format!(
            "{} · {state} · local {} · remote {}",
            folder.name, folder.local_path, folder.remote_path
        )
    }
}

impl Render for SyncView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div()
            .id("sync-page")
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll()
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::SECTION_GAP))
            .child(
                div()
                    .id("sync-heading")
                    .test_support()
                    .role(gpui_kit::Role::Heading)
                    .aria_level(1)
                    .text_size(px(theme::TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Sync", "同步")),
            )
            .child({
                let intro = text(
                    cx,
                    "Reads the canonical local sync file. It does not read a legacy file, write a password, contact WebDAV, or report that a sync succeeded.",
                    "读取本地规范同步文件。不会读取旧文件、写入密码、联系 WebDAV，也不会报告同步已经成功。",
                );
                div()
                    .id("sync-intro")
                    .test_support()
                    .aria_label(intro)
                    .child(intro)
            })
            .child(semantics::button(
                Button::new("sync-read")
                    .label(if self.busy {
                        text(cx, "Reading", "读取中")
                    } else {
                        text(cx, "Read local config", "读取本地配置")
                    })
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                self.busy,
                self.busy,
                cx,
            ));
        if self.busy {
            root = root.child(
                div()
                    .id("sync-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .child(text(
                        cx,
                        "Reading the local sync config.",
                        "正在读取本地同步配置。",
                    )),
            );
        }
        if let Some(message) = self.message {
            let line = service_error(cx, message);
            root = root.child(
                div()
                    .id("sync-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(line)
                    .child(line),
            );
        }
        if let Some(snapshot) = self.snapshot.clone() {
            match snapshot {
                SyncSnapshot::Absent => {
                    let empty = text(
                        cx,
                        "No canonical sync config file was found. Legacy sync config is not read. Remote access was not tested.",
                        "没有找到规范同步配置文件。不会读取旧的同步配置。未测试远程访问。",
                    );
                    root = root.child(
                        div()
                            .id("sync-absent")
                            .test_support()
                            .aria_label(empty)
                            .child(empty),
                    );
                }
                SyncSnapshot::Present(local) => {
                    let lines = local_lines(cx, &local);
                    let summary = lines.join(" ");
                    let mut detail = div()
                        .id("sync-local")
                        .test_support()
                        .aria_label(SharedString::from(summary))
                        .flex()
                        .flex_col()
                        .gap(px(theme::CONTROL_GAP));
                    for line in lines {
                        detail = detail.child(line);
                    }
                    root = root.child(detail);
                    for (index, folder) in local.folders.iter().enumerate() {
                        let label = folder_label(cx, folder);
                        root = root.child(
                            div()
                                .id(SharedString::from(format!("sync-folder-{index}")))
                                .test_support()
                                .h(px(theme::ROW_HEIGHT))
                                .flex()
                                .items_center()
                                .aria_label(SharedString::from(label.clone()))
                                .child(label),
                        );
                    }
                }
            }
        }
        root
    }
}

#[cfg(test)]
mod tests;
