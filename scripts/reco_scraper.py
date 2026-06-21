#!/usr/bin/env python3
"""Scrape reco.nz for ZB reconstructions and extract ZBLL/ZBLS algorithms.

Phase 1 collects all solve IDs + method from the paginated index (the site's
filters are client-side only, so every page must be walked). Phase 2 fetches
each ZB solve page and parses the ``// ZBLL`` / ``// ZBLS`` comment lines.
"""

import json
import re
import time
from pathlib import Path
from typing import Any

import requests
from bs4 import BeautifulSoup

BASE = "https://reco.nz"
SESSION = requests.Session()
SESSION.headers["User-Agent"] = "Mozilla/5.0 (research scraper)"
DELAY = 0.3  # seconds between requests, to stay polite to the source site

# A scraped solve-index row or parsed solve record; loosely typed by design
# because the upstream HTML schema is not guaranteed stable.
Record = dict[str, Any]


def _cell(cells: list[Any], idx: int) -> str:
    """Return the stripped text of cell ``idx``, or "" if it is absent."""
    if idx < len(cells):
        return cells[idx].get_text().strip()
    return ""


def get_rows_on_page(page: int) -> tuple[list[Record], bool]:
    """Fetch one index page and return its solve rows.

    Args:
        page: 1-based index page number.

    Returns:
        A ``(rows, has_next)`` tuple where ``rows`` is the parsed solve rows
        on the page and ``has_next`` indicates whether a further page exists.
    """
    url = f"{BASE}/?page={page}"
    resp = SESSION.get(url, timeout=15)
    soup = BeautifulSoup(resp.text, "html.parser")
    rows: list[Record] = []
    for tr in soup.find_all("tr", class_="solve-row"):
        sid = tr.get("data-id", "")
        if not sid.isdigit():
            continue
        tds = tr.find_all("td")
        if len(tds) < 5:
            continue
        rows.append(
            {
                "id": int(sid),
                "puzzle": _cell(tds, 1),
                "result": _cell(tds, 2),
                "solver": _cell(tds, 3),
                "method": _cell(tds, 4),
                "date": _cell(tds, 5),
                "competition": _cell(tds, 6),
                "tags": _cell(tds, 7),
            }
        )
    has_next = bool(soup.find("a", string=re.compile(r"Next")))
    return rows, has_next


def collect_all_zb_solves() -> list[Record]:
    """Walk every index page and return all rows whose method is ``ZB``."""
    all_zb: list[Record] = []
    page = 1
    while True:
        rows, has_next = get_rows_on_page(page)
        zb_rows = [r for r in rows if r["method"] == "ZB"]
        all_zb.extend(zb_rows)
        if page % 10 == 0 or not has_next:
            print(
                f"  Page {page}: {len(rows)} rows, {len(zb_rows)} ZB | "
                f"total ZB so far: {len(all_zb)}"
            )
        if not has_next:
            break
        page += 1
        time.sleep(DELAY)
    return all_zb


def _extract_case(tag: str, comment: str | None) -> str:
    """Append a case label parsed from ``comment`` to ``tag``.

    Recognises forms such as ``ZBLL T6``, ``ZBLL (T6)`` and ``ZBLLT6``.

    Args:
        tag: Base tag to prefix, e.g. ``"ZBLL"`` or ``"ZBLS"``.
        comment: The reconstruction comment to scan, if any.

    Returns:
        ``tag`` with the parsed case suffix, or ``tag`` unchanged.
    """
    if comment is None:
        return tag
    m = re.search(r"ZBL[LS]+\s*[-_]?\s*([A-Z]\d+)", comment, re.IGNORECASE)
    if m:
        return tag + m.group(1).upper()
    return tag


def parse_solve_page(solve_id: int) -> Record | None:
    """Fetch a solve page and extract its ZBLL/ZBLS algs and comments.

    Args:
        solve_id: The reco.nz solve identifier.

    Returns:
        A record describing the solve's ZBLL/ZBLS algs, or ``None`` if the
        page is unreachable or contains neither.
    """
    url = f"{BASE}/solve/{solve_id}"
    try:
        resp = SESSION.get(url, timeout=15)
    except requests.RequestException:
        return None
    if resp.status_code != 200:
        return None

    soup = BeautifulSoup(resp.text, "html.parser")
    lines = [
        ln.strip() for ln in soup.get_text("\n").splitlines() if ln.strip()
    ]

    algs: dict[str, str | None] = {"ZBLL": None, "ZBLS": None}
    comments: dict[str, str | None] = {"ZBLL": None, "ZBLS": None}
    for line in lines:
        if "//" not in line:
            continue
        before, _, comment = line.partition("//")
        before = before.strip()
        comment = comment.strip()
        cup = comment.upper()
        for tag in ("ZBLL", "ZBLS"):
            if tag in cup and before:
                algs[tag] = before
                comments[tag] = comment

    if algs["ZBLL"] is None and algs["ZBLS"] is None:
        return None

    return {
        "solve_id": solve_id,
        "url": url,
        "zbll_case": _extract_case("ZBLL", comments["ZBLL"]),
        "zbll_alg": algs["ZBLL"],
        "zbll_comment": comments["ZBLL"],
        "zbls_case": _extract_case("ZBLS", comments["ZBLS"]),
        "zbls_alg": algs["ZBLS"],
        "zbls_comment": comments["ZBLS"],
    }


def _save_json(path: str, data: Any) -> None:
    """Write ``data`` to ``path`` as indented JSON."""
    with Path(path).open("w") as f:
        json.dump(data, f, indent=2)


def _fetch_records(zb_solves: list[Record]) -> list[Record]:
    """Fetch and merge per-solve ZBLL/ZBLS records for every ZB solve."""
    results: list[Record] = []
    total = len(zb_solves)
    for i, solve in enumerate(zb_solves):
        rec = parse_solve_page(solve["id"])
        if rec:
            rec.update(
                {
                    "solver": solve["solver"],
                    "result": solve["result"],
                    "date": solve["date"],
                    "competition": solve["competition"],
                    "puzzle": solve["puzzle"],
                }
            )
            results.append(rec)
            print(
                f"  [{i + 1}/{total}] {solve['id']} {solve['solver']} | "
                f"ZBLS={rec['zbls_case']} | ZBLL={rec['zbll_case']} | "
                f"{rec['zbll_alg'] or ''}"
            )
        elif (i + 1) % 50 == 0:
            print(f"  [{i + 1}/{total}] no ZBLL/ZBLS in last 50")
        time.sleep(DELAY)
    return results


def _print_zbll_by_case(results: list[Record]) -> None:
    """Print every ZBLL alg grouped by case, with per-alg attribution."""
    by_case: dict[str, dict[str, list[str]]] = {}
    for r in results:
        if r["zbll_alg"]:
            case = r["zbll_case"] or "ZBLL?"
            alg = r["zbll_alg"]
            attribution = f"{r['solver']} (solve {r['solve_id']})"
            by_case.setdefault(case, {}).setdefault(alg, []).append(attribution)
    for case in sorted(by_case):
        print(f"\n{case}:")
        for alg, uses in by_case[case].items():
            print(f"  {alg}")
            for use in uses:
                print(f"    {use}")


def _print_zbls(results: list[Record]) -> None:
    """Print every ZBLS alg with its case, solver, and any comment."""
    for r in results:
        if r["zbls_alg"]:
            print(
                f"  [{r['zbls_case']}] {r['solver']} "
                f"(solve {r['solve_id']}): {r['zbls_alg']}"
            )
            if r["zbls_comment"] and r["zbls_comment"] != "ZBLS":
                print(f"    comment: {r['zbls_comment']}")


def _print_solver_summary(results: list[Record]) -> None:
    """Print each solver's distinct ZBLL case count, busiest first."""
    solver_cases: dict[str, set[str]] = {}
    for r in results:
        if r["zbll_alg"]:
            solver_cases.setdefault(r["solver"], set()).add(r["zbll_case"])
    for solver, cases in sorted(solver_cases.items(), key=lambda x: -len(x[1])):
        joined = ", ".join(sorted(cases))
        print(f"  {solver}: {len(cases)} distinct cases — {joined}")


def main() -> None:
    """Run both scrape phases and print the ZBLL/ZBLS summaries."""
    print("=== Phase 1: Collecting ZB solve IDs from index pages ===")
    zb_solves = collect_all_zb_solves()
    print(f"\nTotal ZB solves found: {len(zb_solves)}")
    _save_json("reco_zb_metadata.json", zb_solves)
    print("Saved metadata to reco_zb_metadata.json")

    print("\n=== Phase 2: Fetching solve pages for ZBLL/ZBLS ===")
    results = _fetch_records(zb_solves)
    _save_json("reco_zbll_raw.json", results)
    print(f"\nSaved {len(results)} ZBLL/ZBLS records to reco_zbll_raw.json")

    print("\n=== ZBLL Algorithms by Case ===")
    _print_zbll_by_case(results)
    print("\n=== ZBLS Algorithms ===")
    _print_zbls(results)
    print("\n=== Solver ZBLL Summary ===")
    _print_solver_summary(results)


if __name__ == "__main__":
    main()
