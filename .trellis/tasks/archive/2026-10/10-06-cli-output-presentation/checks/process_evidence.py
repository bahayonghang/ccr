"""Run CLI evidence with synthetic credentials and process-local path overrides."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


def fixture():
    home = Path(tempfile.mkdtemp(prefix="ccr-output-evidence-"))
    for child in ("ccr", "codex", "claude", "grok", "locks", "backups"):
        (home / child).mkdir()
    environment = os.environ.copy()
    for key in (
        "CCR_CONFIG_PATH", "CCR_DATA_DIR", "CCR_CODEX_DIR", "CODEX_HOME",
        "ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY", "CLAUDE_CODE_OAUTH_TOKEN",
        "CLAUDE_CODE_USE_BEDROCK", "CLAUDE_CODE_USE_VERTEX", "CLAUDE_CODE_USE_FOUNDRY",
        "XDG_CONFIG_HOME", "XDG_DATA_HOME", "NO_COLOR", "CLICOLOR_FORCE",
        "OPENAI_API_KEY", "XAI_API_KEY", "ANTHROPIC_BASE_URL", "FORCE_COLOR",
    ):
        environment.pop(key, None)
    overrides = {
        "HOME": str(home), "USERPROFILE": str(home),
        "CCR_ROOT": str(home / "ccr"), "CCR_DATA_DIR": str(home / "ccr"),
        "CCR_CONFIG_PATH": str(home / "ccr/config.toml"),
        "CCR_CODEX_DIR": str(home / "codex"), "CODEX_HOME": str(home / "codex"),
        "CLAUDE_CONFIG_DIR": str(home / "claude"),
        "CLAUDE_JSON_PATH": str(home / ".claude.json"),
        "CCR_SETTINGS_PATH": str(home / "claude/settings.json"),
        "CCR_BACKUP_DIR": str(home / "backups"), "GROK_HOME": str(home / "grok"),
        "CCR_LOCK_DIR": str(home / "locks"), "CCR_LOG_LEVEL": "off",
        "TERM": "xterm-256color", "CLICOLOR": "0", "COLUMNS": "120",
    }
    environment.update(overrides)
    encode = lambda value: base64.urlsafe_b64encode(json.dumps(value).encode()).decode().rstrip("=")
    jwt = encode({"alg": "none", "typ": "JWT"}) + "." + encode({"email": "teacher@example.test", "sub": "synthetic"}) + ".signature"
    (home / "codex/config.toml").write_text('cli_auth_credentials_store = "file"\nmodel_provider = "openai"\n', encoding="utf-8")
    (home / "codex/auth.json").write_text(json.dumps({"OPENAI_API_KEY": None, "tokens": {"id_token": jwt, "access_token": "SYNTHETIC_ACCESS_SENTINEL", "refresh_token": "SYNTHETIC_REFRESH_SENTINEL", "account_id": "synthetic-account"}, "last_refresh": "2026-01-08T03:09:53Z"}), encoding="utf-8")
    (home / "claude/.credentials.json").write_text(json.dumps({"claudeAiOauth": {"accessToken": "SYNTHETIC_ACCESS_SENTINEL", "refreshToken": "SYNTHETIC_REFRESH_SENTINEL", "expiresAt": "2099-01-01T00:00:00Z", "subscriptionType": "pro", "rateLimitTier": "default_claude_ai", "scopes": ["user:profile"]}}), encoding="utf-8")
    (home / ".claude.json").write_text(json.dumps({"oauthAccount": {"accountUuid": "synthetic-account", "emailAddress": "teacher@example.test", "billingType": "apple_subscription"}}), encoding="utf-8")
    (home / "grok/auth.json").write_text(json.dumps({"https://auth.x.ai::client": {"auth_mode": "oidc", "oidc_issuer": "https://auth.x.ai", "oidc_client_id": "client", "key": "SYNTHETIC_ACCESS_SENTINEL", "user_id": "synthetic", "create_time": "2026-01-01T00:00:00Z"}}), encoding="utf-8")
    return home, environment, overrides


def run(binary, environment, arguments, stdin=""):
    result = subprocess.run([str(binary), *arguments], env=environment, cwd=environment["HOME"], input=stdin, text=True, encoding="utf-8", capture_output=True, timeout=30)
    combined = result.stdout + result.stderr
    assert "SYNTHETIC_ACCESS_SENTINEL" not in combined
    assert "SYNTHETIC_REFRESH_SENTINEL" not in combined
    return {"arguments": arguments, "exit_code": result.returncode, "stdout": result.stdout, "stderr": result.stderr}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--skip-doctor", action="store_true", help="Keep Windows Known Folder reads outside synthetic evidence")
    args = parser.parse_args()
    binary = Path(args.binary).resolve()
    home, environment, overrides = fixture()
    cases = []
    for platform in ("codex", "claude", "grok"):
        for suffix in (["list"], ["current", "--json"], ["save", "teacher"], ["save", "teacher"], ["current", "--json"], ["list"], ["delete", "teacher"]):
            cases.append(run(binary, environment, [platform, "auth", *suffix], "n\n"))
    extra_arguments = [["platform", "list", "--json"]]
    if not args.skip_doctor:
        extra_arguments.extend([["doctor", "--platform", "codex", "--json"], ["doctor", "--platform", "codex", "--verbose"]])
    for arguments in extra_arguments:
        cases.append(run(binary, environment, arguments))
    report = {"binary": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "fixture": str(home), "environment_overrides": overrides, "cases": cases, "doctor": "NOT_RUN_WINDOWS_ISOLATION" if args.skip_doctor else "RUN"}
    Path(args.output).write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": args.output, "cases": len(cases), "fixture": str(home), "exit_codes": [case["exit_code"] for case in cases]}))


if __name__ == "__main__":
    main()
