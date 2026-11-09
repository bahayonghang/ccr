# Context manifest preflight

2026-09-28：root 对父任务和全部 11 个子任务执行 `task.py validate`，12 个返回码均为 0，路径引用可解析。

T06 的 implement/check manifest 各报告同一警告：`.trellis/spec/ccr/backend/llmusage-provider-adapter.md` 为 37537 bytes，超过 context_injection.max_file_bytes 32768；自动注入可能截断。该警告不能证明此前 reviewer 没有手动读取全文。

T10 规范收敛时应核对新增生命周期契约是否处于截断区域；需要拆分时只提取本次新增、独立的生命周期契约并保留明确索引，不调高全局注入上限，不重构无关 Trellis 引擎。最终重新验证相关 manifest 和规范链接。
