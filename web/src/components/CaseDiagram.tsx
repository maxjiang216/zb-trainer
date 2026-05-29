import type { CubeState } from "../cube/cube";
import {
  cornerSideColor,
  cornerTopColor,
  edgeSideColor,
  edgeTopColor,
  FACE_COLOR,
} from "../cube/facelets";

/**
 * Top-down last-layer recognition diagram, speedcubedb-style: a 3x3 grid of the
 * U-face stickers surrounded by the side stickers of the top-layer pieces
 * (front at the bottom). Pure SVG, sized by `size` (pixels, square-ish).
 */
interface CaseDiagramProps {
  state: CubeState;
  size?: number;
}

const CELL = 34;
const SIDE = 13;
const GAP = 3;
const PAD = 4;
const GRID_ORIGIN = PAD + SIDE + GAP;
const SPAN = 3 * CELL;
const TOTAL = 2 * PAD + 2 * SIDE + 2 * GAP + SPAN;

const STICKER_STROKE = "#222";
const BG = "#1b1b1f";

interface Tile {
  x: number;
  y: number;
  w: number;
  h: number;
  color: string;
}

function buildTiles(state: CubeState): Tile[] {
  const tiles: Tile[] = [];
  const cellAt = (col: number, row: number) => ({
    x: GRID_ORIGIN + col * CELL,
    y: GRID_ORIGIN + row * CELL,
  });

  // ── U-face 3x3 ──────────────────────────────────────────────────────────
  // Grid mapping (front at bottom): rows back→front, cols left→right.
  const uFace: { col: number; row: number; color: string }[] = [
    { col: 0, row: 0, color: cornerTopColor(state, 2) }, // ULB
    { col: 1, row: 0, color: edgeTopColor(state, 3) }, // UB
    { col: 2, row: 0, color: cornerTopColor(state, 3) }, // UBR
    { col: 0, row: 1, color: edgeTopColor(state, 2) }, // UL
    { col: 1, row: 1, color: FACE_COLOR.U }, // center
    { col: 2, row: 1, color: edgeTopColor(state, 0) }, // UR
    { col: 0, row: 2, color: cornerTopColor(state, 1) }, // UFL
    { col: 1, row: 2, color: edgeTopColor(state, 1) }, // UF
    { col: 2, row: 2, color: cornerTopColor(state, 0) }, // URF
  ];
  for (const f of uFace) {
    const { x, y } = cellAt(f.col, f.row);
    tiles.push({ x, y, w: CELL, h: CELL, color: f.color });
  }

  // ── Side stickers ─────────────────────────────────────────────────────────
  // Back (top of diagram), left→right by column.
  const back = [
    cornerSideColor(state, 2, "B"),
    edgeSideColor(state, 3).color,
    cornerSideColor(state, 3, "B"),
  ];
  back.forEach((color, col) => {
    tiles.push({ x: GRID_ORIGIN + col * CELL, y: PAD, w: CELL, h: SIDE, color });
  });

  // Front (bottom), left→right by column.
  const front = [
    cornerSideColor(state, 1, "F"),
    edgeSideColor(state, 1).color,
    cornerSideColor(state, 0, "F"),
  ];
  const frontY = GRID_ORIGIN + SPAN + GAP;
  front.forEach((color, col) => {
    tiles.push({ x: GRID_ORIGIN + col * CELL, y: frontY, w: CELL, h: SIDE, color });
  });

  // Left, top→bottom by row.
  const left = [
    cornerSideColor(state, 2, "L"),
    edgeSideColor(state, 2).color,
    cornerSideColor(state, 1, "L"),
  ];
  left.forEach((color, row) => {
    tiles.push({ x: PAD, y: GRID_ORIGIN + row * CELL, w: SIDE, h: CELL, color });
  });

  // Right, top→bottom by row.
  const right = [
    cornerSideColor(state, 3, "R"),
    edgeSideColor(state, 0).color,
    cornerSideColor(state, 0, "R"),
  ];
  const rightX = GRID_ORIGIN + SPAN + GAP;
  right.forEach((color, row) => {
    tiles.push({ x: rightX, y: GRID_ORIGIN + row * CELL, w: SIDE, h: CELL, color });
  });

  return tiles;
}

export function CaseDiagram({ state, size = 132 }: CaseDiagramProps) {
  const tiles = buildTiles(state);
  return (
    <svg
      viewBox={`0 0 ${TOTAL} ${TOTAL}`}
      width={size}
      height={size}
      role="img"
      aria-label="ZBLL case diagram"
    >
      <rect x={0} y={0} width={TOTAL} height={TOTAL} rx={6} fill={BG} />
      {tiles.map((t, i) => (
        <rect
          key={i}
          x={t.x}
          y={t.y}
          width={t.w}
          height={t.h}
          rx={2}
          fill={t.color}
          stroke={STICKER_STROKE}
          strokeWidth={1.5}
        />
      ))}
    </svg>
  );
}
