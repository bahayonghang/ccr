//! Command workbench layout: page header, catalog list, editor and job log.
use super::{
    InstallRequested, JobsView,
    collections::Collection,
    ledger::{self, Truncation},
    model::{self, Availability, Blocked, Filter},
};
use crate::{
    locales::{is_chinese, localized_line, service_error, text},
    shell::{badge, semantics},
    theme::{self, Tone},
};
use ccr_app::jobs::{
    commands::{Channel, Snapshot, Status},
    policy::CommandInfo,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, Icon, IconName,
    button::{Button, ButtonVariants},
    input::Input,
    tooltip::Tooltip,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    AnyElement, App, Context, Div, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Pixels, Render, Role, SharedString, Stateful, StatefulInteractiveElement,
    Styled, Window, div, px, uniform_list,
};

/// Run and Cancel share one slot, so swapping them never moves the row.
const RUN_WIDTH: f32 = 96.;
/// List column: content width × 34%, a multiple of 8, within 320–400px.
const LIST_SHARE: f32 = 0.34;
const LIST_MIN: f32 = 320.;
const LIST_MAX: f32 = 400.;

type Panel = gpui_kit::base::ObservedElement<Stateful<Div>>;

fn list_width(viewport: Pixels) -> f32 {
    let content = f32::from(viewport)
        - theme::sidebar_width(viewport)
        - 2. * theme::content_padding(viewport);
    ((content * LIST_SHARE / 8.).floor() * 8.).clamp(LIST_MIN, LIST_MAX)
}

fn reading_panel(id: &'static str, cx: &App) -> Panel {
    div()
        .id(id)
        .test_support()
        .flex()
        .flex_col()
        .min_w_0()
        .min_h_0()
        .gap(px(theme::CONTROL_GAP))
        .p(px(theme::SECTION_GAP))
        .rounded(px(theme::PANEL_RADIUS))
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().border)
}

fn eyebrow(label: &'static str, cx: &App) -> Div {
    div()
        .text_size(theme::text_size(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .child(label)
}

/// Header and ledger tag for the latest job: info while it runs, outcome afterwards.
fn status_tag(cx: &App, status: Status) -> (&'static str, Tone) {
    match status {
        Status::Queued => (text(cx, "Queued", "排队中"), Tone::Info),
        Status::Running => (text(cx, "Running", "运行中"), Tone::Info),
        Status::Success => (text(cx, "Succeeded", "已成功"), Tone::Success),
        Status::Failed => (text(cx, "Failed", "已失败"), Tone::Danger),
        Status::Cancelled => (text(cx, "Cancelled", "已取消"), Tone::Neutral),
        Status::TimedOut => (text(cx, "Timed out", "已超时"), Tone::Warning),
        Status::CleanupFailed => (text(cx, "Cleanup failed", "清理失败"), Tone::Danger),
        Status::Unavailable => (text(cx, "Unavailable", "不可用"), Tone::Warning),
    }
}

fn job_state(cx: &App, snapshot: &Snapshot) -> String {
    let status = status_tag(cx, snapshot.status).0;
    let period = text(cx, ".", "。");
    let cancel = if snapshot.cancel_requested {
        text(cx, "Cancel was requested.", "已请求取消。")
    } else {
        ""
    };
    let exit = match snapshot.exit_code {
        Some(code) if is_chinese(cx) => format!("退出码 {code}。"),
        Some(code) => format!("Exit code {code}."),
        None if is_chinese(cx) => "没有退出码。".to_string(),
        None => "No exit code.".to_string(),
    };
    let dropped = if is_chinese(cx) {
        format!("丢弃了 {} 行。", snapshot.dropped_lines)
    } else {
        format!("Dropped {} lines.", snapshot.dropped_lines)
    };
    let truncated = if snapshot.truncated {
        text(cx, "Output was truncated.", "输出被截断。")
    } else {
        text(cx, "Output was not truncated.", "输出没有被截断。")
    };
    format!("{status}{period} {cancel}{exit} {dropped} {truncated}")
}

fn availability_label(cx: &App, availability: Availability) -> &'static str {
    match availability {
        Availability::Runnable => text(cx, "Runnable", "可运行"),
        Availability::PreviewOnly => text(cx, "Preview only", "仅预览"),
        Availability::DedicatedPage => text(cx, "Needs its page", "需专用页面"),
    }
}

/// Why Run is disabled, when the reason is not the argument split itself.
fn blocked_reason(cx: &App, blocked: &Blocked) -> Option<String> {
    Some(match blocked {
        Blocked::UnknownCommand => text(
            cx,
            "This command is not in the local catalog.",
            "这条命令不在本地目录中。",
        )
        .to_string(),
        Blocked::NotRunnable(Availability::DedicatedPage) => text(
            cx,
            "Needs a dedicated page. Complete it on its page in this app.",
            "需专用页面，请在应用中对应的页面完成。",
        )
        .to_string(),
        Blocked::NotRunnable(_) => text(
            cx,
            "Preview only. Run this command in a terminal.",
            "仅预览，请在终端运行这条命令。",
        )
        .to_string(),
        Blocked::MissingRequired(names) => {
            let names = names.join(", ");
            if is_chinese(cx) {
                format!("缺少必填参数：{names}。")
            } else {
                format!("Missing required argument: {names}.")
            }
        }
        Blocked::Tokenize(_) | Blocked::Running => return None,
    })
}

fn hints_line(cx: &App, entry: &CommandInfo) -> String {
    let hints = model::hints(entry);
    if hints.is_empty() {
        return text(
            cx,
            "This command declares no arguments.",
            "这条命令没有声明参数。",
        )
        .to_string();
    }
    let required = text(cx, " (required)", "（必填）");
    let list = hints
        .iter()
        .map(|hint| {
            if hint.required {
                format!("{}{required}", hint.syntax)
            } else {
                hint.syntax.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" · ");
    format!("{}{list}", text(cx, "Available arguments: ", "可用参数："))
}

impl JobsView {
    fn environment_label(&self, cx: &App) -> String {
        let environment = match self.environment_name.as_deref() {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => match self.requests.environment() {
                ccr_app::runtime::EnvironmentId::Local(_) => text(cx, "Local", "本地").to_string(),
                ccr_app::runtime::EnvironmentId::Ssh(id) => format!("SSH · {id}"),
                _ => text(cx, "Unavailable", "不可用").to_string(),
            },
        };
        format!(
            "{} · {environment}",
            text(cx, "Execution environment", "执行环境")
        )
    }

    fn header(&self, cx: &App) -> AnyElement {
        let colors = cx.theme();
        let counts = self.catalog.counts();
        let runnable = if is_chinese(cx) {
            format!("{} 条可运行", counts.runnable)
        } else {
            format!("{} runnable", counts.runnable)
        };
        let title = text(cx, "Commands", "命令");
        let mut tags = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(
                badge::tag(
                    "command-environment",
                    self.environment_label(cx),
                    Tone::Neutral,
                    cx,
                )
                .test_support(),
            )
            .child(
                badge::tag("command-catalog-summary", runnable, Tone::Neutral, cx).test_support(),
            );
        if let Some(snapshot) = &self.snapshot {
            let (label, tone) = status_tag(cx, snapshot.status);
            tags = tags.child(badge::tag("command-job-status", label, tone, cx).test_support());
        }
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .id("command-heading")
                            .test_support()
                            .role(Role::Heading)
                            .aria_level(1)
                            .aria_label(title)
                            .text_size(theme::text_size(theme::TITLE_SIZE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(title),
                    )
                    .child(
                        div()
                            .text_size(theme::text_size(theme::META_SIZE))
                            .text_color(colors.muted_foreground)
                            .child(text(
                                cx,
                                "Run allowlisted CCR commands and watch their output.",
                                "运行白名单内的 CCR 命令并查看输出。",
                            )),
                    ),
            )
            .child(tags)
            .into_any_element()
    }

    fn filter_chip(
        &self,
        id: &'static str,
        filter: Filter,
        label: String,
        cx: &mut Context<Self>,
    ) -> Button {
        badge::filter_chip(
            Button::new(id)
                .label(label)
                .on_click(cx.listener(move |this, _, _, cx| this.set_filter(filter, cx))),
            self.filter == filter,
            false,
            cx,
        )
    }

    fn list_panel(&self, narrow: bool, cx: &mut Context<Self>) -> AnyElement {
        let counts = self.catalog.counts();
        let rows = self.visible(cx);
        let search = badge::search_field(
            Input::new(&self.search)
                .id("command-search")
                .aria_label(text(cx, "Search name or description", "搜索命令名或说明")),
            cx,
        );
        let chips = [
            (
                "command-filter-runnable",
                Filter::Runnable,
                text(cx, "Runnable", "可运行"),
                counts.runnable,
            ),
            (
                "command-filter-all",
                Filter::All,
                text(cx, "All", "全部"),
                counts.all,
            ),
            (
                "command-filter-preview",
                Filter::PreviewOnly,
                text(cx, "Preview only", "仅预览"),
                counts.preview_only,
            ),
            (
                "command-filter-dedicated",
                Filter::DedicatedPage,
                text(cx, "Needs its page", "需专用页面"),
                counts.dedicated_page,
            ),
        ];
        let mut filters = div()
            .id("command-filters")
            .test_support()
            .role(Role::Group)
            .aria_label(text(cx, "Catalog filter", "目录筛选"))
            .flex()
            .flex_wrap()
            .gap(px(theme::SELECTION_RADIUS + 2.));
        for (id, filter, label, count) in chips {
            filters = filters.child(self.filter_chip(id, filter, format!("{label} {count}"), cx));
        }
        let mut panel = reading_panel("command-list", cx)
            .track_focus(&self.records.region_focus)
            .tab_stop(false)
            .flex_1();
        if narrow && self.busy {
            let cmd = self
                .snapshot
                .as_ref()
                .map(|snapshot| snapshot.command.as_str())
                .unwrap_or(&self.selected);
            let banner_label = if is_chinese(cx) {
                format!("正在运行 ccr {cmd} · 查看账本")
            } else {
                format!("Running ccr {cmd} · View ledger")
            };
            let tone = theme::tone_colors(Tone::Info, cx.theme().mode);
            panel = panel.child(semantics::ghost_button(
                Button::new("command-running-banner")
                    .label(banner_label)
                    .w_full()
                    .text_color(tone.ink)
                    .bg(tone.fill)
                    .border_1()
                    .border_color(tone.edge)
                    .on_click(cx.listener(|this, _, window, cx| this.reveal_work(window, cx))),
                false,
                false,
                cx,
            ));
        }
        panel = panel.child(self.collection_bar(cx)).child(search);
        if self.collection == Collection::Catalog {
            let body = if rows.is_empty() {
                self.empty_list(counts.all == 0, cx)
            } else {
                self.listbox(rows.len(), cx)
            };
            panel = panel.child(filters).child(body);
        } else {
            panel = panel.children(self.records_panel(cx));
        }
        panel.into_any_element()
    }

    fn empty_list(&self, catalog_empty: bool, cx: &mut Context<Self>) -> AnyElement {
        if catalog_empty {
            return badge::empty_panel(
                "command-empty",
                IconName::Inbox,
                text(
                    cx,
                    "The local command catalog is empty.",
                    "本地命令目录是空的。",
                ),
                text(
                    cx,
                    "There is no command to run here.",
                    "这里没有可运行的命令。",
                ),
                None,
                cx,
            )
            .test_support()
            .into_any_element();
        }
        let clear = Button::new("command-clear-filters")
            .label(text(cx, "Clear search and filter", "清除搜索与筛选"))
            .ghost()
            .on_click(cx.listener(|this, _, window, cx| this.clear_search_and_filter(window, cx)))
            .into_any_element();
        badge::empty_panel(
            "command-empty",
            IconName::Search,
            text(cx, "No matching commands", "没有匹配的命令"),
            text(
                cx,
                "No command in this filter matches the search.",
                "当前筛选中没有命令匹配搜索。",
            ),
            Some(clear),
            cx,
        )
        .test_support()
        .into_any_element()
    }

    fn listbox(&self, total: usize, cx: &mut Context<Self>) -> AnyElement {
        let list = uniform_list(
            "command-rows",
            total,
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                let rows = this.visible(cx);
                range
                    .filter_map(|index| {
                        let name = *rows.get(index)?;
                        let entry = this.catalog.get(name)?.clone();
                        let highlighted = this.cursor.as_deref() == Some(name);
                        Some(catalog_row(index, total, &entry, highlighted, cx))
                    })
                    .collect::<Vec<_>>()
            }),
        )
        .track_scroll(&self.list_scroll)
        .flex_1()
        .min_h_0();
        div()
            .id("command-listbox")
            .test_support()
            .track_focus(&self.list_focus)
            .tab_stop(true)
            .role(Role::ListBox)
            .aria_label(text(cx, "Command catalog", "命令目录"))
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "down" => this.move_cursor(true, cx),
                    "up" => this.move_cursor(false, cx),
                    "enter" => this.load_cursor(window, cx),
                    _ => return,
                }
                cx.stop_propagation();
            }))
            .child(list)
            .into_any_element()
    }

    fn editor_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let entry = self.catalog.get(&self.selected);
        let draft = self.args.read(cx).value().to_string();
        let readiness = self.catalog.readiness(&self.selected, &draft, self.busy);
        let preview = model::preview(&self.selected, &draft);
        let (muted, danger, mono, mono_size, well) = {
            let colors = cx.theme();
            (
                colors.muted_foreground,
                colors.danger,
                colors.mono_font_family.clone(),
                colors.mono_font_size,
                colors.secondary,
            )
        };
        let run = if self.busy {
            semantics::ghost_button(
                Button::new("command-cancel")
                    .label(text(cx, "Cancel", "取消"))
                    .text_color(danger)
                    .w(px(RUN_WIDTH))
                    .on_click(cx.listener(|this, _, _, cx| this.cancel(cx))),
                false,
                false,
                cx,
            )
        } else {
            semantics::button(
                Button::new("command-start")
                    .label(text(cx, "Run", "运行"))
                    .icon(IconName::Play)
                    .primary()
                    .w(px(RUN_WIDTH))
                    .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                readiness.is_err(),
                false,
                cx,
            )
        };
        let actions = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(semantics::ghost_button(
                Button::new("command-help")
                    .icon(gpui_kit::assets::IconName::CircleQuestionMark)
                    .label(text(cx, "Command help", "命令帮助"))
                    .on_click(cx.listener(|this, _, _, cx| this.help(cx))),
                self.busy || entry.is_none(),
                false,
                cx,
            ))
            .child(self.favorite_toggle(cx))
            .child(run);
        let title = entry.map(|entry| entry.title).unwrap_or_default();
        let description = entry.map(|entry| entry.description).unwrap_or_default();
        let mut panel = reading_panel("command-editor", cx)
            .flex_shrink_0()
            .child(eyebrow(text(cx, "Command editor", "命令编排"), cx))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div()
                            .id("command-editor-name")
                            .test_support()
                            .aria_label(self.selected.clone())
                            .min_w_0()
                            .font_family(mono.clone())
                            .text_size(theme::text_size(theme::TITLE_SIZE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.selected.clone()),
                    )
                    .child(actions),
            )
            .when(!title.is_empty(), |panel| {
                panel.child(
                    div()
                        .id("command-editor-title")
                        .test_support()
                        .aria_label(title)
                        .font_weight(FontWeight::MEDIUM)
                        .child(title),
                )
            })
            .when(!description.is_empty(), |panel| {
                panel.child(
                    div()
                        .id("command-editor-description")
                        .test_support()
                        .aria_label(description)
                        .text_color(muted)
                        .child(description),
                )
            })
            .child(
                div()
                    .id("command-preview")
                    .test_support()
                    .aria_label(preview.clone())
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .px(px(12.))
                    .py(px(theme::CONTROL_GAP))
                    .rounded(px(theme::INPUT_RADIUS))
                    .bg(well)
                    .font_family(mono)
                    .text_size(mono_size)
                    .child(div().flex_shrink_0().text_color(muted).child("➜"))
                    .child(div().min_w_0().child(preview)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(theme::SELECTION_RADIUS))
                    .child(
                        div()
                            .text_size(theme::text_size(theme::META_SIZE))
                            .font_weight(FontWeight::MEDIUM)
                            .child(text(cx, "Arguments", "参数")),
                    )
                    .child(
                        div()
                            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                if event.keystroke.key == "enter"
                                    && event.keystroke.modifiers.control
                                {
                                    cx.stop_propagation();
                                    this.run_if_ready(cx);
                                }
                            }))
                            .child(Input::new(&self.args).id("command-args").aria_label(text(
                                cx,
                                "Arguments",
                                "参数",
                            ))),
                    ),
            );
        if model::tokenize(&draft).is_err() {
            let line = text(
                cx,
                "A double quote is not closed. Close it before running.",
                "双引号未闭合，闭合后才能运行。",
            );
            panel = panel.child(
                div()
                    .id("command-args-error")
                    .test_support()
                    .aria_label(line)
                    .text_size(theme::text_size(theme::META_SIZE))
                    .text_color(danger)
                    .child(line),
            );
        }
        if let Some(entry) = entry {
            let line = hints_line(cx, entry);
            panel = panel.child(
                div()
                    .id("command-args-hints")
                    .test_support()
                    .aria_label(line.clone())
                    .text_size(theme::text_size(theme::META_SIZE))
                    .text_color(muted)
                    .child(line),
            );
        }
        if let Some(reason) = readiness
            .err()
            .and_then(|blocked| blocked_reason(cx, &blocked))
        {
            let ink = theme::tone_colors(Tone::Neutral, cx.theme().mode).ink;
            panel = panel.child(
                div()
                    .id("command-run-blocked")
                    .test_support()
                    .aria_label(reason.clone())
                    .flex()
                    .items_center()
                    .gap(px(theme::SELECTION_RADIUS + 2.))
                    .text_color(ink)
                    .child(
                        Icon::new(gpui_kit::assets::IconName::Lock)
                            .size(px(theme::NAV_ICON_SIZE))
                            .text_color(ink),
                    )
                    .child(reason),
            );
        }
        if let Some(message) = self.message {
            let line = service_error(cx, message);
            panel = panel.child(
                div()
                    .id("command-message")
                    .test_support()
                    .aria_label(line)
                    .text_color(danger)
                    .child(line),
            );
        }
        panel.into_any_element()
    }

    fn ledger_panel(&self, narrow: bool, cx: &mut Context<Self>) -> AnyElement {
        let zh = is_chinese(cx);
        let mode = cx.theme().mode;
        let (muted, danger, code_bg, border, hairline) = {
            let colors = cx.theme();
            (
                colors.muted_foreground,
                colors.danger,
                colors.secondary,
                colors.border,
                theme::hairline(mode),
            )
        };
        let info_ink = theme::tone_colors(Tone::Info, mode).ink;
        let warning_ink = theme::tone_colors(Tone::Warning, mode).ink;

        let count = self.snapshot.as_ref().map_or(0, ledger::line_count);
        let subtitle = self
            .snapshot
            .as_ref()
            .map(|snapshot| ledger::subtitle(snapshot, zh))
            .unwrap_or_else(|| {
                text(
                    cx,
                    "Select a command and run it to see job status and output here",
                    "选择命令并运行后，任务状态与输出显示在这里",
                )
                .to_string()
            });
        let clear_disabled = self.busy
            || (self.snapshot.is_none() && self.job_id.is_none() && self.observe_error.is_none());

        let header = div()
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .items_start()
            .justify_between()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(eyebrow(text(cx, "Run ledger", "运行账本"), cx))
                    .child(
                        div()
                            .text_size(theme::text_size(theme::SECTION_TITLE_SIZE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(text(cx, "Output", "输出")),
                    )
                    .child(
                        div()
                            .id("command-ledger-subtitle")
                            .test_support()
                            .aria_label(subtitle.clone())
                            .truncate()
                            .text_size(theme::text_size(theme::META_SIZE))
                            .text_color(muted)
                            .child(subtitle),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::ghost_button(
                        Button::new("command-ledger-copy")
                            .icon(IconName::Copy)
                            .label(text(cx, "Copy output", "复制输出"))
                            .on_click(cx.listener(|this, _, _, cx| this.copy_output(cx))),
                        count == 0,
                        false,
                        cx,
                    ))
                    .child(semantics::ghost_button(
                        Button::new("command-ledger-clear")
                            .label(text(cx, "Clear", "清空"))
                            .on_click(cx.listener(|this, _, _, cx| this.clear_ledger(cx))),
                        clear_disabled,
                        false,
                        cx,
                    )),
            );

        let mut panel = reading_panel("command-ledger", cx)
            .when(!narrow, |panel| panel.flex_1())
            .when(narrow, |panel| panel.flex_shrink_0().min_h(px(260.)))
            .child(header);

        if let Some(snapshot) = self.snapshot.as_ref() {
            let (tag_label, tag_tone) = status_tag(cx, snapshot.status);
            let duration_value = ledger::duration(snapshot.duration_ms, zh);
            let exit_value = ledger::exit_code(snapshot.exit_code);
            let truncation = ledger::truncation(snapshot);
            let lines_value = match truncation {
                Some(Truncation::Dropped(n)) if zh => format!("{count} · 已丢弃 {n} 行"),
                Some(Truncation::Dropped(n)) => format!("{count} · {n} dropped"),
                Some(Truncation::ClippedOnly) if zh => format!("{count} · 已截断"),
                Some(Truncation::ClippedOnly) => format!("{count} · truncated"),
                None => count.to_string(),
            };
            let status_title = text(cx, "Status", "状态");
            let duration_title = text(cx, "Duration", "耗时");
            let exit_title = text(cx, "Exit code", "退出码");
            let lines_title = text(cx, "Lines", "输出行数");

            let readout_cell = |id: &'static str,
                                title: &'static str,
                                aria: String,
                                divider: bool,
                                value: AnyElement| {
                div()
                    .id(id)
                    .test_support()
                    .aria_label(aria)
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .when(divider, |cell| {
                        cell.pl(px(theme::CONTROL_GAP + 4.))
                            .border_l_1()
                            .border_color(hairline)
                    })
                    .child(
                        div()
                            .text_size(theme::text_size(theme::META_SIZE))
                            .text_color(muted)
                            .child(title),
                    )
                    .child(value)
            };

            let readouts = div()
                .id("command-ledger-readouts")
                .test_support()
                .flex()
                .items_stretch()
                .gap(px(theme::CONTROL_GAP))
                .py(px(theme::CONTROL_GAP))
                .border_t_1()
                .border_b_1()
                .border_color(hairline)
                .child(readout_cell(
                    "command-ledger-readout-status",
                    status_title,
                    format!("{status_title}: {tag_label}"),
                    false,
                    div()
                        .flex()
                        .items_center()
                        .child(badge::tag(
                            "command-ledger-status-tag",
                            tag_label,
                            tag_tone,
                            cx,
                        ))
                        .into_any_element(),
                ))
                .child(readout_cell(
                    "command-ledger-readout-duration",
                    duration_title,
                    format!("{duration_title}: {duration_value}"),
                    true,
                    div()
                        .font_features(theme::tabular_figures())
                        .font_weight(FontWeight::MEDIUM)
                        .child(duration_value)
                        .into_any_element(),
                ))
                .child(readout_cell(
                    "command-ledger-readout-exit",
                    exit_title,
                    format!("{exit_title}: {exit_value}"),
                    true,
                    div()
                        .font_features(theme::tabular_figures())
                        .font_weight(FontWeight::MEDIUM)
                        .child(exit_value)
                        .into_any_element(),
                ))
                .child(readout_cell(
                    "command-ledger-readout-lines",
                    lines_title,
                    format!("{lines_title}: {lines_value}"),
                    true,
                    div()
                        .font_features(theme::tabular_figures())
                        .font_weight(FontWeight::MEDIUM)
                        .truncate()
                        .child(lines_value)
                        .into_any_element(),
                ));
            panel = panel.child(readouts);
        }

        // Polite status region: visible info line while running, screen-reader
        // announcement on terminal states (without duplicating readouts visually).
        let (status_announcement, show_running_line) = match self.snapshot.as_ref() {
            Some(snapshot) if !snapshot.status.is_terminal() => {
                (text(cx, "Running…", "正在运行…").to_string(), true)
            }
            Some(snapshot) => (job_state(cx, snapshot), false),
            None => (String::new(), false),
        };
        let status_live = div()
            .id("command-ledger-status")
            .test_support()
            .role(Role::Status)
            .aria_label(status_announcement.clone())
            .a11y_synthetic_children(|tree| {
                tree.parent_node()
                    .set_live(gpui_kit::accesskit::Live::Polite)
            })
            .when(show_running_line, |row| {
                row.text_size(theme::text_size(theme::META_SIZE))
                    .text_color(info_ink)
                    .child(status_announcement)
            })
            .when(!show_running_line, |row| row.h(px(0.)).overflow_hidden());
        panel = panel.child(status_live);

        if let Some(error) = self.observe_error {
            let line = service_error(cx, error);
            panel = panel.child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div()
                            .id("command-ledger-observe-error")
                            .test_support()
                            .aria_label(line)
                            .flex_1()
                            .min_w_0()
                            .text_size(theme::text_size(theme::META_SIZE))
                            .text_color(danger)
                            .child(line),
                    )
                    .child(semantics::ghost_button(
                        Button::new("command-ledger-retry")
                            .icon(IconName::RefreshCw)
                            .label(text(cx, "Retry", "重试"))
                            .on_click(cx.listener(|this, _, _, cx| this.retry_observe(cx))),
                        false,
                        false,
                        cx,
                    )),
            );
        }

        if let Some(error) = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.error.clone())
        {
            let line = localized_line(cx, &error);
            panel = panel.child(
                div()
                    .id("command-run-error")
                    .test_support()
                    .aria_label(line.clone())
                    .text_size(theme::text_size(theme::META_SIZE))
                    .text_color(danger)
                    .child(line),
            );
        }

        let ssh_unavailable = matches!(
            self.requests.environment(),
            ccr_app::runtime::EnvironmentId::Ssh(_)
        ) && self
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.status == Status::Unavailable);
        if ssh_unavailable {
            panel = panel.child(semantics::button(
                Button::new("jobs-install-remote-cli")
                    .label(text(
                        cx,
                        "Install CCR on this SSH host",
                        "在此 SSH 主机安装 CCR",
                    ))
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(InstallRequested))),
                false,
                false,
                cx,
            ));
        }

        if let Some(truncation) = self.snapshot.as_ref().and_then(ledger::truncation) {
            let notice = match truncation {
                Truncation::Dropped(dropped) if zh => {
                    format!("输出已截断，已丢弃 {dropped} 行。")
                }
                Truncation::Dropped(dropped) => {
                    format!("Output was truncated; {dropped} lines dropped.")
                }
                Truncation::ClippedOnly if zh => "单行输出超限，已截断。".to_string(),
                Truncation::ClippedOnly => "Long output lines were truncated.".to_string(),
            };
            panel = panel.child(
                div()
                    .id("command-ledger-truncated")
                    .test_support()
                    .aria_label(notice.clone())
                    .text_size(theme::text_size(theme::META_SIZE))
                    .text_color(warning_ink)
                    .child(notice),
            );
        }

        if self.snapshot.is_some() {
            let logs = uniform_list(
                "command-job-logs",
                count,
                cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                    let Some(snapshot) = this.snapshot.as_ref() else {
                        return Vec::new();
                    };
                    let (mono, mono_size, muted, fg, err_ink) = {
                        let colors = cx.theme();
                        (
                            colors.mono_font_family.clone(),
                            colors.mono_font_size,
                            colors.muted_foreground,
                            colors.foreground,
                            theme::tone_colors(Tone::Warning, colors.mode).ink,
                        )
                    };
                    let row_h = ledger::row_height(f32::from(mono_size));
                    let job_id = this.job_id.as_deref().unwrap_or("none");
                    range
                        .filter_map(|index| {
                            let (channel, line) = ledger::line(snapshot, index)?;
                            let gutter_color = if channel == Channel::Stderr {
                                err_ink
                            } else {
                                muted
                            };
                            let aria = format!("[{}] {line}", ledger::channel_name(channel));
                            let text = line.into_owned();
                            let tip = SharedString::from(text.clone());
                            Some(
                                div()
                                    .id(SharedString::from(format!("command-log-{job_id}-{index}")))
                                    .test_support()
                                    .aria_label(aria)
                                    .h(px(row_h))
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .gap(px(theme::CONTROL_GAP))
                                    .px(px(theme::CONTROL_GAP))
                                    .font_family(mono.clone())
                                    .text_size(mono_size)
                                    .child(
                                        div()
                                            .w(px(ledger::GUTTER_WIDTH))
                                            .flex_shrink_0()
                                            .text_color(gutter_color)
                                            .child(ledger::gutter(channel)),
                                    )
                                    .child(
                                        div()
                                            .id(SharedString::from(format!(
                                                "command-log-text-{job_id}-{index}"
                                            )))
                                            .flex_1()
                                            .min_w_0()
                                            .truncate()
                                            .text_color(fg)
                                            .tooltip(move |window, cx| {
                                                Tooltip::new(tip.clone()).build(window, cx)
                                            })
                                            .child(text),
                                    ),
                            )
                        })
                        .collect::<Vec<_>>()
                }),
            )
            .h_full()
            .track_scroll(&self.log_scroll);

            panel = panel.child(
                div()
                    .id("command-ledger-log-surface")
                    .flex_1()
                    .min_h(px(160.))
                    .overflow_hidden()
                    .rounded(px(theme::INPUT_RADIUS))
                    .bg(code_bg)
                    .border_1()
                    .border_color(border)
                    .py(px(4.))
                    .child(logs),
            );
        } else if self.observe_error.is_none() {
            panel = panel.child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .child(
                        badge::empty_panel(
                            "command-ledger-empty",
                            IconName::SquareTerminal,
                            text(cx, "Ready to run a command", "准备运行命令"),
                            text(
                                cx,
                                "Select a command and run it to see job status and output here.",
                                "选择命令并运行后，任务状态与输出显示在这里。",
                            ),
                            None,
                            cx,
                        )
                        .test_support(),
                    ),
            );
        }

        panel.into_any_element()
    }
}

// 56px 两行：mono 命令名与异常标签 / title · description（单行省略 + tooltip）。
fn catalog_row(
    index: usize,
    total: usize,
    entry: &CommandInfo,
    highlighted: bool,
    cx: &mut Context<JobsView>,
) -> AnyElement {
    let name = entry.name;
    let availability = model::availability(entry);
    let state = availability_label(cx, availability);
    let label = format!("{name} · {} · {state}", entry.title);
    let detail = format!("{} · {}", entry.title, entry.description);
    let (selected_bg, hover_bg, muted, mono) = {
        let colors = cx.theme();
        (
            colors.secondary,
            colors.muted,
            colors.muted_foreground,
            colors.mono_font_family.clone(),
        )
    };
    let mut first = div()
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .font_family(mono)
                .text_size(theme::text_size(theme::BODY_SIZE))
                .font_weight(FontWeight::MEDIUM)
                .child(name),
        );
    if availability != Availability::Runnable {
        first = first.child(badge::tag(
            SharedString::from(format!("command-row-state:{name}")),
            state,
            Tone::Neutral,
            cx,
        ));
    }
    if entry.requires_confirmation {
        first = first.child(badge::tag(
            SharedString::from(format!("command-row-confirm:{name}")),
            text(cx, "Needs confirmation", "需确认"),
            Tone::Danger,
            cx,
        ));
    }
    let tip = SharedString::from(detail.clone());
    let weak = cx.weak_entity();
    div()
        .id(SharedString::from(format!("command-row:{name}")))
        .test_support()
        .role(Role::ListBoxOption)
        .aria_label(label)
        .aria_selected(highlighted)
        .aria_position_in_set(index + 1)
        .aria_size_of_set(total)
        .on_a11y_action(gpui_kit::AccessibleAction::Click, move |_, window, cx| {
            let _ = weak.update(cx, |this, cx| this.load(name, window, cx));
        })
        .h(theme::data_row_height(theme::COMMAND_ROW_HEIGHT, cx))
        .w_full()
        .flex()
        .flex_col()
        .justify_center()
        .gap(px(2.))
        .px(px(theme::CONTROL_GAP))
        .rounded(px(theme::SELECTION_RADIUS))
        .bg(if highlighted {
            selected_bg
        } else {
            gpui_kit::transparent_black()
        })
        .cursor_pointer()
        .hover(move |style| style.bg(hover_bg))
        .on_click(cx.listener(move |this, _, window, cx| this.load(name, window, cx)))
        .child(first)
        .child(
            div()
                .id(SharedString::from(format!("command-row-detail:{name}")))
                .truncate()
                .text_size(theme::text_size(theme::META_SIZE))
                .text_color(muted)
                .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                .child(detail),
        )
        .into_any_element()
}

impl Render for JobsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport = window.viewport_size().width;
        let split = viewport >= px(theme::SPLIT_WIDTH);
        let header = self.header(cx);
        let warning = self.requests.is_fixture().then(|| {
            div()
                .id("jobs-fixture-warning")
                .test_support()
                .aria_label(
                    "Synthetic owned-process fixture. No CCR command or real configuration is accessed.",
                )
                .text_color(cx.theme().warning)
                .child(text(
                    cx,
                    "Synthetic owned-process fixture. No CCR command or real configuration is accessed.",
                    "合成进程生命周期验证，不执行真实 CCR 命令",
                ))
        });
        let workspace = if split {
            let list = self.list_panel(false, cx);
            let editor = self.editor_panel(cx);
            let ledger = self.ledger_panel(false, cx);
            div()
                .id("command-workspace")
                .test_support()
                .flex()
                .flex_1()
                .min_h_0()
                .gap(px(theme::SECTION_GAP))
                .child(
                    div()
                        .flex_shrink_0()
                        .w(px(list_width(viewport)))
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(list),
                )
                .child(
                    div()
                        .id("command-work-column")
                        .test_support()
                        .track_focus(&self.work_focus)
                        .tab_stop(false)
                        .flex_1()
                        .min_w_0()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .gap(px(theme::SECTION_GAP))
                        .child(editor)
                        .child(ledger),
                )
        } else if self.narrow_detail {
            let editor = self.editor_panel(cx);
            let ledger = self.ledger_panel(true, cx);
            let back = semantics::ghost_button(
                Button::new("command-back-to-list")
                    .icon(IconName::ArrowLeft)
                    .label(text(cx, "Back to list", "返回列表"))
                    .on_click(cx.listener(|this, _, window, cx| this.back_to_list(window, cx))),
                false,
                false,
                cx,
            );
            div()
                .id("command-workspace")
                .test_support()
                .flex()
                .flex_1()
                .min_h_0()
                .child(
                    div()
                        .id("command-work-column")
                        .test_support()
                        .track_focus(&self.work_focus)
                        .tab_stop(false)
                        .flex_1()
                        .min_w_0()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(theme::CONTROL_GAP))
                                .pb(px(theme::CONTROL_GAP))
                                .child(div().flex_shrink_0().flex().items_center().child(back))
                                .child(editor)
                                .child(ledger),
                        ),
                )
        } else {
            let list = self.list_panel(true, cx);
            div()
                .id("command-workspace")
                .test_support()
                .flex()
                .flex_1()
                .min_h_0()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(list),
                )
        };
        div()
            .id("native-command-workbench")
            .test_support()
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .p(px(theme::content_padding(viewport)))
            .gap(px(theme::SECTION_GAP))
            .text_color(cx.theme().foreground)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "f6"
                    && this.focus_region(event.keystroke.modifiers.shift, window, cx)
                {
                    cx.stop_propagation();
                }
            }))
            .child(header)
            .children(warning)
            .child(workspace)
    }
}
