//! Native draft and action surface, backed by the bounded domain adapter.
mod view;
use crate::{
    locales::text,
    runtime::{
        Services,
        profiles::{ProfileCommand, ProfileError, ProfileResult, merge_patch_json, string_patch},
    },
    shell::semantics,
};
use ccr_app::{
    config::{ConfigRow, source::SourceSaveOutcome},
    runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent},
};
use gpui_kit::component::input::{InputState, TextareaState};
use gpui_kit::{AppContext, Context, Entity, FocusHandle, Focusable, Task, Window};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Closed,
    Edit,
    Duplicate,
    Source,
    Import,
    Restore,
    Output,
}
#[derive(Clone, Copy)]
enum Action {
    Create,
    Edit,
    Duplicate,
    Delete,
    Apply,
    Off,
    Validate,
    Source,
    Inspect,
    ForceSource,
    Export,
    ExportSecrets,
    Import,
    ImportReplace,
    Backups,
    Restore,
    RestoreExecute,
    Clean,
    History,
    ClearHistory,
    Advanced,
    Environment,
    ShellEnvironment,
    GrokForceDelete,
}
struct Confirmation {
    generation: u64,
    entry_pending: bool,
    action: Action,
    return_focus: Option<FocusHandle>,
    subject: String,
}
pub struct ProfileTools {
    services: Services,
    platform: Platform,
    target: Option<ConfigRow>,
    original: Option<ConfigRow>,
    mode: Mode,
    name: Entity<InputState>,
    description: Entity<InputState>,
    model: Entity<InputState>,
    credential: Entity<InputState>,
    content: Entity<TextareaState>,
    revision: String,
    advanced: bool,
    activation_conflict: bool,
    can_off: Option<bool>,
    generic: bool,
    clear_sensitive: bool,
    base_url: Entity<InputState>,
    auth_mode: Entity<InputState>,
    request: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    busy: bool,
    confirmation: Option<Confirmation>,
    confirmation_generation: u64,
    focus: FocusHandle,
    confirmation_focus: FocusHandle,
    return_focus: Option<FocusHandle>,
    pub refresh_version: u64,
    pub message: Option<String>,
    failed: bool,
    draft_conflict: bool,
    pending_content: Option<String>,
    source_location: Option<String>,
    grok_recovery: Option<(String, String, bool)>,
    grok_force_delete: Option<String>,
}
impl ProfileTools {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            services,
            platform: Platform::Claude,
            target: None,
            original: None,
            mode: Mode::Closed,
            name: cx.new(|cx| InputState::new(window, cx)),
            description: cx.new(|cx| InputState::new(window, cx)),
            model: cx.new(|cx| InputState::new(window, cx)),
            credential: cx.new(|cx| InputState::new(window, cx).masked(true)),
            content: cx.new(|cx| TextareaState::new(window, cx)),
            revision: String::new(),
            advanced: false,
            activation_conflict: false,
            can_off: None,
            generic: false,
            clear_sensitive: false,
            base_url: cx.new(|cx| InputState::new(window, cx)),
            auth_mode: cx.new(|cx| InputState::new(window, cx)),
            request: 0,
            gate: ResponseGate::default(),
            task: None,
            busy: false,
            confirmation: None,
            confirmation_generation: 0,
            focus: cx.focus_handle().tab_stop(false),
            confirmation_focus: cx.focus_handle().tab_stop(false),
            return_focus: None,
            refresh_version: 0,
            message: None,
            failed: false,
            draft_conflict: false,
            pending_content: None,
            source_location: None,
            grok_recovery: None,
            grok_force_delete: None,
        }
    }
    pub fn busy(&self) -> bool {
        self.busy
    }
    pub fn expanded(&self) -> bool {
        self.mode != Mode::Closed || self.confirmation.is_some()
    }
    /// Dropping this subscription detaches an application-owned write; it never
    /// cancels a committed operation. A late response cannot reopen plaintext.
    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.request += 1;
        self.busy = false;
        self.message = None;
        self.grok_recovery = None;
        self.grok_force_delete = None;
        self.return_focus = None;
        self.close(window, cx);
    }
    pub fn set_target(&mut self, target: Option<ConfigRow>, cx: &mut Context<Self>) {
        if !self.expanded() {
            if self.target.as_ref().map(|r| &r.name) != target.as_ref().map(|r| &r.name) {
                self.grok_force_delete = None;
            }
            self.target = target;
            self.submit(ProfileCommand::Status, cx);
            cx.notify();
        }
    }
    pub fn set_platform(
        &mut self,
        platform: Platform,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.expanded() || self.busy {
            return false;
        }
        if self.platform != platform {
            self.message = None;
            self.failed = false;
            self.draft_conflict = false;
            self.grok_recovery = None;
            self.grok_force_delete = None;
        }
        self.platform = platform;
        self.generic = false;
        self.can_off = None;
        self.target = None;
        self.close(window, cx);
        true
    }
    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if self.draft_conflict {
            self.message = None;
            self.failed = false;
            self.draft_conflict = false;
        }
        self.mode = Mode::Closed;
        self.confirmation = None;
        self.original = None;
        self.revision.clear();
        self.advanced = false;
        self.activation_conflict = false;
        self.pending_content = None;
        self.source_location = None;
        for input in [
            &self.name,
            &self.description,
            &self.model,
            &self.credential,
            &self.base_url,
            &self.auth_mode,
        ] {
            input.update(cx, |state, cx| state.set_value("", window, cx));
        }
        self.content
            .update(cx, |state, cx| state.set_value("", window, cx));
        if let Some(focus) = self.return_focus.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }
    fn act(&mut self, action: Action, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if self.mode == Mode::Closed {
            self.return_focus = window.focused(cx);
        }
        match action {
            Action::Create | Action::Edit | Action::Duplicate => {
                let row = self.target.clone();
                if !matches!(action, Action::Create) && row.is_none() {
                    return;
                }
                self.original = if matches!(action, Action::Create) {
                    None
                } else {
                    row.clone()
                };
                self.revision = row.as_ref().map(|r| r.version.clone()).unwrap_or_default();
                self.mode = if matches!(action, Action::Duplicate) {
                    Mode::Duplicate
                } else {
                    Mode::Edit
                };
                let name = if matches!(action, Action::Create | Action::Duplicate) {
                    ""
                } else {
                    row.as_ref().map(|r| r.name.as_str()).unwrap_or_default()
                };
                self.name
                    .update(cx, |state, cx| state.set_value(name, window, cx));
                self.description.update(cx, |state, cx| {
                    state.set_value(
                        if matches!(action, Action::Create) {
                            ""
                        } else {
                            row.as_ref()
                                .map(|r| r.description.as_str())
                                .unwrap_or_default()
                        },
                        window,
                        cx,
                    )
                });
                self.model.update(cx, |state, cx| {
                    state.set_value(
                        if matches!(action, Action::Create) {
                            ""
                        } else {
                            row.as_ref()
                                .and_then(|r| r.model.as_deref())
                                .unwrap_or_default()
                        },
                        window,
                        cx,
                    )
                });
                window.focus(&self.name.read(cx).focus_handle(cx), cx);
            }
            Action::Delete
            | Action::Off
            | Action::Source
            | Action::Inspect
            | Action::ForceSource
            | Action::ExportSecrets
            | Action::Clean
            | Action::ClearHistory
            | Action::Import
            | Action::ImportReplace
            | Action::Restore
            | Action::RestoreExecute
            | Action::Advanced
            | Action::Environment
            | Action::ShellEnvironment
            | Action::GrokForceDelete => {
                self.confirmation_generation += 1;
                let generation = self.confirmation_generation;
                self.confirmation = Some(Confirmation {
                    generation,
                    entry_pending: true,
                    action,
                    return_focus: window.focused(cx),
                    subject: if matches!(action, Action::RestoreExecute) {
                        self.name.read(cx).value().to_string()
                    } else if matches!(action, Action::GrokForceDelete) {
                        self.grok_force_delete.clone().unwrap_or_default()
                    } else {
                        self.target
                            .as_ref()
                            .map(|r| r.name.clone())
                            .unwrap_or_else(|| "catalog".into())
                    },
                });
            }
            Action::Apply => {
                if let Some(row) = &self.target {
                    self.submit(
                        ProfileCommand::Apply {
                            name: row.name.clone(),
                            enable: !row.enabled,
                        },
                        cx,
                    );
                }
            }
            Action::Validate => self.submit(
                if self.generic {
                    ProfileCommand::GenericValidate
                } else {
                    ProfileCommand::Validate
                },
                cx,
            ),
            Action::Export => self.submit(ProfileCommand::Export { secrets: false }, cx),
            Action::Backups => self.submit(ProfileCommand::Backups, cx),
            Action::History => self.submit(ProfileCommand::History, cx),
        }
        cx.notify();
    }
    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(confirm) = self.confirmation.take() else {
            return;
        };
        match confirm.action {
            Action::Delete => {
                if self.target.is_some() {
                    self.submit(
                        if self.generic {
                            ProfileCommand::GenericDelete {
                                name: confirm.subject.clone(),
                            }
                        } else {
                            ProfileCommand::Delete {
                                name: confirm.subject.clone(),
                            }
                        },
                        cx,
                    )
                }
            }
            Action::GrokForceDelete => {
                if let Some(name) = self.grok_force_delete.take() {
                    self.submit(ProfileCommand::GrokDelete { name, force: true }, cx)
                }
            }
            Action::Off => self.submit(ProfileCommand::Off, cx),
            Action::Environment | Action::ShellEnvironment => {
                if let Some(row) = &self.target {
                    self.submit(
                        ProfileCommand::ExportEnvironment {
                            name: row.name.clone(),
                            shell: matches!(confirm.action, Action::ShellEnvironment),
                        },
                        cx,
                    )
                }
            }
            Action::Source => {
                self.mode = Mode::Source;
                self.submit(ProfileCommand::Source, cx)
            }
            Action::Inspect => self.submit(
                ProfileCommand::Inspect {
                    name: confirm.subject.clone(),
                },
                cx,
            ),
            Action::ForceSource => self.submit(
                ProfileCommand::SaveSource {
                    content: self.content.read(cx).value().to_string(),
                    revision: self.revision.clone(),
                    force: true,
                },
                cx,
            ),
            Action::Advanced => {
                self.advanced = true;
                self.content
                    .update(cx, |state, cx| state.set_value("{}", window, cx));
            }
            Action::ImportReplace => self.submit(
                ProfileCommand::Import {
                    content: self.content.read(cx).value().to_string(),
                    replace: true,
                },
                cx,
            ),
            Action::RestoreExecute => self.submit(
                ProfileCommand::Restore {
                    leaf: confirm.subject.clone(),
                },
                cx,
            ),
            Action::ExportSecrets => self.submit(ProfileCommand::Export { secrets: true }, cx),
            Action::Clean => self.submit(ProfileCommand::CleanBackups, cx),
            Action::ClearHistory => self.submit(ProfileCommand::ClearHistory, cx),
            Action::Import => {
                self.mode = Mode::Import;
                window.focus(&self.content.read(cx).focus_handle(cx), cx);
            }
            Action::Restore => {
                self.mode = Mode::Restore;
                window.focus(&self.name.read(cx).focus_handle(cx), cx);
            }
            _ => {}
        }
        cx.notify();
    }
    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.confirmation.is_some() {
            return;
        }
        let name = self.name.read(cx).value().to_string();
        let command = match self.mode {
            Mode::Edit => {
                let description = self.description.read(cx).value().to_string();
                let model = self.model.read(cx).value().to_string();
                let mut fields = Vec::new();
                if self
                    .original
                    .as_ref()
                    .is_none_or(|r| r.description != description)
                {
                    fields.push(("description".into(), description));
                }
                if self
                    .original
                    .as_ref()
                    .is_none_or(|r| r.model.as_deref().unwrap_or_default() != model)
                {
                    fields.push(("model".into(), model));
                }
                // Masked state's value intentionally contains bullets. Read the scoped
                // input buffer only for an explicit credential replacement.
                let credential = self.credential.read(cx).text().to_string();
                if !credential.is_empty() {
                    if self.platform == Platform::Grok {
                        fields.push(("api_key".into(), credential));
                        fields.push(("credential_action".into(), "replace_api_key".into()));
                    } else {
                        fields.push(("auth_token".into(), credential));
                    }
                }
                let url = self.base_url.read(cx).value().to_string();
                if !url.is_empty() {
                    fields.push(("base_url".into(), url));
                }
                let auth = self.auth_mode.read(cx).value().to_string();
                if !auth.is_empty() {
                    fields.push((
                        if self.platform == Platform::Grok {
                            "profile_kind"
                        } else {
                            "auth_mode"
                        }
                        .into(),
                        auth,
                    ));
                }
                let mut patch = string_patch(fields);
                if self.advanced
                    && merge_patch_json(&mut patch, &self.content.read(cx).value()).is_err()
                {
                    self.failed = true;
                    self.message = Some(ProfileError::Invalid.message().into());
                    cx.notify();
                    return;
                }
                if self.platform == Platform::Grok
                    && self.original.is_some()
                    && (patch.fields.contains_key("profile_kind")
                        || patch.fields.contains_key("provider_type"))
                {
                    self.failed = true;
                    self.message = Some(
                        "Existing Grok profile kind is read-only. / 现有 Grok 配置类型不可修改"
                            .into(),
                    );
                    cx.notify();
                    return;
                }
                if let Some(original) = &self.original {
                    if self.generic {
                        ProfileCommand::GenericSave {
                            name: original.name.clone(),
                            target: name,
                            revision: self.revision.clone(),
                            patch,
                        }
                    } else {
                        ProfileCommand::Save {
                            name: original.name.clone(),
                            target: name,
                            revision: self.revision.clone(),
                            patch,
                        }
                    }
                } else {
                    if self.generic {
                        ProfileCommand::GenericCreate { name, patch }
                    } else {
                        ProfileCommand::Create { name, patch }
                    }
                }
            }
            Mode::Duplicate => {
                let Some(original) = &self.original else {
                    return;
                };
                ProfileCommand::Duplicate {
                    name: original.name.clone(),
                    target: name,
                }
            }
            Mode::Source => ProfileCommand::SaveSource {
                content: self.content.read(cx).value().to_string(),
                revision: self.revision.clone(),
                force: false,
            },
            Mode::Import => ProfileCommand::Import {
                content: self.content.read(cx).value().to_string(),
                replace: false,
            },
            Mode::Restore => {
                self.act(Action::RestoreExecute, window, cx);
                return;
            }
            _ => return,
        };
        self.submit(command, cx);
    }
    fn submit(&mut self, command: ProfileCommand, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let is_status = matches!(command, ProfileCommand::Status);
        self.request += 1;
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
            generation: self.request,
            request_id: self.request,
        };
        self.gate.replace(key.clone());
        self.busy = true;
        // 状态刷新不能抹掉已提交操作的持久警告或恢复结果。
        if !is_status {
            self.message = None;
            self.failed = false;
            self.draft_conflict = false;
        }
        match self.services.profile_command(key.clone(), command) {
            Ok(mut subscription) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) { return; }
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(result) => this.accept(result, cx),
                                    Completion::Failed(error) | Completion::CleanupFailed(error) => {
                                        this.failed = true;
                                        if is_status { this.can_off = None; }
                                        if !is_status || this.message.is_none() {
                                            this.draft_conflict = !is_status
                                                && *error == ProfileError::Conflict
                                                && this.mode != Mode::Closed;
                                            this.message = Some(error.message().into());
                                        }
                                    }
                                    Completion::Cancelled => {
                                        if !is_status { this.message = Some("Cancelled before starting / 操作开始前已取消".into()); }
                                    }
                                    Completion::WorkerFailed => {
                                        this.failed = true;
                                        if !is_status || this.message.is_none() { this.message = Some("Worker stopped. Inspect state before retrying. / 工作线程已停止，请先检查实际状态".into()); }
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
                self.failed = true;
                if !is_status || self.message.is_none() {
                    self.message = Some("Queue busy or application stopping. Retry. / 队列繁忙或应用正在退出，请重试".into());
                }
            }
        }
        cx.notify();
    }
    fn accept(&mut self, result: &ProfileResult, cx: &mut Context<Self>) {
        if !matches!(result, ProfileResult::Status { .. }) {
            self.draft_conflict = false;
        }
        match result {
            ProfileResult::GrokDelete {
                name,
                reason,
                attempted_force,
            } => {
                use ccr_cli::platforms::grok::GrokProfileDeleteBlockedReason;
                self.failed = reason.is_some();
                self.grok_force_delete = match reason {
                    Some(
                        GrokProfileDeleteBlockedReason::Active
                        | GrokProfileDeleteBlockedReason::Drifted,
                    ) if !attempted_force => Some(name.clone()),
                    _ => None,
                };
                self.message=Some(match reason {None=>"Profile deleted / 配置已删除",Some(GrokProfileDeleteBlockedReason::UnsafeMissingEntryState)=>"Entry state is missing. Back up and inspect Grok runtime before recovery. Force deletion is unavailable. / 入口状态缺失，请先备份并检查 Grok 运行配置，不能强制删除",Some(_)=>"Active or drifted profile was retained. Review Off and delete confirmation. / 激活或漂移配置已保留，请审阅停用并删除确认"}.into());
                if reason.is_none() {
                    self.refresh_version += 1;
                }
            }
            ProfileResult::GrokUpdate {
                status,
                old_name,
                new_name,
            } => {
                self.message = Some(format!("{status}: {old_name} → {new_name}"));
                self.failed = matches!(*status, "rename_apply_failed" | "rename_cleanup_failed");
                self.grok_recovery = if self.failed {
                    Some((
                        old_name.clone(),
                        new_name.clone(),
                        *status == "rename_apply_failed",
                    ))
                } else {
                    None
                };
                self.mode = Mode::Closed;
                self.clear_sensitive = true;
                self.refresh_version += 1;
            }
            ProfileResult::Outcome(outcome) => {
                use ccr_cli::application::profile_lifecycle::ProfileStatus;
                let warnings = format!("{:?}", outcome.warnings);
                let status = outcome.message();
                // Status copy has no Chinese half, so the whole sentence stays English.
                let recovery = if outcome.recovery_paths.is_empty() {
                    String::new()
                } else {
                    " · Inspect recovery records before retrying".to_string()
                };
                self.message = Some(format!("{status} · warnings: {warnings}{recovery}"));
                if matches!(
                    outcome.status,
                    ProfileStatus::Applied | ProfileStatus::AppliedWithWarning
                ) {
                    self.grok_recovery = None;
                    self.mode = Mode::Closed;
                    self.original = None;
                    self.clear_sensitive = true;
                } else {
                    self.failed = !outcome.activation_committed;
                }
                self.refresh_version += 1;
            }
            ProfileResult::Status { can_off } => self.can_off = Some(*can_off),
            ProfileResult::Done(message) => {
                self.message = Some(message.clone());
                self.mode = Mode::Closed;
                self.original = None;
                self.clear_sensitive = true;
                self.refresh_version += 1;
            }
            ProfileResult::Text(content) => {
                self.mode = Mode::Output;
                let value = content.clone();
                self.message = Some("Export ready in scoped output / 导出已就绪".into());
                self.set_content(value, cx);
            }
            ProfileResult::Lines(lines) => {
                self.mode = Mode::Output;
                self.set_content(lines.join("\n"), cx);
            }
            ProfileResult::Source {
                content,
                revision,
                path,
                exists,
            } => {
                self.source_location = Some(format!(
                    "{} · {}",
                    path.display(),
                    if *exists {
                        text(cx, "catalog exists", "目录存在")
                    } else {
                        text(cx, "new catalog", "新目录")
                    }
                ));
                self.revision = revision.clone();
                self.activation_conflict = false;
                self.mode = Mode::Source;
                self.set_content(content.clone(), cx);
            }
            ProfileResult::SourceSave(SourceSaveOutcome::Saved { .. }) => {
                self.mode = Mode::Closed;
                self.clear_sensitive = true;
                self.refresh_version += 1;
                self.message = Some("Catalog saved / 配置目录已保存".into());
            }
            ProfileResult::SourceSave(SourceSaveOutcome::Conflict) => {
                self.activation_conflict = false;
                self.failed = true;
                self.draft_conflict = true;
                self.message = Some(ProfileError::Conflict.message().into());
            }
            ProfileResult::SourceSave(SourceSaveOutcome::ActivationConflict { .. }) => {
                self.activation_conflict = true;
                self.failed = true;
                self.draft_conflict = true;
                self.message=Some("Active profile would be removed. Cancel or restore it in the source before saving. / 源码将移除当前配置，请取消或保留当前配置后保存".into());
            }
            ProfileResult::SourceSave(SourceSaveOutcome::Invalid { kind, position, .. }) => {
                self.activation_conflict = false;
                self.failed = true;
                let place = position
                    .map(|(line, column)| {
                        format!(
                            " ({} {line}, {} {column})",
                            text(cx, "line", "行"),
                            text(cx, "column", "列"),
                        )
                    })
                    .unwrap_or_default();
                self.message = Some(format!(
                    "{} {kind:?}{place}; {}",
                    text(cx, "Invalid", "配置无效"),
                    text(cx, "no writes", "未写入"),
                ));
            }
        }
    }
    fn set_content(&mut self, value: String, _cx: &mut Context<Self>) {
        self.pending_content = Some(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{
        ConfigSource,
        profiles::{context, tests::fixture},
    };
    use ccr_app::{config::ConfigReadService, runtime::RuntimeOwner};
    use gpui_kit::base::TestSupportExt;
    use gpui_kit::test::{TestAppContextExt, TestWindowExt};
    use gpui_kit::{
        InputEvent, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
        MouseUpEvent, ParentElement, Render, Styled, TestAppContext, WindowOptions, div,
    };
    use std::time::Duration;
    fn selected(source: &ConfigSource) -> ConfigRow {
        let ConfigSource::Fixture {
            profiles, registry, ..
        } = source
        else {
            panic!("fixture")
        };
        ConfigReadService::from_paths(profiles.clone(), registry.clone())
            .list()
            .unwrap_or_else(|_| panic!("list"))
            .into_iter()
            .find(|r| r.name == "beta")
            .unwrap_or_else(|| panic!("beta"))
    }
    struct ConfirmationHost {
        tools: Entity<ProfileTools>,
        outside: FocusHandle,
        _tools_subscription: gpui_kit::Subscription,
    }
    impl Render for ConfirmationHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .size_full()
                .child(
                    div()
                        .id("profile-outside-focus")
                        .test_support()
                        .track_focus(&self.outside)
                        .tab_stop(true)
                        .child("Outside confirmation"),
                )
                .child(self.tools.clone())
        }
    }
    #[gpui_kit::test]
    fn confirmation_recovers_outside_focus_before_first_frame(cx: &mut TestAppContext) {
        confirmation_focus_boundary(cx, false);
    }
    #[gpui_kit::test]
    fn confirmation_enters_after_native_callback_before_draw_order(cx: &mut TestAppContext) {
        confirmation_focus_boundary(cx, true);
    }
    #[gpui_kit::test]
    fn catalog_decisions_enter_from_outside_without_mutating(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (_root, source) = fixture();
        let row = selected(&source);
        let context =
            context(&source, ccr_config::Platform::Claude).unwrap_or_else(|_| panic!("context"));
        let revision = context
            .manager()
            .snapshot()
            .unwrap_or_else(|_| panic!("snapshot"))
            .version;
        let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
        let services = Services::new(owner.handle(), source);
        let (handle, host) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    let tools = cx.new(|cx| {
                        let mut tools = ProfileTools::new(services, window, cx);
                        tools.target = Some(row);
                        tools
                    });
                    let subscription = cx.observe(&tools, |_, _, cx| cx.notify());
                    ConfirmationHost {
                        tools,
                        outside: cx.focus_handle(),
                        _tools_subscription: subscription,
                    }
                })
            })
            .unwrap_or_else(|_| panic!("window"))
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            for (policy, decisions) in [
                (
                    "profile-policy-configurations",
                    &[
                        "profile-delete",
                        "profile-import",
                        "profile-restore",
                        "profile-clean-backups",
                        "profile-clear-history",
                    ][..],
                ),
                (
                    "profile-policy-profiles",
                    &[
                        "profile-delete",
                        "profile-source",
                        "profile-inspect",
                        "profile-export-secrets",
                    ][..],
                ),
            ] {
                window.click(policy, cx);
                for id in decisions {
                    let outside = host.read(cx).outside.clone();
                    window.focus(&outside, cx);
                    window.click(*id, cx);
                    window.simulate_next_frame(cx);
                    window.render_frame(cx);
                    assert_eq!(
                        window.find("profile-confirm-cancel").focused(),
                        Some(true),
                        "{id} must initially focus Cancel from outside"
                    );
                    window.press("tab", cx);
                    assert_eq!(window.find("profile-confirm").focused(), Some(true));
                    window.press("shift-tab", cx);
                    assert_eq!(window.find("profile-confirm-cancel").focused(), Some(true));
                    window.press("escape", cx);
                    assert!(window.try_find("profile-confirm").is_none());
                    assert_eq!(window.find("profile-outside-focus").focused(), Some(true));
                    assert!(!host.read(cx).tools.read(cx).busy);
                    assert_eq!(
                        context
                            .manager()
                            .snapshot()
                            .unwrap_or_else(|_| panic!("snapshot"))
                            .version,
                        revision
                    );
                }
            }
            window.remove_window();
        })
        .unwrap_or_else(|_| panic!("decision matrix"));
        owner.shutdown();
    }
    #[gpui_kit::test]
    fn nested_confirmation_preserves_editor_return_focus(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (_root, source) = fixture();
        let row = selected(&source);
        let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
        let services = Services::new(owner.handle(), source);
        let (handle, host) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    let tools = cx.new(|cx| {
                        let mut tools = ProfileTools::new(services, window, cx);
                        tools.target = Some(row);
                        tools
                    });
                    let subscription = cx.observe(&tools, |_, _, cx| cx.notify());
                    ConfirmationHost {
                        tools,
                        outside: cx.focus_handle(),
                        _tools_subscription: subscription,
                    }
                })
            })
            .unwrap_or_else(|_| panic!("window"))
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let outside = host.read(cx).outside.clone();
            window.focus(&outside, cx);
            window.click("profile-edit", cx);
            for _ in 0..12 {
                window.press("tab", cx);
                if window.find("profile-advanced").focused() == Some(true) {
                    break;
                }
            }
            assert_eq!(window.find("profile-advanced").focused(), Some(true));
            window.press("enter", cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("profile-confirm-cancel").focused(), Some(true));
            window.press("escape", cx);
            assert_eq!(window.find("profile-advanced").focused(), Some(true));
            window.click("profile-editor-cancel", cx);
            window.render_frame(cx);
            assert_eq!(
                window.find("profile-outside-focus").focused(),
                Some(true),
                "closing an editor after a nested decision must restore its original trigger"
            );
            window.remove_window();
        })
        .unwrap_or_else(|_| panic!("nested confirmation"));
        owner.shutdown();
    }
    #[gpui_kit::test]
    fn platform_change_clears_only_previous_platform_recovery(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (_root, source) = fixture();
        let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
        let services = Services::new(owner.handle(), source);
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    let mut tools = ProfileTools::new(services, window, cx);
                    tools.platform = Platform::Grok;
                    tools.accept(
                        &ProfileResult::GrokUpdate {
                            status: "rename_apply_failed",
                            old_name: "old-grok".into(),
                            new_name: "new-grok".into(),
                        },
                        cx,
                    );
                    tools.grok_force_delete = Some("old-grok".into());
                    tools
                })
            })
            .unwrap_or_else(|_| panic!("window"))
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(window.find("profile-grok-retry-apply").visible());
            assert!(window.find("profile-grok-force-delete").visible());
            let warning = view.read(cx).message.clone();
            view.update(cx, |tools, cx| {
                assert!(tools.set_platform(Platform::Grok, window, cx));
                tools.accept(&ProfileResult::Status { can_off: true }, cx);
            });
            window.render_frame(cx);
            assert_eq!(view.read(cx).message, warning);
            assert!(window.find("profile-grok-retry-apply").visible());
            view.update(cx, |tools, cx| {
                assert!(tools.set_platform(Platform::Claude, window, cx));
            });
            window.render_frame(cx);
            assert!(
                window.try_find("profile-grok-retry-apply").is_none(),
                "a Grok recovery action must never dispatch through the next platform"
            );
            assert!(window.try_find("profile-grok-force-delete").is_none());
            assert!(view.read(cx).message.is_none());
            assert!(!view.read(cx).failed);
            window.remove_window();
        })
        .unwrap_or_else(|_| panic!("platform recovery"));
        owner.shutdown();
    }
    fn confirmation_focus_boundary(cx: &mut TestAppContext, callback_before_draw: bool) {
        cx.executor().allow_parking();
        let (_root, source) = fixture();
        let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
        let services = Services::new(owner.handle(), source);
        let (handle, host) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    let tools = cx.new(|cx| {
                        let mut tools = ProfileTools::new(services, window, cx);
                        tools.accept(&ProfileResult::Status { can_off: true }, cx);
                        tools
                    });
                    let subscription = cx.observe(&tools, |_, _, cx| cx.notify());
                    ConfirmationHost {
                        tools,
                        outside: cx.focus_handle(),
                        _tools_subscription: subscription,
                    }
                })
            })
            .unwrap_or_else(|_| panic!("window"))
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let outside = host.read(cx).outside.clone();
            window.focus(&outside, cx);
            window.render_frame(cx);
            assert_eq!(window.find("profile-outside-focus").focused(), Some(true));
            if callback_before_draw {
                // click() 会在 MouseUp 后自动绘制，掩盖原生先回调后绘制的顺序。
                let position = window.find("profile-off").bounds().center();
                window.dispatch_event(
                    MouseMoveEvent {
                        position,
                        pressed_button: None,
                        modifiers: Default::default(),
                    }
                    .to_platform_input(),
                    cx,
                );
                window.render_frame(cx);
                window.dispatch_event(
                    MouseDownEvent {
                        button: MouseButton::Left,
                        position,
                        modifiers: Default::default(),
                        click_count: 1,
                        first_mouse: false,
                    }
                    .to_platform_input(),
                    cx,
                );
                window.render_frame(cx);
                window.dispatch_event(
                    MouseUpEvent {
                        button: MouseButton::Left,
                        position,
                        modifiers: Default::default(),
                        click_count: 1,
                    }
                    .to_platform_input(),
                    cx,
                );
                assert!(host.read(cx).tools.read(cx).confirmation.is_some());
                // 对齐 gpui-pre 的平台帧：先排空既有回调，再绘制确认子树。
                window.simulate_next_frame(cx);
                window.render_frame(cx);
            } else {
                window.click("profile-off", cx);
            }
            assert!(host.read(cx).tools.read(cx).confirmation.is_some());
        })
        .unwrap_or_else(|_| panic!("open confirmation"));
        cx.run_until_parked();
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            // 模拟原生边界：决策树挂载后、首次延迟进入前，焦点恢复到外部。
            let outside = host.read(cx).outside.clone();
            window.focus(&outside, cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert!(window.find("profile-confirm").visible());
            assert_eq!(window.find("profile-confirm-cancel").focused(), Some(true));
            for _ in 0..6 {
                window.press("tab", cx);
                assert!(
                    window.find("profile-confirm-cancel").focused() == Some(true)
                        || window.find("profile-confirm").focused() == Some(true),
                    "opening a confirmation from outside must enter its decision scope"
                );
            }
            for _ in 0..6 {
                window.press("shift-tab", cx);
                assert!(
                    window.find("profile-confirm-cancel").focused() == Some(true)
                        || window.find("profile-confirm").focused() == Some(true),
                    "reverse traversal must stay inside the decision scope"
                );
            }
            window.press("escape", cx);
            assert!(window.try_find("profile-confirm").is_none());
            assert_eq!(window.find("profile-outside-focus").focused(), Some(true));
            window.click("profile-off", cx);
            window.click("profile-confirm-cancel", cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert!(host.read(cx).tools.read(cx).confirmation.is_none());
            assert_eq!(window.find("profile-outside-focus").focused(), Some(true));
            window.click("profile-off", cx);
            host.read(cx).tools.clone().update(cx, |tools, cx| {
                tools.leave(window, cx);
            });
            let outside = host.read(cx).outside.clone();
            window.focus(&outside, cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert!(host.read(cx).tools.read(cx).confirmation.is_none());
            assert_eq!(
                window.find("profile-outside-focus").focused(),
                Some(true),
                "navigation before deferred entry must not steal destination focus"
            );
            window.remove_window();
        })
        .unwrap_or_else(|_| panic!("outside focus"));
        owner.shutdown();
    }
    #[gpui_kit::test]
    fn editor_confirmation_blocks_underlying_save_and_returns_focus(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (_root, source) = fixture();
        let context =
            context(&source, ccr_config::Platform::Claude).unwrap_or_else(|_| panic!("context"));
        let revision = context
            .manager()
            .snapshot()
            .unwrap_or_else(|_| panic!("snapshot"))
            .version;
        let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
        let services = Services::new(owner.handle(), source.clone());
        let row = selected(&source);
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    let mut view = ProfileTools::new(services, window, cx);
                    view.target = Some(row);
                    view
                })
            })
            .unwrap_or_else(|_| panic!("window"))
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("profile-edit", cx);
            window.click("profile-description", cx);
            window.press("ctrl-a", cx);
            window.input("Unconfirmed draft", cx);
            for _ in 0..12 {
                window.press("tab", cx);
                if window.find("profile-advanced").focused() == Some(true) {
                    break;
                }
            }
            assert_eq!(window.find("profile-advanced").focused(), Some(true));
            window.press("enter", cx);
            assert!(view.read(cx).confirmation.is_some());
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            window.click("profile-save", cx);
            assert!(
                !view.read(cx).busy,
                "underlying Save must not run while confirmation is open"
            );
            assert_eq!(
                context
                    .manager()
                    .snapshot()
                    .unwrap_or_else(|_| panic!("snapshot"))
                    .version,
                revision
            );
            // 确认焦点域仅包含两个决策按钮，不手动预置域内焦点。
            for _ in 0..8 {
                window.press("tab", cx);
                assert!(
                    window.find("profile-confirm-cancel").focused() == Some(true)
                        || window.find("profile-confirm").focused() == Some(true),
                    "focused nodes: {:?}",
                    gpui_kit::base::test_support::snapshots(window)
                        .iter()
                        .filter(|node| node.focused() == Some(true))
                        .map(|node| node.label())
                        .collect::<Vec<_>>()
                );
            }
            for _ in 0..8 {
                window.press("shift-tab", cx);
                assert!(
                    window.find("profile-confirm-cancel").focused() == Some(true)
                        || window.find("profile-confirm").focused() == Some(true)
                );
            }
            window.press("escape", cx);
            assert!(view.read(cx).confirmation.is_none());
            assert_eq!(window.find("profile-advanced").focused(), Some(true));
            assert_eq!(
                window.find("profile-description").value(),
                Some("Unconfirmed draft")
            );
            window.click("profile-editor-cancel", cx);
            view.update(cx, |view, cx| {
                view.generic = true;
                cx.notify();
            });
            window.render_frame(cx);
            window.click("profile-restore", cx);
            window.click("profile-confirm", cx);
            window.click("profile-name", cx);
            window.input("profiles.toml.fixture.bak", cx);
            window.press("tab", cx);
            assert_eq!(window.find("profile-save").focused(), Some(true));
            window.press("enter", cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert!(
                window
                    .find("profile-confirmation")
                    .label()
                    .unwrap_or_default()
                    .contains("profiles.toml.fixture.bak")
            );
            window.press("escape", cx);
            assert!(view.read(cx).confirmation.is_none());
            assert_eq!(window.find("profile-save").focused(), Some(true));
            view.update(cx, |view, cx| {
                view.accept(
                    &ProfileResult::GrokUpdate {
                        status: "rename_apply_failed",
                        old_name: "old".into(),
                        new_name: "new".into(),
                    },
                    cx,
                );
            });
            window.render_frame(cx);
            let partial_warning = view.read(cx).message.clone();
            assert!(
                partial_warning
                    .as_deref()
                    .is_some_and(|message| { message.contains("rename_apply_failed") }),
                "a real partial-write warning must exist before checking its lifetime"
            );
            view.update(cx, |view, cx| {
                view.close(window, cx);
                view.accept(&ProfileResult::Status { can_off: true }, cx);
            });
            assert!(view.read(cx).failed);
            assert_eq!(view.read(cx).message, partial_warning);
            assert!(view.read(cx).grok_recovery.is_some());
            assert!(window.find("profile-grok-retry-apply").visible());
            view.update(cx, |view, cx| {
                use ccr_cli::application::profile_lifecycle::{
                    ProfileOutcome, ProfileStatus, ProfileWarning,
                };
                view.accept(
                    &ProfileResult::Outcome(ProfileOutcome {
                        operation_id: "fixture-retry".into(),
                        platform: "grok".into(),
                        profile: "new".into(),
                        previous_profile: Some("old".into()),
                        status: ProfileStatus::AppliedWithWarning,
                        activation_committed: true,
                        warnings: vec![ProfileWarning::HistoryFailed],
                        recovery_paths: vec![],
                    }),
                    cx,
                );
                view.accept(&ProfileResult::Status { can_off: true }, cx);
            });
            window.render_frame(cx);
            assert!(
                view.read(cx).grok_recovery.is_none(),
                "a committed retry must remove the old apply-retry action"
            );
            assert!(
                view.read(cx)
                    .message
                    .as_deref()
                    .unwrap_or_default()
                    .contains("Do not repeat activation")
            );
            let warning = view.read(cx).message.clone();
            view.update(cx, |view, cx| view.close(window, cx));
            assert_eq!(view.read(cx).message, warning);
            view.update(cx, |view, cx| {
                view.target = None;
                cx.notify();
                window.focus(&view.focus, cx);
            });
            window.render_frame(cx);
            window.click("profile-edit", cx);
            assert!(view.read(cx).mode == Mode::Closed);
            for _ in 0..16 {
                window.press("tab", cx);
                for id in ["profile-edit", "profile-delete", "profile-apply"] {
                    assert_ne!(
                        window.find(id).focused(),
                        Some(true),
                        "disabled {id} must not take Tab focus"
                    );
                }
            }
            window.remove_window();
        })
        .unwrap_or_else(|_| panic!("confirmation"));
        owner.shutdown();
    }
    #[gpui_kit::test]
    async fn rendered_edit_save_conflict_cancel_and_delete_confirmation(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (_root, source) = fixture();
        let context =
            context(&source, ccr_config::Platform::Claude).unwrap_or_else(|_| panic!("context"));
        let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
        let services = Services::new(owner.handle(), source.clone());
        let row = selected(&source);
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    let mut view = ProfileTools::new(services, window, cx);
                    view.target = Some(row);
                    view
                })
            })
            .unwrap_or_else(|_| panic!("window"))
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("profile-edit", cx);
            window.click("profile-description", cx);
            window.press("ctrl-a", cx);
            window.input("Native saved", cx);
            window.click("profile-credential", cx);
            window.input("replacement-native-secret", cx);
            assert!(
                !window
                    .find("profile-credential")
                    .value()
                    .unwrap_or_default()
                    .contains("replacement-native-secret")
            );
            window.click("profile-save", cx);
        })
        .unwrap_or_else(|_| panic!("edit"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        let saved = context.manager().load().unwrap_or_else(|_| panic!("read"));
        assert_eq!(
            saved.sections["beta"].description.as_deref(),
            Some("Native saved")
        );
        assert_eq!(
            saved.sections["beta"]
                .auth_token
                .as_ref()
                .map(ccr_core::Secret::expose),
            Some("replacement-native-secret")
        );
        cx.update_window(handle, |_, _, cx| {
            view.update(cx, |view, cx| {
                view.message = Some("Committed warning retained".into());
                view.submit(ProfileCommand::Status, cx);
            })
        })
        .unwrap_or_else(|_| panic!("status refresh"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        assert_eq!(
            cx.read(|cx| view.read(cx).message.clone()).as_deref(),
            Some("Committed warning retained")
        );
        cx.update_window(handle, |_, window, cx| {
            window.click("profile-inspect", cx);
            assert!(window.find("profile-confirm").visible());
            assert!(view.read(cx).content.read(cx).value().is_empty());
            window.click("profile-confirm", cx);
        })
        .unwrap_or_else(|_| panic!("inspect warning"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(view.read(cx).mode == Mode::Output);
            let before = view.read(cx).content.read(cx).value().to_string();
            assert!(
                before.contains("replacement-native-secret")
                    && before.contains("unknown_native")
                    && before.contains("when_native")
            );
            view.update(cx, |view, cx| {
                window.focus(&view.content.read(cx).focus_handle(cx), cx)
            });
            window.input("must-not-edit-readonly-inspection", cx);
            assert_eq!(view.read(cx).content.read(cx).value().as_ref(), before);
            view.update(cx, |view, cx| view.leave(window, cx));
            assert!(view.read(cx).content.read(cx).value().is_empty());
            assert!(view.read(cx).mode == Mode::Closed);
        })
        .unwrap_or_else(|_| panic!("scoped readonly inspect"));
        let row = selected(&source);
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.target = Some(row);
                cx.notify();
            });
            window.render_frame(cx);
            window.click("profile-edit", cx);
            window.click("profile-description", cx);
            window.press("ctrl-a", cx);
            window.input("Retained draft", cx);
        })
        .unwrap_or_else(|_| panic!("draft"));
        context
            .run(|| {
                context.manager().mutate(|config| {
                    config.get_section_mut("beta")?.model = Some("external-model".into());
                    Ok(())
                })
            })
            .unwrap_or_else(|_| panic!("external client"));
        cx.update_window(handle, |_, window, cx| window.click("profile-save", cx))
            .unwrap_or_else(|_| panic!("save"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            assert!(view.read(cx).failed);
            assert!(view.read(cx).mode == Mode::Edit);
            assert!(view.read(cx).draft_conflict);
            assert_eq!(
                view.read(cx).message.as_deref(),
                Some(ProfileError::Conflict.message()),
                "the retained-draft warning must exist before cancellation clears it"
            );
            assert_eq!(
                window.find("profile-description").value(),
                Some("Retained draft")
            );
            window.click("profile-editor-cancel", cx);
            assert!(
                view.read(cx).message.is_none(),
                "discarded draft has no retained-draft status"
            );
            assert!(!view.read(cx).failed);
            view.update(cx, |view, cx| view.submit(ProfileCommand::Status, cx));
        })
        .unwrap_or_else(|_| panic!("cancel stale draft and refresh"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        assert_eq!(
            context
                .manager()
                .load()
                .unwrap_or_else(|_| panic!("read"))
                .sections["beta"]
                .model
                .as_deref(),
            Some("external-model")
        );
        cx.update_window(handle, |_, window, cx| {
            assert!(view.read(cx).message.is_none());
            assert!(!view.read(cx).failed);
            view.update(cx, |view, cx| window.focus(&view.focus, cx));
            for _ in 0..24 {
                window.press("tab", cx);
                if window.find("profile-delete").focused() == Some(true) {
                    break;
                }
            }
            assert_eq!(window.find("profile-delete").focused(), Some(true));
            window.press("enter", cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert!(window.find("profile-confirm").visible());
            window.press("escape", cx);
            assert!(view.read(cx).confirmation.is_none());
            assert_eq!(window.find("profile-delete").focused(), Some(true));
            window.click("profile-delete", cx);
            window.click("profile-confirm-cancel", cx);
            assert!(view.read(cx).confirmation.is_none());
            window.click("profile-delete", cx);
            window.click("profile-confirm", cx);
        })
        .unwrap_or_else(|_| panic!("confirmation"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        assert!(
            !context
                .manager()
                .load()
                .unwrap_or_else(|_| panic!("read"))
                .sections
                .contains_key("beta")
        );
        cx.update_window(handle, |_, window, cx| {
            window.click("profile-source", cx);
            window.click("profile-confirm", cx);
        })
        .unwrap_or_else(|_| panic!("source warning"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(
                view.read(cx)
                    .content
                    .read(cx)
                    .value()
                    .contains("sk-native-profile-secret-sentinel")
            );
        })
        .unwrap_or_else(|_| panic!("source read"));
        context
            .run(|| {
                context.manager().mutate(|config| {
                    config.get_section_mut("alpha")?.description =
                        Some("External source change".into());
                    Ok(())
                })
            })
            .unwrap_or_else(|_| panic!("external source client"));
        cx.update_window(handle, |_, window, cx| window.click("profile-save", cx))
            .unwrap_or_else(|_| panic!("save stale source"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            assert!(view.read(cx).draft_conflict);
            assert!(view.read(cx).failed);
            assert!(view.read(cx).mode == Mode::Source);
            assert_eq!(
                view.read(cx).message.as_deref(),
                Some(ProfileError::Conflict.message())
            );
            assert!(
                view.read(cx)
                    .content
                    .read(cx)
                    .value()
                    .contains("sk-native-profile-secret-sentinel"),
                "a source CAS refusal must retain the original source draft"
            );
            window.click("profile-editor-cancel", cx);
            assert!(view.read(cx).message.is_none());
            assert!(!view.read(cx).failed);
            view.update(cx, |view, cx| view.submit(ProfileCommand::Status, cx));
        })
        .unwrap_or_else(|_| panic!("cancel stale source and refresh"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            assert!(view.read(cx).message.is_none());
            assert!(!view.read(cx).failed);
            window.click("profile-source", cx);
            window.click("profile-confirm", cx);
        })
        .unwrap_or_else(|_| panic!("reload source after cancel"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(view.read(cx).message.is_none());
            assert!(
                view.read(cx)
                    .content
                    .read(cx)
                    .value()
                    .contains("External source change")
            );
            view.update(cx, |view, cx| view.leave(window, cx));
            window.render_frame(cx);
            assert!(view.read(cx).mode == Mode::Closed);
            assert!(view.read(cx).pending_content.is_none());
            assert!(view.read(cx).content.read(cx).value().is_empty());
            assert!(view.read(cx).credential.read(cx).text().len_chars() == 0);
        })
        .unwrap_or_else(|_| panic!("scoped source leave"));
        cx.update_window(handle, |_, window, _| window.remove_window())
            .unwrap_or_else(|_| panic!("close"));
        owner.shutdown();
    }
}
