"""Apply the reviewed status-argument and field transformations for C3."""

import json
import re
from pathlib import Path
from review_calls import PATHS, calls

FIELDS = {
    '平台名称', '描述', '启用状态', '当前 Profile', '最后使用', '平台类型', '显示名称',
    '当前 profile', '当前平台', '配置文件', '默认配置', '当前配置', '默认 Profile', '使用平台',
    '最后更新', '导出路径', '导出文件', '总操作数', '总成本', '记录数', '时间范围',
    '文件', '格式', '行数', '清理日期', '备份目录', '清理策略', '保留文件', '将删除文件',
    '将释放空间', '命中数量', '扫描目录', '目标文件', '扫描范围', 'settings.json', '目标目录',
    'Trash root', 'Codex home', 'Backup', 'Provider at backup time', '后端', '前端',
    '检测到开发环境', '检测到用户目录版本', '建议迁移到新目录', '桌面端后端', '前端静态资源',
    '克隆仓库', '临时目录', '安装位置', '本地路径', '远程路径', 'Config file',
}
COLORS = re.compile(r'\.(?:bright_(?:red|green|yellow|blue|magenta|cyan|white|black)|red|green|yellow|blue|magenta|cyan|white|black|bold|dimmed|italic|underline)\(\)')
ICONS = re.compile(r'^(?:[✓✅✗❌⚠!×→🔄📊📋🌍💡💰📅🎫🏷🤖📁🏆📈📄⚡🎉🎯📝🔧📍🔍📦📥🏗🆕⏳🚀][\ufe0f\u20e3]?\s*)+')
changes = []

for filename in PATHS:
    path = Path(filename)
    original = path.read_text(encoding='utf-8')
    source = original
    for start, end, method, line in reversed(list(calls(original))):
        if method not in ('success', 'info', 'warning', 'error', 'step'):
            continue
        call = original[start:end]
        updated = COLORS.sub('', call)
        match = re.search(r'"([^"\\]*(?:\\.[^"\\]*)*)"', updated)
        if match:
            text = match.group(1)
            plain = ICONS.sub('', text)
            if method == 'warning':
                plain = re.sub(r'^警告[:：]\s*', '', plain)
            if method in ('success', 'error'):
                plain = re.sub(r'\s*[✓✗]$', '', plain)
            updated = updated[:match.start(1)] + plain + updated[match.end(1):]
        field = re.fullmatch(r'ColorOutput::info\(&format!\(\s*"([^"\\]+): ([^"\\]*(?:\\.[^"\\]*)*)"\s*,(.*)\)\)', updated, re.S)
        if field and field.group(1) in FIELDS:
            key, value, arguments = field.groups()
            updated = f'ColorOutput::key_value("{key}", &format!("{value}",{arguments}), 2)'
        if updated != call:
            source = source[:start] + updated + source[end:]
            changes.append({'file': filename, 'line': line, 'method': method,
                            'reason': 'plain message argument and marker deduplication' if not field
                            else 'indent field beneath result; preserve original value'})
    if source != original:
        path.write_text(source, encoding='utf-8', newline='')

Path('.trellis/tasks/10-06-cli-feedback-rollout/checks/initial-transforms.json').write_text(
    json.dumps(changes, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print('Transformed presentation calls:', len(changes))
