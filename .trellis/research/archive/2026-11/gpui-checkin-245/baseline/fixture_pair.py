"""Check-in v1 synthetic fixture. Produce it under an empty, disposable directory."""
import hashlib
from contextlib import closing
import json
import re
import sqlite3
import subprocess
from pathlib import Path

BASE_SHA = '4f0a02387a221f9f1822eede2787ebe050aa75da'
FIXTURE_ID = 'checkin-pair-v1'
STATES = ('unconfigured', 'configured')
SIZES = {'wide': (1280, 800), 'narrow': (960, 640)}
STAMP = '2026-11-20T08:30:00+00:00'
PROVIDERS = (('p-aurora', '极光演示站'), ('p-harbor', 'Harbor Demo'))
ACCOUNTS = (
    ('a-morning', 'p-aurora', '晨光测试账户', 1, '2026-11-22T10:00:00+00:00', STAMP, STAMP),
    ('a-long', 'p-harbor', 'Harbor International Research Sandbox', 1, '2026-11-21T10:00:00+00:00', None, STAMP),
    ('a-paused', 'p-aurora', '备用账户', 0, '2026-11-20T10:00:00+00:00', None, None),
)
BALANCES = (('b-morning', 'a-morning', 80., 17.5, 62.5, 'USD', STAMP),
            ('b-long', 'a-long', 20., 20., 0., 'CNY', STAMP))

def digest(path):
    if not path.exists():
        return None
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def make(root, repo, state, size, theme):
    assert state in STATES and size in SIZES and theme in ('light', 'dark')
    root.mkdir()
    config = root / 'desktop-gpui'
    config.mkdir()
    w, h = SIZES[size]
    prefs = dict(version=1, language='zh', theme=theme, confirm_quit=False,
                 environment='local', ui_font_size=14, mono_font_size=12,
                 window=dict(x=20, y=40, width=w, height=h, maximized=False))
    (config / 'preferences.json').write_text(json.dumps(prefs), encoding='utf8')
    # The paired paths explicitly select the ordinary fixture source. There is
    # deliberately no profiles catalog or registry. This page needs neither.
    if state == 'configured':
        schema = subprocess.check_output(['git', 'show', BASE_SHA + ':crates/ccr-db/src/database/schema.rs'],
                                         cwd=repo, encoding='utf8')
        sql = re.search(r'pub const CREATE_TABLES_SQL: &str = r#"(.*?)"#;', schema, re.S).group(1)
        with closing(sqlite3.connect(root / 'checkin.db')) as db, db:
            db.execute('PRAGMA journal_mode=DELETE')
            db.executescript(sql)
            db.executemany('INSERT INTO checkin_providers VALUES(?,?,?,?,?,?,?,?,?,?,?,?)',
                           [(i, n, 'https://example.invalid', '/checkin', '/balance', '/user', '', '', 1, STAMP, None, None)
                            for i, n in PROVIDERS])
            db.executemany('INSERT INTO checkin_accounts VALUES(?,?,?,?,?,?,?,?,?,?,?)',
                           [(i, p, n, '', '', on, created, None, cin, bal, '{}')
                            for i, p, n, on, created, cin, bal in ACCOUNTS])
            db.executemany('INSERT INTO checkin_balances VALUES(?,?,?,?,?,?,?)', BALANCES)
    return ['--fixture-profiles', str(root/'profiles.toml'), '--fixture-registry', str(root/'config.toml')]

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--root', type=Path, required=True, help='New child path below an existing temporary directory')
    parser.add_argument('--state', choices=STATES, required=True)
    parser.add_argument('--size', choices=SIZES, required=True)
    parser.add_argument('--theme', choices=('light','dark'), required=True)
    args = parser.parse_args()
    declared = make(args.root.resolve(), args.repo, args.state, args.size, args.theme)
    print(json.dumps(dict(fixture_id=FIXTURE_ID, launch_args=declared, db_sha256=digest(args.root/'checkin.db')), indent=2))
