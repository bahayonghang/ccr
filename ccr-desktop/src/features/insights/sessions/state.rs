//! Sessions surface state: the list snapshot and a separate detail slot.
//!
//! 只有 `Surface::Sessions` 使用这里的状态；Workbench 与 Usage 仍走单 snapshot 路径。
//! 列表与详情各有自己的 gate 与 generation（RequestKey environment 分别为
//! `insights-explicit` / `insights-detail`），晚到的旧响应会被丢弃。
use crate::features::insights::{InsightsPage, InsightsView, Surface};
use crate::runtime::insights::{InsightsError, InsightsQuery, InsightsSnapshot};
use ccr_app::runtime::{
    AdmissionError, Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent,
};
use ccr_app::sessions::{SessionDetail, SessionListRequest};
use ccr_db::database::repositories::usage_repo::AgentSessionArchiveRow;
use ccr_store::sessions::AgentSessionAgentId;
use gpui_kit::{Context, Task, Window, px};
use std::sync::Arc;
use std::time::Duration;

/// Search typing waits this long before querying; Enter queries at once.
pub(in crate::features::insights) const SEARCH_DEBOUNCE: Duration = Duration::from_millis(300);

/// Rows per list page.
const PAGE_LIMIT: usize = 100;

/// Fidelity filter values accepted by `ccr_app::sessions::list_sessions`.
pub(in crate::features::insights) const FIDELITIES: [(&str, &str, &str); 3] = [
    ("full", "Full record", "完整记录"),
    ("partial", "Partial", "部分"),
    ("locked", "Locked", "锁定"),
];

/// Source-state filter values accepted by `ccr_app::sessions::list_sessions`.
pub(in crate::features::insights) const SOURCE_STATES: [(&str, &str, &str); 3] = [
    ("live", "Live", "现存"),
    ("missing", "Missing", "缺失"),
    ("deleted_by_user", "Deleted", "已删除"),
];

/// The opened session on the Sessions surface, independent of the list snapshot.
#[derive(Default)]
pub(in crate::features::insights) struct SessionDetailSlot {
    pub(in crate::features::insights) archive_id: Option<String>,
    /// The list row that opened this session: title, ids and counts for the pane
    /// heading. `SessionDetail` itself carries no title.
    pub(in crate::features::insights) row: Option<AgentSessionArchiveRow>,
    pub(in crate::features::insights) before: Option<u32>,
    /// Always an `InsightsSnapshot::Detail`; kept as the loaded `Arc` to avoid a copy.
    pub(in crate::features::insights) detail: Option<Arc<InsightsSnapshot>>,
    pub(in crate::features::insights) loading: bool,
    pub(in crate::features::insights) error: Option<&'static str>,
    pub(in crate::features::insights) gate: ResponseGate,
    pub(in crate::features::insights) generation: u64,
    pub(in crate::features::insights) task: Option<Task<()>>,
}

impl SessionDetailSlot {
    /// 清空并让 generation 前进：在途请求与延迟焦点回调都随之作废。
    fn reset(&mut self) {
        *self = Self {
            generation: self.generation + 1,
            ..Default::default()
        };
    }
}

/// Session family shown for a platform-scoped page; `None` keeps all providers.
pub(in crate::features::insights) fn platform_agent(
    platform: Platform,
) -> Option<AgentSessionAgentId> {
    match platform {
        Platform::Claude => Some(AgentSessionAgentId::Claude),
        Platform::Codex => Some(AgentSessionAgentId::Codex),
        _ => None,
    }
}

impl InsightsView {
    /// The loaded detail of the opened session on the Sessions surface.
    pub(in crate::features::insights) fn session_detail(&self) -> Option<&SessionDetail> {
        match self.detail.detail.as_deref() {
            Some(InsightsSnapshot::Detail(detail)) => Some(detail),
            _ => None,
        }
    }

    /// The opened session id: the detail slot on the Sessions surface, else the
    /// workbench drill-in id.
    pub(in crate::features::insights) fn opened_session(&self) -> Option<&str> {
        if self.surface == Surface::Sessions {
            self.detail.archive_id.as_deref()
        } else {
            self.archive_id.as_deref()
        }
    }

    /// Provider family locked by the platform scope.
    pub(in crate::features::insights) fn locked_agent(&self) -> Option<AgentSessionAgentId> {
        self.platform_scope.and_then(platform_agent)
    }

    /// Clears the session filters; a platform scope locks its own family.
    pub(in crate::features::insights) fn reset_session_filters(&mut self) {
        self.agents = self.locked_agent().into_iter().collect();
        self.fidelity = None;
        self.source_state = None;
    }

    pub(in crate::features::insights) fn session_list_request(
        &self,
        cx: &Context<Self>,
    ) -> Result<SessionListRequest, InsightsError> {
        let agents = match self.locked_agent() {
            Some(agent) => Some(vec![agent]),
            None if self.agents.is_empty() => None,
            None => Some(self.agents.iter().copied().collect()),
        };
        let query = self.fields[6].read(cx).value().trim().to_string();
        Ok(SessionListRequest {
            agents,
            query: (!query.is_empty()).then_some(query),
            fidelity: self.fidelity.map(str::to_string),
            source_state: self.source_state.map(str::to_string),
            cursor: self.cursor.clone(),
            limit: Some(PAGE_LIMIT),
            ..Default::default()
        })
    }

    /// 关闭详情槽。废纸篓打开时只关详情，不清废纸篓与它在途的恢复核对。
    pub(in crate::features::insights) fn reset_session_detail(&mut self) {
        self.detail.reset();
        if !self.session_ops_trash_open() {
            self.clear_session_draft();
        }
    }

    /// Filters changed: drop the opened detail and read the first list page.
    pub(in crate::features::insights) fn apply_session_filters(&mut self, cx: &mut Context<Self>) {
        self.search_generation += 1;
        self.search_task = None;
        self.reset_session_detail();
        self.cursor = None;
        self.submit(cx);
    }

    pub(in crate::features::insights) fn toggle_session_agent(
        &mut self,
        agent: AgentSessionAgentId,
        cx: &mut Context<Self>,
    ) {
        if self.locked_agent().is_some() {
            return;
        }
        if !self.agents.remove(&agent) {
            self.agents.insert(agent);
        }
        self.apply_session_filters(cx);
    }

    pub(in crate::features::insights) fn set_session_fidelity(
        &mut self,
        value: Option<&'static str>,
        cx: &mut Context<Self>,
    ) {
        self.fidelity = value;
        self.apply_session_filters(cx);
    }

    pub(in crate::features::insights) fn set_session_source_state(
        &mut self,
        value: Option<&'static str>,
        cx: &mut Context<Self>,
    ) {
        self.source_state = value;
        self.apply_session_filters(cx);
    }

    /// 输入变化后防抖查询；带 generation token，过期的 token 到期后不执行。
    pub(in crate::features::insights) fn schedule_session_search(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if self.surface != Surface::Sessions || self.page != InsightsPage::Sessions {
            return;
        }
        self.search_generation += 1;
        let token = self.search_generation;
        self.search_task = Some(cx.spawn(async move |weak, cx| {
            cx.background_executor().timer(SEARCH_DEBOUNCE).await;
            let _ = weak.update(cx, |this, cx| {
                if this.search_generation == token {
                    this.apply_session_filters(cx);
                }
            });
        }));
    }

    /// Enter in the search field queries immediately and cancels a pending debounce.
    pub(in crate::features::insights) fn search_sessions_now(&mut self, cx: &mut Context<Self>) {
        if self.surface != Surface::Sessions || self.page != InsightsPage::Sessions {
            return;
        }
        self.apply_session_filters(cx);
    }

    /// Back to the newest list page; the opened detail stays.
    pub(in crate::features::insights) fn first_session_page(&mut self, cx: &mut Context<Self>) {
        if self.cursor.is_none() {
            return;
        }
        self.cursor = None;
        self.submit(cx);
    }

    /// Opens a list row from a pointer or keyboard; a narrow window drills in.
    pub(in crate::features::insights) fn open_session(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select(index, cx);
        if self.detail.archive_id.is_some() {
            self.reveal_session_pane(window, cx);
        }
    }

    /// Loads one page of the opened session into the detail slot.
    pub(in crate::features::insights) fn submit_detail(&mut self, cx: &mut Context<Self>) {
        let Some(archive_id) = self.detail.archive_id.clone() else {
            return;
        };
        // 详情变化后，旧的导出、确认与废纸篓都不再适用。
        self.clear_session_draft();
        self.detail.task = None;
        self.detail.gate.clear();
        self.detail.generation += 1;
        self.detail.detail = None;
        self.detail.error = None;
        let key = RequestKey {
            environment: EnvironmentId::Local("insights-detail".into()),
            platform: Platform::Claude,
            generation: self.detail.generation,
            request_id: self.detail.generation,
        };
        self.detail.gate.replace(key.clone());
        self.detail.loading = true;
        let query = InsightsQuery::Detail {
            archive_id,
            before: self.detail.before,
        };
        match self.requests.load(key.clone(), query) {
            Ok(mut subscription) => {
                self.detail.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.detail.gate.accepts(&key) {
                                    return;
                                }
                                this.detail.loading = false;
                                this.detail.error = match result.as_ref() {
                                    Completion::Success(snapshot) => {
                                        if matches!(**snapshot, InsightsSnapshot::Detail(_)) {
                                            this.detail.detail = Some(snapshot.clone());
                                            None
                                        } else {
                                            Some(InsightsError::Query.message())
                                        }
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => Some(error.message()),
                                    Completion::Cancelled => Some("Read cancelled / 已取消读取"),
                                    Completion::WorkerFailed => {
                                        Some("Read worker stopped. Retry. / 读取线程已停止，请重试")
                                    }
                                };
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(error) => {
                self.detail.loading = false;
                self.detail.error = Some(match error {
                    AdmissionError::Full => "Read queue busy. Retry. / 读取队列繁忙，请重试",
                    AdmissionError::Stopped => "Application is shutting down / 应用正在退出",
                });
            }
        }
        cx.notify();
    }

    /// "Earlier messages": the next older page of the opened session.
    pub(in crate::features::insights) fn older_session_messages(&mut self, cx: &mut Context<Self>) {
        let Some(before) = self
            .session_detail()
            .filter(|detail| detail.page.has_older)
            .and_then(|detail| detail.page.next_before)
        else {
            return;
        };
        self.detail.before = Some(before);
        self.submit_detail(cx);
    }

    /// 窄窗 drill-in：pane 在下一帧取代列表后，把焦点移到 pane 的第一个控件。
    fn reveal_session_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if window.viewport_size().width >= px(crate::theme::SPLIT_WIDTH) {
            return;
        }
        let guard = (self.generation, self.detail.generation);
        let view = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                // 离页、刷新或又打开了别的会话：延迟回调不再抢焦点。
                if (this.generation, this.detail.generation) != guard {
                    return;
                }
                window.focus(&this.session_pane_focus, cx);
                window.focus_next(cx);
            });
        });
    }

    /// 窄窗从详情返回列表；下一帧焦点回到列表，选中项仍是原来的行。
    pub(in crate::features::insights) fn back_to_session_list(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let row = self.detail.archive_id.clone();
        self.reset_session_detail();
        self.selected = row;
        let guard = (self.generation, self.detail.generation);
        let view = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                if (this.generation, this.detail.generation) != guard {
                    return;
                }
                window.focus(&this.session_list_focus, cx);
            });
        });
        cx.notify();
    }
}
