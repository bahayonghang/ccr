"""Render illustrative cell-based proposals, not a screenshot of the product.

Only fixture values are used. Run from any directory with Pillow installed.
"""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
from right_panel_preview import draw_panel, standalone

OUT = Path(__file__).resolve().parent
CW, CH, PAD = 10, 24, 16
COLS, ROWS = 200, 44
BG, TEXT, MUTED, RULE = '#1e1e2e', '#cdd6f4', '#a6adc8', '#585b70'
SURFACE, SELECT, INK = '#313244', '#cba6f7', '#11111b'
GOOD, WARN, BLUE, PEACH = '#a6e3a1', '#f9e2af', '#89b4fa', '#fab387'
font = ImageFont.truetype('C:/Windows/Fonts/consola.ttf', 18)
bold = ImageFont.truetype('C:/Windows/Fonts/consolab.ttf', 18)

def render(platform, accent, model, auth, variant="A"):
    img = Image.new('RGB', (COLS * CW + PAD * 2, ROWS * CH + PAD * 2), BG)
    d = ImageDraw.Draw(img)

    def text(x, y, value, color=TEXT, strong=False, width=None):
        value = str(value)
        if width is not None:
            value = value[:width]
        d.text((PAD + x * CW, PAD + y * CH), value, fill=color, font=bold if strong else font)

    def fill(x, y, w, h, color):
        d.rectangle((PAD+x*CW, PAD+y*CH, PAD+(x+w)*CW-1, PAD+(y+h)*CH-1), fill=color)

    def hr(x, y, w):
        d.line((PAD+x*CW, PAD+y*CH+CH//2, PAD+(x+w)*CW, PAD+y*CH+CH//2), fill=RULE)

    def box(x, y, w, h, label):
        d.rectangle((PAD+x*CW, PAD+y*CH+CH//2, PAD+(x+w-1)*CW, PAD+(y+h-1)*CH+CH//2), outline=RULE)
        fill(x+1, y, len(label)+2, 1, BG)
        text(x+2, y, label, accent, True)

    def pair(x, y, label, value, width=46, color=TEXT):
        text(x, y, label, MUTED, width=15)
        numeric = label in {'Switches', 'Input', 'Output', 'Reasoning', 'Cache read', 'Cache write'} and any(c.isdigit() for c in value)
        value_x = x+width-len(value) if numeric else x+16
        text(value_x, y, value, color, width=width-16)

    text(0, 0, 'DESIGN PROPOSAL  /  fixture data  /  text fallback  /  not a runtime screenshot', MUTED)
    box(0, 1, 200, 3, 'CCR')
    for i, name in enumerate(['Codex', 'Claude', 'Grok']):
        x = 3+i*25
        if name == platform:
            fill(x-1, 2, 21, 1, accent)
        text(x, 2, name+' Profile', INK if name == platform else MUTED, name == platform)
    text(91, 2, 'Codex Auth     |     Claude Auth     |     Grok Auth', MUTED)

    box(0, 4, 200, 4, 'Current runtime')
    for x, label in [(3, 'MODE'), (48, 'CURRENT PROFILE / MODEL'), (119, 'AUTH')]:
        text(x, 5, label, MUTED)
    text(3, 6, 'PROFILE ACTIVE', GOOD, True)
    text(48, 6, 'production  /  '+model, accent, True)
    text(119, 6, auth, BLUE, True)

    box(0, 9, 91, 32, platform+' Profiles (25)')
    fill(1, 10, 89, 1, SURFACE)
    for x, value in [(2,'ST'), (7,'PROFILE'), (32,'DESCRIPTION'), (55,'MODEL'), (76,'LAST USED')]:
        text(x, 10, value, MUTED, True)
    hr(1, 11, 89)
    for x in [5, 30, 53, 74]:
        d.line((PAD+x*CW, PAD+10*CH, PAD+x*CW, PAD+37*CH), fill=RULE)
    names = ['production', 'work', 'research', 'team', 'personal']
    names += ['profile-'+str(i).zfill(2) for i in range(6, 26)]
    for i, name in enumerate(names):
        y = 12+i
        selected = i == 1
        if selected:
            fill(1, y, 89, 1, SELECT)
        color = INK if selected else (GOOD if i == 0 else TEXT)
        text(2, y, ' *' if i == 0 else ('>o' if selected else ' o'), color)
        text(7, y, name, color, selected, 22)
        text(32, y, ['Main route', 'Daily development', 'Long context', 'Shared provider', 'Personal'][i%5], color if selected else MUTED, width=20)
        text(55, y, model, color if selected else TEXT, width=18)
        text(76, y, ['now', '2h ago', 'yesterday', 'Oct 08 10:42', 'No record'][i%5], color if selected else MUTED, width=13)
    text(2, 38, '* current   > selected   o available', MUTED)
    text(2, 39, 'Last used = last recorded successful activation', MUTED)

    draw_panel(img, platform, accent, model, variant=variant)
    text(2, 42, 'Tab switch   |   Up/Down select   |   Enter/Space apply   |   PgUp/PgDn details   |   r reload   |   Ctrl+L language   |   q quit', BLUE)
    path = OUT / (platform.lower()+'-design-v2-'+variant.lower()+'.png')
    img.save(path)
    print(path)

render('Codex', BLUE, 'gpt-example', 'provider key  /  environment')
render('Claude', PEACH, 'claude-example', 'API key  /  profile configuration')
render('Grok', TEXT, 'grok-example', 'environment key  /  responses backend')

render('Codex', BLUE, 'gpt-example', 'provider key  /  environment', 'B')
render('Codex', BLUE, 'gpt-example', 'provider key  /  environment', 'C')
for variant in ['A', 'B', 'C']:
    print(standalone('right-v2-'+variant.lower(), variant=variant))
for name,kw in [('right-v2-narrow', {'w':74,'h':36}), ('right-v2-short', {'w':60,'h':20}), ('right-v2-zh', {'zh':True}), ('right-v2-empty', {'empty':True})]:
    print(standalone(name, **kw))
print(standalone('right-v2-long', w=74, h=36, model='a-long-model-name-for-a-multilingual-profile-fixture', endpoint='https://long-provider-name.example.invalid/custom/compatible/responses/v1'))
