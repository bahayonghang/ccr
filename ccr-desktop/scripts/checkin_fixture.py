"""Create a fresh native check-in fixture and serve its synthetic providers."""
import argparse
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
from pathlib import Path
import tempfile
import time

OUTPUTS = Path(__file__).resolve().parents[2] / "outputs" / "desktop"
USERS = {"101": "Ready", "102": "Today", "103": "Expired", "104": "Disabled", "105": "Paused provider"}


def prepare(root, port):
    now = datetime.now(timezone.utc).isoformat()
    base = f"http://127.0.0.1:{port}"
    providers = [
        {"id": key, "name": name, "base_url": base + "/" + key, "enabled": enabled, "created_at": now}
        for key, name, enabled in [("demo", "Check-in demo", True), ("paused", "Paused demo", False)]
    ]
    accounts = [
        {"id": "fixture-" + user, "name": name, "provider_id": "paused" if user == "105" else "demo",
         "enabled": user != "104", "api_user": user, "cookies_json_encrypted": False,
         "cookies_json": json.dumps({"session": "synthetic-" + user}), "created_at": now}
        for user, name in USERS.items()
    ]
    config = {"version": "1.0", "exported_at": now, "providers": providers, "accounts": accounts}
    (root / "config.toml").write_text("", encoding="utf-8")
    (root / "profiles.toml").write_text("", encoding="utf-8")
    (root / ".ccr-gpui-checkin-fixture").write_bytes(b"ccr-gpui-checkin-fixture-v1")
    (root / "accounts.json").write_text(json.dumps(config, indent=2), encoding="utf-8")


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def respond(self):
        user = self.headers.get("new-api-user")
        name = USERS.get(user)
        paths = [f"/{prefix}/api/user/{end}" for prefix in ("demo", "paused") for end in ("self", "checkin")]
        if self.path not in paths or not name:
            status, body = 404, {"message": "Unknown fixture request"}
        elif self.headers.get("Cookie") != f"session=synthetic-{user}" or user == "103":
            status, body = 401, {"success": False, "message": "Unauthorized"}
        elif self.command == "POST" and self.path.endswith("/checkin"):
            status, body = 200, {"success": True, "message": "Fixture check-in saved", "data": {"reward": "1 USD"}}
        elif self.command == "GET" and self.path.endswith("/self"):
            status, body = 200, {"success": True, "data": {"check_in_today": user == "102", "quota": 500000, "used_quota": 100000}}
        else:
            status, body = 405, {"message": "Unexpected method"}
        time.sleep(self.server.delay)
        payload = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)
        self.server.count += 1
        print(f"{self.server.count} {self.command} {name or 'unknown'} {status}", flush=True)

    do_GET = respond
    do_POST = respond


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--delay-ms", type=int, default=250, choices=range(0, 3001), metavar="0..3000")
    args = parser.parse_args()
    OUTPUTS.mkdir(parents=True, exist_ok=True)
    if OUTPUTS.resolve() != OUTPUTS:
        raise ValueError("outputs/desktop must be inside this checkout")
    with HTTPServer(("127.0.0.1", 0), Handler) as server:
        root = Path(tempfile.mkdtemp(prefix="checkin-", dir=OUTPUTS))
        prepare(root, server.server_port)
        server.delay = args.delay_ms / 1000
        server.count = 0
        print(f'just gpui-dev --fixture-checkin-loopback --fixture-profiles "{root / "profiles.toml"}" --fixture-registry "{root / "config.toml"}"', flush=True)
        print(f'Import: {root / "accounts.json"}', flush=True)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
