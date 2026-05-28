use crate::moves::{FaceMove, MoveSet};

#[derive(Debug)]
pub enum ParseError {
    /// Token contains big-cube notation (3r, 4r, …) — skip whole alg.
    BigCube,
    Unknown(String),
}

/// One primitive step a token expands into: either a whole-cube rotation or a
/// single-face turn. Wide and slice moves expand to a rotation plus a face
/// turn (e.g. `Rw = x L`, `M = L' x' R`); plain rotations expand to a single
/// `Rot`. Keeping the two kinds distinct lets the parser handle rotations by
/// re-framing the moves that follow, rather than permuting cube pieces.
#[derive(Clone, Copy)]
enum Prim {
    Rot(FaceMove),
    Face(FaceMove),
}

impl Prim {
    fn inverse(self) -> Prim {
        match self {
            Prim::Rot(g) => Prim::Rot(g.inverse()),
            Prim::Face(m) => Prim::Face(m.inverse()),
        }
    }
}

/// Parse an algorithm string into a sequence of plain face-move `FaceMove`s.
///
/// Rotations never permute pieces here: we track the running cube orientation
/// and, for each face turn, emit the equivalent fixed-frame turn (the
/// conjugate `orient · face · orient⁻¹`). Wide and slice moves are expanded to
/// a face turn plus the rotation they carry, so they feed through the same
/// machinery.
///
/// The output is a sequence of plain face turns. Any net cube rotation the alg
/// leaves behind (from wide/slice moves) is intentionally dropped — downstream
/// classification reorients over all 24 cube rotations anyway, so the net
/// rotation never needs to be applied to pieces.
///
/// Returns `Err(ParseError::BigCube)` if any big-cube token is found.
pub fn parse_alg(s: &str, ms: &MoveSet) -> Result<Vec<FaceMove>, ParseError> {
    let mut prims = Vec::new();
    for token in s.split_whitespace() {
        token_prims(token, ms, &mut prims)?;
    }

    let mut out = Vec::new();
    let mut orient = FaceMove::identity();
    for p in prims {
        match p {
            // A rotation re-frames everything after it; it permutes no pieces
            // now, it just updates the running orientation.
            Prim::Rot(g) => orient = orient.then(&g),
            // A face turn in the current frame is that turn conjugated back
            // into the fixed frame.
            Prim::Face(m) => {
                out.push(orient.then(&m).then(&orient.inverse()));
            }
        }
    }
    // Append the net cube rotation so the output is exactly equivalent to the
    // input alg. Setup rotations (a leading `y`) and intrinsic conjugations
    // (`x' … x`) are thus both preserved faithfully; downstream classification
    // reorients over all 24 rotations, which absorbs whatever net rotation
    // remains. Skip a no-op identity so it never blocks trailing-AUF stripping.
    if !is_identity(&orient) {
        out.push(orient);
    }
    Ok(out)
}

fn is_identity(m: &FaceMove) -> bool {
    let id = FaceMove::identity();
    m.cp == id.cp && m.co == id.co && m.ep == id.ep && m.eo == id.eo
}

/// Expand one token into its primitive steps, appended to `out`.
fn token_prims(token: &str, ms: &MoveSet, out: &mut Vec<Prim>) -> Result<(), ParseError> {
    let bytes = token.as_bytes();
    if bytes.first().is_some_and(|b| b.is_ascii_digit()) {
        return Err(ParseError::BigCube);
    }

    let (base, suffix) = split_base_suffix(token)?;
    let prime = suffix.contains('\'');
    let double = suffix.contains('2');

    // Quarter-turn (amount = 1) expansion of the base move.
    let quarter = quarter_prims(base, ms)?;

    if double {
        out.extend_from_slice(&quarter);
        out.extend_from_slice(&quarter);
    } else if prime {
        // Inverse of a sequence: reverse the order and invert each step.
        out.extend(quarter.iter().rev().map(|p| p.inverse()));
    } else {
        out.extend_from_slice(&quarter);
    }
    Ok(())
}

/// The quarter-turn primitive expansion of a base move (no suffix applied).
///
/// Wide/slice identities (all verified by the move-table composition):
/// `Rw = x L`, `Lw = x' R`, `Uw = y D`, `Dw = y' U`, `Fw = z B`, `Bw = z' F`,
/// `M = L' x' R`, `E = D' y' U`, `S = F' z B`.
fn quarter_prims(base: &str, ms: &MoveSet) -> Result<Vec<Prim>, ParseError> {
    use Prim::{Face, Rot};

    // Wide notation "Rw" is the same as lowercase "r".
    let is_wide = base.ends_with('w');
    let core = if is_wide {
        &base[..base.len() - 1]
    } else {
        base
    };

    let prims = match (core, is_wide) {
        // ── plain faces ──────────────────────────────────────────────────
        ("U", false) => vec![Face(ms.u)],
        ("D", false) => vec![Face(ms.d)],
        ("R", false) => vec![Face(ms.r)],
        ("L", false) => vec![Face(ms.l)],
        ("F", false) => vec![Face(ms.f)],
        ("B", false) => vec![Face(ms.b)],

        // ── wide moves = face turn + the rotation they carry ─────────────
        ("u", _) | ("U", true) => vec![Rot(ms.y), Face(ms.d)],
        ("d", _) | ("D", true) => vec![Rot(ms.yp), Face(ms.u)],
        ("r", _) | ("R", true) => vec![Rot(ms.x), Face(ms.l)],
        ("l", _) | ("L", true) => vec![Rot(ms.xp), Face(ms.r)],
        ("f", _) | ("F", true) => vec![Rot(ms.z), Face(ms.b)],
        ("b", _) | ("B", true) => vec![Rot(ms.zp), Face(ms.f)],

        // ── slices = two faces straddling a rotation ─────────────────────
        ("M", false) => vec![Face(ms.lp), Rot(ms.xp), Face(ms.r)],
        ("E", false) => vec![Face(ms.dp), Rot(ms.yp), Face(ms.u)],
        ("S", false) => vec![Face(ms.fp), Rot(ms.z), Face(ms.b)],

        // ── rotations ────────────────────────────────────────────────────
        ("x", _) => vec![Rot(ms.x)],
        ("y", _) => vec![Rot(ms.y)],
        ("z", _) => vec![Rot(ms.z)],

        _ => return Err(ParseError::Unknown(format!("{core} wide={is_wide}"))),
    };
    Ok(prims)
}

fn split_base_suffix(token: &str) -> Result<(&str, &str), ParseError> {
    let bytes = token.as_bytes();
    let end = bytes
        .iter()
        .position(|b| !b.is_ascii_alphabetic())
        .unwrap_or(bytes.len());

    let base = &token[..end];
    let suffix = &token[end..];

    if base.is_empty() {
        return Err(ParseError::Unknown(token.to_string()));
    }
    Ok((base, suffix))
}

/// Invert a sequence of moves (reverse + invert each).
pub fn invert_alg(moves: &[FaceMove]) -> Vec<FaceMove> {
    moves.iter().rev().map(|m| m.inverse()).collect()
}

/// Returns true if this move is a U-family move (U/U'/U2) or a
/// whole-cube rotation (x/y/z family). Used for stripping.
pub fn is_auf(m: &FaceMove, ms: &MoveSet) -> bool {
    is_u_family(m, ms) || is_rotation(m, ms)
}

pub fn is_u_family(m: &FaceMove, ms: &MoveSet) -> bool {
    let u_moves = [ms.u, ms.up, ms.u2];
    u_moves.iter().any(|u| moves_eq(m, u))
}

pub fn is_rotation(m: &FaceMove, ms: &MoveSet) -> bool {
    let rotations = [ms.x, ms.xp, ms.x2, ms.y, ms.yp, ms.y2, ms.z, ms.zp, ms.z2];
    rotations.iter().any(|r| moves_eq(m, r))
}

fn moves_eq(a: &FaceMove, b: &FaceMove) -> bool {
    a.cp == b.cp && a.co == b.co && a.ep == b.ep && a.eo == b.eo
}
