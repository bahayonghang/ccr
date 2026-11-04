#!/usr/bin/env bash
# Download one pinned prebuilt tool onto PATH. Hosted workflows call this
# instead of compiling the tool from source.
set -euo pipefail
cd "$(dirname "$0")/../.."
tool="${1:?tool name required}"
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*) py=python ;;
  *) py=python3 ;;
esac
if ! command -v "$py" >/dev/null 2>&1; then
  py=python
fi
exec "$py" scripts/ci/install_prebuilt_tool.py "$tool"
