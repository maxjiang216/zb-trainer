import { type CubeState, apply, applyAlg, llEdgesOriented, f2lSolved, solved } from "./cube";
import { type FaceMove, type MoveSet, U } from "./moves";
import { invertAlg, isAuf, isUFamily } from "./parse";

/**
 * Classifier ported from `zb-trainer/src/normalize.rs`. A ZBLL "case" is
 * identified by a canonical fingerprint of its last-layer state, normalized over
 * both whole-cube reorientation and AUF.
 */

/** Strip leading AUF + rotations and trailing U-family moves. */
export function stripAuf(moves: FaceMove[], ms: MoveSet): FaceMove[] {
  let start = moves.findIndex((m) => !isAuf(m, ms));
  if (start < 0) start = moves.length;
  const mid = moves.slice(start);
  let end = -1;
  for (let i = mid.length - 1; i >= 0; i--) {
    if (!isUFamily(mid[i], ms)) {
      end = i + 1;
      break;
    }
  }
  if (end < 0) end = 0;
  return mid.slice(0, end);
}

/** U-layer corners + edges (slots 0-3): (piece, orientation) pairs. */
function extractLlFingerprint(state: CubeState): string {
  const v: number[] = [];
  for (let i = 0; i < 4; i++) {
    v.push(state.cp[i], state.co[i]);
  }
  for (let i = 0; i < 4; i++) {
    v.push(state.ep[i], state.eo[i]);
  }
  return v.join(",");
}

export interface ZbllClass {
  fingerprint: string;
  edgesOriented: boolean;
  /** The case state (LL scramble) read in the canonical orientation/AUF. */
  state: CubeState;
}

/**
 * Classify an AUF-stripped alg as a ZBLL case. Applies the inverse alg to a
 * solved cube, tries every cube rotation that lands F2L solved, then normalizes
 * over the 4 AUF turns, keeping the lexicographically minimum fingerprint.
 * Returns null if no rotation solves F2L (not a genuine last-layer alg).
 */
export function zbllCanonical(
  core: FaceMove[],
  rots: FaceMove[],
): ZbllClass | null {
  const state = applyAlg(solved(), invertAlg(core));

  let best: string | null = null;
  let bestState: CubeState | null = null;
  let edgesOriented = false;
  for (const g of rots) {
    const s = apply(state, g);
    if (!f2lSolved(s)) continue;
    edgesOriented = llEdgesOriented(s);
    let t = s;
    for (let k = 0; k < 4; k++) {
      const fp = extractLlFingerprint(t);
      if (best === null || fp < best) {
        best = fp;
        bestState = t;
      }
      t = apply(t, U);
    }
  }
  if (best === null || bestState === null) return null;
  return { fingerprint: best, edgesOriented, state: bestState };
}
