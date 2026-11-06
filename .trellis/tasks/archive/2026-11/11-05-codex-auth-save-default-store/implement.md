# Implementation and Validation

1. 阅读相关规范与所有默认存储判断调用点。新增 unit + binary 回归，用合成 ChatGPT 凭据复现缺省 store 保存失败，先运行失败版本并记录。
2. 修复共享默认值、缺失配置与错误配置区分，CLI save 错误返回与建议。保留安全写入及真实 home 不变。
3. 运行窄测试，然后 ccr-codex 与 ccr-cli 相关测试、ccr binary integration tests，保持默认并行和 --skip export_bindings。运行 cargo fmt --all -- --check 和相关 all-target clippy。跨 crate 最终 just ci；遇到无关失败记录，不擅自修复。
4. Trellis check 子代理审查并处理授权范围内缺陷；主会话同步 spec、记录准确测试和交付状态。
5. 编译可运行 CLI，使用隔离目录验证实际命令保存 khanh。不自动替换用户已安装 ccr，不执行真实账号保存；交付明确新二进制路径及使用方式。

授权：用户明确要求创建任务并修改代码确保当前条件可保存，后续明确要求提交相关改动并归档本任务。授权本地提交与归档，不包含 push。

## Completion receipt

All five execution steps completed. See `research/implementation-evidence.md`
for red-green receipts, `research/review.md` for independent review, and
`research/verification.md` for the full gate/environment recovery and release
smoke. No installed binary replacement or real-account mutation. Local commit
and archival are authorized by the user's follow-up; no push is authorized.
