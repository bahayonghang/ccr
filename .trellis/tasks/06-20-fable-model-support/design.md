# Design — ccr Claude profile fable 层与显示名端到端支持

## 设计原则

fable 层与各层显示名是**纯增量字段**，与现有 `default_opus_model` / `custom_model_option` 完全同构。不新增抽象、不改 auth_mode/resolve 语义，只在「四处登记点」对齐。前置任务 `06-19` 已为 `custom_model_option` 走通同一条链路，本任务是其复制扩展。

## 字段与 env 对照表

| typed 字段                  | env 变量                              | 现状    |
| --------------------------- | ------------------------------------- | ------- |
| `default_opus_model`        | `ANTHROPIC_DEFAULT_OPUS_MODEL`        | ✅ 已有 |
| `default_sonnet_model`      | `ANTHROPIC_DEFAULT_SONNET_MODEL`      | ✅ 已有 |
| `default_haiku_model`       | `ANTHROPIC_DEFAULT_HAIKU_MODEL`       | ✅ 已有 |
| `default_fable_model`       | `ANTHROPIC_DEFAULT_FABLE_MODEL`       | 🆕 新增 |
| `default_opus_model_name`   | `ANTHROPIC_DEFAULT_OPUS_MODEL_NAME`   | 🆕 新增 |
| `default_sonnet_model_name` | `ANTHROPIC_DEFAULT_SONNET_MODEL_NAME` | 🆕 新增 |
| `default_haiku_model_name`  | `ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME`  | 🆕 新增 |
| `default_fable_model_name`  | `ANTHROPIC_DEFAULT_FABLE_MODEL_NAME`  | 🆕 新增 |

## 「四处登记点」契约（每个新字段都必须同时出现，否则会串档/丢配置）

1. **类型定义**：`ConfigSection`（types.rs）+ `ProfileConfig`（platform.rs），`Option<String>` + `skip_serializing_if`。两个结构体之间的转换函数（`profile_to_section` / `section_to_profile`，在 ccr-config `base.rs` 内）也要带上新字段。
2. **写出**：`settings.rs::update_from_config` 增 `if let Some(v) = &section.<field> { env.insert(<CONST>, v.clone()) }`。
3. **登记**：`claude.rs::get_env_var_names` 追加新 env 名（用于 switch 展示）。
4. **src-tauri 手工映射**：`ccr-ui/src-tauri/src/commands/claude.rs` 的请求解析（`apply_*_update`）与 `profile_to_json` 两处手工 JSON 映射也必须带上新字段，否则前端表单字段保存/回填会被静默丢弃。

> ⚠️ 清理修正：`clear_anthropic_vars` 用的是**前缀清理** `!key.starts_with("ANTHROPIC_")`，新增的 `ANTHROPIC_DEFAULT_FABLE_MODEL*` / `*_MODEL_NAME` 全是该前缀，apply 时**自动被清掉**，无需改清理集合。故本任务未动 clear 函数；真正的缺口只在写出侧（无字段/常量/映射，ccr 生成不出 fable）。原 PRD/design 关于「fable 会串档」的判断对清理侧不成立，已订正。`clear_managed_vars` 额外清的是非 `ANTHROPIC_` 前缀键（CLAUDE_CODE_*），与本任务无关。

## 数据流

```
ccr-ui 表单 / profiles.toml
   → ProfileConfig (platform.rs)
   → profile_to_section → ConfigSection (types.rs)
   → update_from_config → ClaudeSettings.env
   → save_atomic → ~/.claude/settings.json
```

apply 前 `clear_managed_vars()` 先清掉全部受管 env（含新增常量），再按当前 profile 重新写入 → 切换不残留。

## 迁移（R3）

参考前置任务对 `custom_model_option` 的处理：在 profile 加载（`load_profiles`）或 `section_to_profile` 阶段，检测 `other`/`platform_data` 里的字符串键（`default_fable_model`、`default_*_model_name`），若 typed 字段为空则抬升、并 `remove` 残留键。保存时即落为 typed 形态。一次性自愈，无需用户重存。

## 前端（R4）

- `ClaudeProfileEditorSections.vue` 的「高级模型映射」分区，按现有 opus/sonnet/haiku 输入框模式追加：fable 模型框 + 四个 `*_MODEL_NAME` 框（显示名）。
- 类型：`claudeProfileEditor.ts` / `claude.ts` 增对应可选字段，命名与后端 JSON 字段一致。
- i18n：`zh-CN.ts` / `en-US.ts` 增 label + helper（helper 说明：显示名仅影响 `/model` 列表展示文案，不影响实际请求模型 ID）。

## 兼容性 / 回滚

- 全部字段 `Option`，旧 profile 反序列化不受影响（缺字段 = None）。
- 回滚 = 还原新增字段与四处登记；已写入 settings.json 的 fable env 因仍在清理集合（若回滚则不在）需注意：回滚后旧 fable env 不会被 ccr 清理 —— 回滚前建议手动从 settings.json 移除。implement.md 记此回滚点。

## 风险

- R-1：`*_MODEL_NAME` 拼写/语义未经官方文档二次确认（A2）。缓解：以截图真实配置为权威，apply 后人工核对 `/model` 行为；implement 列为验证项。
- R-2：两个结构体（ConfigSection / ProfileConfig）字段需手动保持同步，漏一处即丢配置。缓解：单测覆盖 round-trip（profile→section→env）。
