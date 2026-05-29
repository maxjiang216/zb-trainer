/**
 * Cube move tables, ported verbatim from the verified Rust reference
 * (`zb-trainer/src/moves.rs`). The tables encode the standard Kociemba cubie
 * model:
 *   - cp[i] = j  : the piece in slot i moves to slot j.
 *   - co[i]      : orientation delta added when leaving slot i (mod 3 corners).
 *   - ep/eo      : same for edges (eo mod 2).
 *
 * Corner slots: 0=URF 1=UFL 2=ULB 3=UBR 4=DFR 5=DLF 6=DBL 7=DRB
 * Edge slots:   0=UR 1=UF 2=UL 3=UB 4=DR 5=DF 6=DL 7=DB 8=FR 9=FL 10=BL 11=BR
 *
 * These values are the corrected, handedness-consistent tables proven by the
 * Rust test suite (472 ZBLL algs classify 1:1, rotation group = 24). Do not
 * "tidy" them without re-validating against that suite.
 */
export interface FaceMove {
  cp: number[];
  co: number[];
  ep: number[];
  eo: number[];
}

export function identity(): FaceMove {
  return {
    cp: [0, 1, 2, 3, 4, 5, 6, 7],
    co: [0, 0, 0, 0, 0, 0, 0, 0],
    ep: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  };
}

/** Compose: apply `a`, then `b`. */
export function then(a: FaceMove, b: FaceMove): FaceMove {
  const r = identity();
  for (let i = 0; i < 8; i++) {
    const mid = a.cp[i];
    r.cp[i] = b.cp[mid];
    r.co[i] = (a.co[i] + b.co[mid]) % 3;
  }
  for (let i = 0; i < 12; i++) {
    const mid = a.ep[i];
    r.ep[i] = b.ep[mid];
    r.eo[i] = (a.eo[i] + b.eo[mid]) % 2;
  }
  return r;
}

export function inverse(m: FaceMove): FaceMove {
  const r = identity();
  for (let i = 0; i < 8; i++) {
    const dst = m.cp[i];
    r.cp[dst] = i;
    r.co[dst] = (3 - m.co[i]) % 3;
  }
  for (let i = 0; i < 12; i++) {
    const dst = m.ep[i];
    r.ep[dst] = i;
    r.eo[dst] = m.eo[i]; // flipping is self-inverse
  }
  return r;
}

export function pow2(m: FaceMove): FaceMove {
  return then(m, m);
}

export function movesEq(a: FaceMove, b: FaceMove): boolean {
  return (
    a.cp.every((v, i) => v === b.cp[i]) &&
    a.co.every((v, i) => v === b.co[i]) &&
    a.ep.every((v, i) => v === b.ep[i]) &&
    a.eo.every((v, i) => v === b.eo[i])
  );
}

export function isIdentity(m: FaceMove): boolean {
  return movesEq(m, identity());
}

// ── Base face turns (clockwise) ─────────────────────────────────────────────

export const U: FaceMove = {
  cp: [1, 2, 3, 0, 4, 5, 6, 7],
  co: [0, 0, 0, 0, 0, 0, 0, 0],
  ep: [1, 2, 3, 0, 4, 5, 6, 7, 8, 9, 10, 11],
  eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

export const D: FaceMove = {
  cp: [0, 1, 2, 3, 7, 4, 5, 6],
  co: [0, 0, 0, 0, 0, 0, 0, 0],
  ep: [0, 1, 2, 3, 7, 4, 5, 6, 8, 9, 10, 11],
  eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

export const R: FaceMove = {
  cp: [3, 1, 2, 7, 0, 5, 6, 4],
  co: [2, 0, 0, 1, 1, 0, 0, 2],
  ep: [11, 1, 2, 3, 8, 5, 6, 7, 0, 9, 10, 4],
  eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

export const L: FaceMove = {
  cp: [0, 5, 1, 3, 4, 6, 2, 7],
  co: [0, 1, 2, 0, 0, 2, 1, 0],
  ep: [0, 1, 9, 3, 4, 5, 10, 7, 8, 6, 2, 11],
  eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

export const F: FaceMove = {
  cp: [4, 0, 2, 3, 5, 1, 6, 7],
  co: [1, 2, 0, 0, 2, 1, 0, 0],
  ep: [0, 8, 2, 3, 4, 9, 6, 7, 5, 1, 10, 11],
  eo: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
};

export const B: FaceMove = {
  cp: [0, 1, 6, 2, 4, 5, 7, 3],
  co: [0, 0, 1, 2, 0, 0, 2, 1],
  ep: [0, 1, 2, 10, 4, 5, 6, 11, 8, 9, 7, 3],
  eo: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
};

const M: FaceMove = {
  cp: [0, 1, 2, 3, 4, 5, 6, 7],
  co: [0, 0, 0, 0, 0, 0, 0, 0],
  ep: [0, 5, 2, 1, 4, 7, 6, 3, 8, 9, 10, 11],
  eo: [0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0],
};

const E: FaceMove = {
  cp: [0, 1, 2, 3, 4, 5, 6, 7],
  co: [0, 0, 0, 0, 0, 0, 0, 0],
  ep: [0, 1, 2, 3, 4, 5, 6, 7, 11, 8, 9, 10],
  eo: [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
};

const S: FaceMove = {
  cp: [0, 1, 2, 3, 4, 5, 6, 7],
  co: [0, 0, 0, 0, 0, 0, 0, 0],
  ep: [4, 1, 0, 3, 6, 5, 2, 7, 8, 9, 10, 11],
  eo: [1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0],
};

/**
 * The six faces and three whole-cube rotations, each with prime and double.
 * Wide/slice moves are expanded by the parser (face turn + rotation), so they
 * are not stored here. Rotations x/y/z are derived from the private slice
 * constants exactly as in the Rust reference (`x = R·M'·L'`, `y = U·E'·D'`,
 * `z = F·S·B'`), which keeps them consistent with the corrected face tables.
 */
export interface MoveSet {
  u: FaceMove;
  up: FaceMove;
  u2: FaceMove;
  d: FaceMove;
  dp: FaceMove;
  d2: FaceMove;
  r: FaceMove;
  rp: FaceMove;
  r2: FaceMove;
  l: FaceMove;
  lp: FaceMove;
  l2: FaceMove;
  f: FaceMove;
  fp: FaceMove;
  f2: FaceMove;
  b: FaceMove;
  bp: FaceMove;
  b2: FaceMove;
  x: FaceMove;
  xp: FaceMove;
  x2: FaceMove;
  y: FaceMove;
  yp: FaceMove;
  y2: FaceMove;
  z: FaceMove;
  zp: FaceMove;
  z2: FaceMove;
}

export function buildMoveSet(): MoveSet {
  const mp = inverse(M);
  const epMove = inverse(E);
  const x = then(then(R, mp), inverse(L));
  const y = then(then(U, epMove), inverse(D));
  const z = then(then(F, S), inverse(B));
  return {
    u: U,
    up: inverse(U),
    u2: pow2(U),
    d: D,
    dp: inverse(D),
    d2: pow2(D),
    r: R,
    rp: inverse(R),
    r2: pow2(R),
    l: L,
    lp: inverse(L),
    l2: pow2(L),
    f: F,
    fp: inverse(F),
    f2: pow2(F),
    b: B,
    bp: inverse(B),
    b2: pow2(B),
    x,
    xp: inverse(x),
    x2: pow2(x),
    y,
    yp: inverse(y),
    y2: pow2(y),
    z,
    zp: inverse(z),
    z2: pow2(z),
  };
}

/**
 * The 24 whole-cube orientations, each as a composed FaceMove. Built by closure
 * under the rotation generators. Classification tries every rotation and keeps
 * the ones that land F2L solved, absorbing any net rotation a wide/slice move
 * left behind.
 */
export function cubeRotations(ms: MoveSet): FaceMove[] {
  const gens = [ms.x, ms.xp, ms.y, ms.yp, ms.z, ms.zp];
  const set: FaceMove[] = [identity()];
  for (;;) {
    let added = false;
    for (const r of [...set]) {
      for (const g of gens) {
        const c = then(r, g);
        if (!set.some((e) => movesEq(e, c))) {
          set.push(c);
          added = true;
        }
      }
    }
    if (!added) break;
  }
  return set;
}
