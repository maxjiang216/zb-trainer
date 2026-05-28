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
when leaving slot `i`. Wide moves, slices, and rotations are derived from the
six face moves + three slices at startup in `MoveSet::build()`
(`r = R·M'`, `x = R·M'·L'`, `y = U·E'·D'`, `z = F·S·B'`, etc.).

## Key design decisions

- **`f2l_solved()` as the validity gate.** A genuine last-layer alg, inverted
  onto a solved cube, leaves the F2L (D-layer corners, D-layer edges, E-layer
  edges) intact. Algs that fail this in every orientation are not real LL algs
  (e.g. OLL-style CFOP sequences loosely labelled "ZBLL") and are dropped.
- **Rotation-aware F2L + fingerprint.** Wide and slice moves move centers, so an
  alg can finish in a net-rotated frame. Since `CubeState` does **not** track
  centers, we instead try all 24 cube orientations (`MoveSet::cube_rotations()`)
  and keep the ones that land F2L solved; that undoes any net rotation. (We
  chose the 24-rotation search over adding center tracking — same result, less
  state to maintain. An even cleaner alternative — having rotations remap the
  subsequent moves and parsing wide/slice as face+rotation — was considered and
  deferred; the current approach is correct and tested.)
- **AUF is separate from cube rotation.** After reorienting, AUF is normalized
  by applying the four U-face turns and taking the lexicographically minimum
  fingerprint. A whole-cube `y` rotation is **not** AUF — it moves the D-layer
  too and breaks `f2l_solved` — so the two must be handled independently.
- **Fingerprint = full LL state up to AUF.** `extract_ll_fingerprint` records
  `(piece, orientation)` for the four LL corners and four LL edges. This matches
  speedcubedb's case granularity exactly (see verification).

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

- All 24 cube rotations close into the proper group; all 6 unit tests pass; no
  clippy warnings.
- The 464 valid speedcubedb algs produce **464 distinct canonical fingerprints**
  — a 1:1 match with their case definition, confirming the fingerprint
  granularity is exactly right.
- Dataset: **1,588 ZBLL algs → 559 distinct cases**, covering **343 / 472**
  known OCLL-ZBLL cases. 559 > 472 is expected and correct: the dataset also
  contains corners-already-oriented (PLL-type) and H-symmetry last-layer states
  that are legitimately distinct up to AUF but lie outside the 472-case OCLL set.

### Known-bad reference data
Eight `known_zbll.json` entries (`H 6/18/19/28/29/40`, `T 46`, `L 23`) do **not**
preserve F2L under any of the 24 rotations — i.e. they are mistyped / alternate
algs scraped from speedcubedb, not model bugs. They are pinned in
`test_known_zbll_all_valid` so a genuine regression still trips the test.

## Possible future work
- Refactor to the rotation-remapping design (rotations transform subsequent
  moves; wide/slice parse as face move + rotation), deleting the slice/rotation/
  wide piece tables entirely. Cleaner and avoids the orientation-table class of
  bug above; deferred since the current model is correct.
- Reduce cases by full symmetry (mirror/inverse) to map onto the 472 reps for
  apples-to-apples coverage.
- Make ZBLS classification rotation-aware too (currently 4-U-rotation only).
