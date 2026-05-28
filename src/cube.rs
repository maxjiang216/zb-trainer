use crate::moves::FaceMove;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CubeState {
    /// cp[i] = which piece is currently in slot i.
    pub cp: [u8; 8],
    /// co[i] = orientation of the piece in slot i (0–2).
    pub co: [u8; 8],
    pub ep: [u8; 12],
    pub eo: [u8; 12],
}

impl CubeState {
    pub fn solved() -> Self {
        Self {
            cp: [0, 1, 2, 3, 4, 5, 6, 7],
            co: [0; 8],
            ep: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            eo: [0; 12],
        }
    }

    pub fn apply(&self, m: &FaceMove) -> CubeState {
        let mut new = CubeState {
            cp: [0; 8],
            co: [0; 8],
            ep: [0; 12],
            eo: [0; 12],
        };
        // Piece at slot i moves to slot m.cp[i].
        for i in 0..8 {
            let dst = m.cp[i] as usize;
            new.cp[dst] = self.cp[i];
            new.co[dst] = (self.co[i] + m.co[i]) % 3;
        }
        for i in 0..12 {
            let dst = m.ep[i] as usize;
            new.ep[dst] = self.ep[i];
            new.eo[dst] = (self.eo[i] + m.eo[i]) % 2;
        }
        new
    }

    pub fn apply_alg(&self, moves: &[FaceMove]) -> CubeState {
        moves.iter().fold(*self, |s, m| s.apply(m))
    }

    /// True if every D-layer corner, E-layer edge, and D-layer edge is solved.
    /// Used to verify an alg is a genuine LL alg.
    pub fn f2l_solved(&self) -> bool {
        // D-layer corners: slots 4–7, each must hold its own piece, oriented 0
        for i in 4..8 {
            if self.cp[i] != i as u8 || self.co[i] != 0 {
                return false;
            }
        }
        // D-layer edges: slots 4–7
        for i in 4..8 {
            if self.ep[i] != i as u8 || self.eo[i] != 0 {
                return false;
            }
        }
        // E-layer (equatorial) edges: slots 8–11
        for i in 8..12 {
            if self.ep[i] != i as u8 || self.eo[i] != 0 {
                return false;
            }
        }
        true
    }

    /// True if all LL edges (slots 0–3) are oriented (eo == 0).
    pub fn ll_edges_oriented(&self) -> bool {
        self.eo[0..4].iter().all(|&x| x == 0)
    }
}
