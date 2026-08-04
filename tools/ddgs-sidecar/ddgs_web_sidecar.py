"""MANTIS entry point for the bundled DDGS Web-footprint sidecar.

It uses DDGS's standard metasearch selection and emits a small, stable JSON
contract. No cache, DHT node, API server or background process is started.
"""

import argparse
import json
import os
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed

import certifi
from ddgs import DDGS

# Some locked-down Windows profiles deny access to the per-user certificate
# store. Use the bundled Mozilla CA file so public HTTPS searches still work
# without weakening TLS verification or asking for network permissions.
os.environ.setdefault("SSL_CERT_FILE", certifi.where())
os.environ.setdefault("REQUESTS_CA_BUNDLE", certifi.where())

# PyInstaller inherits the Windows ANSI code page for redirected output on some
# machines. MANTIS consumes this contract as UTF-8, independently of locale.
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")


def quoted(value: str) -> str:
    return f'"{value.replace(chr(34), " ").strip()}"'


def build_queries(target: str, kind: str, contexts: list[str]) -> list[str]:
    exact = quoted(target)
    if kind == "pseudo":
        return [
            exact,
            f"site:github.com {exact}",
            f"site:linkedin.com/in {exact}",
            f"site:reddit.com {exact}",
        ]
    if kind == "email":
        return [exact]
    if kind == "telephone":
        # These identifiers are queried only as an exact phrase. MANTIS then
        # verifies the value in the destination page before creating a fact.
        return [exact]
    if kind == "adresse":
        postal_codes = [context.strip() for context in contexts if context.strip() and any(char.isdigit() for char in context)]
        if not postal_codes:
            return []
        postal = quoted(postal_codes[0])
        # A neighbourhood or street name is highly ambiguous on its own. Every
        # address query is deliberately paired with the user-declared postcode.
        return [f"{exact} {postal}", f"{target} {postal}"]
    # A name is never queried in isolation with a first name. Contextual
    # queries are only built from values explicitly recorded by the user.
    # Keep a small, bounded platform pass as well: public profile pages are
    # often ranked below directories in a generic exact-name query, while
    # being the most useful signals for the person reviewing the scan.
    queries = [
        exact,
        target,
        f"{exact} filetype:pdf",
        f"{exact} inurl:pdf",
        f"site:instagram.com {exact}",
        f"site:linkedin.com/in {exact}",
    ]
    for context in contexts[:3]:
        if context and context.casefold() != target.casefold():
            queries.append(f"{exact} {quoted(context)}")
    return queries


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True)
    parser.add_argument(
        "--kind",
        required=True,
        choices=("email", "nom", "pseudo", "telephone", "adresse"),
    )
    parser.add_argument("--context", action="append", default=[])
    parser.add_argument("--max-results", type=int, default=5)
    args = parser.parse_args()

    target = args.target.strip()
    if not target or len(target) > 512:
        print(json.dumps({"version": 1, "results": [], "errors": ["Identité invalide."]}, ensure_ascii=False))
        return 2

    # Do not silently reduce the depth requested by MANTIS.  The provider can
    # still return fewer results, but a useful result must not disappear here
    # just because it sits after an arbitrary first-page cutoff.
    limit = max(args.max_results, 1)
    results: list[dict[str, str]] = []
    errors: list[str] = []
    queries = build_queries(target, args.kind, args.context)
    if not queries and args.kind == "adresse":
        errors.append("Recherche d’adresse non lancée : aucun code postal actif n’est associé à cette identité.")
    # DDGS can select a different upstream engine on separate calls.  Keep two
    # passes and two attempts, but execute independent queries concurrently so
    # a slow provider cannot make an otherwise complete collection time out.
    def search_query(query: str, pass_index: int) -> tuple[int, str, list[dict[str, str]], str | None]:
        last_error = ""
        for _attempt in range(2):
            try:
                items = DDGS().text(query, max_results=limit, backend="auto")
                return pass_index, query, list(items), None
            except Exception as error:
                last_error = str(error)
        return pass_index, query, [], last_error or "réponse indisponible"

    jobs = [(pass_index, query) for pass_index in range(2) for query in queries]
    collected: dict[tuple[int, str], list[dict[str, str]]] = {}
    # Four workers improve recall under transient failures without flooding the
    # public engines or competing with the desktop UI for all CPU resources.
    with ThreadPoolExecutor(max_workers=max(1, min(4, len(jobs)))) as executor:
        futures = [executor.submit(search_query, query, pass_index) for pass_index, query in jobs]
        for future in as_completed(futures):
            pass_index, query, items, error = future.result()
            if error:
                errors.append(f"La recherche {query} (passe {pass_index + 1}) n’a pas répondu : {error}")
            else:
                collected[(pass_index, query)] = items

    # Preserve a stable raw order for traceability. URL deduplication belongs to
    # MANTIS after this point; every returned observation remains in the raw log.
    for pass_index, query in jobs:
        for item in collected.get((pass_index, query), []):
            url = str(item.get("href", "")).strip()
            if url:
                results.append({
                    "query": query,
                    "title": str(item.get("title", "")).strip(),
                    "url": url,
                    "snippet": str(item.get("body", "")).strip(),
                    "backend": "auto",
                })

    print(json.dumps({"version": 1, "results": results, "errors": errors}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
