#!/usr/bin/env python3
"""
Scrape reco.nz for all ZB reconstructions and extract ZBLL/ZBLS algorithms.

Phase 1: collect all solve IDs + method from paginated index (filters are client-side only)
Phase 2: fetch each ZB solve page and parse // ZBLL / // ZBLS lines
"""

import re
import time
import json
import requests
from bs4 import BeautifulSoup

BASE = "https://reco.nz"
SESSION = requests.Session()
SESSION.headers["User-Agent"] = "Mozilla/5.0 (research scraper)"
DELAY = 0.3  # seconds between requests


def get_rows_on_page(page):
    url = f"{BASE}/?page={page}"
    r = SESSION.get(url, timeout=15)
    soup = BeautifulSoup(r.text, "html.parser")
    rows = []
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
                "puzzle": tds[1].get_text().strip(),
                "result": tds[2].get_text().strip(),
                "solver": tds[3].get_text().strip(),
                "method": tds[4].get_text().strip(),
                "date": tds[5].get_text().strip() if len(tds) > 5 else "",
                "competition": tds[6].get_text().strip()
                if len(tds) > 6
                else "",
                "tags": tds[7].get_text().strip() if len(tds) > 7 else "",
            }
        )
    has_next = bool(soup.find("a", string=re.compile(r"Next")))
    return rows, has_next


def collect_all_zb_solves():
    all_zb = []
    page = 1
    while True:
        rows, has_next = get_rows_on_page(page)
        zb_rows = [r for r in rows if r["method"] == "ZB"]
        all_zb.extend(zb_rows)
        if page % 10 == 0 or not has_next:
            print(
                f"  Page {page}: {len(rows)} rows, {len(zb_rows)} ZB | total ZB so far: {len(all_zb)}"
            )
        if not has_next:
            break
        page += 1
        time.sleep(DELAY)
    return all_zb


def parse_solve_page(solve_id):
    url = f"{BASE}/solve/{solve_id}"
    try:
        r = SESSION.get(url, timeout=15)
    except Exception as e:
        return None
    if r.status_code != 200:
        return None

    soup = BeautifulSoup(r.text, "html.parser")
    lines = [l.strip() for l in soup.get_text("\n").splitlines() if l.strip()]

    zbll_alg = None
    zbls_alg = None
    zbll_comment = None
    zbls_comment = None

    for line in lines:
        if "//" not in line:
            continue
        before, _, comment = line.partition("//")
        before = before.strip()
        comment = comment.strip()
        cup = comment.upper()

        if "ZBLL" in cup and before:
            zbll_alg = before
            zbll_comment = comment
        if "ZBLS" in cup and before:
            zbls_alg = before
            zbls_comment = comment

    if zbll_alg is None and zbls_alg is None:
        return None

    # Try to extract case name from comments (e.g. "ZBLL T6", "ZBLL (T6)", "ZBLLT6")
    def extract_case(tag, comment):
        if comment is None:
            return tag
        m = re.search(r"ZBL[LS]+\s*[-_]?\s*([A-Z]\d+)", comment, re.IGNORECASE)
        if m:
            return tag + m.group(1).upper()
        return tag

    zbll_case = extract_case("ZBLL", zbll_comment)
    zbls_case = extract_case("ZBLS", zbls_comment)

    return {
        "solve_id": solve_id,
        "url": url,
        "zbll_case": zbll_case,
        "zbll_alg": zbll_alg,
        "zbll_comment": zbll_comment,
        "zbls_case": zbls_case,
        "zbls_alg": zbls_alg,
        "zbls_comment": zbls_comment,
    }


def main():
    # --- Phase 1: collect all ZB solve metadata ---
    print("=== Phase 1: Collecting ZB solve IDs from index pages ===")
    zb_solves = collect_all_zb_solves()
    print(f"\nTotal ZB solves found: {len(zb_solves)}")

    with open("reco_zb_metadata.json", "w") as f:
        json.dump(zb_solves, f, indent=2)
    print("Saved metadata to reco_zb_metadata.json")

    # --- Phase 2: fetch each solve page ---
    print("\n=== Phase 2: Fetching solve pages for ZBLL/ZBLS ===")
    results = []
    meta_by_id = {s["id"]: s for s in zb_solves}

    for i, solve in enumerate(zb_solves):
        sid = solve["id"]
        rec = parse_solve_page(sid)
        if rec:
            # Merge metadata
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
                f"  [{i + 1}/{len(zb_solves)}] {sid} {solve['solver']} | "
                f"ZBLS={rec['zbls_case']} | ZBLL={rec['zbll_case']} | {rec['zbll_alg'] or ''}"
            )
        elif (i + 1) % 50 == 0:
            print(f"  [{i + 1}/{len(zb_solves)}] no ZBLL/ZBLS in last 50")
        time.sleep(DELAY)

    # Save raw JSON
    with open("reco_zbll_raw.json", "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved {len(results)} ZBLL/ZBLS records to reco_zbll_raw.json")

    # --- Summary ---
    print("\n=== ZBLL Algorithms by Case ===")
    by_case = {}
    for r in results:
        if r["zbll_alg"]:
            case = r["zbll_case"] or "ZBLL?"
            by_case.setdefault(case, {})
            alg = r["zbll_alg"]
            by_case[case].setdefault(alg, [])
            by_case[case][alg].append(f"{r['solver']} (solve {r['solve_id']})")

    for case in sorted(by_case):
        print(f"\n{case}:")
        for alg, uses in by_case[case].items():
            print(f"  {alg}")
            for u in uses:
                print(f"    {u}")

    print("\n=== ZBLS Algorithms ===")
    for r in results:
        if r["zbls_alg"]:
            print(
                f"  [{r['zbls_case']}] {r['solver']} (solve {r['solve_id']}): {r['zbls_alg']}"
            )
            if r["zbls_comment"] and r["zbls_comment"] != "ZBLS":
                print(f"    comment: {r['zbls_comment']}")

    print("\n=== Solver ZBLL Summary ===")
    solver_cases = {}
    for r in results:
        if r["zbll_alg"]:
            solver_cases.setdefault(r["solver"], set()).add(r["zbll_case"])
    for solver, cases in sorted(solver_cases.items(), key=lambda x: -len(x[1])):
        print(
            f"  {solver}: {len(cases)} distinct cases — {', '.join(sorted(cases))}"
        )


if __name__ == "__main__":
    main()
