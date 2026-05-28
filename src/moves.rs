/// Complete permutation table for one move.
/// cp[i] = j means the piece currently in slot i moves to slot j.
/// co[i] = orientation delta added to the piece moving FROM slot i (mod 3).
/// ep/eo: same for edges (eo mod 2).
#[derive(Clone, Copy, Debug)]
pub struct FaceMove {
    pub cp: [u8; 8],
    pub co: [u8; 8],
    pub ep: [u8; 12],
    pub eo: [u8; 12],
}

impl FaceMove {
    pub const fn identity() -> Self {
        Self {
            cp: [0, 1, 2, 3, 4, 5, 6, 7],
            co: [0; 8],
            ep: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            eo: [0; 12],
        }
    }

    /// Compose: apply self then other.
    pub fn then(&self, other: &FaceMove) -> FaceMove {
        let mut r = FaceMove::identity();
        for i in 0..8 {
            let mid = self.cp[i] as usize;
            r.cp[i] = other.cp[mid];
            r.co[i] = (self.co[i] + other.co[mid]) % 3;
        }
        for i in 0..12 {
            let mid = self.ep[i] as usize;
            r.ep[i] = other.ep[mid];
            r.eo[i] = (self.eo[i] + other.eo[mid]) % 2;
        }
        r
    }

    pub fn inverse(&self) -> FaceMove {
        let mut r = FaceMove::identity();
        for i in 0..8 {
            let dst = self.cp[i] as usize;
            r.cp[dst] = i as u8;
            // If self adds co[i] when moving FROM i TO dst,
            // the inverse adds (3 - co[i]) % 3 when moving FROM dst TO i.
            r.co[dst] = (3 - self.co[i]) % 3;
        }
        for i in 0..12 {
            let dst = self.ep[i] as usize;
            r.ep[dst] = i as u8;
            r.eo[dst] = self.eo[i]; // flipping is self-inverse
        }
        r
    }

    pub fn pow2(&self) -> FaceMove {
        self.then(self)
    }
}

// ── Corner slots ───────────────────────────────────────────────────────────
// 0=URF  1=UFL  2=ULB  3=UBR
// 4=DFR  5=DLF  6=DBL  7=DRB
//
// ── Edge slots ─────────────────────────────────────────────────────────────
// 0=UR   1=UF   2=UL   3=UB
// 4=DR   5=DF   6=DL   7=DB
// 8=FR   9=FL  10=BL  11=BR

// U  (CW from top: UBR→URF→UFL→ULB,  UB→UR→UF→UL)
pub const U: FaceMove = FaceMove {
    cp: [1, 2, 3, 0, 4, 5, 6, 7],
    co: [0; 8],
    ep: [1, 2, 3, 0, 4, 5, 6, 7, 8, 9, 10, 11],
    eo: [0; 12],
};

// D  (CW from bottom: DFR→DRB→DBL→DLF,  DF→DR→DB→DL)
// Original table encoded D' (it sent DF→DL, mirroring U; a real D sends DF→DR).
// Corrected so D matches the handedness of U/F and standard alg strings work.
pub const D: FaceMove = FaceMove {
    cp: [0, 1, 2, 3, 7, 4, 5, 6],
    co: [0; 8],
    ep: [0, 1, 2, 3, 7, 4, 5, 6, 8, 9, 10, 11],
    eo: [0; 12],
};

// R  (CW from right: URF→UBR→DRB→DFR,  UR→BR→DR→FR)
// The original table encoded R' (it sent UR→FR, i.e. up→front). A real R sends
// up→back; verified via the canonical edge cycle UR→BR→DR→FR. Corrected below.
pub const R: FaceMove = FaceMove {
    cp: [3, 1, 2, 7, 0, 5, 6, 4],
    co: [2, 0, 0, 1, 1, 0, 0, 2],
    ep: [11, 1, 2, 3, 8, 5, 6, 7, 0, 9, 10, 4],
    eo: [0; 12],
};

// L  (CW from left: UFL→DLF→DBL→ULB,  UL→FL→DL→BL)
// Original table encoded L' (mixed handedness vs U/D/F). Corrected to the true
// clockwise direction so the model is internally consistent.
pub const L: FaceMove = FaceMove {
    cp: [0, 5, 1, 3, 4, 6, 2, 7],
    co: [0, 1, 2, 0, 0, 2, 1, 0],
    ep: [0, 1, 9, 3, 4, 5, 10, 7, 8, 6, 2, 11],
    eo: [0; 12],
};

// F  (CW from front: UFL→URF→DFR→DLF,  UF→FR→DF→FL)
pub const F: FaceMove = FaceMove {
    cp: [4, 0, 2, 3, 5, 1, 6, 7],
    co: [1, 2, 0, 0, 2, 1, 0, 0],
    ep: [0, 8, 2, 3, 4, 9, 6, 7, 5, 1, 10, 11],
    eo: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
};

// B  (CW from back: ULB→DBL→DRB→UBR,  UB→BL→DB→BR)
// Original table encoded B' (mixed handedness vs U/D/F). Corrected to the true
// clockwise direction so the model is internally consistent.
pub const B: FaceMove = FaceMove {
    cp: [0, 1, 6, 2, 4, 5, 7, 3],
    co: [0, 0, 1, 2, 0, 0, 2, 1],
    ep: [0, 1, 2, 10, 4, 5, 6, 11, 8, 9, 7, 3],
    eo: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
};

// M  (middle slice, same direction as L: UF→DF→DB→UB)
// Original table encoded M' (UF→UB, opposite to corrected L's UL→FL). Flipped to
// follow L so wide moves (r = R M') and x rotations come out consistent.
pub const M: FaceMove = FaceMove {
    cp: [0, 1, 2, 3, 4, 5, 6, 7],
    co: [0; 8],
    ep: [0, 5, 2, 1, 4, 7, 6, 3, 8, 9, 10, 11],
    eo: [0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0],
};

// E  (equatorial, same direction as D: FR→BR→BL→FL)
// Original table encoded E' (FR→FL, opposite to corrected D's DF→DR). Flipped to
// follow D so y rotations (y = U E' D') come out consistent.
pub const E: FaceMove = FaceMove {
    cp: [0, 1, 2, 3, 4, 5, 6, 7],
    co: [0; 8],
    ep: [0, 1, 2, 3, 4, 5, 6, 7, 11, 8, 9, 10],
    // E flips the 4 equatorial edges it moves, just as M and S flip theirs.
    // Without this, y = U E' D' is not a clean rotation (the rotation group
    // came out as 48 instead of 24).
    eo: [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
};

// S  (standing slice, same direction as F: UL→UR→DR→DL, flips edges)
pub const S: FaceMove = FaceMove {
    cp: [0, 1, 2, 3, 4, 5, 6, 7],
    co: [0; 8],
    ep: [4, 1, 0, 3, 6, 5, 2, 7, 8, 9, 10, 11],
    eo: [1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0],
};

/// The move table the parser draws from: the six faces and three whole-cube
/// rotations, each with its prime and double. Wide and slice moves are *not*
/// stored — the parser expands them into a face turn plus a rotation (see
/// `parse::quarter_prims`), so they need no dedicated tables here. The slice
/// constants `M`/`E`/`S` still exist privately, used only to derive `x/y/z`.
///
/// This is the complete face+rotation catalog; the current parser references
/// most but not every variant (e.g. it builds doubles by repetition rather
/// than reading `r2`), so a few entries are unused outside tests.
#[allow(dead_code)]
pub struct MoveSet {
    pub u: FaceMove,
    pub up: FaceMove,
    pub u2: FaceMove,
    pub d: FaceMove,
    pub dp: FaceMove,
    pub d2: FaceMove,
    pub r: FaceMove,
    pub rp: FaceMove,
    pub r2: FaceMove,
    pub l: FaceMove,
    pub lp: FaceMove,
    pub l2: FaceMove,
    pub f: FaceMove,
    pub fp: FaceMove,
    pub f2: FaceMove,
    pub b: FaceMove,
    pub bp: FaceMove,
    pub b2: FaceMove,
    pub x: FaceMove,
    pub xp: FaceMove,
    pub x2: FaceMove,
    pub y: FaceMove,
    pub yp: FaceMove,
    pub y2: FaceMove,
    pub z: FaceMove,
    pub zp: FaceMove,
    pub z2: FaceMove,
}

impl MoveSet {
    pub fn build() -> Self {
        // Slice inverses are only needed to derive the rotations below.
        let mp = M.inverse();
        let ep_move = E.inverse();

        // x = R M' L', y = U E' D', z = F S B'  (whole-cube rotations).
        let x = R.then(&mp).then(&L.inverse());
        let y = U.then(&ep_move).then(&D.inverse());
        let z = F.then(&S).then(&B.inverse());

        Self {
            u: U,
            up: U.inverse(),
            u2: U.pow2(),
            d: D,
            dp: D.inverse(),
            d2: D.pow2(),
            r: R,
            rp: R.inverse(),
            r2: R.pow2(),
            l: L,
            lp: L.inverse(),
            l2: L.pow2(),
            f: F,
            fp: F.inverse(),
            f2: F.pow2(),
            b: B,
            bp: B.inverse(),
            b2: B.pow2(),
            x,
            xp: x.inverse(),
            x2: x.pow2(),
            y,
            yp: y.inverse(),
            y2: y.pow2(),
            z,
            zp: z.inverse(),
            z2: z.pow2(),
        }
    }

    /// The 24 whole-cube orientations, each as a composed `FaceMove`.
    ///
    /// Wide moves and slice moves rotate the cube's centers, so an algorithm
    /// can finish in a net-rotated frame. Checking F2L / reading the LL case
    /// against fixed slots would then spuriously fail. Callers reorient by
    /// trying every rotation here and keeping the ones that land F2L solved.
    pub fn cube_rotations(&self) -> Vec<FaceMove> {
        let gens = [self.x, self.xp, self.y, self.yp, self.z, self.zp];
        let mut set = vec![FaceMove::identity()];
        loop {
            let mut added = false;
            for r in set.clone() {
                for g in &gens {
                    let c = r.then(g);
                    let dup = set
                        .iter()
                        .any(|e| e.cp == c.cp && e.co == c.co && e.ep == c.ep && e.eo == c.eo);
                    if !dup {
                        set.push(c);
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }
        set
    }
}
