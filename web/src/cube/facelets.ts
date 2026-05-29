import type { CubeState } from "./cube";

/**
 * Derives the visible sticker colors of the last layer from a CubeState, for
 * the top-down recognition diagram.
 *
 * Orientation convention (matches the cubie `apply` in this codebase, verified
 * empirically — see `npm run gen:cases` validation and the moves port):
 *   - A corner piece's three facelets are listed in CLOCKWISE order viewed from
 *     above, starting from its primary (U/D-axis) facelet. For a corner in a
 *     U-layer slot, the facelet that ends up on top is facelet[co]; the two side
 *     facelets follow clockwise.
 *   - An edge piece's two facelets are [primary, secondary]; in a U-layer slot
 *     the top facelet is facelet[eo].
 */

export type Face = "U" | "D" | "F" | "B" | "R" | "L";

export const FACE_COLOR: Record<Face, string> = {
  U: "#FFD500", // yellow (top)
  D: "#FFFFFF", // white
  F: "#00A651", // green
  B: "#0051BA", // blue
  R: "#C41E3A", // red
  L: "#FF7300", // orange
};

// Corner facelets in clockwise-from-above order, starting at the U/D facelet.
const CORNER_FACELETS: Face[][] = [
  ["U", "R", "F"], // 0 URF
  ["U", "F", "L"], // 1 UFL
  ["U", "L", "B"], // 2 ULB
  ["U", "B", "R"], // 3 UBR
  ["D", "F", "R"], // 4 DFR
  ["D", "L", "F"], // 5 DLF
  ["D", "B", "L"], // 6 DBL
  ["D", "R", "B"], // 7 DRB
];

// Edge facelets: [primary (U/D for U/D edges, F/B for E-slice), secondary].
const EDGE_FACELETS: Face[][] = [
  ["U", "R"], // 0 UR
  ["U", "F"], // 1 UF
  ["U", "L"], // 2 UL
  ["U", "B"], // 3 UB
  ["D", "R"], // 4 DR
  ["D", "F"], // 5 DF
  ["D", "L"], // 6 DL
  ["D", "B"], // 7 DB
  ["F", "R"], // 8 FR
  ["F", "L"], // 9 FL
  ["B", "L"], // 10 BL
  ["B", "R"], // 11 BR
];

/**
 * For each U-layer corner slot, the clockwise-from-above side directions
 * following the top facelet. Index k (1 or 2) maps facelet[(co + k) % 3] to
 * that side. Index 0 (the top) is omitted.
 */
const CORNER_SLOT_SIDES: Record<number, [Face, Face]> = {
  0: ["R", "F"], // URF
  1: ["F", "L"], // UFL
  2: ["L", "B"], // ULB
  3: ["B", "R"], // UBR
};

/** For each U-layer edge slot, the single side direction. */
const EDGE_SLOT_SIDE: Record<number, Face> = {
  0: "R", // UR
  1: "F", // UF
  2: "L", // UL
  3: "B", // UB
};

/** Color of the U-facelet of the corner currently in U-slot `slot` (0-3). */
export function cornerTopColor(state: CubeState, slot: number): string {
  const piece = state.cp[slot];
  const co = state.co[slot];
  return FACE_COLOR[CORNER_FACELETS[piece][co]];
}

/** Color of the U-facelet of the edge currently in U-slot `slot` (0-3). */
export function edgeTopColor(state: CubeState, slot: number): string {
  const piece = state.ep[slot];
  const eo = state.eo[slot];
  return FACE_COLOR[EDGE_FACELETS[piece][eo]];
}

/**
 * Color of the side sticker of the corner in U-slot `slot` facing direction
 * `dir`. `dir` must be one of the slot's two side directions.
 */
export function cornerSideColor(
  state: CubeState,
  slot: number,
  dir: Face,
): string {
  const piece = state.cp[slot];
  const co = state.co[slot];
  const sides = CORNER_SLOT_SIDES[slot];
  // Side directions sit at clockwise offsets 1 and 2 from the top facelet.
  const k = dir === sides[0] ? 1 : 2;
  return FACE_COLOR[CORNER_FACELETS[piece][(co + k) % 3]];
}

/** Color of the side sticker of the edge in U-slot `slot`. */
export function edgeSideColor(state: CubeState, slot: number): {
  dir: Face;
  color: string;
} {
  const piece = state.ep[slot];
  const eo = state.eo[slot];
  return {
    dir: EDGE_SLOT_SIDE[slot],
    color: FACE_COLOR[EDGE_FACELETS[piece][(eo + 1) % 2]],
  };
}
