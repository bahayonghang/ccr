"""Render exported Ratatui TestBackend cells without reconstructing the layout.

Symbols, colors, modifiers and positions come from the actual Rust renderer.
These isolated-fixture buffers are not native terminal or personal-data captures.
"""
from pathlib import Path
import json
import re
import unicodedata
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent
OUT = ROOT / 'rendered-tui'
OUT.mkdir(exist_ok=True)
CW, CH, PAD, CAPTION = 10, 24, 16, 32
NORMAL = ImageFont.truetype('C:/Windows/Fonts/consola.ttf', 18)
BOLD = ImageFont.truetype('C:/Windows/Fonts/consolab.ttf', 18)
CJK = ImageFont.truetype('C:/Windows/Fonts/msyh.ttc', 17)
SYMBOL = ImageFont.truetype('C:/Windows/Fonts/seguisym.ttf', 18)

def rgb(value, fallback):
    match = re.fullmatch(r'Rgb\((\d+), (\d+), (\d+)\)', value)
    if match:
        return tuple(map(int, match.groups()))
    if value == 'Reset':
        return fallback
    raise ValueError(f'Unmapped terminal color: {value}')

for path in sorted((ROOT / 'render-cells').glob('*.json')):
    data = json.loads(path.read_text(encoding='utf-8'))
    assert data['fixture_only'] and data['renderer'] == 'Ratatui TestBackend'
    cols, rows, cells = data['width'], data['height'], data['cells']
    assert len(cells) == cols * rows
    light = 'latte' in path.stem
    bg, fg = ((239, 241, 245), (76, 79, 105)) if light else ((30, 30, 46), (205, 214, 244))
    img = Image.new('RGB', (cols * CW + 2 * PAD, rows * CH + 2 * PAD + CAPTION), bg)
    draw = ImageDraw.Draw(img)
    draw.text((PAD, 6), 'Actual Ratatui cells / isolated fixture / ' + path.stem, font=NORMAL, fill=fg)
    # Ratatui leaves a reset continuation after a wide grapheme. A terminal
    # writes the leading grapheme across both cells; do not paint that reset.
    continuation = set()
    for i, (symbol, _, _, _) in enumerate(cells):
        if i not in continuation and any(unicodedata.east_asian_width(c) in 'WF' for c in symbol):
            continuation.add(i + 1)
    for i, (symbol, _, background, _) in enumerate(cells):
        if i in continuation:
            continue
        x, y = PAD + (i % cols) * CW, PAD + CAPTION + (i // cols) * CH
        span = 2 if i + 1 in continuation else 1
        draw.rectangle((x, y, x + CW * span - 1, y + CH - 1), fill=rgb(background, bg))
    for i, (symbol, foreground, _, modifiers) in enumerate(cells):
        if i in continuation or not symbol.strip():
            continue
        x, y = PAD + (i % cols) * CW, PAD + CAPTION + (i // cols) * CH
        font = CJK if any(unicodedata.east_asian_width(c) in 'WF' for c in symbol) else (SYMBOL if any(0x2500 <= ord(c) <= 0x27ff for c in symbol) else (BOLD if modifiers & 1 else NORMAL))
        color = rgb(foreground, fg)
        if len(symbol) == 1 and 0x2588 <= ord(symbol) <= 0x258f:
            # Draw the exact Unicode eighth-block cell geometry; no font tofu.
            fraction = (0x2590 - ord(symbol)) / 8
            draw.rectangle((x, y, x + CW * fraction - 1, y + CH - 1), fill=color)
        else:
            draw.text((x, y), symbol, font=font, fill=color)
        if modifiers & 8:
            draw.line((x, y + CH - 4, x + CW - 1, y + CH - 4), fill=color)
    target = OUT / (path.stem + '.png')
    img.save(target)
    print(target.name)
