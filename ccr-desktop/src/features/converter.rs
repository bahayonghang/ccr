//! Scoped conversion preview. It never applies files or executes generated commands.
use crate::{locales::text, runtime::Services, shell::semantics, theme};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, TaskEvent};
use ccr_db::{
    models::converter::{CliType, ConverterRequest},
    services::converter_service::ConfigConverter,
};
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    input::{Textarea, TextareaState},
};
use gpui_kit::{
    AppContext, Context, Entity, Focusable, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Task, Window, div, px,
};
pub struct Converter {
    services: Services,
    acknowledged: bool,
    reverse: bool,
    input: Entity<TextareaState>,
    output: Entity<TextareaState>,
    pending: Option<String>,
    task: Option<Task<()>>,
    generation: u64,
    busy: bool,
    message: Option<String>,
}
impl Converter {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            services,
            acknowledged: false,
            reverse: false,
            input: cx.new(|cx| TextareaState::new(window, cx)),
            output: cx.new(|cx| TextareaState::new(window, cx)),
            pending: None,
            task: None,
            generation: 0,
            busy: false,
            message: None,
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.input.read(cx).focus_handle(cx), cx);
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.generation += 1;
        self.busy = false;
        cx.notify();
    }
    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.suspend(cx);
        self.pending = None;
        self.message = None;
        self.acknowledged = false;
        self.input
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.output
            .update(cx, |state, cx| state.set_value("", window, cx));
    }
    fn convert(&mut self, cx: &mut Context<Self>) {
        if self.busy || !self.acknowledged {
            return;
        }
        self.generation += 1;
        let generation = self.generation;
        let key = RequestKey {
            environment: EnvironmentId::Local("local".into()),
            platform: Platform::Claude,
            generation,
            request_id: generation,
        };
        let request = ConverterRequest {
            source_format: if self.reverse {
                CliType::Codex
            } else {
                CliType::ClaudeCode
            },
            target_format: if self.reverse {
                CliType::ClaudeCode
            } else {
                CliType::Codex
            },
            config_data: self.input.read(cx).value().to_string(),
            convert_mcp: true,
            convert_commands: true,
            convert_agents: true,
        };
        self.busy = true;
        match self.services.executor.read(key,move|_,_|ConfigConverter::convert(request).map_err(|_|())){
            Ok(mut read)=>self.task=Some(cx.spawn(async move|weak,cx|{while let Some(event)=read.next().await{if let TaskEvent::Terminal(result)=event{let _=weak.update(cx,|this,cx|{if this.generation!=generation{return;}this.busy=false;match result.as_ref(){Completion::Success(value)=>{this.pending=Some(value.converted_data.clone());this.message=Some(format!("{}: MCP {}; commands {}; agents {}; profiles {}. {}",value.format,value.stats.mcp_servers,value.stats.slash_commands,value.stats.agents,value.stats.profiles,if value.warnings.is_empty(){"No warnings / 无警告"}else{"Conversion has warnings / 转换存在警告"}));},_=>this.message=Some("Invalid input or unsupported conversion. Input retained; no files changed. / 输入无效或格式不支持，输入已保留，未修改文件".into())}cx.notify();});break;}}})),
            Err(_)=>{self.busy=false;self.message=Some("Queue busy. Retry. / 队列繁忙，请重试".into());}
        }
        cx.notify();
    }
}
impl Render for Converter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(output) = self.pending.take() {
            self.output
                .update(cx, |state, cx| state.set_value(output, window, cx));
        }
        let mut root=div().flex().flex_col().size_full().p(px(theme::CONTENT_PADDING)).gap(px(theme::SECTION_GAP)).child(div().text_size(px(theme::TITLE_SIZE)).child(text(cx,"Configuration converter","配置转换"))).child(div().text_color(cx.theme().muted_foreground).child(text(cx,"Claude Code JSON ↔ Codex TOML. Preview only; generated text is never applied or executed. Gemini and Qwen are not supported by the existing converter.","Claude Code JSON ↔ Codex TOML，仅预览，不写入或执行。现有转换器不支持 Gemini 和 Qwen。")));
        if !self.acknowledged {
            return root.child(text(cx,"Input and output may contain plaintext credentials. They remain in this window session. Do not include them in screenshots.","输入和输出可能包含明文凭据，仅保留在当前窗口会话内，请勿截图。" )).child(semantics::button(Button::new("converter-acknowledge").label(text(cx,"Open scoped converter","打开临时转换区")).on_click(cx.listener(|this,_,window,cx|{this.acknowledged=true;this.focus_main(window,cx);cx.notify();})),false,false,cx));
        }
        root = root
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(semantics::button(
                        Button::new("converter-direction")
                            .label(if self.reverse {
                                "Codex → Claude Code"
                            } else {
                                "Claude Code → Codex"
                            })
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.reverse = !this.reverse;
                                cx.notify();
                            })),
                        self.busy,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("converter-run")
                            .label(text(cx, "Convert preview", "转换预览"))
                            .on_click(cx.listener(|this, _, _, cx| this.convert(cx))),
                        self.busy,
                        self.busy,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("converter-clear")
                            .label(text(cx, "Clear sensitive text", "清除敏感文本"))
                            .ghost()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.suspend(cx);
                                this.input
                                    .update(cx, |state, cx| state.set_value("", window, cx));
                                this.output
                                    .update(cx, |state, cx| state.set_value("", window, cx));
                                this.message = None;
                                this.acknowledged = false;
                                cx.notify();
                            })),
                        false,
                        false,
                        cx,
                    )),
            )
            .child(
                Textarea::new(&self.input)
                    .accessibility_id("converter-input")
                    .aria_label(text(cx, "Source configuration", "源配置"))
                    .h(px(180.)),
            )
            .child(
                Textarea::new(&self.output)
                    .accessibility_id("converter-output")
                    .aria_label(text(cx, "Converted preview", "转换预览结果"))
                    .h(px(180.))
                    .readonly(true),
            );
        if let Some(message) = &self.message {
            root = root.child(
                div()
                    .id("converter-status")
                    .role(gpui_kit::Role::Status)
                    .child(message.clone()),
            );
        }
        root
    }
}
