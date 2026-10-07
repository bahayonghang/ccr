"""Remove only imports and formats made redundant by the C3 migration."""

import re
from pathlib import Path
from review_calls import PATHS, calls

for filename in PATHS:
    path = Path(filename)
    original = path.read_text(encoding='utf-8')
    source = original
    for start, end, _, _line in reversed(list(calls(original, r'&(format)!\('))):
        call = original[start:end]
        matched = re.fullmatch(r'&format!\(\s*"\{\}",\s*(.*?)\s*,?\s*\)', call, re.S)
        if not matched:
            continue
        expression = matched.group(1).rstrip(',').strip()
        if expression.startswith('format!('):
            updated = '&' + expression
        elif expression.startswith('if '):
            updated = '&(' + expression + ').to_string()'
        else:
            updated = '&' + expression + '.to_string()'
        source = source[:start] + updated + source[end:]
    if not re.search(r'\.(?:(?:bright_)?(?:red|green|yellow|blue|cyan|white|black|magenta)|bold|dimmed|italic|underline|color)\(', source):
        source = re.sub(r'^use colored::(?:Colorize|\*);\n', '', source, flags=re.M)
    if source != original:
        path.write_text(source, encoding='utf-8', newline='')
print('Migration-created redundant formats and imports removed.')
