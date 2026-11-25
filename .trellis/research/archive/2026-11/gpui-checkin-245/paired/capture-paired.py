"""Repeat the eight first screens using the same data and Windows capture as the baseline."""
import argparse
from functools import partial
import importlib.util
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace

def main():
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--prefix', choices=['after'], required=True)
    opts, _ = parser.parse_known_args()
    repo = opts.repo.resolve(strict=True)
    baseline = repo / '.trellis/research/archive/2026-11/gpui-checkin-245/baseline'
    sys.path.insert(0, str(baseline))
    spec = importlib.util.spec_from_file_location('paired_window', baseline / 'capture-windows.py')
    native = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native)
    outputs = repo / 'outputs' / 'desktop'
    outputs.mkdir(parents=True, exist_ok=True)
    assert outputs.resolve() == outputs

    make_pair = native.make
    def make_marked_pair(root, checkout, state, size, theme):
        launch = make_pair(root, checkout, state, size, theme)
        assert root.resolve().is_relative_to(outputs)
        for name in ('config.toml', 'profiles.toml'):
            (root / name).write_bytes(b'')
        (root / '.ccr-gpui-checkin-fixture').write_bytes(b'ccr-gpui-checkin-fixture-v1')
        return ['--fixture-checkin-loopback', *launch]

    native.make = make_marked_pair
    native.tempfile = SimpleNamespace(TemporaryDirectory=partial(tempfile.TemporaryDirectory, dir=outputs))
    # The private-desktop worker must re-enter this wrapper to use the same marker and flags.
    native.__file__ = __file__
    return native.main()

if __name__ == '__main__':
    sys.exit(main())
