use crate::moves::{FaceMove, MoveSet};

#[derive(Debug)]
pub enum ParseError {
    /// Token contains big-cube notation (3r, 4r, …) — skip whole alg.
    BigCube,
    Unknown(String),
}

/// Parse an algorithm string into a sequence of `FaceMove`s.
/// Returns `Err(ParseError::BigCube)` if any big-cube token is found.
pub fn parse_alg(s: &str, ms: &MoveSet) -> Result<Vec<FaceMove>, ParseError> {
    let mut out = Vec::new();
    for token in s.split_whitespace() {
        let m = parse_token(token, ms)?;
        out.push(m);
    }
    Ok(out)
}

fn parse_token(token: &str, ms: &MoveSet) -> Result<FaceMove, ParseError> {
    let bytes = token.as_bytes();

    // Big-cube prefix: digit(s) before the move letter (e.g., "3r", "4Rw").
    if bytes.first().is_some_and(|b| b.is_ascii_digit()) {
        return Err(ParseError::BigCube);
    }

    // Separate base letter(s) from suffix.
    // Suffix can be: '', "2", "2'", or a bare "'"
    // Base: one or two chars (e.g., "Rw" = wide R)
    let (base, suffix) = split_base_suffix(token)?;

    let prime = suffix.contains('\'');
    let double = suffix.contains('2');

    let fm = lookup_base(base, ms, prime, double)?;
    Ok(fm)
}

fn split_base_suffix(token: &str) -> Result<(&str, &str), ParseError> {
    let bytes = token.as_bytes();
    // Find where the base ends: it's alpha chars (and 'w' for wide).
    // Suffix = everything after the last alpha char that is part of base.
    // Strategy: base is the leading alpha chars (possibly ending with 'w'),
    // suffix is trailing ' and/or 2.
    let end = bytes
        .iter()
        .position(|b| !b.is_ascii_alphabetic())
        .unwrap_or(bytes.len());

    // Handle 'w' suffix on wide moves like "Rw", "Lw"
    let base = &token[..end];
    let suffix = &token[end..];

    if base.is_empty() {
        return Err(ParseError::Unknown(token.to_string()));
    }
    Ok((base, suffix))
}

fn lookup_base(
    base: &str,
    ms: &MoveSet,
    prime: bool,
    double: bool,
) -> Result<FaceMove, ParseError> {
    // Normalize wide notation: "Rw" → same as lowercase "r"
    let is_wide = base.ends_with('w');
    let core = if is_wide {
        &base[..base.len() - 1]
    } else {
        base
    };

    let fm: FaceMove = match (core, is_wide, prime, double) {
        // ── U-layer ──────────────────────────────────────────────────────
        ("U", false, false, false) => ms.u,
        ("U", false, true, false) => ms.up,
        ("U", false, _, true) => ms.u2,
        ("u", _, false, false) | ("U", true, false, false) => ms.uw,
        ("u", _, true, false) | ("U", true, true, false) => ms.uwp,
        ("u", _, _, true) | ("U", true, _, true) => ms.uw2,

        // ── D-layer ──────────────────────────────────────────────────────
        ("D", false, false, false) => ms.d,
        ("D", false, true, false) => ms.dp,
        ("D", false, _, true) => ms.d2,
        ("d", _, false, false) | ("D", true, false, false) => ms.dw,
        ("d", _, true, false) | ("D", true, true, false) => ms.dwp,
        ("d", _, _, true) | ("D", true, _, true) => ms.dw2,

        // ── R-layer ──────────────────────────────────────────────────────
        ("R", false, false, false) => ms.r,
        ("R", false, true, false) => ms.rp,
        ("R", false, _, true) => ms.r2,
        ("r", _, false, false) | ("R", true, false, false) => ms.rw,
        ("r", _, true, false) | ("R", true, true, false) => ms.rwp,
        ("r", _, _, true) | ("R", true, _, true) => ms.rw2,

        // ── L-layer ──────────────────────────────────────────────────────
        ("L", false, false, false) => ms.l,
        ("L", false, true, false) => ms.lp,
        ("L", false, _, true) => ms.l2,
        ("l", _, false, false) | ("L", true, false, false) => ms.lw,
        ("l", _, true, false) | ("L", true, true, false) => ms.lwp,
        ("l", _, _, true) | ("L", true, _, true) => ms.lw2,

        // ── F-layer ──────────────────────────────────────────────────────
        ("F", false, false, false) => ms.f,
        ("F", false, true, false) => ms.fp,
        ("F", false, _, true) => ms.f2,
        ("f", _, false, false) | ("F", true, false, false) => ms.fw,
        ("f", _, true, false) | ("F", true, true, false) => ms.fwp,
        ("f", _, _, true) | ("F", true, _, true) => ms.fw2,

        // ── B-layer ──────────────────────────────────────────────────────
        ("B", false, false, false) => ms.b,
        ("B", false, true, false) => ms.bp,
        ("B", false, _, true) => ms.b2,
        ("b", _, false, false) | ("B", true, false, false) => ms.bw,
        ("b", _, true, false) | ("B", true, true, false) => ms.bwp,
        ("b", _, _, true) | ("B", true, _, true) => ms.bw2,

        // ── Slices ───────────────────────────────────────────────────────
        ("M", false, false, false) => ms.m,
        ("M", false, true, false) => ms.mp,
        ("M", false, _, true) => ms.m2,
        ("E", false, false, false) => ms.e,
        ("E", false, true, false) => ms.ep_move,
        ("E", false, _, true) => ms.e2,
        ("S", false, false, false) => ms.s,
        ("S", false, true, false) => ms.sp,
        ("S", false, _, true) => ms.s2,

        // ── Rotations ────────────────────────────────────────────────────
        ("x", _, false, false) => ms.x,
        ("x", _, true, false) => ms.xp,
        ("x", _, _, true) => ms.x2,
        ("y", _, false, false) => ms.y,
        ("y", _, true, false) => ms.yp,
        ("y", _, _, true) => ms.y2,
        ("z", _, false, false) => ms.z,
        ("z", _, true, false) => ms.zp,
        ("z", _, _, true) => ms.z2,

        _ => return Err(ParseError::Unknown(format!("{core} wide={is_wide}"))),
    };
    Ok(fm)
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
    let rotations =
        [ms.x, ms.xp, ms.x2, ms.y, ms.yp, ms.y2, ms.z, ms.zp, ms.z2];
    rotations.iter().any(|r| moves_eq(m, r))
}

fn moves_eq(a: &FaceMove, b: &FaceMove) -> bool {
    a.cp == b.cp && a.co == b.co && a.ep == b.ep && a.eo == b.eo
}
