# 验收协议和本轮证据

## 当前状态（2026-10-26 实施后）

AG01–03 展示实现已应用，AG04 继续验收。完整证据见 `../10-26-ccr-ag04-acceptance/evidence/integration-summary.md`；原规划研究和原始失败／成功样本均保留。

| 检查 | 状态 |
|---|---|
| 方案批准、既有任务启动、设计／spec／sidecar同步 | PASS |
| before 及最终源码指纹／四组合代表页 | PASS，AG01 evidence |
| 完整前端测试、coverage、类型、lint、构建预算 | PASS；完整套件918项，后续修改另有定向记录 |
| 全 catalog Web 页面与重定向 | PASS，75条×亮暗=150访问；layout父记录单列 |
| 四组合窗口／减少透明度／材质层数 | PASS，32集成样本；无支持引擎仅源代码守卫 |
| 控件／有数据配置／实际焦点环 | PASS，AG02最新截图与计算样式 |
| 已覆盖路由与500条日志滚动性能 | PASS，稳定复采；Profiles此前22.8%波动轮次保留 |
| 真实确认关闭焦点恢复 | FAIL_BASELINE，四组合返回BODY，原处理未变 |
| 独立 check:i18n | FAIL_BASELINE，计数4501/4510漂移；主test的i18n通道通过 |
| 原生WebView／托盘／DPI、其他平台 | NOT_RUN，旧可执行文件及隔离缺口见native-feasibility |
| 浏览器200%缩放／真实无blur支持引擎／真实数据图表和虚拟列表性能 | NOT_RUN |
| 聚合frontend-check／just ci | NOT_RUN，含未授权安装；已有依赖下子门分别记录 |
| 提交／推送／发行／归档 | 未执行，不在授权范围 |

## 覆盖矩阵
全路由在light-neutral/dark-neutral核查；公共组件、首页、Profiles、Settings、Usage扩展到light/dark×neutral/clay四组合。先断言data-theme/flavor/resolved-flavor/accent，再采样，不能仅贴截图标签。

偏好覆盖空/有效/legacy/非法存储、system首帧和实时切换、有效/非法runtime custom accent、默认/存在/缺失字体回退。自定义accent的启发式对比色不能夸大为任意颜色WCAG合规。

窗口：1200×800、800×600、1440×900；Web另测390×844，959/960材质阈值与1023/1024导航阈值分别检查；200%缩放、原生100%/150%DPI。独立tray用实际popup尺寸，不套主窗口宽度。

状态：normal/loading/empty/error/disabled/selected/hover/focus/destructive。检查弹窗、菜单、移动侧栏、sticky滚动并发下有效blur≤1；reduce-motion/reduce-transparency/无blur支持下有真实降级证据，不能用普通截图冒充仿真。

数据：现有fixture、未知/零/空、有界最大列表、九来源统计；不用真实密钥/个人配置做截图。列表数据量和虚拟节点数按当前实现记录，不借用其他项目的一万条指标。

## 视觉与行为
截图标明路由/参数/状态/主题/flavor/窗口/版本/数据来源；包含稳定后的首屏和滚动后状态。长中英文、报错、焦点环、控制边界、portal定位、sticky遮挡、resize、图例均检查。验证实际页面标识，避免redirect/placeholder假通过。

Web fixture只能证明展示，不能证明真实IPC；原生独立PASS/FAIL/NOT_RUN。macOS等不可用平台明确未验，不用mock或Windows结果代替。核心原生验收未完成时不得宣布桌面最终验收完成。

## 对比度
内置四组合保留primary≥12、secondary≥7、muted≥4.5；普通文字至少4.5，大字和关键非文本边界/状态至少3。纸面浅边线不能替代控件边界。低对比图表继续提供精确表格和非颜色编码。

验证实底、alpha合成、渐变两端/中间/实际最差位置、modal遮罩及交互状态。目录亮色静态比例不等于CCR运行时或暗色合格。

## 性能协议（建议门槛，批准后先采基线）
同机器/运行模式/浏览器或WebView版本/窗口DPI/数据/主题；记录工作树指纹。warmup后每场景至少3轮，保留原始trace与汇总；噪声目标RSD≤15%，不稳定先重采。不能混合native与headless比较。

场景：首页滚动、长列表、Usage图表tab、弹窗开关、跨路由返回。记录route-ready、帧间隔p95、>50ms长任务累计时间、DOM数、blur数，比较三轮中位数：
- route-ready ≤ max(baseline×1.10, baseline+50ms)
- 帧间隔p95 ≤ max(baseline×1.10, baseline+2ms)
- 长任务累计 ≤ max(baseline×1.10, baseline+50ms)
- blur≤1、无逐行blur、无持续blur动画为硬门槛；稳定复现的新冻结/卡顿直接失败。

这些是建议回归预算，不表示原有绝对性能合格。已有perf脚本含Vue/旧路由/错误分支假设，AG01先校准选择器、真实滚动距离、目标页面和错误路径并成功采before；必要时最小修复验证脚本，不借机重做性能平台。AG04沿用同一方法复核after。没有成功before数据不能宣称无性能回归。

## 记录
命令附cwd、开始/结束、退出码、通过/失败/跳过数；未运行项写原因和下一步。证据放任务evidence/；原始日志不改写。验证规划文本空白时排除原始日志。禁止降低coverage、串行化既有并行测试或将缺证据写成PASS。

## 2026-10-27 提交归档

用户明确要求提交全部改动并归档相关任务。本任务树已按该指令归档，状态 completed 仅为归档工具的机械状态；meta.final_acceptance 仍为 open。上表的未测／失败项目不变。截图仅本地保留，原始日志与结构化证据随归档提交。
