use crate::cube::CubeState;
use crate::moves::{FaceMove, MoveSet, U};
use crate::parse::{invert_alg, is_auf, is_u_family};

/// Canonical fingerprint for a ZBLL case.
/// Derived from the U-layer corner and edge state, normalized over 4 AUF.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZbllCase(pub Vec<u8>);

/// Canonical fingerprint for a ZBLS case.
/// Derived from LL edge orientations + DFR corner + FR edge, normalized
/// over 4 pre-AUF rotations of the U layer.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZblsCase(pub Vec<u8>);

/// Strip leading AUF (U/U'/U2) and whole-cube rotations (x/y/z) from the
/// start, and trailing U/U'/U2 from the end.
pub fn strip_auf<'a>(moves: &'a [FaceMove], ms: &MoveSet) -> &'a [FaceMove] {
    let start = moves
        .iter()
        .position(|m| !is_auf(m, ms))
        .unwrap_or(moves.len());
    let moves = &moves[start..];
    let end = moves
        .iter()
        .rposition(|m| !is_u_family(m, ms))
        .map(|i| i + 1)
        .unwrap_or(0);
    &moves[..end]
}

/// Like strip_auf but only strips from the start (for ZBLS pre-AUF).
pub fn strip_auf_start<'a>(
    moves: &'a [FaceMove],
    ms: &MoveSet,
) -> &'a [FaceMove] {
    let start = moves
        .iter()
        .position(|m| !is_auf(m, ms))
        .unwrap_or(moves.len());
    &moves[start..]
}

fn apply_u_k(state: &CubeState, k: usize) -> CubeState {
    let u_move = FaceMove { ..U };
    let mut s = *state;
    for _ in 0..k {
        s = s.apply(&u_move);
    }
    s
}

fn extract_ll_fingerprint(state: &CubeState) -> Vec<u8> {
    // U-layer corners (slots 0–3): (piece_id, orientation)
    // U-layer edges (slots 0–3): (piece_id, orientation)
    let mut v = Vec::with_capacity(16);
    for i in 0..4 {
        v.push(state.cp[i]);
        v.push(state.co[i]);
    }
    for i in 0..4 {
        v.push(state.ep[i]);
        v.push(state.eo[i]);
    }
    v
}

/// Classify an alg (already stripped of AUF) as a ZBLL case.
///
/// Applies the inverse alg to a solved cube, then tries every whole-cube
/// rotation in `rots` (pass `MoveSet::cube_rotations()`). The rotations that
/// land F2L solved are exactly the alg's AUF variants — possibly composed with
/// a net cube rotation left behind by wide/slice moves. We read the LL
/// fingerprint from each and take the lexicographic minimum as the canonical
/// case, which folds AUF and net-rotation normalization into one step.
///
/// Returns `None` if no rotation solves F2L (the alg is not a genuine
/// last-layer alg). The bool is whether the LL edges are oriented (true ZBLL
/// vs. an OLL-style case loosely labeled ZBLL); it is invariant across the AUF
/// rotations, so reading it from any qualifying state is sufficient.
pub fn zbll_canonical(
    core: &[FaceMove],
    rots: &[FaceMove],
) -> Option<(ZbllCase, bool)> {
    let inv = invert_alg(core);
    let state = CubeState::solved().apply_alg(&inv);

    let mut best: Option<Vec<u8>> = None;
    let mut edges_oriented = false;
    for g in rots {
        let s = state.apply(g);
        if !s.f2l_solved() {
            continue;
        }
        edges_oriented = s.ll_edges_oriented();
        // The rotation fixed the frame (F2L back on the bottom). AUF is a
        // separate freedom — a U-face turn of the last layer — so normalize
        // over the 4 U turns here. A whole-cube y rotation would NOT do this:
        // it moves the D-layer too and breaks f2l_solved.
        let mut t = s;
        for _ in 0..4 {
            let fp = extract_ll_fingerprint(&t);
            if best.as_ref().is_none_or(|b| &fp < b) {
                best = Some(fp);
            }
            t = t.apply(&U);
        }
    }
    best.map(|b| (ZbllCase(b), edges_oriented))
}

fn extract_zbls_fingerprint(state: &CubeState) -> Vec<u8> {
    // LL edge orientations in their current slots.
    let mut v = Vec::with_capacity(8);
    for i in 0..4 {
        v.push(state.eo[i]);
    }
    // Position + orientation of the DFR corner piece (piece 4).
    let dfr_slot = state.cp.iter().position(|&p| p == 4).unwrap_or(99) as u8;
    let dfr_ori = if dfr_slot < 8 {
        state.co[dfr_slot as usize]
    } else {
        0
    };
    v.push(dfr_slot);
    v.push(dfr_ori);
    // Position + orientation of the FR edge piece (piece 8).
    let fr_slot = state.ep.iter().position(|&p| p == 8).unwrap_or(99) as u8;
    let fr_ori = if fr_slot < 12 {
        state.eo[fr_slot as usize]
    } else {
        0
    };
    v.push(fr_slot);
    v.push(fr_ori);
    v
}

/// Compute the canonical ZBLS case for an alg (leading AUF stripped).
pub fn zbls_canonical(core: &[FaceMove]) -> ZblsCase {
    let inv = invert_alg(core);
    let state = CubeState::solved().apply_alg(&inv);

    let best = (0..4)
        .map(|k| extract_zbls_fingerprint(&apply_u_k(&state, k)))
        .min()
        .unwrap();

    ZblsCase(best)
}
