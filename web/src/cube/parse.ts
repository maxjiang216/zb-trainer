import {
  type FaceMove,
  type MoveSet,
  identity,
  inverse,
  isIdentity,
  movesEq,
  then,
} from "./moves";

/**
 * Parser ported from `zb-trainer/src/parse.rs`. Rotations never permute pieces:
 * we track a running cube orientation and emit each face turn conjugated back
 * into the fixed frame (`orient · face · orient⁻¹`). Wide and slice moves expand
 * to a face turn plus the rotation they carry, feeding the same machinery. Any
 * net rotation the alg leaves behind is appended at the end so the output is
 * exactly equivalent to the input alg (downstream classification reorients over
 * all 24 rotations to absorb it).
 */

export class BigCubeError extends Error {}
export class UnknownTokenError extends Error {}

type Prim = { kind: "rot" | "face"; m: FaceMove };

function primInverse(p: Prim): Prim {
  return { kind: p.kind, m: inverse(p.m) };
}

/** Parse an algorithm string into a sequence of fixed-frame face moves. */
export function parseAlg(s: string, ms: MoveSet): FaceMove[] {
  const prims: Prim[] = [];
  for (const token of s.split(/\s+/).filter((t) => t.length > 0)) {
    tokenPrims(token, ms, prims);
  }

  const out: FaceMove[] = [];
  let orient = identity();
  for (const p of prims) {
    if (p.kind === "rot") {
      orient = then(orient, p.m);
    } else {
      out.push(then(then(orient, p.m), inverse(orient)));
    }
  }
  if (!isIdentity(orient)) {
    out.push(orient);
  }
  return out;
}

function tokenPrims(token: string, ms: MoveSet, out: Prim[]): void {
  if (token.length > 0 && token[0] >= "0" && token[0] <= "9") {
    throw new BigCubeError(token);
  }
  const [base, suffix] = splitBaseSuffix(token);
  const prime = suffix.includes("'");
  const double = suffix.includes("2");

  const quarter = quarterPrims(base, ms);

  if (double) {
    out.push(...quarter, ...quarter);
  } else if (prime) {
    for (let i = quarter.length - 1; i >= 0; i--) {
      out.push(primInverse(quarter[i]));
    }
  } else {
    out.push(...quarter);
  }
}

/**
 * Quarter-turn primitive expansion of a base move. Wide/slice identities (all
 * verified by move-table composition): `Rw = x L`, `Lw = x' R`, `Uw = y D`,
 * `Dw = y' U`, `Fw = z B`, `Bw = z' F`, `M = L' x' R`, `E = D' y' U`,
 * `S = F' z B`.
 */
function quarterPrims(base: string, ms: MoveSet): Prim[] {
  const rot = (m: FaceMove): Prim => ({ kind: "rot", m });
  const face = (m: FaceMove): Prim => ({ kind: "face", m });

  const isWide = base.endsWith("w");
  const core = isWide ? base.slice(0, base.length - 1) : base;

  switch (core) {
    case "U":
      return isWide ? [rot(ms.y), face(ms.d)] : [face(ms.u)];
    case "D":
      return isWide ? [rot(ms.yp), face(ms.u)] : [face(ms.d)];
    case "R":
      return isWide ? [rot(ms.x), face(ms.l)] : [face(ms.r)];
    case "L":
      return isWide ? [rot(ms.xp), face(ms.r)] : [face(ms.l)];
    case "F":
      return isWide ? [rot(ms.z), face(ms.b)] : [face(ms.f)];
    case "B":
      return isWide ? [rot(ms.zp), face(ms.f)] : [face(ms.b)];

    // Lowercase wide moves.
    case "u":
      return [rot(ms.y), face(ms.d)];
    case "d":
      return [rot(ms.yp), face(ms.u)];
    case "r":
      return [rot(ms.x), face(ms.l)];
    case "l":
      return [rot(ms.xp), face(ms.r)];
    case "f":
      return [rot(ms.z), face(ms.b)];
    case "b":
      return [rot(ms.zp), face(ms.f)];

    // Slices.
    case "M":
      return [face(ms.lp), rot(ms.xp), face(ms.r)];
    case "E":
      return [face(ms.dp), rot(ms.yp), face(ms.u)];
    case "S":
      return [face(ms.fp), rot(ms.z), face(ms.b)];

    // Rotations.
    case "x":
      return [rot(ms.x)];
    case "y":
      return [rot(ms.y)];
    case "z":
      return [rot(ms.z)];

    default:
      throw new UnknownTokenError(`${core} wide=${isWide}`);
  }
}

function splitBaseSuffix(token: string): [string, string] {
  let end = token.length;
  for (let i = 0; i < token.length; i++) {
    const c = token[i];
    const isAlpha = (c >= "a" && c <= "z") || (c >= "A" && c <= "Z");
    if (!isAlpha) {
      end = i;
      break;
    }
  }
  const base = token.slice(0, end);
  const suffix = token.slice(end);
  if (base.length === 0) {
    throw new UnknownTokenError(token);
  }
  return [base, suffix];
}

/** Invert a sequence of moves (reverse + invert each). */
export function invertAlg(moves: FaceMove[]): FaceMove[] {
  return moves.map(inverse).reverse();
}

export function isUFamily(m: FaceMove, ms: MoveSet): boolean {
  return [ms.u, ms.up, ms.u2].some((u) => movesEq(m, u));
}

export function isRotation(m: FaceMove, ms: MoveSet): boolean {
  return [ms.x, ms.xp, ms.x2, ms.y, ms.yp, ms.y2, ms.z, ms.zp, ms.z2].some((r) =>
    movesEq(m, r),
  );
}

export function isAuf(m: FaceMove, ms: MoveSet): boolean {
  return isUFamily(m, ms) || isRotation(m, ms);
}
