//! Claude extensions: read-only source list, same-name facts and scoped legacy exports.
use super::{Decision, ResourcesView};
use crate::{
    locales::{service_error, text},
    runtime::resources::ResourceCommand,
    shell::semantics,
};
use ccr_app::resources::claude_extensions::{self as extensions, SameNameRule, Source, State};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, FocusTrapElement,
    button::{Button, ButtonVariants},
    input::Textarea,
};
use gpui_kit::{
    AnyElement, App, Context, Div, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px,
};

fn source_label(cx: &App, label: &str) -> &'static str {
    match label {
        extensions::SUBAGENTS => text(cx, "Subagents · user", "子代理 · 用户级"),
        extensions::SKILLS => text(cx, "Skills · user", "技能 · 用户级"),
        extensions::LEGACY_COMMANDS => {
            text(cx, "Legacy slash commands · user", "旧斜杠命令 · 用户级")
        }
        extensions::PLUGINS => text(cx, "Plugins · installation registry", "插件 · 安装登记清单"),
        extensions::PROJECT_SCOPE => text(
            cx,
            "Project scope (no project selected)",
            "项目范围（未选择项目）",
        ),
        extensions::PLUGIN_CONTENTS => text(
            cx,
            "Plugin-bundled agents, skills and commands · install paths not followed",
            "插件自带的子代理、技能与命令 · 不跟随安装路径",
        ),
        _ => text(
            cx,
            "Legacy CCR entries — Claude does not read these arrays",
            "旧版 CCR 条目 — Claude 不读取这些数组",
        ),
    }
}

fn state_label(cx: &App, state: &State) -> &'static str {
    match state {
        State::Discovered => text(
            cx,
            "Discovered; client loading unverified",
            "已发现；客户端加载未验证",
        ),
        State::Empty => text(cx, "No entries discovered", "未发现条目"),
        State::Missing => text(cx, "Source not found", "来源未找到"),
        State::NotConnected => text(cx, "Not connected", "未接入"),
        State::Invalid => text(cx, "Invalid manifest", "清单格式无效"),
        State::Limit => text(cx, "Source exceeds scan limit", "来源超出扫描限额"),
        State::Unreadable => text(cx, "Source unreadable", "来源不可读"),
        State::Unsafe => text(cx, "Unsafe source path refused", "来源路径不安全，已拒绝"),
    }
}

/// 只陈述本地发现的同名事实；优先规则仅来自研究票已写明的文档，其余写「未验证」。
fn same_name_line(cx: &App, same: &extensions::SameName) -> String {
    let peers = same
        .peers
        .iter()
        .map(|peer| format!("{} · {}", source_label(cx, peer.source), peer.location))
        .collect::<Vec<_>>()
        .join("; ");
    let rule = match same.rule {
        SameNameRule::SkillOverCommand => text(
            cx,
            "Documented: a skill takes precedence over a same-name command; client loading unverified.",
            "文档写明：同名时技能优先于旧命令；客户端加载未验证。",
        ),
        SameNameRule::OneSubagentLoaded => text(
            cx,
            "Documented: only one same-name subagent is loaded; which one is unverified.",
            "文档写明：同名子代理只加载其中一个；具体哪个未验证。",
        ),
        SameNameRule::Unverified => text(
            cx,
            "No documented precedence; client effect unverified.",
            "未见文档优先规则；客户端生效未验证。",
        ),
    };
    format!(
        "{} “{}” {} {peers}. {rule}",
        text(cx, "Same name", "同名"),
        same.name,
        text(cx, "also in:", "也见于："),
    )
}

impl ResourcesView {
    pub(super) fn legacy_confirm_text(cx: &App) -> &'static str {
        text(
            cx,
            "Show the full legacy CCR entries, including credentials? The copy stays in this view temporarily and nothing is saved until you choose a file. Do not include it in screenshots or logs.",
            "显示含凭据的完整旧版 CCR 条目？副本只在本页临时显示，选择文件后才保存；请勿截图或记录。",
        )
    }

    /// 确认框挂载后的下一帧把焦点交给取消；过期 generation 的回调不生效。
    pub(super) fn schedule_decision_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !std::mem::take(&mut self.decision_entry_pending) {
            return;
        }
        let generation = self.decision_generation;
        let view = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                if this.decision.is_some() && this.decision_generation == generation {
                    window.focus(&this.confirmation_focus, cx);
                    window.focus_next(cx);
                }
            });
        });
    }

    pub(super) fn accept_legacy_export(&mut self, content: &str, full: bool) {
        self.legacy_pending = Some(content.to_owned());
        self.legacy_full = Some(full);
        self.message = Some(if full {
            "Full copy shown below temporarily; not saved until you choose a file. / 完整副本仅在下方临时显示，选择文件后才保存。"
        } else {
            "Redacted copy shown below temporarily; not saved until you choose a file. / 脱敏副本仅在下方临时显示，选择文件后才保存。"
        });
    }

    pub(super) fn clear_legacy_output(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.legacy_pending = None;
        self.legacy_full = None;
        self.legacy_output
            .update(cx, |state, cx| state.set_value("", window, cx));
    }

    fn close_legacy_output(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.decision.is_some() {
            return;
        }
        // 关闭即作废尚未返回的保存对话框。
        self.generation += 1;
        self.task = None;
        self.message = None;
        self.clear_legacy_output(window, cx);
        cx.notify();
    }

    /// 文件只经显式保存入口落盘；对话框结果按 generation 与仍在显示的输出校验。
    fn save_legacy_output(&mut self, cx: &mut Context<Self>) {
        if self.busy || self.decision.is_some() || self.legacy_full.is_none() {
            return;
        }
        let Some(directory) = self.services.profile_export_directory() else {
            return;
        };
        let dialog = cx.prompt_for_new_path(&directory, Some("ccr-claude-legacy-entries.json"));
        self.generation += 1;
        let generation = self.generation;
        self.task = Some(cx.spawn(async move |weak, cx| {
            let selected = dialog.await;
            let _ = weak.update(cx, |this, cx| {
                if this.generation != generation || this.legacy_full.is_none() {
                    return;
                }
                match selected {
                    Ok(Ok(Some(destination))) => {
                        let content = this.legacy_output.read(cx).value().to_string();
                        this.request(
                            ResourceCommand::SaveClaudeLegacyExport {
                                content,
                                destination,
                            },
                            cx,
                        );
                    }
                    Ok(Ok(None)) => {}
                    _ => {
                        this.message = Some(
                            "Save dialog unavailable; temporary output retained. / 保存对话框不可用，临时输出保留。",
                        );
                        cx.notify();
                    }
                }
            });
        }));
    }

    // 原生 1 MiB 线程栈：每段在独立且不内联的帧里构建，返回 AnyElement 后即释放 builder 临时值。
    #[inline(never)]
    fn source_card(
        &self,
        index: usize,
        source: &Source,
        blocked: bool,
        cx: &Context<Self>,
    ) -> AnyElement {
        let heading = format!(
            "{}: {}",
            source_label(cx, source.label),
            state_label(cx, &source.state)
        );
        let mut card = div()
            .id(format!("claude-extension-source-{index}"))
            .test_support()
            .aria_label(heading.clone())
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(8.))
            .p(px(16.))
            .bg(cx.theme().background)
            .rounded(px(12.))
            .child(heading);
        for (row, entry) in source.entries.iter().enumerate() {
            card = card.child(div().text_size(px(12.)).child(entry.text.clone()));
            if let Some(same) = &entry.same_name {
                card = card.child(same_name_row(index, row, same, cx));
            }
        }
        if source.label == extensions::LEGACY_ENTRIES && source.state == State::Discovered {
            card = card.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .child(self.legacy_export_button(blocked, cx))
                    .child(self.legacy_full_button(blocked, cx)),
            );
        }
        card.into_any_element()
    }

    #[inline(never)]
    fn legacy_export_button(&self, blocked: bool, cx: &Context<Self>) -> AnyElement {
        semantics::ghost_button(
            Button::new("claude-extension-legacy-export")
                .label(text(cx, "Redacted export", "脱敏导出"))
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.request(ResourceCommand::ExportClaudeLegacy { full: false }, cx)
                })),
            blocked,
            false,
            cx,
        )
        .into_any_element()
    }

    #[inline(never)]
    fn legacy_full_button(&self, blocked: bool, cx: &Context<Self>) -> AnyElement {
        semantics::ghost_button(
            Button::new("claude-extension-legacy-export-full")
                .label(text(
                    cx,
                    "Export full copy (with credentials)",
                    "导出完整副本（含凭据）",
                ))
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.decide(Decision::LegacyFullExport, window, cx)
                })),
            blocked,
            false,
            cx,
        )
        .into_any_element()
    }

    #[inline(never)]
    fn legacy_cancel_button(&self, cx: &Context<Self>) -> AnyElement {
        semantics::button(
            Button::new("claude-extension-legacy-cancel")
                .label(text(cx, "Cancel", "取消"))
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| this.dismiss(window, cx))),
            false,
            false,
            cx,
        )
        .into_any_element()
    }

    #[inline(never)]
    fn legacy_confirm_button(&self, cx: &Context<Self>) -> AnyElement {
        semantics::button(
            Button::new("claude-extension-legacy-confirm")
                .label(text(cx, "Confirm", "确认"))
                .on_click(cx.listener(|this, _, window, cx| this.confirm(window, cx))),
            false,
            false,
            cx,
        )
        .into_any_element()
    }

    #[inline(never)]
    fn legacy_confirmation(&self, cx: &Context<Self>) -> AnyElement {
        div()
            .id("claude-extension-legacy-confirmation")
            .test_support()
            .role(gpui_kit::Role::AlertDialog)
            .aria_label(text(
                cx,
                "Confirm full legacy export",
                "确认导出完整旧版条目",
            ))
            .flex_shrink_0()
            .child(Self::legacy_confirm_text(cx))
            .child(
                div()
                    .tab_group()
                    .tab_stop(false)
                    .flex()
                    .gap(px(8.))
                    .child(self.legacy_cancel_button(cx))
                    .child(self.legacy_confirm_button(cx))
                    .focus_trap("claude-extension-legacy-trap", &self.confirmation_focus),
            )
            .into_any_element()
    }

    #[inline(never)]
    fn legacy_save_button(&self, blocked: bool, cx: &Context<Self>) -> AnyElement {
        semantics::button(
            Button::new("claude-extension-legacy-save-file")
                .label(text(
                    cx,
                    "Save this output to a new file…",
                    "将当前输出保存到新文件…",
                ))
                .primary()
                .on_click(cx.listener(|this, _, _, cx| this.save_legacy_output(cx))),
            blocked,
            self.busy,
            cx,
        )
        .into_any_element()
    }

    #[inline(never)]
    fn legacy_close_button(&self, blocked: bool, cx: &Context<Self>) -> AnyElement {
        semantics::ghost_button(
            Button::new("claude-extension-legacy-close")
                .label(text(cx, "Close and clear", "关闭并清除"))
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| this.close_legacy_output(window, cx))),
            blocked,
            false,
            cx,
        )
        .into_any_element()
    }

    #[inline(never)]
    fn legacy_output_panel(&self, full: bool, blocked: bool, cx: &Context<Self>) -> AnyElement {
        let caption = if full {
            text(
                cx,
                "Full copy with credentials · temporary, not saved",
                "含凭据完整副本 · 临时显示，尚未保存",
            )
        } else {
            text(
                cx,
                "Redacted copy · temporary, not saved",
                "脱敏副本 · 临时显示，尚未保存",
            )
        };
        div()
            .id("claude-extension-legacy-output")
            .test_support()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .child(self.legacy_save_button(blocked, cx))
                    .child(self.legacy_close_button(blocked, cx)),
            )
            .child(div().text_color(cx.theme().muted_foreground).child(caption))
            .child(
                Textarea::new(&self.legacy_output)
                    .accessibility_id("claude-extension-legacy-content")
                    .aria_label(text(cx, "Temporary legacy export", "临时旧版条目导出"))
                    .h(px(240.))
                    .readonly(true),
            )
            .into_any_element()
    }

    /// 扩展页的全部内容；外层滚动容器由调用方提供。
    #[inline(never)]
    pub(super) fn render_extensions(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let blocked = self.busy || self.decision.is_some();
        let confirming = matches!(self.decision, Some(Decision::LegacyFullExport));
        if confirming {
            self.schedule_decision_focus(window, cx);
        }
        let mut children = vec![extensions_intro(cx)];
        for (index, source) in self.extension_sources.iter().enumerate() {
            children.push(self.source_card(index, source, blocked, cx));
            // 确认与临时输出紧跟旧版条目卡片，靠近触发它们的动作。
            if source.label != extensions::LEGACY_ENTRIES {
                continue;
            }
            if confirming {
                children.push(self.legacy_confirmation(cx));
            }
            if let Some(full) = self.legacy_full {
                children.push(self.legacy_output_panel(full, blocked, cx));
            }
        }
        if let Some(message) = self.message {
            children.push(extensions_status(cx, message));
        }
        div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(crate::theme::SECTION_GAP))
            .children(children)
    }
}

#[inline(never)]
fn extensions_intro(cx: &App) -> AnyElement {
    div()
        .text_size(px(12.))
        .child(text(
            cx,
            "Read-only user metadata. Discovery does not verify client loading; no installation, network or execution. Same-name entries stay listed under each source. Legacy CCR arrays are preserved and are not read by Claude.",
            "用户级元数据只读清单。发现不证明客户端已加载；不安装、不联网、不执行。同名条目仍在各自来源下列出。旧 CCR 数组保留，Claude 不读取。",
        ))
        .into_any_element()
}

#[inline(never)]
fn extensions_status(cx: &App, message: &'static str) -> AnyElement {
    div()
        .id("claude-extension-status")
        .test_support()
        .role(gpui_kit::Role::Status)
        .child(service_error(cx, message))
        .into_any_element()
}

#[inline(never)]
fn same_name_row(index: usize, row: usize, same: &extensions::SameName, cx: &App) -> AnyElement {
    let line = same_name_line(cx, same);
    div()
        .id(format!("claude-extension-same-name-{index}-{row}"))
        .test_support()
        .aria_label(line.clone())
        .text_size(px(12.))
        .text_color(cx.theme().muted_foreground)
        .child(line)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locales::Language;

    // PRD 8.3「扩展」：缺失、空、不可读、不安全、超限、坏清单、未接入、已发现但未验证加载必须各自表达，
    // 任何两种状态都不能共用同一句话（例如把不安全路径并入“不可读”）。
    #[gpui_kit::test]
    fn every_extension_source_state_reads_differently_in_both_languages(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        let states = [
            State::Discovered,
            State::Empty,
            State::Missing,
            State::Unreadable,
            State::Invalid,
            State::Unsafe,
            State::Limit,
            State::NotConnected,
        ];
        for language in [Language::English, Language::Chinese] {
            cx.update(|cx| {
                cx.set_global(language);
                let labels: Vec<_> = states.iter().map(|state| state_label(cx, state)).collect();
                let unique: std::collections::BTreeSet<_> = labels.iter().collect();
                assert_eq!(unique.len(), states.len(), "{labels:?}");
                // 只有真实发现才陈述“客户端加载未验证”；其余状态不能冒充已发现。
                let unverified = if language == Language::English {
                    "client loading unverified"
                } else {
                    "客户端加载未验证"
                };
                for (state, label) in states.iter().zip(&labels) {
                    assert_eq!(
                        label.contains(unverified),
                        *state == State::Discovered,
                        "{state:?}: {label}"
                    );
                }
            });
        }
    }
}
