/**
 * Build-time data pipeline. Classifies the scraped `reco` canonical algs onto
 * the 472 known speedcubedb ZBLL cases (using the ported, verified classifier)
 * and emits `src/data/cases.json` for the web app.
 *
 * Each output case carries: its identity (subset + number), the canonical
 * last-layer state for rendering, and the list of algs that solve it — the
 * speedcubedb reference alg plus every reco alg, annotated with which top
 * cubers use it.
 *
 * Run with `npm run gen:cases`.
 */
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import type { CubeState } from "../src/cube/cube";
import { buildMoveSet, cubeRotations } from "../src/cube/moves";
import { parseAlg, BigCubeError, UnknownTokenError } from "../src/cube/parse";
import { stripAuf, zbllCanonical } from "../src/cube/classify";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", ".."); // zb-trainer/
const projectsRoot = resolve(repoRoot, ".."); // projects/

interface KnownCase {
  subset: string;
  n: number;
  alg: string;
}

interface CanonicalAlg {
  canonical_alg: string;
  uses: number;
  unique_solvers: string[];
}

interface OutAlg {
  alg: string;
  source: "speedcubedb" | "reco.nz";
  solvers: string[];
  uses: number;
}

interface OutCase {
  id: string;
  subset: string;
  n: number;
  state: CubeState;
  algs: OutAlg[];
}

const ms = buildMoveSet();
const rots = cubeRotations(ms);
console.log(`rotation group size: ${rots.length} (expect 24)`);

/** Parse + strip + classify; returns the fingerprint, or null if not a ZBLL. */
function classify(alg: string): { fingerprint: string; state: CubeState } | null {
  let moves;
  try {
    moves = parseAlg(alg, ms);
  } catch (e) {
    if (e instanceof BigCubeError || e instanceof UnknownTokenError) return null;
    throw e;
  }
  const core = stripAuf(moves, ms);
  if (core.length === 0) return null;
  const c = zbllCanonical(core, rots);
  if (c === null) return null;
  return { fingerprint: c.fingerprint, state: c.state };
}

// ── Known speedcubedb cases: the canonical case set + reference algs ─────────
const known: KnownCase[] = JSON.parse(
  readFileSync(resolve(repoRoot, "known_zbll.json"), "utf8"),
);

const byFingerprint = new Map<string, OutCase>();
let knownUnclassified = 0;
for (const k of known) {
  const c = classify(k.alg);
  if (c === null) {
    knownUnclassified++;
    console.warn(`known case ${k.subset}${k.n} did not classify: ${k.alg}`);
    continue;
  }
  const id = `${k.subset}${k.n}`;
  const out: OutCase = {
    id,
    subset: k.subset,
    n: k.n,
    state: c.state,
    algs: [{ alg: k.alg, source: "speedcubedb", solvers: [], uses: 0 }],
  };
  byFingerprint.set(c.fingerprint, out);
}
console.log(
  `known cases: ${known.length}, classified: ${byFingerprint.size}, ` +
    `failed: ${knownUnclassified}`,
);

// ── reco canonical algs: attach to matching cases with attribution ───────────
const canonical: CanonicalAlg[] = JSON.parse(
  readFileSync(
    resolve(projectsRoot, "reco_scraper", "reco_zbll_canonical.json"),
    "utf8",
  ),
);

let matched = 0;
let unmatched = 0;
const coveredFps = new Set<string>();
for (const c of canonical) {
  const cls = classify(c.canonical_alg);
  if (cls === null) {
    unmatched++;
    continue;
  }
  const target = byFingerprint.get(cls.fingerprint);
  if (!target) {
    // A real LL case that is not among the 472 speedcubedb cases (e.g. an
    // edges-mis-oriented case loosely labeled ZBLL). Skip for the ZBLL trainer.
    unmatched++;
    continue;
  }
  matched++;
  coveredFps.add(cls.fingerprint);
  target.algs.push({
    alg: c.canonical_alg,
    source: "reco.nz",
    solvers: c.unique_solvers ?? [],
    uses: c.uses ?? 0,
  });
}

console.log(
  `reco canonical algs: ${canonical.length}, matched: ${matched}, ` +
    `unmatched: ${unmatched}`,
);
console.log(
  `coverage: ${coveredFps.size}/${byFingerprint.size} cases have a reco alg`,
);

// ── Assemble + sort ─────────────────────────────────────────────────────────
const subsetOrder = ["H", "T", "U", "L", "Pi", "S", "AS"];
const cases = [...byFingerprint.values()];
for (const c of cases) {
  // Within a case: speedcubedb first, then reco algs by popularity.
  c.algs.sort((a, b) => {
    if (a.source !== b.source) return a.source === "speedcubedb" ? -1 : 1;
    return b.uses - a.uses;
  });
}
cases.sort((a, b) => {
  const sa = subsetOrder.indexOf(a.subset);
  const sb = subsetOrder.indexOf(b.subset);
  if (sa !== sb) return sa - sb;
  return a.n - b.n;
});

const outDir = resolve(here, "..", "src", "data");
mkdirSync(outDir, { recursive: true });
const outPath = resolve(outDir, "cases.json");
writeFileSync(outPath, JSON.stringify(cases));
console.log(`wrote ${cases.length} cases to ${outPath}`);
