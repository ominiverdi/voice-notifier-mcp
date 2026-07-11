#!/usr/bin/env python3
"""Smoke-test a packaged voice-notifier-mcp executable over stdio."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

SUPPORTED_PROTOCOLS = ("2024-11-05", "2025-06-18", "2025-11-25")


def exchange(process: subprocess.Popen[str], request: dict[str, Any]) -> dict[str, Any]:
    assert process.stdin is not None
    assert process.stdout is not None
    process.stdin.write(json.dumps(request, separators=(",", ":")) + "\n")
    process.stdin.flush()
    line = process.stdout.readline()
    if not line:
        stderr = process.stderr.read() if process.stderr else ""
        raise RuntimeError(f"server exited without a response: {stderr}")
    return json.loads(line)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit(f"usage: {Path(sys.argv[0]).name} PATH_TO_BINARY")

    binary = Path(sys.argv[1]).resolve()
    if not binary.is_file():
        raise SystemExit(f"binary not found: {binary}")

    with tempfile.TemporaryDirectory(prefix="voice-notifier-mcp-smoke-") as directory:
        voice_directory = Path(directory)
        (voice_directory / "bf_emma.bin").touch()
        (voice_directory / "custom-voice.bin").touch()
        (voice_directory / "ignored.txt").touch()
        environment = os.environ.copy()
        environment["VOICE_NOTIFIER_VOICE_PATH"] = directory
        environment["VOICE_NOTIFIER_VOICE"] = "bf_emma"
        process = subprocess.Popen(
            [str(binary)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
            env=environment,
        )
        run_checks(process)


def run_checks(process: subprocess.Popen[str]) -> None:
    try:
        for request_id, protocol in enumerate(SUPPORTED_PROTOCOLS, start=1):
            response = exchange(
                process,
                {
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "method": "initialize",
                    "params": {"protocolVersion": protocol},
                },
            )
            assert response["result"]["protocolVersion"] == protocol, response
            assert response["result"]["serverInfo"]["name"] == "voice-notifier", response

        response = exchange(
            process,
            {"jsonrpc": "2.0", "id": 10, "method": "tools/list"},
        )
        tool = response["result"]["tools"][0]
        assert tool["name"] == "voice_notify", response
        assert tool["annotations"] == {
            "readOnlyHint": False,
            "destructiveHint": False,
            "idempotentHint": False,
            "openWorldHint": False,
        }, response
        voice_schema = tool["inputSchema"]["properties"]["voice_name"]
        assert voice_schema["enum"] == ["bf_emma", "custom-voice"], response
        assert voice_schema["default"] == "bf_emma", response
    finally:
        if process.stdin:
            process.stdin.close()
        try:
            return_code = process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.terminate()
            return_code = process.wait(timeout=10)
        if process.stdout:
            process.stdout.close()
        if process.stderr:
            process.stderr.close()

    if return_code != 0:
        raise SystemExit(f"server exited with status {return_code}")
    print(f"Package smoke test passed: {process.args[0]}")


if __name__ == "__main__":
    main()
