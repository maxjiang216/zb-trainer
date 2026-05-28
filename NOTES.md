# zb-trainer — Design Notes & Bug Log

A Rust cube simulator that takes scraped ZBLL/ZBLS reconstructions, applies the
inverse of each algorithm to a solved cube, and reads off a canonical
fingerprint so structurally different alg strings that solve the **same case**
group together. Input: `../reco_scraper/reco_zbll_fixed.json` (1,588 ZBLL algs
scraped from reco.nz). Reference: `known_zbll.json` (472 speedcubedb cases).

## What it does

1. Parse each alg string into a sequence of `FaceMove`s (`parse.rs`).
2. Strip AUF / leading rotations (`normalize.rs::strip_auf`).
3. Apply the **inverse** alg to a solved cube (`cube.rs`).
4. Reorient + AUF-normalize and read the last-layer state as a canonical
   fingerprint (`normalize.rs::zbll_canonical`).
5. Group algs by fingerprint; report distinct cases + coverage vs. the 472
   known speedcubedb cases (`main.rs`).

## Cube representation

Standard Kociemba cubie model in `cube.rs`:
- `cp[i]` / `co[i]` — which corner piece is in slot `i`, and its orientation (mod 3).
- `ep[i]` / `eo[i]` — same for edges (orientation mod 2).
- Solved = identity permutation, zero orientation.

Slot numbering (fixed frame):
- Corners: `0=URF 1=UFL 2=ULB 3=UBR 4=DFR 5=DLF 6=DBL 7=DRB`
- Edges: `0=UR 1=UF 2=UL 3=UB 4=DR 5=DF 6=DL 7=DB 8=FR 9=FL 10=BL 11=BR`

A move (`moves.rs::FaceMove`) is `cp/co/ep/eo` arrays where `cp[i]=j` means "the
piece in slot `i` moves to slot `j`", and `co[i]` is the orientation delta added
when leaving slot `i`. `MoveSet` stores the six faces and three rotations (`x`,
`y`, `z`, derived once from the private slice constants via `x = R·M'·L'` etc.).
Wide and slice moves are **not** stored — the parser expands them.

## Key design decisions

- **`f2l_solved()` as the validity gate.** A genuine last-layer alg, inverted
  onto a solved cube, leaves the F2L (D-layer corners, D-layer edges, E-layer
  edges) intact. Algs that fail this in every orientation are not real LL algs
  (e.g. OLL-style CFOP sequences loosely labelled "ZBLL") and are dropped.
- **Rotations remap moves, not pieces (`parse.rs`).** Wide and slice moves
  rotate the cube's centers, so an alg can finish in a net-rotated frame. Rather
  than give rotations piece tables, the parser tracks a running orientation and
  emits each face turn conjugated back into the fixed frame
  (`orient · face · orient⁻¹`). Wide/slice moves expand to a face turn plus the
  rotation they carry (`Rw = x L`, `M = L' x' R`, …), feeding the same path. The
  net cube rotation is appended at the end so the output is exactly equivalent
  to the alg. This handles setup rotations (a leading `y`) and intrinsic
  conjugations (`x' … x`) uniformly — it fixed three speedcubedb algs the old
  string-stripping approach mis-handled (`H 28`, `U 41`, `U 42`).
- **Rotation-aware F2L + fingerprint.** Since `CubeState` does **not** track
  centers, classification tries all 24 cube orientations
  (`MoveSet::cube_rotations()`) and keeps the ones that land F2L solved; that
  absorbs whatever net rotation the parser appended. (Chosen over center
  tracking — same result, less state.)
- **AUF is separate from cube rotation.** After reorienting, AUF is normalized
  by applying the four U-face turns and taking the lexicographically minimum
  fingerprint. A whole-cube `y` rotation is **not** AUF — it moves the D-layer
  too and breaks `f2l_solved` — so the two must be handled independently.
- **Fingerprint = full LL state up to AUF.** `extract_ll_fingerprint` records
  `(piece, orientation)` for the four LL corners and four LL edges. This matches
  speedcubedb's case granularity exactly (see verification).
- **ZBLS uses the same scheme.** `zbls_canonical` is rotation-aware too: it
  keeps rotations where everything *except* the FR last slot is solved
  (`cube.rs::f2l_minus_fr_solved`), then reads the DFR-corner + FR-edge + LL-edge
  fingerprint over the 4 AUF turns. Algs that don't reduce to a last-slot case
  under any rotation are skipped rather than false-grouped.

## Bugs found and fixed

The original move tables had a mixed-handedness model: `U` and `F` were correct
clockwise turns, but `R`, `L`, `B`, `D` encoded the **prime (CCW)** turn. This
is invisible to `X⁴ = identity` and `X·X' = identity` checks (both hold for
either direction), so it slipped through. It surfaced as: with the F2L filter
off, ~746 bogus "cases"; with it on, only ~22 — because genuine ZBLLs like Sune
spuriously failed `f2l_solved`.

| Move(s) | Symptom | Root cause | Fix |
|---|---|---|---|
| `R`, `L`, `B` | Sune broke F2L; AntiSune passed | Tables encoded `R'`/`L'`/`B'` (e.g. `R` did `UR→FR`, up→front, instead of `UR→BR`) | Reverse the corner+edge 4-cycles |
| `D` | All `D`-using known algs failed | Encoded `D'` (`DF→DL`, mirroring `U`) instead of `DF→DR` | Reverse the cycle |
| `M`, `E` | Wide/rotation algs failed (220→70→46) | Slices followed the *old* (wrong) face direction | Flip `M` and `E` permutations |
| `M`, `E` (orientation) | Wide moves left LL edges mis-oriented; `⟨x,y,z⟩` was 48 not 24 | Slices weren't flipping the edges they move | Add edge-orientation flips to `M` and `E` (all three slices flip their edges) |
| Normalization | 559 → should be ≤472; AUF not collapsing | Normalized over cube rotations only — a `y` rotation breaks `f2l_solved`, so AUF never collapsed | Normalize over cube rotation **and** the 4 U-turns |
| Net rotation | 12 known algs failing | `CubeState` has no centers; wide/slice algs end net-rotated | Rotation-aware `f2l_solved` over the 24 orientations |

### Debugging method that worked
`X⁴=id` tests were useless here (direction-agnostic). The decisive tests use
**real algorithms as ground truth**:
- `test_ll_algs_preserve_f2l`: Sune/AntiSune/T-/Y-/H-/U-perms must preserve F2L
  and edge orientation. (Caught the inverted faces — and a wrong "AntiSune"
  string I'd typed; `R' U2 R U' R' U' R` is not actually a pure LL alg.)
- `test_rotation_group_is_24`: `⟨x,y,z⟩` must have exactly 24 elements. (Caught
  the `E`-slice edge-orientation bug — it was 48.)
- `test_known_zbll_all_valid`: every speedcubedb alg must classify as a valid
  ZBLL. (Drove the count down 380 → 70 → 46 → 12 → 8 as each layer was fixed.)

## Verification & results

- All 24 cube rotations close into the proper group; all 5 unit tests pass;
  default `cargo clippy` is clean.
- The valid speedcubedb algs produce a 1:1 set of distinct canonical
  fingerprints, confirming the fingerprint granularity exactly matches their
  case definition.
- Dataset: **1,588 ZBLL algs → 572 distinct cases**, covering **343 / 472**
  known OCLL-ZBLL cases; **710 ZBLS algs → 221 distinct cases**. 572 > 472 is
  expected and correct: the dataset also contains corners-already-oriented
  (PLL-type) and H-symmetry last-layer states that are legitimately distinct up
  to AUF but lie outside the 472-case OCLL set.

### Known-bad reference data
Seven `known_zbll.json` entries (`H 6/18/19/29/40`, `T 46`, `L 23`) do **not**
preserve F2L under any of the 24 rotations — i.e. they are mistyped / alternate
algs scraped from speedcubedb, not model bugs. They are pinned in
`test_known_zbll_all_valid` so a genuine regression still trips the test.
(`H 28`, `U 41`, `U 42` were previously in this list; the rotation-remapping
parser handles their wrapping rotations correctly and they now validate.)

## Possible future work
- Reduce cases by full symmetry (mirror/inverse) to map onto the 472 reps for
  apples-to-apples coverage.
- The full `lint.sh` runs clippy with `-D warnings` + pedantic + nursery; the
  codebase predates that bar and has pre-existing findings (integer casts,
  `use_self`, etc.). Default `cargo clippy` is clean. Bringing it to full
  pedantic-clean is a separate cleanup.
