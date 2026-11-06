//! Metadata-only source inventory. No managed layer is an editable document.
use super::*;
use ccr_app::platform_settings::layers::{Layer, State};

pub(super) struct LayersView {
    services: Services,
    platform: Platform,
    project: Entity<InputState>,
    pub(super) rows: Vec<Layer>,
    task: Option<Task<()>>,
    generation: u64,
    pub(super) busy: bool,
    message: Option<&'static str>,
}
impl LayersView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            services,
            platform: Platform::Claude,
            project: cx.new(|cx| InputState::new(window, cx)),
            rows: Vec::new(),
            task: None,
            generation: 0,
            busy: false,
            message: None,
        }
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.generation += 1;
        self.rows.clear();
        self.busy = false;
        self.message = None;
        cx.notify();
    }
    pub fn open(&mut self, platform: Platform, cx: &mut Context<Self>) {
        self.suspend(cx);
        self.platform = platform;
        self.refresh(cx);
    }
    fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.generation += 1;
        let generation = self.generation;
        let project = self.project.read(cx).value().trim().to_string();
        let project = (!project.is_empty()).then(|| std::path::PathBuf::from(project));
        self.rows.clear();
        self.message = None;
        self.busy = true;
        let key = RequestKey {
            environment: EnvironmentId::Local(
                if self.services.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: self.platform,
            generation,
            request_id: generation,
        };
        match self.services.platform_layers_request(key, project) {
            Ok(mut subscription) => self.task = Some(cx.spawn(async move |weak, cx| {
                while let Some(event) = subscription.next().await {
                    if let TaskEvent::Terminal(result) = event {
                        let _ = weak.update(cx, |this, cx| {
                            if this.generation != generation { return; }
                            this.busy = false;
                            match result.as_ref() {
                                Completion::Success(rows) => this.rows = rows.clone(),
                                Completion::Failed(error) | Completion::CleanupFailed(error) => this.message = Some(error.message()),
                                _ => this.message = Some("Source inventory interrupted. Retry. / 来源读取已中断，请重试"),
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            })),
            Err(_) => { self.busy = false; self.message = Some("Source inventory queue unavailable. Retry. / 来源读取队列不可用，请重试"); }
        }
        cx.notify();
    }
    fn row(&self, index: usize, cx: &gpui_kit::App) -> gpui_kit::AnyElement {
        let row = &self.rows[index];
        let state = match row.state {
            State::Unresolved => text(cx, "Project scope not selected", "尚未选择项目作用域"),
            State::Missing => text(cx, "Not found", "不存在"),
            State::Present => text(cx, "Present", "已存在"),
            State::Unreadable => text(cx, "Cannot read metadata", "无法读取元数据"),
        };
        let access = if row.editable {
            text(
                cx,
                "User source · edit from Settings",
                "用户来源 · 可从设置编辑",
            )
        } else {
            text(cx, "Read-only layer", "只读配置层")
        };
        let path = row
            .path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| text(cx, "No project path captured", "未捕获项目路径").into());
        let label = match row.id.as_str() {
            "managed" => text(cx, "Managed policy", "托管策略").to_string(),
            "project" => text(cx, "Project", "项目").to_string(),
            "local" => text(cx, "Project local", "项目本地").to_string(),
            "user" => text(cx, "User", "用户").to_string(),
            "managed_user" => text(cx, "Managed policy (user)", "托管策略（用户）").to_string(),
            "managed_system" => text(cx, "Managed policy (system)", "托管策略（系统）").to_string(),
            "requirements_user" => text(cx, "Requirements (user)", "约束（用户）").to_string(),
            "requirements_system" => text(cx, "Requirements (system)", "约束（系统）").to_string(),
            _ => row.label.clone(),
        };
        div()
            .id(("platform-layer", index))
            .h(px(88.))
            .flex()
            .flex_col()
            .gap(px(4.))
            .border_b_1()
            .border_color(cx.theme().border)
            .py(px(8.))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(label)
                    .child(format!("{state} · {access}")),
            )
            .child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .truncate()
                    .child(path),
            )
            .child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(match (row.size, row.mtime) {
                        (Some(size), Some(mtime)) => {
                            let modified = i64::try_from(mtime)
                                .ok()
                                .and_then(chrono::DateTime::from_timestamp_millis)
                                .map(|value| {
                                    value
                                        .with_timezone(&chrono::Local)
                                        .format("%Y-%m-%d %H:%M:%S %:z")
                                        .to_string()
                                })
                                .unwrap_or_else(|| "—".into());
                            format!(
                                "{size} bytes · {} {modified}",
                                text(cx, "Modified", "修改时间")
                            )
                        }
                        _ => "—".into(),
                    }),
            )
            .into_any_element()
    }
}
impl Render for LayersView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div().id("platform-layers").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(div().text_size(px(theme::TITLE_SIZE)).child(text(cx,"Configuration sources","配置来源")))
            .child(text(cx,"Source inventory only; it does not infer merged values. Managed policy and requirements stay read-only. A missing project scope is unknown, not an absent file.","此清单不推断合并后的配置值。托管策略与 requirements 保持只读。未选择项目时，项目来源未知，不表示文件不存在。"));
        if self.platform != Platform::Codex {
            root = root.child(text(
                cx,
                "Optional absolute project directory",
                "可选的项目绝对路径",
            ));
            root = root.child(
                Input::new(&self.project)
                    .id("platform-layers-project")
                    .aria_label(text(
                        cx,
                        "Project scope for configuration sources",
                        "配置来源的项目作用域",
                    )),
            );
        }
        root = root.child(semantics::button(
            Button::new("platform-layers-refresh")
                .label(text(cx, "Read source metadata", "读取来源元数据"))
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
            self.busy,
            self.busy,
            cx,
        ));
        if let Some(message) = self.message {
            root = root.child(crate::locales::service_error(cx, message));
        }
        root.child(
            gpui_kit::uniform_list(
                "platform-layers-list",
                self.rows.len(),
                cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                    range.map(|index| this.row(index, cx)).collect()
                }),
            )
            .h(px(360.))
            .w_full(),
        )
    }
}
