//! Ordinary fields only; source documents and credentials remain backend-owned.
use super::*;
use serde_json::{Value, json};
#[cfg(test)]
mod tests;

struct Field {
    key: &'static str,
    path: &'static str,
    label: &'static str,
    choices: &'static [&'static str],
    input: Entity<InputState>,
    initial: String,
    selected: String,
}
pub(super) struct SettingsForm {
    fields: Vec<Field>,
}
fn label(en: &'static str, cx: &gpui_kit::App) -> &'static str {
    let zh = match en {
        "Model" => "模型",
        "Effort" | "Reasoning effort" => "推理强度",
        "Theme" => "主题",
        "Language" => "语言",
        "Update channel" => "更新通道",
        "Permission mode" => "权限模式",
        "Provider" => "供应商",
        "Approval policy" => "审批策略",
        "Sandbox" => "沙箱",
        "Personality" => "交互风格",
        "Default model" => "默认模型",
        "Automatic updates" => "自动更新",
        "Auto compact threshold (0–100)" => "自动压缩阈值（0–100）",
        "Load envrc" => "加载 envrc",
        "Show tips" => "显示提示",
        "New-session worktree" => "新会话工作树",
        "Fork worktree" => "分叉会话工作树",
        "Small model" => "轻量模型",
        "Default agent" => "默认 Agent",
        "Sharing" => "共享",
        "Snapshots" => "快照",
        "Mouse" => "鼠标",
        "Provider name" => "供应商名称",
        "Base URL" => "服务地址",
        "Website URL" => "官网地址",
        "API key page URL" => "密钥管理页",
        "Replace API key" => "替换 API 密钥",
        "New key name" => "新密钥名称",
        "NPM package" => "NPM 包",
        _ => en,
    };
    text(cx, en, zh)
}
impl SettingsForm {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }
    pub fn clear(&mut self) {
        self.fields.clear();
    }
    pub fn load(
        &mut self,
        platform: Platform,
        surface: Surface,
        facts: &[crate::runtime::platforms::FieldFact],
        window: &mut Window,
        cx: &mut Context<PlatformsView>,
    ) {
        self.clear();
        let specifications: &[(&str, &str, &str, &[&str])] = match (platform, surface) {
            (Platform::Claude, Surface::Settings) => &[
                ("model", "model", "Model", &[]),
                ("effortLevel", "effortLevel", "Effort", &[]),
                ("theme", "theme", "Theme", &[]),
                ("language", "language", "Language", &[]),
                (
                    "autoUpdatesChannel",
                    "autoUpdatesChannel",
                    "Update channel",
                    &["stable", "latest"],
                ),
                (
                    "permissionMode",
                    "permissions.defaultMode",
                    "Permission mode",
                    &["default", "plan", "bypassPermissions"],
                ),
            ],
            (Platform::Codex, Surface::Settings) => &[
                ("model", "model", "Model", &[]),
                ("model_provider", "model_provider", "Provider", &[]),
                (
                    "model_reasoning_effort",
                    "model_reasoning_effort",
                    "Reasoning effort",
                    &["minimal", "low", "medium", "high"],
                ),
                ("approval_policy", "approval_policy", "Approval policy", &[]),
                ("sandbox_mode", "sandbox_mode", "Sandbox", &[]),
                ("personality", "personality", "Personality", &[]),
            ],
            (Platform::Grok, Surface::Settings) => &[
                ("default_model", "models.default", "Default model", &[]),
                (
                    "reasoning_effort",
                    "models.default_reasoning_effort",
                    "Reasoning effort",
                    &["none", "minimal", "low", "medium", "high", "xhigh", "max"],
                ),
                ("theme", "ui.theme", "Theme", &["system", "light", "dark"]),
                (
                    "channel",
                    "cli.channel",
                    "Update channel",
                    &["stable", "alpha"],
                ),
                (
                    "auto_update",
                    "cli.auto_update",
                    "Automatic updates",
                    &["true", "false"],
                ),
                (
                    "auto_compact",
                    "session.auto_compact_threshold_percent",
                    "Auto compact threshold (0–100)",
                    &[],
                ),
                (
                    "load_envrc",
                    "session.load_envrc",
                    "Load envrc",
                    &["true", "false"],
                ),
                (
                    "show_tips",
                    "cli.show_tips",
                    "Show tips",
                    &["true", "false"],
                ),
                (
                    "new_session_worktree_mode",
                    "hints.new_session_worktree_mode",
                    "New-session worktree",
                    &["ask", "always", "never"],
                ),
                (
                    "fork_worktree_mode",
                    "hints.fork_worktree_mode",
                    "Fork worktree",
                    &["ask", "always", "never"],
                ),
            ],
            (Platform::OpenCode, Surface::Settings) => &[
                ("model", "model", "Model", &[]),
                ("small_model", "small_model", "Small model", &[]),
                ("default_agent", "default_agent", "Default agent", &[]),
                ("share", "share", "Sharing", &["manual", "auto", "disabled"]),
                ("snapshot", "snapshot", "Snapshots", &["true", "false"]),
                (
                    "autoupdate",
                    "autoupdate",
                    "Automatic updates",
                    &["true", "false", "notify"],
                ),
            ],
            (Platform::OpenCode, Surface::Tui) => &[
                ("theme", "theme", "Theme", &[]),
                ("mouse", "mouse", "Mouse", &["true", "false"]),
            ],
            (Platform::Codex, Surface::Providers) => &[
                ("name", "name", "Provider name", &[]),
                ("base_url", "base_url", "Base URL", &[]),
                ("website_url", "website_url", "Website URL", &[]),
                ("api_key_url", "api_key_url", "API key page URL", &[]),
                ("api_key", "api_key", "Replace API key", &[]),
                ("api_key_name", "api_key_name", "New key name", &[]),
            ],
            (Platform::OpenCode, Surface::Providers) => &[
                ("name", "name", "Provider name", &[]),
                ("npm", "npm", "NPM package", &[]),
                ("baseURL", "options.baseURL", "Base URL", &[]),
                ("apiKey", "options.apiKey", "Replace API key", &[]),
            ],
            _ => &[],
        };
        for &(key, path, label, choices) in specifications {
            let initial = facts
                .iter()
                .find(|f| f.key == key)
                .map(|f| f.value.clone())
                .unwrap_or_default();
            let input = cx
                .new(|cx| InputState::new(window, cx).masked(matches!(key, "api_key" | "apiKey")));
            input.update(cx, |state, cx| state.set_value(initial.clone(), window, cx));
            self.fields.push(Field {
                key,
                path,
                label,
                choices,
                input,
                selected: initial.clone(),
                initial,
            });
        }
    }
    pub fn patch(&self, base: &str, cx: &Context<PlatformsView>) -> Result<String, String> {
        let mut patch: Value = if base.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str(base)
                .map_err(|_| "Invalid advanced draft. / 高级草稿无效".to_string())?
        };
        if !patch.is_object() {
            return Err("Advanced patch must be an object. / 高级补丁必须是对象".into());
        }
        for field in &self.fields {
            let value = if field.choices.is_empty() {
                field.input.read(cx).value().to_string()
            } else {
                field.selected.clone()
            };
            if value == field.initial
                || (matches!(field.key, "api_key" | "apiKey") && value.is_empty())
            {
                continue;
            }
            let value = if field.key == "auto_compact" {
                let number = value
                    .parse::<i64>()
                    .ok()
                    .filter(|v| (0..=100).contains(v))
                    .ok_or_else(|| {
                        format!(
                            "{}: enter an integer from 0 to 100. / 请输入 0–100 的整数",
                            field.label
                        )
                    })?;
                json!(number)
            } else if (field.choices == ["true", "false"] || field.key == "autoupdate")
                && matches!(value.as_str(), "true" | "false")
            {
                json!(value == "true")
            } else {
                json!(value)
            };
            let (section, key) = field
                .path
                .split_once('.')
                .map_or((None, field.path), |(section, key)| (Some(section), key));
            let root = if let Some(section) = section {
                patch
                    .as_object_mut()
                    .ok_or("Invalid draft")?
                    .entry(section)
                    .or_insert_with(|| json!({}))
                    .as_object_mut()
                    .ok_or("Invalid nested draft")?
            } else {
                patch.as_object_mut().ok_or("Invalid draft")?
            };
            root.insert(key.into(), value);
        }
        Ok(patch.to_string())
    }
    pub fn focus(&self, window: &mut Window, cx: &mut Context<PlatformsView>) {
        if let Some(field) = self.fields.first() {
            window.focus(&field.input.read(cx).focus_handle(cx), cx)
        }
    }
    pub fn sync_draft(
        &mut self,
        raw: &str,
        window: &mut Window,
        cx: &mut Context<PlatformsView>,
    ) -> Result<(), String> {
        let patch: Value = serde_json::from_str(raw)
            .map_err(|_| "Invalid advanced draft. / 高级草稿无效".to_string())?;
        if !patch.is_object() {
            return Err("Advanced patch must be an object. / 高级补丁必须是对象".into());
        }
        for field in &mut self.fields {
            let value = field
                .path
                .split('.')
                .try_fold(&patch, |value, key| value.get(key));
            if let Some(value) = value.and_then(|v| match v {
                Value::String(v) => Some(v.clone()),
                Value::Bool(v) => Some(v.to_string()),
                Value::Number(v) => Some(v.to_string()),
                _ => None,
            }) {
                field.initial = value.clone();
                field.selected = value.clone();
                field
                    .input
                    .update(cx, |state, cx| state.set_value(value, window, cx));
            }
        }
        Ok(())
    }
    pub fn render(&self, blocked: bool, cx: &mut Context<PlatformsView>) -> gpui_kit::AnyElement {
        let mut root = div()
            .id("platform-common-form")
            .test_support()
            .flex()
            .flex_wrap()
            .gap(px(theme::SECTION_GAP));
        for field in &self.fields {
            let mut row = div()
                .w(gpui_kit::relative(0.48))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP))
                .child(label(field.label, cx));
            if field.choices.is_empty() {
                row = row.child(
                    Input::new(&field.input)
                        .id(SharedString::from(format!(
                            "platform-setting-{}",
                            field.key
                        )))
                        .aria_label(label(field.label, cx))
                        .disabled(blocked),
                );
            } else {
                let mut choices = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
                let key = field.key;
                for &value in field.choices {
                    choices = choices.child(semantics::button(
                        Button::new(SharedString::from(format!(
                            "platform-setting-{key}-{value}"
                        )))
                        .label(value)
                        .ghost()
                        .toggled(field.selected == value)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(field) = this.form.fields.iter_mut().find(|f| f.key == key)
                            {
                                field.selected = value.into();
                            }
                            cx.notify();
                        })),
                        blocked,
                        false,
                        cx,
                    ));
                }
                row = row.child(choices);
            }
            root = root.child(row);
        }
        root.into_any_element()
    }
}
