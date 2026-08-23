"""批次 6 一次性脚本：src/styles 消费侧 px → token 映射。

规则只做数值精确映射（var 引用值 == 原 px 值，根字号 16px 下 rem 换算精确）。
映射后仍含 px 的非注释行自动收集为豁免清单（hardcode-exemptions.md 的数据源）。
tokens.css 的 token 定义值不在映射范围（定义源头，登记为文件级豁免）。
"""

import json
import re
from pathlib import Path

STYLES = Path('src/styles')

MAP_RULES: list[tuple[str, str]] = [
    (r'\bborder: 1px solid', 'border: var(--space-px) solid'),
    (r'\bborder-top: 1px solid', 'border-top: var(--space-px) solid'),
    (r'\bborder-bottom: 1px solid', 'border-bottom: var(--space-px) solid'),
    (r'\bborder: 2px solid', 'border: var(--space-0-5) solid'),
    (r'\boutline: 2px solid', 'outline: var(--space-0-5) solid'),
    (r'\boutline-offset: 2px', 'outline-offset: var(--space-0-5)'),
    (r'\bgap: 16px', 'gap: var(--space-4)'),
    (r'\bgap: 12px', 'gap: var(--space-3)'),
    (r'\bgap: 10px', 'gap: var(--space-2-5)'),
    (r'\bgap: 8px', 'gap: var(--space-2)'),
    (r'\bgap: 6px', 'gap: var(--space-1-5)'),
    (r'\bmargin-top: 4px', 'margin-top: var(--space-1)'),
    (r'\bpadding: 48px 16px', 'padding: var(--space-12) var(--space-4)'),
    (r'\bpadding: 16px', 'padding: var(--space-4)'),
    (r'\bborder-radius: 12px', 'border-radius: var(--radius-2xl)'),
    (r'\bborder-radius: 8px', 'border-radius: var(--radius-lg)'),
    (r'\bborder-radius: 999px', 'border-radius: var(--radius-full)'),
    (r'\bborder-radius: 9999px', 'border-radius: var(--radius-full)'),
    (r'\bwidth: 64px', 'width: var(--space-16)'),
    (r'\bheight: 64px', 'height: var(--space-16)'),
    (r'\bwidth: 48px', 'width: var(--space-12)'),
    (r'\bheight: 48px', 'height: var(--space-12)'),
    (r'\bwidth: 32px', 'width: var(--space-8)'),
    (r'\bheight: 32px', 'height: var(--space-8)'),
    (r'\bwidth: 8px', 'width: var(--space-2)'),
    (r'\bheight: 8px', 'height: var(--space-2)'),
    (r'--home-card-radius: 12px', '--home-card-radius: var(--radius-2xl)'),
    (r'--home-divider-anchor-width: 32px', '--home-divider-anchor-width: var(--space-8)'),
]

PX_IN_CODE = re.compile(r'[0-9](?:\.[0-9]+)?px')
COMMENT_SPAN = re.compile(r'/\*[\s\S]*?\*/')

mapped: list[dict] = []
exempted: list[dict] = []

for css in sorted(STYLES.rglob('*.css')):
    if css.name == 'tokens.css':
        continue  # token 定义源头，文件级豁免
    lines = css.read_text(encoding='utf-8').splitlines(keepends=True)
    for index, line in enumerate(lines):
        code_only = COMMENT_SPAN.sub('', line)
        if not PX_IN_CODE.search(code_only):
            continue
        original = line.rstrip('\n')
        replaced = original
        for pattern, target in MAP_RULES:
            replaced = re.sub(pattern, target, replaced)
        if replaced != original:
            mapped.append({
                'file': str(css).replace('\\', '/'),
                'line': index + 1,
                'before': original.strip(),
                'after': replaced.strip(),
            })
            lines[index] = replaced + '\n'
        else:
            exempted.append({
                'file': str(css).replace('\\', '/'),
                'line': index + 1,
                'text': original.strip(),
            })
    css.write_text(''.join(lines), encoding='utf-8')

# tokens.css：删除与相邻 token 定义重复的「/* Npx */」标注行（注释，零渲染影响），
# 使 AC1 的 rg 计数只反映真实值。
tokens = STYLES / 'tokens.css'
token_lines = tokens.read_text(encoding='utf-8').splitlines(keepends=True)
removed_labels = 0
kept: list[str] = []
for line in token_lines:
    if re.fullmatch(r'\s*/\* [0-9.]+px \*/\s*', line):
        removed_labels += 1
        continue
    kept.append(line)
tokens.write_text(''.join(kept), encoding='utf-8')

print(json.dumps({
    'mapped_count': len(mapped),
    'exempted_count': len(exempted),
    'tokens_css_comment_labels_removed': removed_labels,
}, ensure_ascii=False))
out = Path('../.trellis/tasks/08-22-design-system/hardcode-transform-records.json')
out.write_text(json.dumps({'mapped': mapped, 'exempted': exempted}, ensure_ascii=False, indent=2), encoding='utf-8')
print(f'records -> {out}')
