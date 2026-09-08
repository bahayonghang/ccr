# Grok 多账号总设计

保存采用 Codex 式只读复制语义：Grok 正在运行时也能保存，副本写入 CCR 的 Grok 目录，不要求退出、不修改当前账号。切换已由用户确定为先结束当前 Grok，再切换，供后续新会话使用。具体服务机制见子任务 09-08-grok-auth-account-service/design.md，界面见 09-08-grok-auth-multi-account-tui/design.md。实施授权已记录，本会话已开始实施。

## 层次与信任边界

```text
主 TUI / GrokAuthApp
  └─ secret-free snapshot + 按用户操作提交
      └─ GrokAuthService（ccr-cli）
          ├─ 单文件 CCR accounts.json：账号别名 -> 单 scope 凭据
          ├─ 原生 auth.json：只替换选中 scope
          ├─ 既有 auth_off 写核：独立登出
          └─ ccr-core：Secret / 操作锁 / 原子写 / CAS / 文件权限
```

不引入 ccr-store 数据库、上游 Rust 依赖或通用认证框架。Grok profile 服务继续拥有 config.toml，账号服务不替换它。

## 操作契约

| 动作 | CCR 保存库 | 原生 auth.json | 其他文件 |
| --- | --- | --- | --- |
| 查看 / r | 只读 | 只读 | 只读 activation，不解析 token 到 UI |
| s 保存 | 增加或明确覆盖一项 | 只读复制，可在 Grok 运行时执行 | 不改 Grok 文件或官方锁，不登录/登出/刷新 |
| Enter 切换 | 先更新可识别的原账号最新凭据 | 只替换目标 scope；保留其余条目 | profile/MCP 不改 |
| d 删除 | 只删选中保存项 | 不改 | 不改 |
| o 登出 | 已识别账号先回存，其余保存项不变 | 保留原共享写核的全文件删除语义 | profile/MCP 不改 |

所有修改接口都在服务层重读/验证。保存/切换确认期间 runtime 或账号库发生变化时，拒绝旧请求并要求刷新；不得只根据 UI 当前选中项写盘。

## 最小存储决策

新文件固定在 `<CCR_ROOT>/platforms/grok/auth/accounts.json`，不新增用户配置选项。每个别名项存 saved_at、scope 和一个 Secret 包装的完整 credential JSON 文本；列表字段/身份比较从该凭据派生。没有旁路 registry/current 指针，没有额外备份轮换或 last-used 日志。

默认位置为 `~/.ccr/platforms/grok/auth/accounts.json`，沿用项目 CCR 根目录解析。与 Codex 对齐的是保存时复制当前凭据、原账号继续使用，以及 CCR 平台目录归属；Grok 原生文件包含多个 scope，因此取选定 scope 的完整副本保存。

保存的是一个认证范围中的凭据对象，不能把 auth.json 全文件当账号写回。个人/团队来自官方生产 OAuth，企业 OIDC/external/API-key 不进入管理范围，仍保留在 runtime 中。多个可保存官方 scope 时让用户选择来源，零个显示原生登录指引，不猜测当前有效配置。

## 界面信息结构

```text
Codex Profile | Claude Code | Grok Profile | Codex Auth | Claude Auth | Grok Auth
┌─ 账号 ────────────────────────┐ ┌─ 选中账号 ──────────────────────┐
│ 名称       本地匹配    凭据状态 │ │ work                            │
│ personal              已保存   │ │ 邮箱/团队：仅展示本地已有字段     │
│ work       ●          已保存   │ │ 保存时间 / 到期信息（若有）      │
│                              │ ├─ 当前会话 ──────────────────────┤
│                              │ │ 本地官方 scope 匹配：work        │
│                              │ │ 实际请求认证：未验证             │
│                              │ ├─ 操作结果 / 下一步 ─────────────┤
│                              │ │ 已保存到 CCR；当前账号保持使用    │
└──────────────────────────────┘ └─────────────────────────────────┘
Keys  ↑↓/jk 选择 | Enter 切换 | s 保存 | d 删除 | o 登出 | r 刷新
      Tab/Shift+Tab 换页 | Ctrl+L 语言 | q 退出
```

不渲染 key/refresh token，不以邮箱单独认定账号；无字段用未知，不编造 5h/7d 或 token 统计。显示“本地匹配”而非“登录有效”。第三方 profile 保持，并提示当前路线可能优先于官方会话；不自动 profile off。

## 一致性和风险取舍

将最新原账号保存成功作为 runtime 覆盖前置。原账号回存成功但 runtime 写失败时，保存库更新保留，这是保留新凭据，不回滚成旧 token。当前状态由重新读取推导，不存在“runtime 已变但 registry 仍指向旧账号”的第二真相。

保存只使用 CCR 自身操作锁和账号库原子写，不获取或创建官方 auth.json.lock，不阻挡 Grok 的原生刷新；保存的是读取时的凭据快照，不承诺冻结后续刷新。切换/登出写 runtime 时才获取官方锁；锁和 CAS 顺序见服务设计。停止后切换是已确定的使用前提，不适用于保存；由用户自行停止 Grok，CCR 不检测或终止进程。官方不守锁路径及热加载不能用“多加一次探测”抹平。

所有 token 时间和未知字段保留。过期只表示本地元数据已过期，是否可由官方 refresh 成功未知；CCR 不联系 token endpoint。明确 switch 可恢复已保存的过期条目，但提示需官方重新认证的可能，不伪造续期。

## 回退和权限

只修改本任务代码。无用户全局目录扫描、凭据迁移、旧配置回填。运行时写失败沿用 ccr-core 保留原目标语义，写后失败按实际回读说明“已应用但持久化/刷新未确认”；不盲目覆盖外部新内容作回滚。

新 secret 文件的 Windows owner-only DACL 与 Unix 0600 在写 secret 内容前设置；已有目标严格权限保留。只补 ccr-core 现有 secret 写入分支，不增加新公开配置或权限策略层。

## 验证

子任务通过源码审查覆盖删除边界、服务/文件错误与交互设计。按用户最新要求，本轮不运行测试、just ci 或真实 Grok 验证；测试源码可按功能改动维护，但不执行。静态审查、未执行用例和真实新会话证据分别记录。不得操作当前 Grok Build 进程、真实凭据/config/profile/MCP，不安装或替换其二进制，不启动登录/登出/切换验证。
