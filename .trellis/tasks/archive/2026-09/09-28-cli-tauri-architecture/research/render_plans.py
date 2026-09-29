"""Render this audit's reviewed task data into Trellis planning documents."""
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
PARENT = HERE.parent
DATA = json.loads((HERE / 'task-designs.json').read_text(encoding='utf-8'))
BY_KEY = {item['key']: item for item in DATA}
LF = chr(10)

def task_dir(item):
    return ROOT / '.trellis/tasks' / ('09-28-' + item['slug'])

def write(path, lines):
    path.write_text(LF.join(lines) + LF, encoding='utf-8')

def link(key):
    item = BY_KEY[key]
    return f"{key} / 09-28-{item['slug']}"

for item in DATA:
    folder = task_dir(item)
    meta = json.loads((folder / 'task.json').read_text(encoding='utf-8'))
    if item['key'] in ['T05', 'T07']:
        meta['priority'] = 'P1'
    if item['key'] == 'T09':
        meta['title'] = '前端查询、编辑会话与错误状态契约'
        meta['description'] = '修复 Grok Auth 错误状态、Settings 草稿刷新及 Configs 语言订阅，保持前端分层。'
    meta['meta'].update({'architecture_task_key': item['key'],
                         'depends_on': ['09-28-' + BY_KEY[k]['slug'] for k in item['depends']],
                         'planning_only': True, 'review_baseline': '34d8a85e0e48b793733835e0304c8ed33940fcee'})
    meta['relatedFiles'] = [x for x in item['scope'] if (ROOT / x).exists()]
    (folder / 'task.json').write_text(json.dumps(meta, ensure_ascii=False, indent=2) + LF, encoding='utf-8')
    requirements = item['requirements']
    deps = '；'.join(link(x) for x in item['depends']) or '无子任务前置；仍须用户批准规划后才可实施。'
    prd = [f"# {item['key']}：{meta['title']}", '', '## 目标与用户价值', '', item['goal'], '',
           '## 已确认事实', '',
           '当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。',
           '- 问题映射：' + '、'.join(item['findings']) + '。',
           '- 代码证据：' + '；'.join('`' + x + '`' for x in item['evidence']) + '。', '',
           '## 范围和兼容约束', '',
           '- 覆盖：' + '；'.join(item['scope']) + '。',
           '- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。',
           '- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。',
           '- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。', '',
           '## 前置任务', '', deps, '',
           '父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。', '',
           '## 需求', '']
    for n, req in enumerate(requirements, 1):
        prd.append(f"- R{n}：{req['r']}")
    prd += ['', '## 验收标准', '']
    for n, req in enumerate(requirements, 1):
        prd.append(f"- [ ] AC{n}（R{n}）：{req['ac']}")
    prd += ['', '## 不在本任务范围', '',
            '生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。', '',
            '## 规划状态', '',
            '本任务保持 planning。当前需求无待用户裁定的阻断项；技术验证在 design.md 和 implement.md 中明确。此文档完成不表示产品问题已修复，也不授权 task.py start。']
    write(folder / 'prd.md', prd)
    design = [f"# {item['key']} 技术设计", '', '## 责任与权威来源', '',
              '本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。', '',
              '## 机制与逐项追溯', '',
              '| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |',
              '| --- | --- | --- |']
    for n, req in enumerate(requirements, 1):
        design.append(f"| R{n} / AC{n} | {req['mechanism'].replace('|', '/')} | {req['test'].replace('|', '/')} |")
    if item.get('design_appendix'):
        design += ['', '## 逐命令控制矩阵', '', 'R1/AC1 的强制逐 ID 策略、N/A 边界及 C01-C06 行为验收：`' + item['design_appendix'] + '`。该矩阵是本设计的一部分。']
    design += ['', '## 依赖和变更顺序', '', deps, '',
               '允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。', '',
               '## 兼容与取舍', '',
               '- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。',
               '- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。',
               '- 不用延迟、吞错误、无条件重试、额外 ROUND/默认值或外层 Promise timeout 掩盖不一致。',
               '- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。', '',
               '## 回滚边界', '', item['rollback'], '',
               '## 未验证边界', '',
               '本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。']
    # Remove a spreadsheet-specific phrase from the common code-review policy.
    design = [x.replace('额外 ROUND/默认值', '伪造默认值') for x in design]
    write(folder / 'design.md', design)
    steps = [f"# {item['key']} 实施顺序与验证", '', '## 启动条件', '',
             '- [ ] 用户批准父任务/本子任务的最新规划；执行前读取 applicable AGENTS 与 specs。',
             '- [ ] 核对前置任务：' + deps,
             '- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。',
             '- [ ] 依 Trellis 流程在获批后单独激活本子任务，本轮不得启动。', '',
             '## 有序实施', '']
    for n, req in enumerate(requirements, 1):
        steps += [f"- [ ] {n}. 固化 R{n}/AC{n} 的旧失败反例：{req['test']}",
                  f"- [ ] {n}.1 在明确 owner 内实现机制：{req['mechanism']}",
                  f"- [ ] {n}.2 运行行为断言并验证 AC{n}，保留兼容成功路径。"]
    steps += ['', '## 验证命令', '',
              '下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。', '', '```text',
              *item['commands'], 'git diff --check', '```', '',
              '## 交付与集成', '',
              '- [ ] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。',
              '- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。',
              '- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。',
              '- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。',
              '- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。', '',
              '## 失败和回滚', '', item['rollback'], '',
              '正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。']
    write(folder / 'implement.md', steps)
    research = ['.trellis/tasks/09-28-cli-tauri-architecture/research/architecture-audit.md',
                '.trellis/tasks/09-28-cli-tauri-architecture/research/governance-audit.md']
    if item['key'] in ['T01','T02','T03','T04','T05','T10']:
        research.append('.trellis/tasks/09-28-cli-tauri-architecture/research/cli-audit.md')
    if item['key'] in ['T02','T03','T05','T06','T07','T10','T11']:
        research.append('.trellis/tasks/09-28-cli-tauri-architecture/research/tauri-audit.md')
    if item['key'] in ['T03','T07','T08','T09','T10']:
        research.append('.trellis/tasks/09-28-cli-tauri-architecture/research/frontend-audit.md')
    if item.get('design_appendix'):
        research.append(item['design_appendix'])
    for name in ['implement.jsonl', 'check.jsonl']:
        entries = [{'file': p, 'reason': '本任务必须保留的现行领域契约'} for p in item['specs']]
        entries += [{'file': p, 'reason': '已验证发现、作用范围和反例验收'} for p in research]
        write(folder / name, [json.dumps(x, ensure_ascii=False) for x in entries])

parent_meta = json.loads((PARENT / 'task.json').read_text(encoding='utf-8'))
parent_meta['priority'] = 'P1'
parent_meta['meta'].update({'planning_only': True, 'review_baseline': '34d8a85e0e48b793733835e0304c8ed33940fcee',
                            'audit_report': str((HERE / 'architecture-audit.md').relative_to(ROOT)).replace(chr(92), '/'),
                            'implementation_authorized': False})
(PARENT / 'task.json').write_text(json.dumps(parent_meta, ensure_ascii=False, indent=2) + LF, encoding='utf-8')

prd = ['# CLI 与 Tauri 架构审查及分层重构', '', '## 目标与用户价值', '',
       '让 CLI、TUI 和桌面界面对相同配置/认证操作产生一致结果，让后台状态与真实执行一致，避免设置保存损失、错误空状态和无法恢复的任务界面。', '',
       '本轮用户授权的交付是深入审查与创建优化父子任务。审查和规划已落盘；下列需求/验收描述后续重构的完成标准，本轮不实施、不启动任务、不提交或归档。', '',
       '## 当前事实与审查依据', '',
       '- 基线：dev / 34d8a85e0e48b793733835e0304c8ed33940fcee，2026-09-28。',
       '- 完整报告：research/architecture-audit.md；三域原始报告和 governance-audit.md 保留精确引用、事实/推断/未验证分级。',
       '- 29 项原始记录合并为 20 组问题（9 P1、10 P2、1 P3）；另列未充分验证的 legacy、环境和性能风险。',
       '- 6 项隔离前端反例已复现；29 项现有相关 smoke 通过。Rust/原生故障未执行，不能预先判为通过。',
       '- 正式 lint:ci 因两个原有临时脚本的 5 条 no-console 失败；其他基线与受控诊断详见 research/baseline-*.json。', '',
       '## 范围', '',
       'CLI/TUI application 与领域调用、共享配置持久化、Tauri typed command 适配、usage/OAuth/command job 生命周期、React Settings/Auth/Commands/Configs 的用户操作链、相关规范和质量门禁。', '',
       '保留现有平台能力、磁盘格式、secret 脱敏、备份、CAS、ACL、确认、进程树管理、llmusage 外部 CLI 与 ccr-usage SQL owner；优先修复已证行为，按责任抽取模块。', '',
       '## 需求与子任务归属', '']
for item in DATA:
    number = int(item['key'][1:])
    prd.append(f"- R{number}：{item['goal']} 责任子任务 {link(item['key'])}。")
prd += ['', '## 父任务验收标准', '']
for item in DATA:
    number = int(item['key'][1:])
    prd.append(f"- [ ] AC{number}（R{number}）：{item['key']} 的全部行为验收通过，相关成功/失败/兼容路径附真实测试结果；报告中的对应问题及状态投影不存在未解释差异。")
prd += ['', '各 AC 的具体反例、观察值、机制和命令分别在对应子任务 prd/design/implement 中定义；父任务另执行跨入口集成矩阵，不把子任务文档存在当作修复完成。', '',
        '## 不在范围', '',
        '全仓物理拆包、全部 legacy IPC 改写、未证性能优化、新平台支持、全面视觉重做、真实用户数据试错、生产发布。checkin/sync/skills/数据库内部算法无逐函数重写范围。未充分验证项保留在报告 D01-D05。', '',
        '## 与已有工作的关系', '',
        '09-24-home-insights-frontend 保持 in_progress；本父任务不接管或归档。共享 eventBridge/query/dashboard 改动在实施时与原任务文件状态核对。两个原有 .tmp 脚本保留，不以删除或忽略它们使正式 lint 通过。', '',
        '## 规划状态', '',
        '11 个子任务均为 planning；本轮没有实施授权。无需要用户补充事实的阻断问题。方案取舍、环境限制和未验证项已记录；用户后续批准最新规划后再按依赖激活子任务。']
write(PARENT / 'prd.md', prd)

design = ['# 父任务技术设计', '', '## 关键决策', '',
          '1. 共享应用用例拥有完整业务规则和提交结果。先收敛现有 ccr-cli::application 与领域 service，再按实际依赖需求判断拆包；不新建全能 facade。',
          '2. Tauri handler 是 DTO/授权/事件适配器，不调用 CLI 终端 handler。通用工作台继续用受控 CCR sidecar，usage 同步继续用 llmusage CLI，usage SQL 继续由 ccr-usage 拥有。',
          '3. 配置仓储锁覆盖读取到提交，operation lock 与 guarded leaf lock 顺序固定。查询纯读，修复与初始化显式执行。',
          '4. 复合 profile 流程先 prepare，再执行，返回结构化 outcome。跨文件写使用版本保护补偿，部分提交明确显示恢复状态，不宣称 OS 级多文件原子性。',
          '5. job/controller 唯一提交终态。取消先到控制 owner，清理成功后确认 Cancelled；失败/超时/cleanup_failed 保留原义。',
          '6. Settings 原始 typed snapshot、dirty patch、锁/层/token、raw callbacks 由平台 descriptor 提供。Base 不按平台名分支；复用 editor 前修正共享层归属。',
          '7. Query snapshot、编辑草稿、后台任务投影与页面局部偏好分开；路由 mount 不决定后台生命周期，迟到数据不能覆盖新会话或终态。', '',
          '## 方案比较', '',
          '| 方案 | 判断与证据 |', '| --- | --- |',
          '| Tauri 全部启动 CLI | 不采用。已有服务可复用，CLI presentation 有永久迁移错误和 process::exit；子进程边界仍用于外部程序拥有的能力。 |',
          '| 一次拆出新业务大 crate | 延后。先证明共享用例和测试边界，避免仅移动 imports 而保留重复副作用。 |',
          '| 各页面局部修补状态 | 不采用。A13/A16/A17 需要对齐 state owner 与结果语义；只增加延迟或默认值不能满足反例。 |',
          '| 按现有 owner 分阶段重构 | 采用。保留 registry、ProcessGateway、guarded writer、Query、生成协议和现有平台能力。 |', '',
          '## 需求到机制、验证的映射', '',
          '| 父需求/验收 | 子任务机制权威 | 具体检查 |', '| --- | --- | --- |']
for item in DATA:
    number = int(item['key'][1:])
    design.append(f"| R{number}/AC{number} | ../09-28-{item['slug']}/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：{'、'.join(item['findings'])} |")
design += ['', '## 显式依赖', '', '| 子任务 | 必要前置 |', '| --- | --- |']
for item in DATA:
    design.append(f"| {link(item['key'])} | {'、'.join(item['depends']) or '无'} |")
design += ['',
           '第一阶段可分别推进 T01/T05/T06/T08；T01+T05 完成后进入 T02；T02 后进入 T03/T04；T05+T06 后进入 T11，继而 T07；T03+T08 后进入 T09；T10 最后。同文件实现不能并行覆盖。', '',
           '## 跨入口验证矩阵', '',
           '| 操作 | 入口 | 必须比较 |', '| --- | --- | --- |',
           '| profile apply/off/rename | CLI application、TUI adapter、Tauri service | runtime、profiles、registry、enabled、计数、history、结果与错误 |',
           '| config CRUD/读取 | CLI/service、Tauri、React Configs | 两进程 RMW、只读前后文件、显式平台、非法 patch 无写入 |',
           '| 诊断 | validate binary、共享 validator | auth-mode、unconfigured/invalid/unreadable、退出码、无隐式修复 |',
           '| 后台任务 | job service、stream executor、React route | admission、start/terminal 顺序、status 恢复、cancel、deadline、cleanup |',
           '| Settings | typed read、表单编辑、patch、reread | 通知 union、未知字段、托管锁、CAS、环境、dirty 草稿 |',
           '| OAuth | controller/storage/listener、Tauri adapter | secret 权限、bind、silent socket、cancel、HTTP deadline、唯一终态 |',
           '| 环境 | Local/WSL/SSH 按已支持能力 | 不支持操作明确拒绝，禁止向其他环境隐式回退 |', '',
           '## 兼容与回滚', '',
           '按单一领域完整调用链为提交批次；生成协议、registry 和消费者成组更新。旧公开命令保留明确兼容 adapter；无法确定平台时拒绝，不恢复隐式全局切换。自由配置保留 OpenJson；仅固定操作结果迁入具名类型。', '',
           '持久化不迁移用户配置 schema。备份命名变化兼容旧文件，禁止删除已有前镜像。跨文件回滚必须检查版本，不覆盖其他进程新写入。活动进程/登录需结束或受控交接后再替换实现。', '',
           '## 风险和明确延后', '',
           '公开 CcrError 变体、String-error command macro、旧配置和备份发现规则是兼容敏感面。规划不授权修改真实凭据、扩大环境支持或删除 legacy adapter。全量性能、原生视觉和外部账户行为没有本轮证据；详见审查 D01-D05。']
write(PARENT / 'design.md', design)

implementation = ['# 父任务执行与最终验收计划', '', '## 本轮已完成的规划交付', '',
                  '- [x] 阅读当前规范、依赖和代表调用链，保存三域研究与治理证据。',
                  '- [x] 执行只读基线和隔离前端反例，记录正式 lint 的已有阻断。',
                  '- [x] 将问题映射到 11 个子任务，建立 PRD、设计、执行计划和 context manifests。',
                  '- [x] 独立规划复核的六项修订闭合，12 个任务机械检查及上下文校验通过；结果见 research/planning-validation.md、plan-precheck.json 和 context-validation.json。', '',
                  '## 后续实施门槛', '',
                  '- [ ] 用户批准本次最新规划；未批准前不运行 task.py start、不派实施代理。',
                  '- [ ] 核对当时的 baseline、其他任务与工作区；父任务只协调，按子任务单独实施。',
                  '- [ ] 第一阶段：T01/T05/T06/T08。',
                  '- [ ] 第二阶段：T02、T11。',
                  '- [ ] 第三阶段：T03/T04/T07；T03 与 T08 就绪后 T09。',
                  '- [ ] 最后 T10：完整契约矩阵、规范与门禁收敛。', '',
                  '## 每个子任务的必需证据', '',
                  '对 R1/AC1 至 R11/AC11：逐项运行子任务全部验收，记录旧反例、修复测试、平台、测试数、退出码、commit、剩余限制。前置依赖通过对应契约后才进入下游。研究中的缺陷刻画测试要迁为正确行为回归并反转旧失败断言，不把旧缺陷测试通过当作已修复。', '',
                  '## 最终综合验证', '',
                  '以下命令仅在获批实施、处理原有工作区边界后执行：', '', '```text',
                  'just version-check', 'just fmt-check', 'just lint-strict', 'just test',
                  'just frontend-check', 'just tauri-ci', 'just ci', 'git diff --check', '```', '',
                  'just ci 有 version-sync/fmt 等改写；本轮未运行。最终门禁在 clean worktree 或明确保留用户改动的受控工作区执行。Formal lint、受控排除诊断、native、visual 分开记录；任何正式失败仍为失败，不降低规则或删用户文件。', '',
                  '## 完成条件', '',
                  '- [ ] 所有父 AC 和子 AC 通过；Local/WSL/SSH 按支持范围验证。',
                  '- [ ] Windows/Linux/macOS 的敏感权限与进程清理按相关契约通过。',
                  '- [ ] 受影响 Web 交互与视觉确认、原生桌面 smoke 完成；未验证不宣称发布就绪。',
                  '- [ ] 规范和生成物同步、独立检查通过、原有 Insights 任务和文件状态无越权变化。',
                  '- [ ] 依用户后续授权和 Trellis 流程处理提交与归档。']
write(PARENT / 'implement.md', implementation)
parent_specs = ['.trellis/spec/ccr/backend/module-decomposition.md', '.trellis/spec/ccr/backend/dependency-governance.md',
                '.trellis/spec/ccr/backend/desktop-command-policy.md', '.trellis/spec/ccr-ui/frontend/layering-contracts.md']
for name in ['implement.jsonl', 'check.jsonl']:
    entries = [{'file': p, 'reason': '父任务共享边界与综合验收规则'} for p in parent_specs]
    entries += [{'file': '.trellis/tasks/09-28-cli-tauri-architecture/research/' + p, 'reason': '原始证据与跨任务问题映射'}
                for p in ['architecture-audit.md','cli-audit.md','tauri-audit.md','frontend-audit.md','governance-audit.md']]
    write(PARENT / name, [json.dumps(x, ensure_ascii=False) for x in entries])
print('Rendered parent and', len(DATA), 'planning children; product code unchanged.')
