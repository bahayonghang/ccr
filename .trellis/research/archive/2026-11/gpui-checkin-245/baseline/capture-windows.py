"""Repeat native first-screen captures on a private (never switched-to) Windows desktop."""
import argparse
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import traceback
import uuid

import win32api
import pywintypes
import win32con
import win32event
import win32gui
import win32process
import win32service
import win32ui
from PIL import Image, ImageStat
from fixture_pair import ACCOUNTS, BASE_SHA, FIXTURE_ID, SIZES, STATES, digest, make

HERE = Path(__file__).resolve().parent
TITLE = 'CCR Desktop (GPUI)'
TARGET_FILES = ('checkin.db', 'checkin.db-wal', 'checkin.db-shm', 'checkin-crypto', 'checkin-migration-home', 'checkin-locks')
user = ctypes.windll.user32

def owned_window(pid):
    found = []
    def inspect(hwnd, _):
        if win32process.GetWindowThreadProcessId(hwnd)[1] == pid and win32gui.GetWindowText(hwnd) == TITLE and win32gui.IsWindowVisible(hwnd):
            found.append(hwnd)
    try:
        win32api.SetLastError(0)
        win32gui.EnumWindows(inspect, None)
    except pywintypes.error as err:
        # An empty private desktop or a window that just closed can race
        # enumeration between processes. The caller re-enumerates until ready.
        if err.winerror not in (0, 122, 1400):
            raise
    if len(found) > 1:
        raise RuntimeError('Two windows match the private process')
    return found[0] if found else None

def uia(hwnd, open_page=False):
    command = ['powershell.exe', '-NoProfile', '-NonInteractive',
               '-File', str(HERE/'window-tree.ps1'), str(hwnd)]
    if open_page:
        command.append('-Open')
    result = subprocess.run(command, capture_output=True, encoding='utf8', timeout=10,
                            creationflags=subprocess.CREATE_NO_WINDOW, check=True)
    return json.loads(result.stdout) if result.stdout.lstrip().startswith('[') else []

def capture(hwnd, path):
    outer = win32gui.GetWindowRect(hwnd)
    frame = wintypes.RECT()
    hr = ctypes.windll.dwmapi.DwmGetWindowAttribute(hwnd, 9, ctypes.byref(frame), ctypes.sizeof(frame))
    assert hr == 0
    w, h = outer[2]-outer[0], outer[3]-outer[1]
    dc = win32gui.GetWindowDC(hwnd)
    src = win32ui.CreateDCFromHandle(dc)
    dst = src.CreateCompatibleDC()
    bitmap = win32ui.CreateBitmap()
    bitmap.CreateCompatibleBitmap(src, w, h)
    dst.SelectObject(bitmap)
    try:
        assert user.PrintWindow(hwnd, dst.GetSafeHdc(), 2), 'PrintWindow failed'
        raw = Image.frombytes('RGB', (w, h), bitmap.GetBitmapBits(True), 'raw', 'BGRX')
        image = raw.crop((frame.left-outer[0], frame.top-outer[1], frame.right-outer[0], frame.bottom-outer[1]))
        assert max(ImageStat.Stat(image).stddev) > 8, 'Nearly blank image'
        image.save(path)
        return list(image.size)
    finally:
        dst.DeleteDC()
        src.DeleteDC()
        win32gui.ReleaseDC(hwnd, dc)
        win32gui.DeleteObject(bitmap.GetHandle())

def resize(hwnd, desired):
    _, _, w, h = win32gui.GetClientRect(hwnd)
    if (w,h) != desired:
        l,t,r,b = win32gui.GetWindowRect(hwnd)
        win32gui.SetWindowPos(hwnd, 0, 0, 0, r-l-w+desired[0], b-t-h+desired[1],
                              win32con.SWP_NOMOVE|win32con.SWP_NOACTIVATE|win32con.SWP_NOZORDER)
        time.sleep(.7)
    assert win32gui.GetClientRect(hwnd)[2:] == desired, (win32gui.GetClientRect(hwnd), desired)

def capture_case(args, state, size, theme):
    case = f'{args.prefix}-{state}-{size}-{theme}'
    with tempfile.TemporaryDirectory(prefix='ccr-checkin-pair-') as tmp:
        root = Path(tmp)/'fixture'
        appargs = make(root, args.repo, state, size, theme)
        before = digest(root/'checkin.db')
        process = subprocess.Popen([str(args.exe), *appargs], cwd=args.repo, stdout=subprocess.DEVNULL,
                                   stderr=subprocess.DEVNULL, creationflags=subprocess.CREATE_NO_WINDOW)
        hwnd = None
        try:
            for _ in range(120):
                hwnd = owned_window(process.pid)
                if hwnd:
                    break
                assert process.poll() is None, f'app exited {process.returncode}'
                time.sleep(.25)
            assert hwnd, 'No visible app window on private desktop'
            dpi = user.GetDpiForWindow(hwnd)
            desired = tuple(round(v*dpi/96) for v in SIZES[size])
            resize(hwnd, desired)
            time.sleep(1.0)
            nav = '签到管理'
            entry = uia(hwnd)
            candidates = [x for x in entry if x['name']==nav and x['role']=='ControlType.Button']
            if len(candidates) != 1:
                raise RuntimeError(f"Navigation is missing: {[(x['name'],x['role']) for x in entry[:50]]}")
            uia(hwnd, open_page=True)
            time.sleep(.55)
            tree = uia(hwnd)
            names = [x['name'] for x in tree]
            # Compare stable state markers, suitable for baseline and candidate with different words.
            if args.prefix == 'before':
                # The legacy plain empty paragraph is visible in pixels, but its
                # text has no UIA control name on this AccessKit build.
                expected = '管理提供商与账户' if state=='unconfigured' else ACCOUNTS[0][2]
            else:
                expected = '添加提供商' if state=='unconfigured' else ACCOUNTS[0][2]
            if not any(expected in n for n in names):
                # AccessKit may report Invoke support without dispatch. A window-only
                # mouse message on this private desktop resolves to the observed UIA row.
                el = candidates[0]
                pos = win32gui.ScreenToClient(hwnd, (int(el['left']+el['width']/2), int(el['top']+el['height']/2)))
                coord = win32api.MAKELONG(*pos)
                win32gui.PostMessage(hwnd, win32con.WM_MOUSEMOVE, 0, coord)
                win32gui.PostMessage(hwnd, win32con.WM_LBUTTONDOWN, 1, coord)
                time.sleep(.08)
                win32gui.PostMessage(hwnd, win32con.WM_LBUTTONUP, 0, coord)
                time.sleep(.8)
                tree = uia(hwnd)
                names = [x['name'] for x in tree]
            assert any(expected in n for n in names), (expected, names)
            assert not any('example.invalid' in n for n in names)
            assert digest(root/'checkin.db') == before
            paths = {p:(root/p).exists() for p in TARGET_FILES}
            assert not any(paths[p] for p in TARGET_FILES[1:]), paths
            basename = args.out / case
            png = basename.with_suffix('.png')
            image_size = capture(hwnd, png)
            receipt = dict(fixture_id=FIXTURE_ID, state=state, account_count=3 if state=='configured' else 0,
                           commit=args.commit, exe_sha256=args.exe_sha256, image=png.name, image_sha256=digest(png),
                           theme=theme, language='zh', ui_font_px=14, mono_font_px=12, environment='local',
                           title=TITLE, logical_client=list(SIZES[size]), physical_client=list(desired), dpi=dpi,
                           captured_frame_px=image_size, capture='PrintWindow(2), DWM extended frame',
                           page='签到管理', scroll='fresh navigation from sidebar, initial top, no wheel or PageDown',
                           db_before_sha256=before, db_after_sha256=digest(root/'checkin.db'),
                           file_state_after=paths, uia_labels=names)
            basename.with_suffix('.json').write_text(json.dumps(receipt, indent=2, ensure_ascii=False), encoding='utf8')
            print(case, image_size, flush=True)
        finally:
            if hwnd and process.poll() is None:
                win32gui.PostMessage(hwnd, win32con.WM_CLOSE, 0, 0)
                try:
                    process.wait(4)
                except subprocess.TimeoutExpired:
                    process.terminate()
            if process.poll() is None:
                process.terminate()
            process.wait(8)

def run(args):
    user.SetThreadDpiAwarenessContext(ctypes.c_void_p(-4))
    args.exe_sha256 = digest(args.exe)
    for state in args.state or STATES:
        for theme in args.theme or ('light','dark'):
            for size in args.size or SIZES:
                capture_case(args, state, size, theme)

def private_desktop():
    name = 'CCRCheckin' + uuid.uuid4().hex
    desktop = win32service.CreateDesktop(name, 0, win32con.MAXIMUM_ALLOWED, None)
    startup = win32process.STARTUPINFO()
    startup.lpDesktop = 'WinSta0\\' + name
    command = subprocess.list2cmdline([sys.executable, str(Path(__file__).resolve()), *sys.argv[1:], '--worker'])
    process, thread, _, _ = win32process.CreateProcess(None, command, None, None, False,
                                                       win32con.CREATE_NO_WINDOW, None, None, startup)
    try:
        win32event.WaitForSingleObject(process, win32event.INFINITE)
        return win32process.GetExitCodeProcess(process)
    finally:
        thread.Close()
        process.Close()
        desktop.CloseDesktop()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--commit', default=BASE_SHA)
    parser.add_argument('--prefix', default='before')
    parser.add_argument('--state', action='append', choices=STATES)
    parser.add_argument('--size', action='append', choices=SIZES)
    parser.add_argument('--theme', action='append', choices=['light','dark'])
    parser.add_argument('--worker', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if not args.worker:
        return private_desktop()
    args.out.mkdir(parents=True, exist_ok=True)
    with (args.out/(args.prefix+'-capture.log')).open('a', encoding='utf8', buffering=1) as log:
        sys.stdout = sys.stderr = log
        try:
            run(args)
            return 0
        except Exception:
            traceback.print_exc()
            return 1

if __name__ == '__main__':
    sys.exit(main())
