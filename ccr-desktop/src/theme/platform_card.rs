//! Frosted reading surfaces for platform profiles and the Antigravity workbench.
use super::{alpha, over, palette, rim};
use ccr_app::runtime::Platform;
use gpui_kit::component::ThemeMode;
use gpui_kit::{Div, Hsla, Styled, linear_color_stop, linear_gradient, px, rgb};

pub struct PlatformCard {
    mode: ThemeMode,
    light: u32,
    paper: u32,
    pub title: Hsla,
    pub secondary: Hsla,
    pub edge: Hsla,
    pub action_edge: Hsla,
    pub action: Hsla,
    pub hover: Hsla,
    pub pressed: Hsla,
}

impl PlatformCard {
    pub fn new(platform: Platform, mode: ThemeMode) -> Self {
        let dark = mode.is_dark();
        // A warm/cool light source, and independently legible inks in both appearances.
        let (tint, day_ink, night_ink) = match platform {
            Platform::Claude => (0xefad54, 0x733914, 0xffc590),
            Platform::Codex => (0x73aff4, 0x204f84, 0xa4d0ff),
            Platform::Grok => (0xa99bbb, 0x493e59, 0xd9cdea),
            Platform::Gemini => (0x60c9b4, 0x165c56, 0x96e8d9),
            _ => (0x9ca8a8, 0x414f53, 0xc9d6d6),
        };
        let p = palette(mode);
        let ink = if dark { night_ink } else { day_ink };
        let paper = over(p.surface, tint, if dark { 0.025 } else { 0.045 });
        let action = over(paper, tint, if dark { 0.095 } else { 0.23 });
        Self {
            mode,
            light: over(p.surface, tint, if dark { 0.16 } else { 0.32 }),
            paper,
            title: rgb(ink).into(),
            secondary: rgb(over(p.secondary_text, ink, 0.12)).into(),
            edge: rgb(over(paper, ink, if dark { 0.30 } else { 0.29 })).into(),
            action_edge: rgb(over(paper, ink, if dark { 0.72 } else { 0.79 })).into(),
            action: rgb(action).into(),
            hover: rgb(over(action, tint, if dark { 0.08 } else { 0.16 })).into(),
            pressed: rgb(over(action, tint, if dark { 0.14 } else { 0.26 })).into(),
        }
    }

    pub(super) fn stops(&self) -> [(u32, f32); 2] {
        [(self.light, 0.95), (self.paper, 0.92)]
    }

    pub fn surface(&self, element: Div, selected: bool) -> Div {
        let [start, end] = self.stops();
        element
            .rounded(px(12.))
            .border_1()
            .border_color(if selected {
                self.action_edge
            } else {
                self.edge
            })
            .bg(linear_gradient(
                125.,
                linear_color_stop(alpha(start), 0.),
                linear_color_stop(alpha(end), 0.88),
            ))
            .shadow(vec![rim(self.mode)])
    }
}
