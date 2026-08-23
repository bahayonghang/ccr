"""批次 8：明暗对比度与迁移前同名 token 的取值一致性核对（AC8 证据生成）。

对比基准：批次 1 之前的 tokens.css（commit 98b08252，未经过设计体系任务改动）。
输出：逐名取值 diff（颜色类 token）+ 四组合 WCAG 对比度表 → contrast-parity.md。
"""

import re
import subprocess
from pathlib import Path

BASELINE_REF = '98b08252'
TOKENS = Path('ccr-ui/src/styles/tokens.css')
OUT = Path('.trellis/tasks/08-22-design-system/contrast-parity.md')

baseline = subprocess.run(
    ['git', 'show', f'{BASELINE_REF}:ccr-ui/src/styles/tokens.css'],
    capture_output=True, text=True, check=True,
).stdout
current = TOKENS.read_text(encoding='utf-8')

VAR_RE = re.compile(r'^\s*(--[a-z0-9-]+)\s*:\s*([^;]+);', re.M)
COLOR_VALUE_RE = re.compile(r'#[0-9a-fA-F]{3,8}\b|rgb|oklch|hsl')


def extract(source: str) -> dict[str, str]:
    source = re.sub(r'/\*[\s\S]*?\*/', '', source)
    return {name: value.strip() for name, value in VAR_RE.findall(source)
            if COLOR_VALUE_RE.search(value)}


base_vars = extract(baseline)
curr_vars = extract(current)

changed = {k: (base_vars[k], curr_vars[k]) for k in base_vars
           if k in curr_vars and base_vars[k] != curr_vars[k]}
removed = [k for k in base_vars if k not in curr_vars]
added = [k for k in curr_vars if k not in base_vars]


def parse_channel(token: float) -> float:
    scaled = token / 255
    return scaled / 12.92 if scaled <= 0.03928 else ((scaled + 0.055) / 1.055) ** 2.4


def hex_to_rgb(value: str) -> tuple[int, int, int]:
    value = value.strip()
    if value.startswith('#'):
        hex_part = value[1:]
        if len(hex_part) == 3:
            hex_part = ''.join(ch * 2 for ch in hex_part)
        return (int(hex_part[0:2], 16), int(hex_part[2:4], 16), int(hex_part[4:6], 16))
    rgb_match = re.match(r'rgba?\(\s*([0-9]+)[\s,]+([0-9]+)[\s,]+([0-9]+)', value)
    if rgb_match:
        return tuple(int(g) for g in rgb_match.groups())  # type: ignore[return-value]
    raise ValueError(f'unsupported color literal: {value}')


def luminance(value: str) -> float:
    r, g, b = hex_to_rgb(value)
    channels = [parse_channel(c) for c in (r, g, b)]
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]


def contrast(fg: str, bg: str) -> float:
    l1, l2 = sorted((luminance(fg), luminance(bg)), reverse=True)
    return (l1 + 0.05) / (l2 + 0.05)


def pick(var: str, theme: str, flavor: str) -> str:
    """按主题/flavor 组合解析变量的生效值（模拟层叠：root < dark < flavor < dark+flavor）。"""
    value = curr_vars.get(var)
    for selector in ('[data-theme="dark"]', f'[data-flavor="{flavor}"]',
                     f'[data-theme="dark"][data-flavor="{flavor}"]'):
        block_re = re.compile(
            re.escape(selector) + r'[^{]*\{[\s\S]*?\n\}')
        block = block_re.search(current)
        if block:
            m = re.search(re.escape(var) + r'\s*:\s*([^;]+);', block.group())
            if m:
                value = m.group(1).strip()
    assert value is not None, var
    return value


PAIRS = [
    ('--color-text-primary', '--color-bg-surface', 12.0),
    ('--color-text-secondary', '--color-bg-surface', 7.0),
    ('--color-text-muted', '--color-bg-surface', 4.5),
    ('--color-accent-primary', '--color-accent-primary-contrast', 3.5),
]

lines = [
    '# 明暗对比度与取值一致性核对（AC8 证据，批次 8）',
    '',
    f'- 基准：`git show {BASELINE_REF}:ccr-ui/src/styles/tokens.css`（批次 1 之前，未受设计体系任务影响）。',
    f'- 对比范围：两版本中取值含颜色字面量的全部变量（基线 {len(base_vars)} 个 / 现行 {len(curr_vars)} 个）。',
    f'- 取值差异：{len(changed)} 个；仅存在于基线：{len(removed)} 个；仅存在于现行：{len(added)} 个。',
]
if changed:
    lines += ['', '| 变量 | 基线值 | 现行值 |', '| --- | --- | --- |']
    lines += [f'| `{k}` | `{v[0]}` | `{v[1]}` |' for k, v in changed.items()]
if removed or added:
    lines += ['', f'- 移除名单：{", ".join(removed) if removed else "无"}',
              f'- 新增名单：{", ".join(added) if added else "无"}']

lines += ['', '## WCAG 对比度（四组合 × 契约色对，阈值同 theme-contrast-contract）', '',
          '| 组合 | 色对 | 对比度 | 阈值 | 判定 |', '| --- | --- | --- | --- | --- |']
all_pass = True
for theme in ('light', 'dark'):
    for flavor in ('neutral', 'clay'):
        combo = f'{theme}/{flavor}'
        for fg_var, bg_var, threshold in PAIRS:
            ratio = contrast(pick(fg_var, theme, flavor), pick(bg_var, theme, flavor))
            ok = ratio >= threshold
            all_pass = all_pass and ok
            lines.append(f'| {combo} | `{fg_var}` vs `{bg_var}` | {ratio:.2f}:1 | ≥{threshold}:1 | '
                         f'{"PASS" if ok else "FAIL"} |')

lines += ['', f'**结论**：{"四组合全部达标" if all_pass else "存在未达标项"}；'
          f'取值与迁移前{"一致" if not changed and not removed and not added else "存在差异（见上表）"}。']
OUT.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print('\n'.join(lines[:8]))
print('...')
print(lines[-1])
print(f'-> {OUT}')
