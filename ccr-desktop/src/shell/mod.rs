//! Native capability shell: only connected Local surfaces are actionable.
mod command_state;
pub(crate) mod semantics;
use crate::{
    app::ConfigurationApp,
    locales::{Language, text},
    runtime::Services,
    theme::{self, Preference},
};
use command_state::CommandState;
use gpui_kit::component::{
    ActiveTheme, Selectable,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, StatefulInteractiveElement, Styled, Subscription, Window, div, px,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Configurations,
    Converter,
    Favorites,
    Recent,
    Settings,
}
pub struct Shell {
    configurations: Entity<ConfigurationApp>,
    commands: Entity<CommandState>,
    converter: Entity<crate::features::converter::Converter>,
    page: Page,
    preference: Preference,
    fixture: bool,
    slow_fixture: bool,
    navigation_focus: FocusHandle,
    main_focus: FocusHandle,
    status_focus: FocusHandle,
    _appearance: Subscription,
    _activation: Subscription,
}
impl Shell {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.set_global(Language::English);
        theme::apply(Preference::System, window, cx);
        let appearance = cx.observe_window_appearance(window, |this, window, cx| {
            if this.preference == Preference::System {
                theme::apply(this.preference, window, cx);
                cx.notify();
            }
        });
        let activation = cx.observe_window_activation(window, |this, window, cx| {
            if window.is_window_active() && window.focused(cx).is_none() {
                this.focus_main(window, cx);
            }
        });
        Self {
            configurations: cx.new(|cx| ConfigurationApp::new(services.clone(), window, cx)),
            commands: cx.new(|cx| CommandState::new(&services, window, cx)),
            converter: cx
                .new(|cx| crate::features::converter::Converter::new(services.clone(), window, cx)),
            page: Page::Configurations,
            preference: Preference::System,
            fixture: services.is_fixture(),
            slow_fixture: services.is_slow_fixture(),
            navigation_focus: cx.focus_handle().tab_stop(false),
            main_focus: cx.focus_handle().tab_stop(false),
            status_focus: cx.focus_handle(),
            _appearance: appearance,
            _activation: activation,
        }
    }
    fn navigate(&mut self, page: Page, window: &mut Window, cx: &mut Context<Self>) {
        if self.page == page {
            return;
        }
        self.configurations
            .update(cx, |view, cx| view.leave(window, cx));
        self.commands.update(cx, |view, cx| view.suspend(cx));
        self.converter.update(cx, |view, cx| view.leave(window, cx));
        self.page = page;
        match page {
            Page::Configurations => self.configurations.update(cx, |view, cx| view.refresh(cx)),
            Page::Favorites | Page::Recent => self
                .commands
                .update(cx, |view, cx| view.open(page == Page::Recent, cx)),
            Page::Settings | Page::Converter => {}
        }
        // The activated navigation button keeps focus. F6/Tab enters content.
        cx.notify();
        let _ = window;
    }
    fn focus_main(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.page {
            Page::Configurations => self
                .configurations
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Page::Converter => self
                .converter
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Page::Favorites => self
                .commands
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Page::Recent | Page::Settings => {
                window.focus(&self.main_focus, cx);
                window.focus_next(cx);
            }
        }
    }
    fn region(&mut self, backward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let in_nav = self.navigation_focus.contains_focused(window, cx);
        let in_status = self.status_focus.contains_focused(window, cx);
        let target = if backward {
            if in_nav {
                2
            } else if in_status {
                1
            } else {
                0
            }
        } else if in_nav {
            1
        } else if in_status {
            0
        } else {
            2
        };
        match target {
            0 => {
                window.focus(&self.navigation_focus, cx);
                window.focus_next(cx);
            }
            1 => self.focus_main(window, cx),
            _ => window.focus(&self.status_focus, cx),
        }
        cx.notify();
    }
    fn choose_theme(
        &mut self,
        preference: Preference,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.preference = preference;
        theme::apply(preference, window, cx);
        cx.notify();
    }
    fn settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut choices = div().flex().gap(px(theme::CONTROL_GAP));
        for (id, preference, en, zh) in [
            ("theme-system", Preference::System, "System", "跟随系统"),
            ("theme-light", Preference::Light, "Light", "浅色"),
            ("theme-dark", Preference::Dark, "Dark", "深色"),
        ] {
            choices = choices.child(semantics::button(
                Button::new(id)
                    .label(text(cx, en, zh))
                    .ghost()
                    .selected(self.preference == preference)
                    .toggled(self.preference == preference)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.choose_theme(preference, window, cx)
                    })),
                false,
                false,
                cx,
            ));
        }
        let language = cx.global::<Language>();
        div().flex().flex_col().p(px(theme::CONTENT_PADDING)).gap(px(theme::SECTION_GAP))
            .child(div().text_size(px(theme::TITLE_SIZE)).font_weight(gpui_kit::FontWeight::SEMIBOLD).child(text(cx,"Appearance and language","外观与语言")))
            .child(text(cx,"Theme","主题")).child(choices)
            .child(text(cx,"Language","语言"))
            .child(div().flex().gap(px(theme::CONTROL_GAP))
                .child(semantics::button(Button::new("language-en").label("English").ghost().selected(*language==Language::English).toggled(*language==Language::English).on_click(cx.listener(|_,_,_,cx|{cx.set_global(Language::English);cx.refresh_windows();})),false,false,cx))
                .child(semantics::button(Button::new("language-zh").label("中文").ghost().selected(*language==Language::Chinese).toggled(*language==Language::Chinese).on_click(cx.listener(|_,_,_,cx|{cx.set_global(Language::Chinese);cx.refresh_windows();})),false,false,cx)))
            .child(div().text_color(cx.theme().muted_foreground).child(text(cx,"Preferences apply to this window session. Keyboard actions are immediate; decorative motion is disabled.","设置在当前窗口会话内生效。键盘操作即时响应，装饰动画已关闭。")))
    }
}
impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let narrow = window.viewport_size().width < px(theme::NARROW_WIDTH);
        let sidebar_width = if narrow {
            theme::NARROW_SIDEBAR
        } else {
            theme::SIDEBAR_WIDTH
        };
        let mut navigation = div()
            .id("shell-navigation")
            .role(gpui_kit::Role::Navigation)
            .aria_label(text(cx, "Navigation", "导航"))
            .track_focus(&self.navigation_focus)
            .tab_stop(false)
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    match event.keystroke.key.as_str() {
                        "down" => {
                            window.focus_next(cx);
                            if !this.navigation_focus.contains_focused(window, cx) {
                                window.focus(&this.navigation_focus, cx);
                                window.focus_next(cx);
                            }
                        }
                        "up" => {
                            window.focus_prev(cx);
                            if !this.navigation_focus.contains_focused(window, cx) {
                                window.focus(&this.main_focus, cx);
                                window.focus_prev(cx);
                            }
                        }
                        _ => return,
                    }
                    cx.stop_propagation();
                }),
            )
            .w(px(sidebar_width))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .p(px(theme::SECTION_GAP))
            .gap(px(theme::CONTROL_GAP))
            .bg(cx.theme().sidebar)
            .child(
                div()
                    .h(px(theme::TOOLBAR_HEIGHT))
                    .flex()
                    .flex_col()
                    .justify_center()
                    .child(text(cx, "Local", "本地"))
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .text_size(px(theme::META_SIZE))
                            .child(if self.fixture {
                                text(cx, "Demo data", "演示数据")
                            } else {
                                text(cx, "This computer", "此计算机")
                            }),
                    ),
            );
        for (id, page, en, zh) in [
            (
                "nav-configurations",
                Page::Configurations,
                "Configurations",
                "配置",
            ),
            (
                "nav-favorites",
                Page::Favorites,
                "Favorite commands",
                "收藏命令",
            ),
            ("nav-recent", Page::Recent, "Recent commands", "最近命令"),
            ("nav-converter", Page::Converter, "Converter", "配置转换"),
        ] {
            navigation = navigation.child(semantics::button(
                Button::new(id)
                    .label(text(cx, en, zh))
                    .ghost()
                    .selected(self.page == page)
                    .toggled(self.page == page)
                    .w_full()
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.navigate(page, window, cx)),
                    ),
                false,
                false,
                cx,
            ));
        }
        navigation = navigation.child(div().flex_1()).child(semantics::button(
            Button::new("nav-settings")
                .label(text(cx, "Settings", "设置"))
                .ghost()
                .selected(self.page == Page::Settings)
                .toggled(self.page == Page::Settings)
                .w_full()
                .on_click(
                    cx.listener(|this, _, window, cx| this.navigate(Page::Settings, window, cx)),
                ),
            false,
            false,
            cx,
        ));
        let content = match self.page {
            Page::Configurations => self.configurations.clone().into_any_element(),
            Page::Favorites | Page::Recent => self.commands.clone().into_any_element(),
            Page::Settings => self.settings(cx).into_any_element(),
            Page::Converter => self.converter.clone().into_any_element(),
        };
        let status = if self.slow_fixture {
            text(
                cx,
                "Local · Demo data · Controlled slow read",
                "本地 · 演示数据 · 受控慢读取",
            )
        } else if self.fixture {
            text(cx, "Local · Demo data", "本地 · 演示数据")
        } else {
            text(
                cx,
                "Local · Desktop command records are shared across environments",
                "本地 · 桌面命令记录跨环境共享",
            )
        };
        div()
            .id("ccr-shell")
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone())
            .text_size(px(theme::BODY_SIZE))
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "f6" {
                        this.region(event.keystroke.modifiers.shift, window, cx);
                        cx.stop_propagation();
                    }
                }),
            )
            .child(
                div().flex().flex_1().min_h_0().child(navigation).child(
                    div()
                        .id("shell-main")
                        .role(gpui_kit::Role::Main)
                        .aria_label(text(cx, "Workspace", "工作区"))
                        .track_focus(&self.main_focus)
                        .tab_stop(false)
                        .flex_1()
                        .min_w_0()
                        .min_h_0()
                        .child(content),
                ),
            )
            .child(
                div()
                    .id("shell-status")
                    .role(gpui_kit::Role::Status)
                    .aria_label(status)
                    .track_focus(&self.status_focus)
                    .tab_stop(true)
                    .h(px(theme::STATUS_HEIGHT))
                    .flex_shrink_0()
                    .px(px(theme::SECTION_GAP))
                    .flex()
                    .items_center()
                    .border_t_1()
                    .border_color(cx.theme().status_bar_border)
                    .focus(|style| style.border_color(cx.theme().ring))
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(status),
            )
    }
}
