"""Bounded MANTIS adapter for the MIT-licensed Maigret username collector."""

from __future__ import annotations

import argparse
import asyncio
import contextlib
import io
import json
import os
import re
import sys
import tempfile
from pathlib import Path
from urllib.parse import urlparse


def public_url(value: object) -> str | None:
    if not isinstance(value, str) or len(value) > 2048:
        return None
    try:
        parsed = urlparse(value.strip())
        return value.strip() if parsed.scheme in {"http", "https"} and parsed.netloc else None
    except ValueError:
        return None


def normalize_report(report: object) -> tuple[list[dict], int]:
    """Accept Maigret simple/legacy JSON shapes but emit one stable MANTIS contract."""
    results: list[dict] = []
    checked = 0
    seen: set[str] = set()

    def visit(node: object, site_hint: str = "") -> None:
        nonlocal checked
        if isinstance(node, list):
            for item in node:
                visit(item, site_hint)
            return
        if not isinstance(node, dict):
            url = public_url(node)
            if url and url not in seen:
                seen.add(url)
                results.append({"site_name": site_hint or urlparse(url).netloc, "url": url, "category": "profil_public"})
            return
        for key, value in node.items():
            if isinstance(value, (dict, list)):
                checked += 1 if isinstance(value, dict) else 0
                candidate = value if isinstance(value, dict) else {}
                status = str(candidate.get("status") or candidate.get("status_text") or "").lower()
                url = public_url(candidate.get("url_user") or candidate.get("url") or candidate.get("profile_url"))
                absent = any(term in status for term in ("not found", "available", "unclaimed", "unknown"))
                if url and not absent and url not in seen:
                    seen.add(url)
                    site_meta = candidate.get("site") if isinstance(candidate.get("site"), dict) else {}
                    status_meta = candidate.get("status") if isinstance(candidate.get("status"), dict) else {}
                    tags = candidate.get("tags") or site_meta.get("tags") or status_meta.get("tags")
                    category = ", ".join(str(tag) for tag in tags[:4]) if isinstance(tags, list) else "profil_public"
                    results.append({"site_name": str(candidate.get("site_name") or key)[:120], "url": url, "category": category[:160]})
                    continue
                visit(value, str(key))
            else:
                visit(value, str(key))

    visit(report)
    return results[:200], checked


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True)
    parser.add_argument("--timeout", type=int, default=8)
    parser.add_argument("--top-sites", type=int, default=150)
    args = parser.parse_args()
    target = args.target.strip().lstrip("@")
    if not target or len(target) > 128 or re.search(r"[\x00-\x20]", target):
        print(json.dumps({"version": 1, "target": target, "results": [], "summary": {"checked": 0, "found": 0}, "error": "Pseudo invalide."}, ensure_ascii=False))
        return 0

    timeout = max(2, min(args.timeout, 15))
    top_sites = max(20, min(args.top_sites, 250))
    noise = io.StringIO()
    try:
        with tempfile.TemporaryDirectory(prefix="mantis-maigret-") as temp_dir, contextlib.redirect_stdout(noise), contextlib.redirect_stderr(noise):
            from maigret.maigret import main as maigret_main

            old_argv = sys.argv
            old_cwd = os.getcwd()
            try:
                sys.argv = ["maigret", target, "--json", "simple", "--folderoutput", temp_dir,
                            "--top-sites", str(top_sites), "--timeout", str(timeout), "--max-connections", "20",
                            "--retries", "0", "--no-autoupdate", "--no-recursion", "--no-extracting",
                            "--no-progressbar", "--no-color"]
                os.chdir(temp_dir)
                asyncio.run(maigret_main())
            finally:
                sys.argv = old_argv
                os.chdir(old_cwd)
            candidates = list(Path(temp_dir).glob("report_*_simple.json"))
            if not candidates:
                raise RuntimeError("Rapport JSON Maigret absent.")
            raw = candidates[0].read_text(encoding="utf-8")
            if len(raw.encode("utf-8")) > 8_000_000:
                raise RuntimeError("Rapport Maigret trop volumineux.")
            report = json.loads(raw)
            results, _ = normalize_report(report)
            checked = top_sites
        print(json.dumps({"version": 1, "collector_version": "maigret-0.6.3", "target": target,
                          "results": results, "summary": {"checked": checked, "found": len(results)}}, ensure_ascii=False))
        return 0
    except Exception as error:
        print(json.dumps({"version": 1, "collector_version": "maigret-0.6.3", "target": target,
                          "results": [], "summary": {"checked": 0, "found": 0},
                          "error": f"Collecte Maigret indisponible : {str(error)[:240]}"}, ensure_ascii=False))
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
