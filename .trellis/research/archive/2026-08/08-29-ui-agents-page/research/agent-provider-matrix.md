# 八 Agent Provider 支持矩阵

## Scope rule

“尽量支持”被收敛为可验证规则：八个 family 都必须出现在 provider registry 和 UI 中；每个 family 至少一个 canonical fixture 必须完成发现、摘要与有界消息页；无法安全解码的来源返回明确 fidelity/status，不以空内容假装成功。

## Evidence-backed matrix

| Family | Reference roots and source shape | Shared implementation | First-version fidelity and fallback |
| --- | --- | --- | --- |
| Claude | `~/.claude/projects/**/*.jsonl`; `CLAUDE_CONFIG_DIR` / `CLAUDE_PROJECTS_DIR`（`ref/repo/agentsview/internal/parser/types.go:136-143`） | 迁移 CCR existing Claude JSONL parser | canonical JSONL full；malformed/truncated 行 fail-soft 并标 partial |
| Codex | `~/.codex/sessions` + `~/.codex/archived_sessions` JSONL（`ref/repo/agentsview/internal/parser/types.go:166-176`） | 迁移 CCR existing Codex parser | live/archived full，source variant 可见 |
| Grok | `~/.grok/sessions/<cwd>/<session>/summary.json`；companions 为 `signals.json`、`chat_history.jsonl`、`updates.jsonl`、`prompt_context.json`（`ref/repo/agentsview/internal/parser/types.go:636-642`, `ref/repo/agentsview/internal/parser/grok_provider.go:27-83,116-178`） | 新 bundle provider | summary + transcript companion full；缺少可选 metadata 不失败，缺 transcript 为 partial |
| OpenCode | `~/.local/share/opencode`; file-backed `storage/session|message|part` 或 `opencode.db`（`ref/repo/agentsview/internal/parser/types.go:242-252`, `ref/repo/agentsview/internal/parser/discovery.go:76-133`） | 新 storage/SQLite dual provider；复用 CCR OpenCode path/usage DB 经验 | 两种 backend 都是首版验收；SQLite 共享容器按 member watermark 增量 |
| Pi | `~/.pi/agent/sessions/**/*.jsonl`（`ref/repo/agentsview/internal/parser/types.go:449-455`） | 参数化 Pi-like parser | full；保留 `pi` identity |
| OMP | OhMyPi，`~/.omp/agent/sessions/**/*.jsonl`（`ref/repo/agentsview/internal/parser/types.go:467-473`）；参考 parser 明确复用 Pi-like 格式（`ref/repo/agentsview/internal/parser/pi.go:101-116`） | 与 Pi 共享 parser，不共享 identity | full；UI 标签 `OMP`，辅助说明 `OhMyPi`，archive ID 使用 `omp` |
| Antigravity | IDE `~/.gemini/antigravity/conversations/<id>.db` + brain/annotation；CLI `~/.gemini/antigravity-cli/conversations/<id>.db|.pb` + implicit/brain/history（`ref/repo/agentsview/internal/parser/types.go:764-789`, `ref/repo/agentsview/internal/parser/antigravity.go:18-38`, `ref/repo/agentsview/internal/parser/antigravity_cli.go:30-48`） | 同 family 两 variants；read-only SQLite + optional existing-key decrypt + plaintext fallback | IDE/CLI DB full；brain/history partial；encrypted-only PB without `ANTIGRAVITY_KEY` locked，绝不请求/保存 key |
| Kimi | `~/.kimi/sessions` 与 `~/.kimi-code/sessions` 的 nested `wire.jsonl`（`ref/repo/agentsview/internal/parser/types.go:540-549`, `ref/repo/agentsview/internal/parser/kimi.go:56-96,158`） | 新 Kimi wire parser | legacy/kimi-code full；Kimi Work paths (`ref/repo/agentsview/internal/parser/types.go:552-570`) 明确排除 |

## Canonical identity

- `platform`/family 只使用八个稳定小写 ID：`grok`, `claude`, `codex`, `opencode`, `pi`, `omp`, `antigravity`, `kimi`。
- variant 解释 format/source 差异，不能裂成 UI 顶层 filter：例如 `antigravity-ide`, `antigravity-cli`, `opencode-storage`, `opencode-sqlite`, `kimi-legacy`, `kimi-code`。
- native session IDs 只在 provider 内解析；archive ID 是 opaque stable ID，避免路径和不同 provider 的同名 ID 冲突。

## Fixture and contract coverage

每个 provider fixture 必须验证：

1. canonical root discovery 只接纳合法 source shape，忽略噪声文件与越界 symlink/path。
2. quick state 未变更时不调用 `parse_summary()`。
3. summary 至少稳定提供 native session ID、family、variant、title/cwd（可缺省）、created/updated、message/tool counts 和 fidelity。
4. `read_message_page(latest/before, limit)` 有硬上限、稳定 cursor/ordinal、UTF-8 安全截断与 role/tool normalization。
5. source 删除/替换/member 不存在时返回 stale/missing，不读取任意替代路径。
6. parser 错误只输出 provider、opaque source/archive identity 和错误类别；fixture 断言日志/DTO 不含 raw message、secret 或 user path。

## Explicit non-claims

- 参考 Go 代码只作为格式和边界证据；CCR 不复制整套 provider framework，也不宣称与 AgentsView 全版本等价。
- fixture-backed canonical format 支持不等于所有历史/未来版本都 full；未知版本必须显示 partial/error。
- Antigravity 的 locked 状态是安全支持的一部分，不是成功解析；没有密钥时不能声称 transcript 完整。
- Kimi family 不包含 Kimi Work；旧 `gemini` archive 行也不因命名相近自动重标为 Antigravity。
