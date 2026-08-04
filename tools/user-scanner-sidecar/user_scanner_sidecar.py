"""MANTIS adapter for the MIT-licensed User Scanner project.

The adapter deliberately uses the library API: it disables its interactive CLI,
auto-update prompt, proxy support and notification-triggering ("loud") modules.
Only a JSON document is emitted on stdout so the Tauri host can preserve the raw
result locally and normalise positive matches into reviewable MANTIS signals.
"""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import sys

from user_scanner.core.email_orchestrator import run_email_full_batch, set_concurrency as set_email_concurrency
from user_scanner.core.helpers import ScanConfig, is_valid_email
from user_scanner.core.orchestrator import run_user_full, set_concurrency as set_user_concurrency
from user_scanner.core.result import Status


def result_to_dict(result):
    return result.to_dict()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True)
    parser.add_argument("--kind", required=True, choices=("email", "pseudo"))
    parser.add_argument("--timeout", type=float, default=8.0)
    parser.add_argument("--concurrency", type=int, default=10)
    args = parser.parse_args()

    target = args.target.strip()
    if not target:
        print(json.dumps({"version": 1, "error": "La cible est vide."}, ensure_ascii=False))
        return 2
    if args.kind == "email" and not is_valid_email(target):
        print(json.dumps({"version": 1, "error": "L’adresse e-mail est invalide."}, ensure_ascii=False))
        return 2

    # A moderate local concurrency, no proxy and no "loud" reset/password flows.
    # NSFW categories remain included, as requested for the full footprint view.
    set_email_concurrency(args.concurrency)
    set_user_concurrency(args.concurrency)
    config = ScanConfig(
        allow_loud=False,
        only_found=False,
        no_nsfw=False,
        verbose=False,
        timeout=max(1.0, min(args.timeout, 20.0)),
    )

    console_noise = io.StringIO()
    with contextlib.redirect_stdout(console_noise):
        results = run_email_full_batch(target, config) if args.kind == "email" else run_user_full(target, config)

    serialized = [result_to_dict(result) for result in results]
    found = [item for item, result in zip(serialized, results) if result.status == Status.TAKEN]
    errors = sum(1 for result in results if result.status == Status.ERROR)
    skipped = sum(1 for result in results if result.status == Status.SKIPPED)
    payload = {
        "version": 1,
        "target": target,
        "target_kind": args.kind,
        "results": found,
        "summary": {
            "checked": len(results),
            "found": len(found),
            "errors": errors,
            "skipped": skipped,
            "notification_checks_excluded": skipped,
        },
    }
    print(json.dumps(payload, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
