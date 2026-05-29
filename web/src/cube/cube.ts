import type { FaceMove } from "./moves";

/**
 * A cube state in the Kociemba cubie model. Ported from `zb-trainer/src/cube.rs`.
 *   - cp[i] = which corner piece is in slot i; co[i] = its orientation (0-2).
 *   - ep/eo  = same for edges (orientation 0-1).
 * Solved = identity permutation, zero orientation.
 */
export interface CubeState {
  cp: number[];
  co: number[];
  ep: number[];
  eo: number[];
}

export function solved(): CubeState {
  return {
    cp: [0, 1, 2, 3, 4, 5, 6, 7],
    co: [0, 0, 0, 0, 0, 0, 0, 0],
    ep: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  };
}

export function apply(s: CubeState, m: FaceMove): CubeState {
  const n: CubeState = {
    cp: new Array(8).fill(0),
    co: new Array(8).fill(0),
    ep: new Array(12).fill(0),
    eo: new Array(12).fill(0),
  };
  for (let i = 0; i < 8; i++) {
    const dst = m.cp[i];
    n.cp[dst] = s.cp[i];
    n.co[dst] = (s.co[i] + m.co[i]) % 3;
  }
  for (let i = 0; i < 12; i++) {
    const dst = m.ep[i];
    n.ep[dst] = s.ep[i];
    n.eo[dst] = (s.eo[i] + m.eo[i]) % 2;
  }
  return n;
}

export function applyAlg(s: CubeState, moves: FaceMove[]): CubeState {
  return moves.reduce(apply, s);
}

/** True if every D-layer corner, E-layer edge, and D-layer edge is solved. */
export function f2lSolved(s: CubeState): boolean {
  for (let i = 4; i < 8; i++) {
    if (s.cp[i] !== i || s.co[i] !== 0) return false;
  }
  for (let i = 4; i < 8; i++) {
    if (s.ep[i] !== i || s.eo[i] !== 0) return false;
  }
  for (let i = 8; i < 12; i++) {
    if (s.ep[i] !== i || s.eo[i] !== 0) return false;
  }
  return true;
}

/** True if all LL edges (slots 0-3) are oriented (eo == 0). */
export function llEdgesOriented(s: CubeState): boolean {
  return s.eo.slice(0, 4).every((x) => x === 0);
}
