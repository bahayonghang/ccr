"""Inspect the approved non-auth presentation calls without running handlers."""

import json
import re
from pathlib import Path

PARENT = Path('.trellis/tasks/10-06-cli-output-presentation/research')
INVENTORY = json.loads((PARENT / 'output-callsite-inventory.json').read_text(encoding='utf-8'))
PATHS = [
    row['file'] for row in INVENTORY['files']
    if ('ccr-cli/' in row['file'] and '/auth/' not in row['file']
        and not row['file'].endswith('/grok/auth.rs'))
    or 'ccr-sync/' in row['file'] or 'ccr-codex/' in row['file']
]


def calls(source, pattern=r'ColorOutput::(\w+)\('):
    """Yield balanced ColorOutput call spans, retaining the original line."""
    for match in re.finditer(pattern, source):
        position = match.end()
        depth = 1
        quoted = False
        escaped = False
        while depth:
            char = source[position]
            if quoted:
                if escaped:
                    escaped = False
                elif char == '\\':
                    escaped = True
                elif char == '"':
                    quoted = False
            elif char == '"':
                quoted = True
            elif char == '(':
                depth += 1
            elif char == ')':
                depth -= 1
            position += 1
        yield (match.start(), position, match.group(1),
               source[:match.start()].count('\n') + 1)


if __name__ == '__main__':
    import sys
    filters = sys.argv[1:]
    for filename in PATHS:
        if filters and not any(fragment in filename for fragment in filters):
            continue
        source = Path(filename).read_text(encoding='utf-8')
        print('\nFILE', filename)
        for start, end, method, line in calls(source):
            if method in ('success', 'info', 'warning', 'error', 'step'):
                print(line, re.sub(r'\s+', ' ', source[start:end]))
