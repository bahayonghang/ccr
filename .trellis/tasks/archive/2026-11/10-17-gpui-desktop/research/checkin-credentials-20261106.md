# 签到账户编辑回填与捕获锁 — 2026-11-06

## 实际差距与实现

Tauri `AccountFormModal.openEditor` 通过 `get_checkin_account_cookies` 为选中账户回填原值。原生此前只提供留空保留，不等同于原值编辑。本批新增 editor-scoped `Services::checkin_account_credentials`，复用原 AccountManager/捕获数据库及 CryptoManager 解密 owner，不新增 SQL、加密协议或网络请求。

仅当前编辑器持有非 Debug/Serialize 的 Secret DTO 和 masked inputs。读取期间禁止编辑/保存，仍可取消或选择另一账户；每次打开独立 request nonce 与账户 ID，迟到响应不能填写新编辑器。返回值不进入 retained OperationState、管理列表、日志或偏好。取消/离页/成功保存释放编辑器及订阅；缺失/损坏密钥使用 `load_existing` 返回失败，不生成替代密钥。失败仍允许保留原值或显式填写替换值。

新测试暴露旧 Add/UpdateAccount 的 crypto.key 创建仍使用默认 LockManager。Storage 为 Fixture/Benchmark 捕获锁目录，仅对同步 domain 调用建立 scope，不跨 await/spawn。空导出提前在捕获 scope 初始化域密钥，再由原 exporter 使用其自己的 captured file-write scope；导入 commit 和 CDK credentials 同步更新亦绑定捕获锁。ImportSession 路径在读 owner 之前再次验证当前 fixture 范围，防止旧 session 先读取另一个目录。

Local 继续原锁解析；旧 Fixture 数据布局不改，增加其内部 `checkin-locks`。Benchmark 使用已验证的 layout locks/crypto/main DB。正常浏览列表仍不解密凭据或创建数据库。

## 已执行验证

- `cargo test --locked --offline --lib checkin -- --skip export_bindings`：**14 passed / 0 failed**，日志 `outputs/desktop/checkin-credentials-20261106-rerun3.log`。包含原 CRUD/导出导入/批量/真实受控 loopback/页面回归，以及新增选中账户读取、missing-key 不生成密钥、masked editor 回填/换账户/离页迟到响应、空 fixture 导出导入首次 crypto 创建。
- 同一旧 loopback 用例在受限沙箱中返回 Checked(Failed)，允许已授权 127.0.0.1 后同一 artifact 精确重跑通过。最终14项在允许 loopback 的环境完整通过；没有改测试预期或产品网络限制。
- 最初 runtime 新测试在自己的 Tokio 内调用同步 shutdown，断言通过后发生 nested-runtime panic；改为既有测试相同的 spawn_blocking shutdown，未修改产品生命周期。
- 既有 Windows 实际 **1 MiB + poisoned CCR_LOCK_DIR** 守卫通过，**22 个真实测试子进程**，包含新增凭据编辑控件和空 fixture transfer，19.85s。日志 `outputs/desktop/checkin-credentials-stack-20261106.log`。未增加栈或降低覆盖。
- 本批文件独立 rustfmt、`git diff --check` 通过。独立 check 与最终集中 GPUI 门禁待完成。

全部为自建临时 fixture/合成 headless 控件证据；未读取真实 Cookie/用户密钥，未请求提供商，未启动原生 GUI。原生验收仍按用户决定延期。

## Independent check follow-up

新editor的缺key失败回退仍触达旧 `AccountManager::update -> get_crypto ->
CryptoManager::new`，只改名字也生成替代key。独立检查的native和domain回归均
实际红于“key不应存在”。经主会话授权，修正在真实domain owner：metadata-only
更新不创建crypto；Cookie替换只load_existing，缺失/损坏key时失败且不写DB。
domain account_manager 9/9和strict Clippy通过；fresh native credentials过滤9/9
通过（其中本批相关3项），覆盖最终Session控件实际Save→重开、other Cookie保留、
advanced入口、缺key None元数据保存/Some拒绝、空fixture transfer。

这不代表其他旧 `CryptoManager::new` 消费点已完成缺key恢复策略：AddAccount、
Export/Import、CDK仍须各自审查。捕获锁与“不生成替代key”是不同合同。完整独立
检查见 `native-consumer-check-20261106.md`；最终整合门禁由主会话执行。

后续原生保护已实现并进入 ReadyForReview，见 `checkin-crypto-guard-20261106.md`。
缺失/不可读密钥现在明确要求恢复原密钥；不再提示填写替换凭据。metadata-only
编辑仍允许，Cookie 替换明确拒绝。上述早期通用失败提示与剩余消费点描述以新收据为准。
