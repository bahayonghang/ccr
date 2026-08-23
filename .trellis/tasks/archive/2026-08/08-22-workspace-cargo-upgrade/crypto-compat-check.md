# 加密依赖兼容性核查记录（AC9）

> 方法：按 design.md §3.1，三种持久格式在升级前用当前版本固化向量（含真实 salt/nonce/KDF 参数），升级后用新版本读回验证。摘要类依赖用固定输入摘要前后比对。向量不含真实凭据。
> 向量存储：`%TEMP%/ccr-crypto-vectors/`；临时测试文件不提交。

## 逐依赖结果

| 依赖 | 升级前 | 升级后 | 是否升级 | 三格式向量读回 | 摘要一致 | 结论 |
|---|---|---|---|---|---|---|
| base64 | 0.22.1 | 0.23.1 | 是 | 全部通过 | — | 接受 |
| sha2 | 0.11.0 | 0.11.0 | 否（已是最新稳定版） | 不适用 | 通过（复验） | 无变化 |
| blake3 | 1.8.5 | 1.8.7 | 是 | — | 固定输入 ×3 一致 | 接受 |
| rand | 0.10.1 | 0.10.2 | 是 | 全部通过 | — | 接受；调用点全部 OsRng（CSPRNG）fill_bytes，无弱生成器 |
| argon2 | 0.5 | 0.5.3 | 是 | codex 导出 + 同步信封通过 | — | 接受；Argon2id V0x13、KdfParams{65536,3,1} 行为不变 |
| aes-gcm | 0.10 | 0.10 | **否，保留** | — | — | 见下方保留原因 |

## aes-gcm 保留 0.10 的原因

- aes-gcm 0.11.x 基于 aead 0.6，移除了 `aes_gcm::aead::OsRng` 再导出（实测 `cargo check` 报 E0432，ccr-sync / ccr-codex 编译失败）。
- 继续升级必须改动 `crates/ccr-sync/src/sync/envelope.rs`、`crates/ccr-codex/src/services/codex_auth_crypto.rs`、`crates/ccr-checkin/src/core/crypto.rs` 的导入与类型路径，违反本任务 C3 约束（crates/** 源文件零功能改动）。
- 按「任一向量不匹配或无法无源码改动升级 → 保留当前版本」规则保留 0.10。lockfile 实际解析 0.10.3。

## 逐格式明细

| 格式 | 向量内容 | 生成版本状态 | 最终读回状态 |
|---|---|---|---|
| Codex auth 导出 | 完整导出信封 JSON（salt、KdfParams{m_cost=262144?, t_cost, p_cost}、version=2.0、format=encrypted、AAD 输入 exported_at=2026-08-23T12:00:00Z、account_count=1） | 升级前生成并自检通过 | base64 0.23 / blake3 1.8.7 / rand 0.10.2 / argon2 0.5.3 下解密明文一致，exit 0 |
| Sync 信封 V2 | 完整 envelope 字节（magic/version=2/algorithm/kdf/kdf_params/salt/nonce/metadata/ciphertext）+ PlaintextV1 样本 | 升级前生成并自检通过 | 同上，EncryptedV2 明文一致 + PlaintextV1 旧读路径命中 |
| CheckIn 凭据 | crypto.key（base64 32B，write_guarded 落盘）+ base64(nonce‖ciphertext) | 升级前生成并自检通过 | 同上，解出同一测试 API key |
| blake3 摘要 | 固定输入 ×3（含空串） | 基线落盘 blake3_digests.txt | 升级后逐字节一致 |
| sha256 摘要 | 固定输入 ×3（含空串） | 基线落盘 sha256_digests.txt | sha2 未升版本；复验一致 |

## 验证命令记录

- 每依赖升级后：`just test` exit 0；五项向量测试 exit 0。
- 组 C 收尾：`just secret-write-check` exit 0（Sensitive persistence policy check passed）。
