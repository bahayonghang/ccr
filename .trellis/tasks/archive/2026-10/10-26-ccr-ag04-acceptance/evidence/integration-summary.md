# 轻盈玻璃迁移：实施与验收记录

2026-10-26。AG01–03 的前端展示实现已应用；AG04 保持 `in_progress`，桌面最终验收未完成。未安装依赖、提交、推送、发行或归档。

## 已应用的改造

- 保留原主题／flavor／accent／字体存储和迁移，在现有令牌体系内增加八个专用场景／纸面变量。唯一纸面由 `.layout-shell::before` 承载，避免给固定导航增加新的 containing block。
- 静态薄荷至淡紫场景与亮暗四组合；卡片、表格、输入、图表、sticky 和浮层实底。959/960 材质断点与 1023/1024 导航断点独立。
- 公共控件、Profiles、配置／资源／MCP／SSH 和观测／Usage／日志／会话／作业／托盘的必要展示适配。保留草稿、请求、确认处理、图表生命周期、虚拟行估高与 IPC。
- 加强所有四种实底阅读面及纸面合成的文字 12/7/4.5 守卫，关键控件边界 ≥3、实际状态文字 ≥4.5，修正实色焦点环、配置筛选换行和刷新操作裁切。没有新增主题 provider、业务抽象或后端隔离功能。
- 同步 DESIGN、AGENTS、sidecar、跟踪的视觉技能来源和 theme spec；保留原始目录来源与研究快照。

## 检查与证据

| 检查 | 结果 | 记录 |
|---|---|---|
| `just version-check` / `just fmt-check` | PASS | `version-check.log`、`fmt-check.log` |
| `bun run test` | PASS，170 文件／918 项，含 i18n 测试通道 | `final-checks/test.log`、`final-checks/commands.json` |
| `bun run test:smoke --coverage` | PASS，行覆盖率 75.23%，保留 70% 门槛 | `final-checks/coverage.log`、`coverage-command.json` |
| 最后主题守卫 | PASS，13 文件／114 项 | `final-theme.log` |
| 最后公共焦点环 | PASS，19 文件／87 项；四组合 12 项实际焦点采样 | `final-checks/final-controls-*`；AG02 `evidence/focus/2026-10-26T16-55-14.532Z/` |
| 最后配置布局／字段 | PASS，12 文件／37 项；lint/type/build PASS | `final-config-layout-*`；最终单项 grid 修正 style/build PASS：`final-grid-*` |
| 环依赖／样式行数／前端文档审计 | PASS | `final-checks/check-cycles.log`、`check-style-lines.log`、`docs-audit.log` |
| 架构边界 | 原命令被坏 `bunx` 启动器阻断；本地等价检查 PASS，四个违规夹具均被拒绝 | `final-checks/arch-local-launcher.log` 与 adaptation receipts |
| 独立 `bun run check:i18n` | FAIL，既有计数期望 4501、两语言实际 4510；相关文件未修改 | `final-checks/check-i18n.log`；主 `bun run test` 的独立 i18n 测试通道通过，不能混为同一命令 |
| 全 catalog Web 渲染 | PASS，57 页面＋18 重定向，亮暗共 150 次；1 条布局父记录单列 | `web-complete/results.json`、父 `route-matrix.md` |
| 四组合窗口／透明度降级 | PASS，32 样本；包括 390/800/959/960/1023/1024/1440 宽度与 CDP 减少透明度 | `web-complete/results.json`；早期 AG01 28 样本另存 |
| 桌面侧栏键盘／指针 resize、移动定位／遮罩关闭、456×620 实底 tray | PASS，亮暗四组记录 | `shell-final/results.json`；只证明 Web 交互 |
| 最新配置控件／有数据页面 | PASS，四组合；1200/800/390 刷新按钮均在内容边界内 | AG02 `evidence/cards/2026-10-26T17-03-33.324Z/`，`refresh-viewport.log` |
| `just frontend-check` / `just ci` | NOT_RUN，含未授权的安装步骤 | 未修改聚合门；已有依赖下的实际子检查分别记录 |

完整测试和 coverage 在最终纯色值／焦点／布局修正前取得。后续修改已运行相应定向测试、类型、lint、构建和浏览器验证；不把旧的完整套件冒充最后一次源码全量重跑。最终审查见 `review-summary.md`。ESLint 两条既有 warning、构建的既有 `@tailwind` warning 未被隐藏。

## 性能范围

before、初次 after、final-after、final-settled 和必要的复采分别保存在 AG01 `evidence/`，未覆盖原始轮次。每轮保留 12 个路由样本、3 轮 500 条日志真实滚动、原始帧间隔、版本／工作树指纹和 20 张截图。对照与最终噪声结论见 `performance-comparison.json`。final-settled 中 Profiles RSD 22.8% 未通过噪声目标，该轮完整保留；同方法 final-confirmation 复采各路由 RSD 1.6%–4.8%，预算与噪声门均通过，未断言波动根因。路由中位数：首页 283.6→236.2ms、Profiles 262.3→220.6ms、Settings 256.5→223.7ms、Usage 259.1→226.4ms；日志滚动帧间隔 p95 7→7ms。

方法为同机、同 Chrome 157 headless／Vite watch:null、1200×800／DPR1／reduced-motion；路由耗时是预热后的整文档导航，不是 SPA 路由延迟。Profiles／Usage 在 Web 中是实际不可用状态。真实数据图表 tab、虚拟会话列表、弹窗与原生性能没有成功 before，仍为 NOT_RUN。没有 CPU trace 就不声称完成了 CPU trace 分析；所保留原始数据是计时、长任务观察和 rAF 帧间隔。

## 尚未关闭的验收

1. **确认框焦点恢复：FAIL_BASELINE。** 实际配置卡片 Switch → Escape 在四组合均把焦点留在 BODY，等待两秒仍未恢复。BaseModal 和相关关闭处理未被视觉工作修改。焦点进入、Tab 留在弹窗、Esc 取消和实色 focus ring 已通过，但不等于焦点恢复通过。证据：AG02 `evidence/populated/2026-10-26T16-46-23.032Z/`。
2. **原生 Windows DPI／系统主题／托盘，以及 macOS/Linux：NOT_RUN。** 已有可执行文件均早于本次改造；Tauri 启动会访问真实目录迁移、数据库和偏好，现有环境变量不构成完整隔离。没有启动旧程序冒充新成果，也没有在视觉任务内新增后端隔离开关。精确哈希、路径和隔离验证途径见 `native-feasibility.md`。
3. **真实浏览器 200% 缩放和无 backdrop-filter 支持引擎：NOT_RUN。** 已完成真实小窗口／断点／减少透明度；不把它们当作不同环境的验收。无支持分支有源代码守卫，实际不支持引擎尚未运行。
4. **既有独立 i18n 检查漂移与缺失翻译 key：仍存在。** 包含实际确认标题显示 key 的现象；不通过修改期望或屏蔽检查把它改写成 PASS。

R1/R2 的已覆盖 Web 外观／材质、R4 自动化偏好合同、R8 范围保护已有证据；R3 全状态、R5 焦点／原生、R6 全环境和 R7 全场景性能仍部分开放。因此父任务与四子任务均保持 `in_progress`，没有归档或声明桌面最终验收完成。

## 可复核性

本机最初的 Scoop 启动器失败后，使用已安装的 Bun 1.4.4、Just 1.58.0 和 Rust 1.98.0 实际路径，仅改变检查进程 PATH。未安装或改写机器工具链／锁文件／聚合命令。所有测试保留既有并行与 coverage 阈值。

最初的 CDP 媒体采样及移动 transform 采样发生在渲染器应用新样式前；修正为等待实际计算样式后通过，初次失败记录仍保留。配置页 800px 裁切则是实际 grid 子项自动最小宽度问题，已在本地 workspace 角色加 `min-width:0` 后重新截图通过；没有把真实问题归咎于采样。

最终范围复核：`final-scope.json` 记录80个跟踪文件差异；没有Rust/GPUI/VSCode运行时改动。最终前端diff指纹与稳定性能复采完全一致。`git diff --check`与AG04上下文校验通过；任务预览服务器和本轮浏览器均已关闭。

## 2026-10-27 用户授权的归档

用户已明确授权忽略临时 PNG、拆分提交全部修改并归档父任务及四个子任务，故不再保持活动任务状态。历史 status/in_progress/未提交/未归档叙述记录上轮状态；当前为已提交并行政归档，final_acceptance=open，所有原 FAIL_BASELINE/NOT_RUN 项不变。临时截图保留在本地归档目录，新的克隆只包含文件名、文本记录和原始日志；归档不是桌面全量验收通过。参见 ../closure.md。
