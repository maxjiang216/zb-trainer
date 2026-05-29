import { type CubeState, apply, applyAlg, f2lSolved, solved } from "./cube";
import { type FaceMove, type MoveSet } from "./moves";
import { invertAlg, parseAlg } from "./parse";

/**
 * The trainer practice loop, per the user's model:
 *
 *   - The virtual cube `V` starts solved (reset → solved).
 *   - Each round the trainer names a case and you APPLY its alg FORWARD to the
 *     current cube. The "resulting" state becomes the next round's starting
 *     cube, so practice chains continuously: V₀ = solved, Vₙ = applyAlg(Vₙ₋₁,
 *     algₙ). Applying an alg to a solved cube yields its "inverse case"; every
 *     ZBLL alg preserves F2L + edge orientation, so the cube stays a
 *     recognizable last-layer state throughout — no scrambling needed.
 *   - Alongside, the trainer shows the case the alg *solves* — the recognition
 *     image — which is always `invert(alg)` applied to a solved cube.
 *
 * No post-AUF is applied; AUF variety arises naturally as the chain progresses.
 */
export interface Round {
  /** The case this alg solves (recognition image): invert(alg) on solved. */
  caseSolved: CubeState;
  /** The cube after applying the alg forward to `base` (the running chain). */
  after: CubeState;
}

/**
 * Re-orient a state so the last layer is back on top (F2L on the bottom). Algs
 * containing rotations or wide moves leave the cube in a net-rotated frame; on a
 * real cube you simply re-grip. We pick the cube rotation that lands F2L solved,
 * matching the diagram's U-up assumption. Falls back to the state unchanged if
 * none qualifies (should not happen for a genuine last-layer alg).
 */
function reorientToTop(state: CubeState, rots: FaceMove[]): CubeState {
  for (const g of rots) {
    const s = apply(state, g);
    if (f2lSolved(s)) return s;
  }
  return state;
}

export function buildRound(
  base: CubeState,
  algString: string,
  ms: MoveSet,
  rots: FaceMove[],
): Round {
  const moves = parseAlg(algString, ms);
  const caseSolved = reorientToTop(applyAlg(solved(), invertAlg(moves)), rots);
  const after = reorientToTop(applyAlg(base, moves), rots);
  return { caseSolved, after };
}
