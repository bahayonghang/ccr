"""Validate actual Windows console receipts without executing product code."""

import argparse
import hashlib
import itertools
import json
from pathlib import Path
import re


EXPECTED_CASES = {
    "save-long-multiline": "save",
    "duplicate-error": "duplicate",
    "current-runtime": "current",
    "doctor-statuses": "doctor",
    "missing-fields": "missing-fields",
    "duplicate-leading": "duplicate-leading",
    "copy-normal": "copy-normal",
    "copy-leading": "copy-leading",
}


def compact(value):
    return re.sub(r"\s+", "", value)


def screen_text(screen):
    return "\n".join(row["Text"] for row in screen["Rows"])


def check_default_body(path, fragments):
    screen = json.loads(path.read_text(encoding="utf-8-sig"))
    characters, attributes = [], []
    for row in screen["Rows"]:
        for run in row["Runs"]:
            for character in run["Text"]:
                if not character.isspace():
                    characters.append(character)
                    attributes.append((run["Foreground"], run["Background"]))
    joined = "".join(characters)
    for fragment in fragments:
        wanted = compact(fragment)
        index = joined.find(wanted)
        assert index >= 0, f"body fragment absent: {path}"
        assert all(value == (7, 0) for value in attributes[index:index + len(wanted)]), (
            f"shared message body or field value styled: {path}"
        )


def load_screen(path, width, background, mode):
    screen = json.loads(path.read_text(encoding="utf-8-sig"))
    assert screen["Width"] == width, f"wrong window width: {path}"
    assert screen["Height"] == 40, f"wrong window height: {path}"
    assert screen["Palette"][0] == (0xFFFFFF if background == "light" else 0)
    assert screen["Palette"][7] == (0 if background == "light" else 0xC0C0C0)
    for row in screen["Rows"]:
        for run in row["Runs"]:
            if not run["Text"].strip():
                continue
            foreground = screen["Palette"][run["Foreground"]]
            backdrop = screen["Palette"][run["Background"]]
            assert foreground != backdrop, f"invisible text: {path}"
            if mode in ("no-color", "dumb"):
                assert run["Foreground"] == 7 and run["Background"] == 0, (
                    f"styled text in {mode}: {path}"
                )
    value = screen_text(screen)
    for label in ("[OK]", "[INFO]", "[ERR]", "[WARN]", "[STEP]", "[SKIP]"):
        assert label not in value, f"legacy status label: {path}"
    assert "\x1b" not in value, f"raw ANSI stored in console cells: {path}"
    return value


def check_receipt(path, binary_sha256):
    receipt = json.loads(path.read_text(encoding="utf-8-sig"))
    width, background, mode = receipt["width"], receipt["background"], receipt["mode"]
    assert receipt["sha256"].lower() == binary_sha256, f"binary mismatch: {path}"
    assert receipt["height"] == 40
    assert not receipt["stdout_redirected"] and not receipt["stderr_redirected"]
    assert receipt["doctor"] == "RUN", f"Doctor missing: {path}"
    assert EXPECTED_CASES.keys() <= set(receipt["cases"]), f"cases missing: {path}"
    assert len(receipt["cases"]) == len(receipt["exit_codes"])
    exits = dict(zip(receipt["cases"], receipt["exit_codes"]))
    for case in receipt["cases"]:
        if case != "doctor-statuses":
            assert exits[case] == 0, f"unexpected exit for {case}: {path}"
    assert exits["doctor-statuses"] in (0, 1), f"invalid Doctor exit: {path}"

    texts = {}
    for case, suffix in EXPECTED_CASES.items():
        screen_path = Path(receipt["snapshot_prefix"] + f"-{suffix}-screen.json")
        texts[case] = load_screen(screen_path, width, background, mode)

    save = texts["save-long-multiline"]
    success = "成功:" if mode == "dumb" else "✓"
    assert compact(success + " 已保存账号 teacher") in compact(save)
    assert compact(receipt["description"]) in compact(save), f"description truncated: {path}"
    assert compact("邮箱: tea***@example.test") in compact(save)
    assert compact("ccr codex auth list") in compact(save)
    assert "[" not in save.split("已保存账号 teacher", 1)[0]
    check_default_body(Path(receipt["snapshot_prefix"] + "-save-screen.json"),
                       ["已保存账号 teacher", receipt["description"], "tea***@example.test"])
    missing = texts["missing-fields"]
    assert compact(success + " 已保存账号") in compact(missing)
    assert "邮箱:" not in missing and "描述:" not in missing
    assert "覆盖会替换该账号已保存的凭据快照" in compact(texts["duplicate-error"])
    error = "错误:" if mode == "dumb" else "× 错误:"
    assert compact(error) in compact(texts["duplicate-error"])

    copied = receipt["copied_commands"]
    assert len(copied) == 2, f"copy evidence missing: {path}"
    expected = {
        "ccr codex auth save --force -- teacher": "copy-normal",
        "ccr codex auth save --force -- -teacher": "copy-leading",
    }
    assert {item["text"] for item in copied} == set(expected)
    for item in copied:
        assert item["exit_code"] == 0
        source_path = Path(item["source_snapshot"])
        source = load_screen(source_path, width, background, mode)
        assert compact(item["text"]) in compact(source), f"command absent from console: {path}"
        result_path = Path(item["result_snapshot"])
        result = load_screen(result_path, width, background, mode)
        name = item["text"].split()[-1]
        assert compact(success + " 已保存账号 " + name) in compact(result)
    return {"receipt": str(path), "width": width, "background": background, "mode": mode,
            "case_count": len(receipt["cases"]), "exit_codes": exits, "status": "PASS"}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--receipts", required=True, help="Glob for one final native matrix round")
    parser.add_argument("--binary", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    binary = Path(args.binary).resolve()
    binary_sha256 = hashlib.sha256(binary.read_bytes()).hexdigest()
    pattern = Path(args.receipts)
    paths = sorted(pattern.parent.glob(pattern.name))
    records, failures = [], []
    for path in paths:
        try:
            records.append(check_receipt(path, binary_sha256))
        except (AssertionError, KeyError, OSError, ValueError) as error:
            failures.append({"receipt": str(path), "error": str(error)})
    expected = set(itertools.product((40, 80, 120), ("dark", "light"), ("normal", "no-color", "dumb")))
    observed = {(record["width"], record["background"], record["mode"]) for record in records}
    if len(paths) != 18 or len(records) != 18 or observed != expected:
        failures.append({"error": "matrix incomplete", "receipts": len(paths),
                         "missing": sorted(expected - observed)})
    result = {"status": "FAIL" if failures else "PASS", "binary": str(binary),
              "sha256": binary_sha256, "records": records, "failures": failures,
              "scope": "Visible console cells, palette, dimensions, copied command receipts; not raw stream/DTO/Doctor full-report verification"}
    Path(args.output).write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": result["status"], "receipts": len(paths), "passed": len(records),
                      "failures": len(failures)}))
    raise SystemExit(bool(failures))


if __name__ == "__main__":
    main()
