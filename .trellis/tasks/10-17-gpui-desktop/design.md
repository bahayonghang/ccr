# 技术设计

## 1. 决策与边界

目标为独立原生 GPUI 客户端；现有 CLI/Tauri 继续工作。第一阶段只交付可验证的原生基础，完整功能完成由父任务 AC5 决定。分层替换允许逐域回退，不做一次性整体后端重写。

候选依赖为 gpui-kit =0.7.3，其绑定的 gpui-pre =0.3.9 由 Kit 管理。实施当日重新核查最新兼容发布；不使用 wildcard、浮动 git main 或无记录的旧版本。独立 Cargo.lock 入库，所有验证 --locked。不开启 GPUI Fast，除非独立基准证明收益且兼容性通过；不为名称里的 Fast 默认付出第二图形栈成本。

## 2. 目录与依赖方向

```text
ccr-desktop/
  Cargo.toml                 # package/bin ccr-gpui，独立 [workspace]
  Cargo.lock                 # GPUI 应用解析图，提交版本库
  .cargo/config.toml         # target-dir 及必要平台配置
  justfile                   # 本地实现，根仅 desktop-* 转发
  DESIGN.md                  # 已从 style:codex 生成，原生规范
  README.md
  src/
    main.rs                  # 平台启动、受控退出
    app.rs                   # composition root，不含领域规则
    runtime/                 # Tokio 生命周期、typed 请求/事件
    theme/                   # 唯一 token 与 Kit Theme 映射
    shell/                   # 导航、快捷键、窗口状态
    features/                # 按迁移域逐步添加，禁止占位空模块
  tests/                     # 原生集成与 fixture
  benches/                   # 第 10 阶段完善
crates/ccr-app/               # 第 02 阶段起按需提取的框架无关编排
```

依赖方向：GPUI feature -> desktop runtime / ccr-app -> 已有领域 crates。Tauri command -> adapter / ccr-app -> 同一领域 crates。ccr-app 不依赖 ccr facade、tauri、gpui 或 UI 实体，不做通用插件框架。优先直接消费已有服务；只有确有两端共享的编排才进入 ccr-app。ccr-app 加入根 workspace，因此这部分必须跑根回归；GPUI 本身不进入根 members/default-members。

独立 workspace 避免 GUI feature/MSRV/图形系统依赖进入 CLI 默认构建。根 justfile 通过子 justfile 的工作目录进入新工程；该目录的 Cargo 配置因此能生效。仅在根传 --manifest-path 不等于切换 Cargo 配置搜索目录，不采用依赖该误解的缓存隔离方案。

Tauri package 保留 ccr-desktop，新 package/bin 为 ccr-gpui。新窗口标题 CCR Desktop (GPUI)，打包 identifier、可执行名和安装位置独立。新 UI 偏好保存在独立 desktop-gpui 子命名空间；业务配置与数据库继续由已有 API 定位，不复制用户数据。

## 3. 数据流与线程

UI action -> typed request {request_id, environment_id, platform_id, input} -> bounded executor -> domain result -> typed event/snapshot -> 对应 GPUI entity 局部更新。

- GPUI 主线程仅处理输入、布局、绘制与小规模状态变更；同步 DB/目录扫描/JSONL 解析移入有限并发 blocking worker。
- 长期持有一个 Tokio runtime；不每次点击新建 runtime，不在 render 中 block_on。
- 每个页面读取请求最多 1 个当前版本；环境/筛选切换增加 generation，晚到结果不得覆盖新上下文。
- 关闭视图取消只读任务订阅；已开始的有副作用任务由 app 级 owner 继续持有并展示状态，不能通过 drop 视图伪装取消。
- UI 进度事件每个任务最多合并发布 10 次/秒；终态必达，通知通道容量 128。快照用最新值替换，日志单独以有界队列处理，不用同一丢弃策略。
- 日志窗口保留至多 2,000 行/2 MiB，单行显示上限 8 KiB；溢出计数明确显示，业务终态不依赖日志。原始进程管道另遵循现有 gateway 上限。
- 首屏先呈现可交互 shell，再按需加载。隐藏页面不轮询；文件 watcher 去抖 200ms，不扫描全仓库。
- 每个列表使用 stable ID 与 virtualization；10,000 条 fixture 不能构造 10,000 个 UI row entity。筛选、排序和分页在查询层处理。

所有容量是新 adapter 的初始预算，与既有进程契约不能混同；更改预算必须给出数据。共享 job 状态机保留 admission、cancel_requested 和 immutable terminal。用量 cleanup 共享五秒预算，保持一小时生产 deadline，不用 UI 超时另写终态。

## 4. 服务提取

第一条纵向链选择配置列表读取：GPUI action -> ccr-app/已有 service -> ccr-config -> snapshot。验证线程边界、依赖隔离和错误显示后，后续子任务逐域扩展。

Tauri 内已存在 State-free usage、home_insights、agent_sessions；优先迁计算/编排，再在两个 adapter 中做映射。Tauri TS wire DTO/export 路径继续由 Tauri 持有，不能机械移动带 ts(export_to) 的结构体导致绑定漂移。ccr-usage 保持 SQL 唯一 owner。

可复用的进程 owner 使用 ccr-core ManagedProcess；命令 allowlist、风险确认、环境限制仍在应用入口。禁止字符串 shell passthrough。网络/命令取消只有 runner 完成清理后发 terminal。

## 5. 数据与安全兼容

复用现有 profiles.db、认证存储、同步、备份、CAS 与原子写 API，不设计第二数据库。读取、保存与应用 profile 是不同动作；失败不把 UI 的当前 profile 先切过去。多客户端测试必须使用独立 fixture root / home provider，禁止修改真实 HOME 或全局环境来串行化测试。

密钥默认脱敏，敏感编辑器只保留生命周期内明文，不写日志/诊断/截图。SSH trust 复用既有 host key/known hosts 确认和 SFTP 规则；WSL 使用结构化参数及路径转换。remote 不支持的本地用量/会话能力显示明确原因，不能落回本地数据造成误读。

## 6. UI 设计与通用性

权威规范为 ccr-desktop/DESIGN.md。纯 GPUI Rust theme/style，CSS 策略为不适用，不引入 HTML 样式层。单一语义 token 层映射 Kit Theme。系统原生字体与 CJK fallback，数据用等宽数字；导航为稳定的环境入口、功能分组、主列表、详情区。

区分三条兼容轴：操作系统 Windows/macOS/Linux；执行环境 Local/WSL/SSH；业务平台 descriptor 的实际 capabilities。不把“GPUI 支持三平台”解释成 CCR 已完成认证/托盘/IME 等测试。

## 7. 命令合同

下列均为待实施命令，不是当前已可运行入口。根转发子目录，并保留退出码。Windows PowerShell 和 Unix shell 均需验证。

| 根入口 | 行为 |
| --- | --- |
| desktop-help / desktop-doctor | 列命令；只读核查 Rust/链接器/GPU/系统库，不自动安装 |
| desktop-dev *args | cargo run --locked --bin ccr-gpui -- args；可见原生开发窗口 |
| desktop-build | cargo build --locked --release --bin ccr-gpui |
| desktop-build-debug | cargo build --locked --bin ccr-gpui |
| desktop-check | cargo check --locked --all-targets |
| desktop-fmt | 显式格式修复，不能被验证命令隐式调用 |
| desktop-fmt-check | cargo fmt --all -- --check |
| desktop-lint | cargo clippy --locked --all-targets -- -D warnings |
| desktop-test | cargo test --locked -- --skip export_bindings，默认并行 |
| desktop-test-ui | 只运行 native UI suite，启用 dev-dependency test-support |
| desktop-ci | fmt-check -> check -> lint -> test -> build，第一失败非零退出 |
| desktop-audit | 审计新 Cargo.lock，不修改全局工具 |
| desktop-bench / desktop-perf | 第 10 阶段接入固定 fixture 与指标报告 |
| desktop-package | 第 10 阶段生成当前平台本地产物，不发布 |
| desktop-clean | cargo clean 仅操作已验证的独立 target；不得指向根/Tauri target |

已有 build/dev/test/ui-*/tauri-* 语义保持。根 just ci 仍保留现有覆盖；第 10 阶段治理评估再将新检查纳入整体路由，不让旧 CLI 用户的普通构建依赖 GPUI。根版本同步的 PowerShell/Bash 两端同时增加新 manifest；不通过每次 build 偷改版本。

## 8. 验证与回退

每个迁移域有 contract fixture、native UI 状态测试、Tauri 对应回归和数据兼容检查。共享边界更改后按影响范围执行根/Tauri gate。正式最终验收运行 just ci 与 desktop-ci/audit/perf；已有测试并行策略、覆盖率阈值和 hosted required context 名保持。

原生 smoke 覆盖渲染可见文本、输入、关闭、focus、DPI 与目标平台；无显示设备的 headless pass 不作为截图证据。阶段失败可以停止新入口，Tauri/CLI 继续可用。若依赖/IME/GPU 硬阻塞，记录复现并重新评审，不能暗换框架或把 mock 标为完成。
