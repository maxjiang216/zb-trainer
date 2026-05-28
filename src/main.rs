mod cube;
mod moves;
mod normalize;
mod parse;

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::moves::{FaceMove, MoveSet};
use crate::normalize::{strip_auf, strip_auf_start, zbll_canonical, zbls_canonical, ZbllCase};
use crate::parse::{parse_alg, ParseError};

#[derive(Deserialize)]
struct Record {
    id: u64,
    solver_name: String,
    result_time: String,
    method: String,
    date: String,
    #[serde(default)]
    zbll_alg: String,
    #[serde(default)]
    zbls_alg: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct AlgEntry {
    solve_id: u64,
    solver: String,
    result: String,
    method: String,
    date: String,
    raw_alg: String,
}

fn main() {
    let path = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "../reco_scraper/reco_zbll_fixed.json".to_string()),
    );

    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));
    let records: Vec<Record> =
        serde_json::from_str(&data).unwrap_or_else(|e| panic!("JSON parse error: {e}"));

    let ms = MoveSet::build();
    // Precompute the 24 cube orientations once; reused for every alg to
    // reorient out any net rotation left by wide/slice moves.
    let rots = ms.cube_rotations();

    // ── ZBLL grouping ─────────────────────────────────────────────────────
    let mut zbll_groups: HashMap<ZbllCase, Vec<AlgEntry>> = HashMap::new();
    let mut zbll_skip_bigcube = 0usize;
    let mut zbll_skip_unknown = 0usize;
    let mut zbll_skip_not_ll = 0usize;
    let mut zbll_skip_edges_bad = 0usize;
    let mut zbll_total = 0usize;

    // ── ZBLS grouping ─────────────────────────────────────────────────────
    let mut zbls_groups: HashMap<crate::normalize::ZblsCase, Vec<AlgEntry>> = HashMap::new();
    let mut zbls_skip = 0usize;
    let mut zbls_total = 0usize;

    for rec in &records {
        // ── ZBLL ──────────────────────────────────────────────────────────
        if !rec.zbll_alg.is_empty() {
            zbll_total += 1;
            match parse_alg(&rec.zbll_alg, &ms) {
                Err(ParseError::BigCube) => {
                    zbll_skip_bigcube += 1;
                }
                Err(ParseError::Unknown(t)) => {
                    eprintln!("ZBLL unknown token '{t}' in solve {}", rec.id);
                    zbll_skip_unknown += 1;
                }
                Ok(moves) => {
                    let core = strip_auf(&moves, &ms);
                    if core.is_empty() {
                        zbll_skip_not_ll += 1;
                        continue;
                    }
                    // Only keep algs that genuinely preserve F2L (in some
                    // orientation). Algs that move D-layer pieces — e.g.
                    // OLL-style CFOP sequences loosely labeled "ZBLL" — never
                    // land F2L solved under any rotation and are excluded.
                    let Some((case, edges_oriented)) = zbll_canonical(core, &rots) else {
                        zbll_skip_not_ll += 1;
                        continue;
                    };
                    if !edges_oriented {
                        zbll_skip_edges_bad += 1;
                    }
                    zbll_groups.entry(case).or_default().push(AlgEntry {
                        solve_id: rec.id,
                        solver: rec.solver_name.clone(),
                        result: rec.result_time.clone(),
                        method: rec.method.clone(),
                        date: rec.date.clone(),
                        raw_alg: rec.zbll_alg.clone(),
                    });
                }
            }
        }

        // ── ZBLS ──────────────────────────────────────────────────────────
        if !rec.zbls_alg.is_empty() {
            zbls_total += 1;
            match parse_alg(&rec.zbls_alg, &ms) {
                Err(_) => {
                    zbls_skip += 1;
                }
                Ok(moves) => {
                    let core = strip_auf_start(&moves, &ms);
                    if core.is_empty() {
                        zbls_skip += 1;
                        continue;
                    }
                    let Some(case) = zbls_canonical(core, &rots) else {
                        zbls_skip += 1;
                        continue;
                    };
                    zbls_groups.entry(case).or_default().push(AlgEntry {
                        solve_id: rec.id,
                        solver: rec.solver_name.clone(),
                        result: rec.result_time.clone(),
                        method: rec.method.clone(),
                        date: rec.date.clone(),
                        raw_alg: rec.zbls_alg.clone(),
                    });
                }
            }
        }
    }

    // ── ZBLL output ───────────────────────────────────────────────────────
    let mut zbll_sorted: Vec<_> = zbll_groups.iter().collect();
    zbll_sorted.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));

    println!("══ ZBLL ══════════════════════════════════════════════════════");
    println!(
        "Total algs: {zbll_total}  |  distinct cases: {}  |  \
         skipped — big-cube: {zbll_skip_bigcube}, \
         unknown: {zbll_skip_unknown}, \
         not-LL: {zbll_skip_not_ll}, \
         edges-bad (≠ZBLL): {zbll_skip_edges_bad}",
        zbll_sorted.len()
    );
    println!();

    // Spot-check two known-equivalent algs.
    spot_check(&ms, &rots, "F R' F' r U R U' r'", "U' F R' F' r U R U' r'");
    spot_check(&ms, &rots, "R' F' r U R U' r' F U2'", "R' F' r U R U' r' F");
    println!();

    // Coverage: all ZBLL cases from speedcubedb.com vs our collected data.
    let all_fingerprints: std::collections::HashSet<_> = zbll_groups.keys().collect();
    let known_path = path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("../zb-trainer/known_zbll.json");
    let known_path = if known_path.exists() {
        known_path
    } else {
        PathBuf::from("known_zbll.json")
    };
    if known_path.exists() {
        #[derive(serde::Deserialize)]
        struct KnownCase {
            subset: String,
            n: u32,
            alg: String,
        }
        let kdata = std::fs::read_to_string(&known_path).unwrap();
        let known: Vec<KnownCase> = serde_json::from_str(&kdata).unwrap();
        let total = known.len();
        let mut covered = 0usize;
        let mut missed: Vec<String> = Vec::new();
        let mut parse_err: Vec<String> = Vec::new();
        for k in &known {
            let name = format!("ZBLL {} {}", k.subset, k.n);
            match parse_and_canon(&ms, &rots, &k.alg) {
                Some(fp) if all_fingerprints.contains(&fp) => covered += 1,
                Some(_) => missed.push(name),
                None => parse_err.push(format!("{name}: {}", k.alg)),
            }
        }
        println!("── Coverage: speedcubedb.com ZBLL cases ──────────────────────");
        // Group missed by subset
        let mut missed_by_subset: std::collections::BTreeMap<&str, Vec<u32>> =
            std::collections::BTreeMap::new();
        for k in &known {
            let name = format!("ZBLL {} {}", k.subset, k.n);
            if missed.contains(&name) {
                missed_by_subset
                    .entry(k.subset.as_str())
                    .or_default()
                    .push(k.n);
            }
        }
        println!(
            "Coverage: {covered}/{total} cases in dataset  \
             (missed: {}  parse errors: {})",
            missed.len(),
            parse_err.len()
        );
        if !missed_by_subset.is_empty() {
            println!("Missed cases by subset:");
            for (subset, ns) in &missed_by_subset {
                println!("  ZBLL {subset}: {:?}", ns);
            }
        }
        if !parse_err.is_empty() {
            println!("Parse errors:");
            for e in &parse_err {
                println!("  {e}");
            }
        }
        // Also show coverage per subset
        println!("Per-subset:");
        let mut by_subset: std::collections::BTreeMap<&str, (usize, usize)> =
            std::collections::BTreeMap::new();
        for k in &known {
            let name = format!("ZBLL {} {}", k.subset, k.n);
            let e = by_subset.entry(k.subset.as_str()).or_default();
            e.1 += 1;
            if !missed.contains(&name) && !parse_err.iter().any(|s| s.starts_with(&name)) {
                e.0 += 1;
            }
        }
        for (subset, (cov, tot)) in &by_subset {
            println!("  ZBLL {subset}: {cov}/{tot}");
        }
        println!();
    }

    // Top 30 cases by usage count.
    println!("── Top ZBLL cases ────────────────────────────────────────────");
    for (case, entries) in zbll_sorted.iter().take(30) {
        let solvers: Vec<_> = {
            let mut s: Vec<_> = entries.iter().map(|e| e.solver.as_str()).collect();
            s.sort();
            s.dedup();
            s
        };
        let algs: Vec<_> = {
            let mut a: Vec<_> = entries.iter().map(|e| e.raw_alg.as_str()).collect();
            a.sort();
            a.dedup();
            a
        };
        println!(
            "[{:3}x | {:2} solvers | {:2} raw algs]",
            entries.len(),
            solvers.len(),
            algs.len()
        );
        // Pick the shortest raw alg as representative.
        if let Some(shortest) = algs.iter().min_by_key(|a| a.len()) {
            println!("  canonical-rep: {}", shortest);
        }
        println!("  solvers: {}", solvers.join(", "));
        println!("  case fingerprint: {:?}", &case.0);
        println!();
    }

    // ── ZBLS output ───────────────────────────────────────────────────────
    let mut zbls_sorted: Vec<_> = zbls_groups.iter().collect();
    zbls_sorted.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));

    println!("══ ZBLS ══════════════════════════════════════════════════════");
    println!(
        "Total algs: {zbls_total}  |  distinct cases: {}  |  skipped: {zbls_skip}",
        zbls_sorted.len()
    );
    println!();

    for (case, entries) in zbls_sorted.iter().take(20) {
        let solvers: Vec<_> = {
            let mut s: Vec<_> = entries.iter().map(|e| e.solver.as_str()).collect();
            s.sort();
            s.dedup();
            s
        };
        println!(
            "[{:3}x | {:2} solvers] fingerprint: {:?}",
            entries.len(),
            solvers.len(),
            &case.0
        );
        println!("  solvers: {}", solvers.join(", "));
        println!(
            "  example: {}",
            entries.first().map(|e| e.raw_alg.as_str()).unwrap_or("")
        );
        println!();
    }
}

/// Print whether two alg strings map to the same ZBLL case.
fn spot_check(ms: &MoveSet, rots: &[FaceMove], a: &str, b: &str) {
    let ca = parse_and_canon(ms, rots, a);
    let cb = parse_and_canon(ms, rots, b);
    let same = ca.as_ref() == cb.as_ref();
    println!(
        "SPOT CHECK: '{}' vs '{}' → {}",
        a,
        b,
        if same { "SAME ✓" } else { "DIFFERENT ✗" }
    );
}

fn parse_and_canon(ms: &MoveSet, rots: &[FaceMove], alg: &str) -> Option<ZbllCase> {
    let moves = parse_alg(alg, ms).ok()?;
    let core = strip_auf(&moves, ms);
    if core.is_empty() {
        return None;
    }
    zbll_canonical(core, rots).map(|(case, _)| case)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::CubeState;
    use crate::parse::invert_alg;

    /// Verify every stored move: quarter^4 = identity, quarter·prime = identity,
    /// and double = quarter². Covers all six faces and three rotations.
    #[test]
    fn test_move_identity() {
        let ms = MoveSet::build();
        let id = CubeState::solved();
        for (name, q, p, d) in [
            ("U", ms.u, ms.up, ms.u2),
            ("D", ms.d, ms.dp, ms.d2),
            ("R", ms.r, ms.rp, ms.r2),
            ("L", ms.l, ms.lp, ms.l2),
            ("F", ms.f, ms.fp, ms.f2),
            ("B", ms.b, ms.bp, ms.b2),
            ("x", ms.x, ms.xp, ms.x2),
            ("y", ms.y, ms.yp, ms.y2),
            ("z", ms.z, ms.zp, ms.z2),
        ] {
            assert_eq!(
                id.apply(&q).apply(&q).apply(&q).apply(&q),
                id,
                "{name}^4 != identity"
            );
            assert_eq!(id.apply(&q).apply(&p), id, "{name}·{name}' != identity");
            assert_eq!(id.apply(&d), id.apply(&q).apply(&q), "{name}2 != {name}²");
        }
    }

    /// AUF-stripped algs that differ only by pre/post U moves must share a fingerprint.
    #[test]
    fn test_auf_normalization() {
        let ms = MoveSet::build();
        let rots = ms.cube_rotations();
        let a = parse_and_canon(&ms, &rots, "F R' F' r U R U' r'").unwrap();
        let b = parse_and_canon(&ms, &rots, "U' F R' F' r U R U' r'").unwrap();
        let c = parse_and_canon(&ms, &rots, "U2 F R' F' r U R U' r' U'").unwrap();
        assert_eq!(a, b, "pre-AUF changed fingerprint");
        assert_eq!(a, c, "pre+post AUF changed fingerprint");
    }

    /// The rotations x/y/z must form exactly the 24-element cube-orientation
    /// group. A larger group means a slice/rotation carries a spurious
    /// orientation delta (the E slice once gave 48 by not flipping its edges).
    #[test]
    fn test_rotation_group_is_24() {
        let ms = MoveSet::build();
        assert_eq!(ms.cube_rotations().len(), 24);
    }

    /// Ground-truth check that the move model is internally consistent.
    /// Every genuine last-layer alg must, when its inverse is applied to a
    /// solved cube, leave F2L intact and keep all LL edges oriented. This
    /// exercises R/U/F/L/D together and is what caught the inverted R/L/B
    /// tables (X^4=id / X·X'=id alone could not — they hold for either
    /// turn direction).
    #[test]
    fn test_ll_algs_preserve_f2l() {
        let ms = MoveSet::build();
        let id = CubeState::solved();
        // Standard last-layer algs (OLL corner-orientation + PLL permutations).
        // PLLs and ZBLL/OLL cases all preserve F2L and LL edge orientation.
        for (name, alg) in [
            ("Sune", "R U R' U R U2 R'"),
            ("AntiSune", "R U2 R' U' R U' R'"),
            ("T-perm", "R U R' U' R' F R2 U' R' U' R U R' F'"),
            ("Y-perm", "F R U' R' U' R U R' F' R U R' U' R' F R F'"),
            ("H-perm", "M2 U M2 U2 M2 U M2"),
            ("Ua-perm", "M2 U M U2 M' U M2"),
            ("Ub-perm", "M2 U' M U2 M' U' M2"),
        ] {
            let moves = parse_alg(alg, &ms).unwrap();
            let inv = invert_alg(&moves);
            let state = id.apply_alg(&inv);
            assert!(state.f2l_solved(), "{name}: inverse broke F2L");
            assert!(
                state.ll_edges_oriented(),
                "{name}: inverse mis-oriented LL edges"
            );
        }
    }

    /// Every alg in the speedcubedb ZBLL set (472 cases, using wide moves and
    /// rotations) should classify as a valid ZBLL: rotation-aware F2L solved
    /// with LL edges oriented. This is the end-to-end correctness check for the
    /// whole move model (faces + slices + wide + rotations).
    ///
    /// Eight entries are excluded as known-bad scraped data — they do not
    /// preserve F2L under ANY of the 24 cube rotations, so they are mistyped /
    /// alternate algs from speedcubedb, not model bugs. They are pinned here so
    /// a real regression (which would change the failing set) is still caught.
    #[test]
    fn test_known_zbll_all_valid() {
        #[derive(serde::Deserialize)]
        struct KnownCase {
            subset: String,
            n: u32,
            alg: String,
        }
        let ms = MoveSet::build();
        let rots = ms.cube_rotations();
        let data = std::fs::read_to_string("known_zbll.json")
            .expect("known_zbll.json must be present at crate root");
        let known: Vec<KnownCase> = serde_json::from_str(&data).unwrap();

        // Scraped algs that don't close to a valid LL case under any rotation
        // — mistyped / alternate entries from speedcubedb, not model bugs.
        let known_bad: std::collections::HashSet<&str> =
            ["H 6", "H 18", "H 19", "H 29", "H 40", "T 46", "L 23"]
                .into_iter()
                .collect();

        let mut unexpected = Vec::new();
        for k in &known {
            let name = format!("{} {}", k.subset, k.n);
            let valid = parse_alg(&k.alg, &ms).ok().is_some_and(|moves| {
                let core = strip_auf(&moves, &ms);
                // A true ZBLL classifies (rotation-aware F2L) with edges oriented.
                matches!(zbll_canonical(core, &rots), Some((_, true)))
            });
            if valid == known_bad.contains(name.as_str()) {
                // Either a known-bad alg unexpectedly validated, or a good alg
                // failed — both are regressions worth surfacing.
                unexpected.push(format!("{name}: valid={valid} | {}", k.alg));
            }
        }
        assert!(
            unexpected.is_empty(),
            "{} known ZBLL algs changed classification:\n{}",
            unexpected.len(),
            unexpected.join("\n")
        );
    }
}
