use super::*;
use crate::runtime::checkin::{AccountJob, JobAction, JobScope, JobStatus, Operation};
use ccr_db::models::checkin::CheckinStatus;

impl CheckinView {
    pub(super) fn job_button(
        &self,
        key: &str,
        label: &'static str,
        action: JobAction,
        scope: JobScope,
        primary: bool,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let blocked = self.busy
            || self.operation.busy
            || self.confirmation.is_some()
            || self.editor.is_some()
            || !scope.can_start(action, self.rows());
        let source = key.to_owned();
        let mut button = Button::new(key.to_owned())
            .label(label)
            .on_click(cx.listener(move |this, _, _, cx| {
                let Some(data) = this.data() else { return };
                let job = match AccountJob::prepare(data, action, scope.clone()) {
                    Ok(job) => job,
                    Err(error) => {
                        Arc::make_mut(&mut this.operation).result = Some(Err(error));
                        cx.notify();
                        return;
                    }
                };
                let names = job
                    .targets
                    .iter()
                    .map(|target| format!("{} · {} ({})", target.name, target.provider, target.id))
                    .collect::<Vec<_>>()
                    .join("\n");
                let notice = format!(
                    "{label} · {} {}\n{names}",
                    job.targets.len(),
                    if job.targets.len() == 1 {
                        text(cx, "account", "个账户")
                    } else {
                        text(cx, "accounts", "个账户")
                    }
                );
                this.confirm_entry(Operation::Job(job), notice, &source, cx);
            }));
        if primary && !blocked {
            button = button.primary();
        } else {
            button = button.ghost();
        }
        self.origin_control(key, semantics::button(button, blocked, false, cx))
            .into_any_element()
    }

    pub(super) fn header_job_actions(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        if self.rows().is_empty() {
            return None;
        }
        let n = self.rows().iter().filter(|row| row.enabled).count();
        let mut actions = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(self.job_button(
                "checkin-all",
                text(cx, "Check in all", "一键签到"),
                JobAction::Checkin,
                JobScope::AllEnabled,
                true,
                cx,
            ))
            .child(self.job_button(
                "checkin-refresh-balances",
                text(cx, "Refresh balances", "刷新余额"),
                JobAction::Balance,
                JobScope::AllEnabled,
                false,
                cx,
            ));
        if n == 0 {
            actions = actions.child(text(cx, "No enabled accounts", "没有已启用的账户"));
        } else if n > crate::runtime::checkin::MAX_JOB_TARGETS {
            actions = actions.child(text(
                cx,
                "Select up to 1,000 accounts",
                "每批最多选择 1000 个账户",
            ));
        }
        Some(actions.into_any_element())
    }

    pub(super) fn selection_actions(&self, cx: &mut Context<Self>) -> Option<gpui_kit::AnyElement> {
        if self.checked_ids.is_empty() {
            return None;
        }
        let scope = JobScope::Selected(self.checked_ids.clone());
        let balance_enabled = scope.can_start(JobAction::Balance, self.rows());
        let hint = if self.checked_ids.len() > crate::runtime::checkin::MAX_JOB_TARGETS {
            Some(text(
                cx,
                "Each batch accepts up to 1,000 target accounts. Balance refresh counts enabled accounts only.",
                "每批最多处理 1000 个目标账户。刷新余额仅计入启用账户。",
            ))
        } else if !balance_enabled {
            Some(text(
                cx,
                "No enabled accounts selected for balance refresh",
                "所选账户中没有可刷新余额的启用账户",
            ))
        } else {
            None
        };
        Some(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(theme::CONTROL_GAP))
                .child(format!(
                    "{} {}",
                    self.checked_ids.len(),
                    text(cx, "selected", "项已选")
                ))
                .child(self.job_button(
                    "checkin-selected-run",
                    text(cx, "Check in selected", "签到所选"),
                    JobAction::Checkin,
                    scope.clone(),
                    false,
                    cx,
                ))
                .child(self.job_button(
                    "checkin-selected-balances",
                    text(cx, "Refresh selected balances", "刷新所选余额"),
                    JobAction::Balance,
                    scope,
                    false,
                    cx,
                ))
                .children(hint.map(|message| {
                    div()
                        .id("checkin-selection-hint")
                        .test_support()
                        .role(gpui_kit::Role::Status)
                        .aria_label(message)
                        .child(message)
                }))
                .into_any_element(),
        )
    }

    pub(super) fn row_job_actions(
        &self,
        row: &CheckinAccountRow,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let id = row.id.clone();
        let scope = JobScope::Single(id.clone());
        let selected = self.checked_ids.contains(&id);
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(semantics::button(
                Button::new(format!("checkin-pick-{id}"))
                    .ghost()
                    .selected(selected)
                    .label(if selected {
                        text(cx, "Selected", "已选")
                    } else {
                        text(cx, "Select", "选择")
                    })
                    .accessibility_label(format!(
                        "{} · {}",
                        text(cx, "Select account", "选择账户"),
                        row.name
                    ))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.checked_ids.remove(&id) {
                            this.checked_ids.insert(id.clone());
                        }
                        cx.notify();
                    })),
                self.operation.busy,
                false,
                cx,
            ))
            .child(self.job_button(
                &format!("checkin-run-{}", row.id),
                text(cx, "Check in", "签到"),
                JobAction::Checkin,
                scope.clone(),
                false,
                cx,
            ))
            .child(self.job_button(
                &format!("checkin-refresh-{}", row.id),
                text(cx, "Balance", "刷新余额"),
                JobAction::Balance,
                scope,
                false,
                cx,
            ))
            .into_any_element()
    }

    pub(super) fn job_report(&self, cx: &mut Context<Self>) -> Option<gpui_kit::AnyElement> {
        let report = self.operation.last_job.as_deref()?;
        let title = match report.action {
            JobAction::Checkin => text(cx, "Last check-in results", "最近签到结果"),
            JobAction::Balance => text(cx, "Last balance results", "最近余额刷新结果"),
        };
        let mut list = div()
            .id("checkin-job-results")
            .test_support()
            .role(gpui_kit::Role::Group)
            .aria_label(title)
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(title);
        for item in &report.items {
            let status = match item.status {
                JobStatus::Checked(CheckinStatus::Success) => text(cx, "Success", "成功"),
                JobStatus::Checked(CheckinStatus::AlreadyCheckedIn) => {
                    text(cx, "Already checked in today", "今日已签到")
                }
                JobStatus::Checked(CheckinStatus::Skipped) => text(cx, "Skipped", "跳过"),
                JobStatus::Failed | JobStatus::Checked(CheckinStatus::Failed) => {
                    text(cx, "Failed", "失败")
                }
                JobStatus::Updated => text(cx, "Updated", "已更新"),
                JobStatus::Recent => text(
                    cx,
                    "Skipped · refreshed within 30 seconds",
                    "跳过 · 30 秒内已刷新",
                ),
            };
            let id = &item.target.id;
            let mut line = format!(
                "{status} · {} · {} ({id})",
                item.target.name, item.target.provider
            );
            for detail in [
                item.error_code.as_deref(),
                item.skip_reason.as_deref(),
                item.message.as_deref(),
                item.admission_error
                    .map(|error| operations::error(cx, error)),
            ]
            .into_iter()
            .flatten()
            {
                line.push_str(" · ");
                line.push_str(detail);
            }
            if (matches!(
                item.status,
                JobStatus::Failed | JobStatus::Checked(CheckinStatus::Failed)
            ) && item.error_code.is_none())
                || (item.status == JobStatus::Checked(CheckinStatus::Skipped)
                    && item.skip_reason.is_none())
            {
                line.push_str(text(cx, " · provider detail unknown", " · 提供商明细未知"));
            }
            let mut row = div()
                .id(format!("checkin-result-{id}"))
                .test_support()
                .role(gpui_kit::Role::Group)
                .aria_label(line.clone())
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(theme::CONTROL_GAP))
                .child(line);
            if crate::runtime::checkin::credential_error_code(item.error_code.as_deref())
                && self.rows().iter().any(|r| &r.id == id)
            {
                let source = format!("result-edit-{id}");
                let button_id = source.clone();
                let id = id.clone();
                row = row.child(
                    self.origin_control(
                        &source,
                        semantics::ghost_button(
                            Button::new(button_id.clone())
                                .label(text(cx, "Edit credentials", "编辑凭据"))
                                .accessibility_label(format!(
                                    "{} · {} · {}",
                                    text(cx, "Edit credentials", "编辑凭据"),
                                    item.target.name,
                                    item.target.provider
                                ))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.open_account_credentials(&id, &button_id, window, cx)
                                })),
                            self.operation.busy,
                            false,
                            cx,
                        ),
                    ),
                );
            }
            list = list.child(row);
        }
        Some(list.into_any_element())
    }
}
