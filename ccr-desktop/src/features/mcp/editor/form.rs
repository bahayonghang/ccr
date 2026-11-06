use super::*;
use gpui_kit::{SharedString, px};
use serde_json::{Value, json};
#[cfg(test)]
mod tests;

struct Pair {
    key: Entity<InputState>,
    value: Entity<InputState>,
}
pub(super) struct ServerForm {
    command: Entity<InputState>,
    initial_command: String,
    source_command: String,
    args: Entity<TextareaState>,
    url: Entity<InputState>,
    env: Vec<Pair>,
    headers: Vec<Pair>,
    pub url_key: &'static str,
    codex: bool,
    creating: bool,
}
impl ServerForm {
    pub fn new(
        target: &McpTarget,
        creating: bool,
        window: &mut Window,
        cx: &mut Context<McpEditor>,
    ) -> Self {
        Self {
            command: cx.new(|cx| InputState::new(window, cx)),
            initial_command: String::new(),
            source_command: String::new(),
            creating,
            args: cx.new(|cx| TextareaState::new(window, cx)),
            url: cx.new(|cx| InputState::new(window, cx).masked(true)),
            env: Vec::new(),
            headers: Vec::new(),
            url_key: if *target == McpTarget::Antigravity {
                "serverUrl"
            } else {
                "url"
            },
            codex: *target == McpTarget::Codex,
        }
    }
    pub fn load_command(
        &mut self,
        value: Option<String>,
        window: &mut Window,
        cx: &mut Context<McpEditor>,
    ) {
        self.initial_command = value.unwrap_or_default();
        if !self.creating {
            self.source_command = self.initial_command.clone();
        }
        self.command.update(cx, |state, cx| {
            state.set_value(self.initial_command.clone(), window, cx)
        });
    }
    pub fn add_env(&mut self, key: &str, window: &mut Window, cx: &mut Context<McpEditor>) {
        let pair = Pair {
            key: cx.new(|cx| InputState::new(window, cx)),
            value: cx.new(|cx| InputState::new(window, cx).masked(true)),
        };
        pair.key
            .update(cx, |state, cx| state.set_value(key.to_owned(), window, cx));
        self.env.push(pair);
    }
    pub fn resume(
        &mut self,
        raw: &str,
        window: &mut Window,
        cx: &mut Context<McpEditor>,
    ) -> Result<(), &'static str> {
        let patch: Value =
            serde_json::from_str(raw).map_err(|_| "Invalid advanced draft. / 高级草稿无效")?;
        if !patch.is_object() {
            return Err("Advanced patch must be an object. / 高级补丁必须是对象");
        }
        self.initial_command = patch
            .get("command")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| self.source_command.clone());
        self.command.update(cx, |state, cx| {
            state.set_value(self.initial_command.clone(), window, cx)
        });
        self.args
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.url
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.env.clear();
        self.headers.clear();
        Ok(())
    }
    pub fn patch(&self, base: &str, cx: &Context<McpEditor>) -> Result<Value, &'static str> {
        let mut patch: Value =
            serde_json::from_str(base).map_err(|_| "Invalid advanced draft. / 高级草稿无效")?;
        let object = patch
            .as_object_mut()
            .ok_or("Advanced patch must be an object. / 高级补丁必须是对象")?;
        let command = self.command.read(cx).value();
        if command.as_ref() != self.initial_command {
            object.insert("command".into(), json!(command.to_string()));
        }
        let args = self.args.read(cx).value();
        if !args.is_empty() {
            object.insert(
                "args".into(),
                json!(args.lines().map(str::to_owned).collect::<Vec<_>>()),
            );
        }
        let url = self.url.read(cx).value();
        if !url.is_empty() {
            if !reqwest::Url::parse(&url).is_ok_and(|url| {
                matches!(url.scheme(), "http" | "https") && url.host_str().is_some()
            }) {
                return Err(
                    "URL: enter an HTTP or HTTPS endpoint. / URL：请输入 HTTP 或 HTTPS 地址",
                );
            }
            object.insert(self.url_key.into(), json!(url.to_string()));
            if command.as_ref() != self.initial_command && !command.is_empty() {
                return Err(
                    "Choose a stdio command or an HTTP URL. / 请选择 Stdio 命令或 HTTP 地址其中之一",
                );
            }
            object.insert("command".into(), Value::Null);
            object.insert("args".into(), Value::Null);
            if !self.codex && self.url_key == "url" {
                object.insert("type".into(), json!("http"));
            }
        }
        if command.as_ref() != self.initial_command && !command.is_empty() {
            object.insert(self.url_key.into(), Value::Null);
            if !self.codex {
                object.insert("type".into(), Value::Null);
            }
        }
        for (key, pairs) in [
            ("env", &self.env),
            (
                if self.codex {
                    "http_headers"
                } else {
                    "headers"
                },
                &self.headers,
            ),
        ] {
            for pair in pairs {
                let name = pair.key.read(cx).value().to_string();
                let value = pair.value.read(cx).value().to_string();
                if value.is_empty() {
                    continue;
                }
                if name.trim().is_empty() || name.contains(['\r', '\n']) {
                    return Err(
                        "Environment/header name must be nonempty and single-line. / 环境变量或请求头名称不能为空或包含换行",
                    );
                }
                if key != "env" && value.contains(['\r', '\n']) {
                    return Err("Header value must be single-line. / 请求头值不能包含换行");
                }
                let target = object
                    .entry(key)
                    .or_insert_with(|| json!({}))
                    .as_object_mut()
                    .ok_or("Invalid map in advanced draft. / 高级草稿映射无效")?;
                target.insert(name, json!(value));
            }
        }
        Ok(patch)
    }
    fn pairs(
        &self,
        headers: bool,
        blocked: bool,
        cx: &mut Context<McpEditor>,
    ) -> gpui_kit::AnyElement {
        let mut root = div().flex().flex_col().gap(px(theme::CONTROL_GAP));
        let prefix = if headers { "mcp-header" } else { "mcp-env" };
        let pairs = if headers { &self.headers } else { &self.env };
        for (index, pair) in pairs.iter().enumerate() {
            root = root.child(
                div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div().flex_1().child(
                            Input::new(&pair.key)
                                .id(SharedString::from(format!("{prefix}-name-{index}")))
                                .aria_label(if headers {
                                    text(cx, "Header name", "请求头名称")
                                } else {
                                    text(cx, "Environment variable name", "环境变量名")
                                })
                                .disabled(blocked),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            Input::new(&pair.value)
                                .id(SharedString::from(format!("{prefix}-value-{index}")))
                                .aria_label(text(cx, "Replacement value", "替换值"))
                                .disabled(blocked),
                        ),
                    ),
            );
        }
        root.child(semantics::button(
            Button::new(if headers {
                "mcp-header-add"
            } else {
                "mcp-env-add"
            })
            .label(if headers {
                text(cx, "Add header replacement", "添加请求头替换")
            } else {
                text(cx, "Add environment replacement", "添加环境变量替换")
            })
            .ghost()
            .on_click(cx.listener(move |this, _, window, cx| {
                let pair = Pair {
                    key: cx.new(|cx| InputState::new(window, cx)),
                    value: cx.new(|cx| InputState::new(window, cx).masked(true)),
                };
                if headers {
                    this.form.headers.push(pair)
                } else {
                    this.form.env.push(pair)
                };
                cx.notify();
            })),
            blocked,
            false,
            cx,
        ))
        .into_any_element()
    }
    pub fn render(&self, blocked: bool, cx: &mut Context<McpEditor>) -> gpui_kit::AnyElement {
        div().id("mcp-common-form").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(text(cx,"Stored arguments, endpoint and credentials remain hidden. Blank replacement fields preserve them. Use Advanced for explicit clearing or other transport options.","已有参数、地址和凭据保持隐藏。替换字段留空会保留原值；显式清空和其他传输选项可使用高级补丁。"))
            .child(div().child(text(cx,"Stdio command","Stdio 命令")).child(Input::new(&self.command).id("mcp-command").aria_label(text(cx,"Stdio command","Stdio 命令")).disabled(blocked)))
            .child(div().child(text(cx,"Arguments (one per line)","参数（每行一个）")).child(Textarea::new(&self.args).accessibility_id("mcp-args").aria_label(text(cx,"Replacement arguments","替换参数")).h(px(theme::ROW_HEIGHT*2.)).disabled(blocked)))
            .child(div().child(format!("HTTP URL ({})",self.url_key)).child(Input::new(&self.url).id("mcp-url").aria_label(text(cx,"Replacement HTTP URL","替换 HTTP 地址")).disabled(blocked)))
            .child(self.pairs(false,blocked,cx)).child(self.pairs(true,blocked,cx)).into_any_element()
    }
}
