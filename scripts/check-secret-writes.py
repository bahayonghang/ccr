#!/usr/bin/env python3
"""Reject unsafe writes in known credential and settings persistence modules."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SENSITIVE_MODULES = (
    "crates/ccr-cli/src/managers/settings.rs",
    "crates/ccr-cli/src/platforms/claude.rs",
    "crates/ccr-codex/src/platforms/codex.rs",
    "crates/ccr-codex/src/services/codex_oauth_token_service.rs",
    "crates/ccr-codex/src/services/codex_quota_service.rs",
    "crates/ccr-codex/src/services/opencode_auth_service.rs",
    "crates/ccr-codex/src/services/opencode_quota_service.rs",
)
DIRECT_ASYNC_WRITE = re.compile(r"(?:tokio::fs|async_fs)::write\s*\(")
ATOMIC_WRITE = re.compile(
    r"(?:Async)?AtomicWriter::new\s*\([^;]{0,300}?\.write(?:_string)?(?:_async)?\s*\(",
    re.DOTALL,
)


def main() -> int:
    violations: list[str] = []
    for relative in SENSITIVE_MODULES:
        path = ROOT / relative
        source = path.read_text(encoding="utf-8")
        for match in DIRECT_ASYNC_WRITE.finditer(source):
            line = source.count("\n", 0, match.start()) + 1
            violations.append(f"{relative}:{line}: direct async write is forbidden")
        for match in ATOMIC_WRITE.finditer(source):
            if ".secret(true)" not in match.group(0):
                line = source.count("\n", 0, match.start()) + 1
                violations.append(
                    f"{relative}:{line}: sensitive AtomicWriter call lacks .secret(true)"
                )

    if violations:
        print("Sensitive persistence policy violations:", file=sys.stderr)
        for violation in violations:
            print(f"  {violation}", file=sys.stderr)
        return 1

    print("Sensitive persistence policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
