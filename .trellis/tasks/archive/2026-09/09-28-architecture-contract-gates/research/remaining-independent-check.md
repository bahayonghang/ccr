# 本轮剩余门禁的非作者复核

更新时间：2026-09-29T02:14:29.003200+00:00。

状态：本轮非作者增量复核已完成并交接。已审查的 Windows 完整 CI 与原生交互存在失败；修复后原生运行及最终整体验收待完成。

## 范围与独立性

审查代理为 /root/implement_t09。使用 trellis-check 和 su-architecture-first；已读取本任务 check/implement context、规范、PRD、design 和 implement。仅写本报告与同名 JSON；本阶段没有执行 Cargo、bindings、前端构建或浏览器操作，没有修改产品、历史证据、矩阵和生命周期。

本报告复核其他作者的新增 delta 和 remaining-* 证据。本代理历史编写过 codex/fix.rs 的基础实现、Doctor 早期证据和 continuation-bindings-startup-*；这些历史内容不属于作者独立审查范围。后续 Doctor fixture 仅对其他作者新增 delta 做有限复核，不声明整文件独立。

本代理不是专职 trellis-check 角色派发。新 spawn 遭遇 thread-limit 的历史失败保留；root 后续成功唤醒既有 /root/check_t01 专职 checker，live agent 列表确认 running，专职审查路径已恢复。本报告不替代该 checker 的最终审查结论。

## 当前结论

- 两份临时输出脚本的 5 项替换通过有限复核。
- Windows frontend-check、Doctor focused/lib/Clippy、Linux tauri-ci 与 coverage-tauri 均已有可追踪通过证据。
- Windows CI 在 Doctor fixture 修复后仍退出 1：Tauri fast-version 测试的 5000ms 断言失败，原因未查明。
- 原生生产 custom-protocol 构建通过；第 3、4 次原生交互捕获共享确认弹窗 scroll-lock 样式缺少 nonce。修复后原生完整验收待提交。
- nonce 最小修复的 5 个文件已完成静态复核，未发现阻断问题；新构建与原生回归证据仍待提交。

## 输出脚本复核

两份保存的原始 base64、字节长度和 SHA 一致。当前代码只替换 5 个输出调用：desktop 3 处，insights 2 处。其余字节一致，浏览器连接、测量、等待、截图和路径未变。

提取真实原/新表达式，用 Node v26.7.0 的 vm、Console、Writable 对照 stdout/stderr，5/5 一致。覆盖中文、百分号、换行样本、JSON 缩进、标签和末尾换行。两份 node --check 和限定 ESLint 退出 0。仅执行输出表达式，不证明 Playwright、真实窗口或 pinned Node 24 行为。首次 reviewer harness 转义失败已记录；修正 harness 后执行成功，产品与阈值未改。

## 门禁证据

| 门禁 | 结果 | 已核验范围 |
| --- | --- | --- |
| Windows frontend-check | exit 0 / 182.238s | 168 smoke 文件、901 smoke 测试、24 i18n 测试，lint/type/cycles/边界/build/docs |
| Windows 初次 remaining CI | exit 1 / 174.077s | CLI 343 passed / 2 failed；Doctor fixture 找不到程序 |
| Doctor 空 PATH focused | exit 0 / 17.986s | 3 passed |
| Doctor 完整 CLI lib | exit 0 / 110.252s | 345 passed / 24 exports filtered，默认并行 |
| Doctor strict Clippy | exit 0 / 9.474s | all-targets / all-features |
| Linux tauri-ci | exit 0 / 209.223s | 397 行为测试、1 既有 ignored、2 guards、24+9+197 exports、1 inventory |
| Linux coverage-tauri | exit 0 / 202.864s | 397 行为测试、2 guards、覆盖率报告 |
| Windows CI after Doctor | exit 1 / 690.743s | Tauri 406 passed / 1 failed / 1 ignored / 197 filtered |
| Linux native custom-protocol build | exit 0 / 65.639s | 实际 dist、生产 CSP、debug automation binary |

各记录的原始日志 SHA 已核验，完整命令、路径、SHA 和实际测试数保存在同名 JSON。零匹配 test-binary 记录保持为零，不计为执行用例。早期失败和后续通过各自保留。

最新 Windows CI 已通过 workspace 测试、release build、audit 和 governance，随后在 commands::system::tests::cli_versions_fast_mode_returns_expected_shape 的 elapsed <= 5000ms 断言失败。记录未捕获该断言的精确 elapsed 值或当时调度。不能从日志断言具体调度原因，不能把该失败归为历史 exporter AV 或 Doctor PATH 问题。

## Doctor fixture 的有限复核

原字节 SHA 与历史冻结一致，测试模块之前的生产字节一致。其他作者的新 delta 仅调整两项 Windows 测试：使用既有 TestHostEnv/ENV_LOCK 清空并恢复 PATH，以绝对 SystemRoot PowerShell 和 PSHOME 子 PowerShell 运行 fixture。readiness、200ms deadline、父子进程及清理断言保留。

原 CI 可执行文件 SHA 与空 PATH 红例一致；红例 1 passed / 2 failed。修复后 focused 3 passed，完整 lib 345 passed，strict Clippy 通过。没有新增 serial annotation 或全局 test-threads 限制。14 个作者报告引用的证据 SHA 已核验。

已确认 fixture 对可变 PATH 的依赖。原 CI 时刻的 PATH 与锁归属没有采集，精确并发交错仍未证实。历史 STATUS_ACCESS_VIOLATION 原因未查明。

## 来源与覆盖率

remaining-source-freeze.json 的 SHA 为 46f5b43cbdcfb664480d9ce1d8f57f0e6a9fa51ef3cd492fe60c0674d597f447。263 个冻结路径包含 261 个实际文件及 2 个 null 删除标记。删除标记为旧 code-source-editor.css 和 config-source-panel.css，不能表述成 263 个实际文件。

来源关系为原始冻结、平台 codex_auth.rs cfg 导入修正、两份输出脚本修正、Windows Doctor fixture 修正。Linux 两项门禁、最新 Windows CI 与 native-build 的 before/after 映射均符合对应冻结。230 个 generated 类型的嵌入字节/hash 与前后快照一致。

覆盖率原始 JSON SHA 为 e347b83c3e5ec8527ed021c6a61f9800cf00fee4814cb9dc83f3064ce4b584a2。独立重算 gateway 为 654/703 = 93.0298719772404%，达到既定 85%。总体为 17929/34415 = 52.096469562690686%，属于 Tauri baseline，不计为 root workspace 总体 70% 门槛通过。

native-build 的 284 个 dist 路径在本次复核时逐项匹配 metadata。生产 CSP 配置 SHA 未变。该构建属于 nonce 修复前源码，未来改动必须使用新 source/bundle/binary 证据链。

## 原生工具及失败记录

静态检查提交 NCR-01 至 NCR-04，作者已修正：显式检查 WebDriver ready；断言 editor flex/13px/28rem 与 style CSP；session 关闭或活跃进程组残留使清理失败；urllib 使用 ProxyHandler({})。实际尝试已运行这些路径，但全流程原生验收尚未通过。

初次发现时未同时采集工具 SHA。edecb2b804f8d933053bbb174b293872aa99b07c8edce25ea57a772d957ba755 为作者完成上述修订后保存的 attempt1，不把该 SHA 标为原始缺陷版本。

| 尝试 | 结果与归属 |
| --- | --- |
| 1 | 等待 #root 超时；应用实际挂载 #app。主页 DOM 已加载，后续未执行。 |
| 2 | source 按钮 regex 遗漏实际“源文件”文案；native boot 已通过，后续未执行。 |
| 3 | 真实确认弹窗显示且 editorCount=0；编辑器 flex、13px、448px/28rem 正确；出现 style-src-elem，零违例断言失败。 |
| 4 | 增加诊断数据，确认在 CodeMirror 加载前共享弹窗已插入无 nonce 的 body[data-scroll-locked] 样式；sourceFile 为 base-modal-CepZGn4m.js。 |

第 3、4 次 CodeMirror 注入样式自身具有 nonce。零 style CSP 断言没有放宽。4 次尝试的 save 和 inline-script negative control 都未执行到，不能计为通过。每次 session 清理后记录均无活跃进程组成员，driver 经结束信号退出。已保存全部 attempt 脚本和记录 SHA；第 1、2 次原始 receipt 没有 harness_sha256 字段，后续保存脚本提供来源补充，不宣称当时已有该字段。

运行使用合成 HOME/USERPROFILE/XDG/CCR/Claude/Codex/llmusage 目录、限定 PATH 和 private D-Bus。环境重定向不等于操作系统文件系统隔离。本证据仅覆盖 Linux WebKitGTK/WSLg，不覆盖 Windows WebView2、macOS WKWebView、真实物理输入、硬件或发布安装包。

## nonce 修复责任层审查

最小责任层为 src/utils/cspNonce.ts 和 main.tsx 启动初始化；feature 文件保留 re-export。该方向符合 utils 低层依赖约束，避免共享 UI 依赖 feature。

本地 get-nonce@1.0.1 源码和 README 明确支持全局 __webpack_nonce__，react-style-singleton 在首个 style 创建时同步读取。源码未发现其他 setNonce/__webpack_nonce__ 写入。仅在页面 nonce 非空时用 Object.assign(globalThis, ...) 初始化该兼容入口，且在 createRoot/render 前执行，属于当前依赖链支持的机制；没有直接 import 传递依赖，也无须放宽 CSP。

须保留真实 ConfirmModal mount、无 nonce web、CodeMirror 回归；测试恢复全局属性并清理 modal。新原生构建须验证 scroll-lock style 的 nonce、sheet 有效、computed overflow、style CSP 零违例、inline-script 被拒绝、真实 UI save 与 fixture 字段保存，以及清理成功。

实现复核：utils、新 main 调用、feature 重导出、3 个 ConfirmModal 用例和 confirm-interaction-contracts 共 5 文件已读取。未发现阻断问题。main:24 的初始化早于 render:48；测试恢复原全局属性描述符，卸载 modal 后验证 scroll-lock 移除。package.json、bun.lock、tauri.conf.json 对应已冻结输入未变。

remaining-csp-focused.log（SHA 9cce2878424d184a1912a8a9594799864dc97430414c5530dd4a26781a8fc91c）记录 3 files / 7 tests 通过。原日志自身不含确切调用命令、测试文件名和退出码；后续 focused-final receipt 已补齐该信息并完成核验。本代理未重复运行测试。

局部最终证据已核验：focused-final 3 files / 7 tests、完整 type-check、限定 ESLint、production UI build 均 exit 0。前三项 receipt 的路径相对 ccr-ui，4 个源码 hash 前后和已审查内容一致。UI build 冻结 1060 个前端源码路径，前后无差异；284 个 dist hash 逐项一致。保留 513kB chunk-size 与 Vite plugin timing 警告。

初次 type-check 的 TS2550 保存在 remaining-csp-types.log：当前 compiler target 不支持 Object.hasOwn。作者将同一“全局属性不存在”断言改为 Object.getOwnPropertyDescriptor(...).toBeUndefined()，没有更改 compiler target 或放宽断言；复读更正后的测试与最终 receipts 一致。

新版 native harness 的 AST 通过。新增 build receipt SHA、scroll-lock nonce/sheet/rules/overflow、释放等待及 CodeMirror sheet/scroller flex/gutter 对齐断言，原 style 零违例和 inline-script 拒绝断言保留。NCR-05 已静态闭合：作者在磁盘 JSON 精确比对后等待编辑器关闭和源文件按钮恢复，再记录 styles_after_save 并断言为空。等待条件符合 ConfigSourcePanel:176–180 和 BaseSettings:49–52 的保存回调；终检版 harness SHA 为 475797bf8466010dabc223cf2f1fda3663487e6b55c2ca726901fa8f4c0da7c1，AST 通过。实际运行仍待提交。

## 保留边界

Windows aggregate 新失败的原因、修复和重跑仍待完成。共享样式 nonce 实现已静态复核，新冻结与原生验证待完成。macOS 必需矩阵保持开放；既有专职 checker 已恢复运行，最终审查结果待交接。原始 AV、前端 5 个 no-console、驱动 --version、Doctor CI 和 native 1–4 次失败全部保留。

增量复核截止：2026-09-29T02:22:37.315602+00:00。后续 Windows system.rs 修复、统一 source freeze、native rebuild/roundtrip 和专职终审由对应 owner 继续，本报告没有将尚未提交的结果计为通过。
