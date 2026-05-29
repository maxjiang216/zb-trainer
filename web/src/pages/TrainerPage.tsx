import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";

import { CaseDiagram } from "../components/CaseDiagram";
import { type CubeState, solved } from "../cube/cube";
import { buildMoveSet, cubeRotations } from "../cube/moves";
import { buildRound } from "../cube/trainer";
import { CASES, useChosenAlgs, useSelectedCases } from "../store";

interface ActiveRound {
  caseId: string;
  alg: string;
  caseSolved: CubeState;
  after: CubeState;
}

/** The practice loop. See `cube/trainer.ts` for the exact chaining mechanic. */
export function TrainerPage() {
  const ms = useMemo(() => buildMoveSet(), []);
  const rots = useMemo(() => cubeRotations(ms), [ms]);
  const { selected } = useSelectedCases();
  const { chosenAlg } = useChosenAlgs();

  const pool = useMemo(
    () => CASES.filter((c) => selected.has(c.id)),
    [selected],
  );

  // `v` is the current cube (the "before" panel); start solved.
  const [v, setV] = useState<CubeState>(() => solved());
  const [round, setRound] = useState<ActiveRound | null>(null);
  const [revealed, setRevealed] = useState(false);
  const [count, setCount] = useState(0);

  // chosenAlg changes identity when the alg map updates; a ref keeps newRound
  // from needing it as a dependency.
  const chosenAlgRef = useRef(chosenAlg);
  chosenAlgRef.current = chosenAlg;

  const newRound = useCallback(
    (base: CubeState) => {
      if (pool.length === 0) {
        setRound(null);
        return;
      }
      const c = pool[Math.floor(Math.random() * pool.length)];
      const alg = chosenAlgRef.current(c.id);
      const { caseSolved, after } = buildRound(base, alg, ms, rots);
      setRound({ caseId: c.id, alg, caseSolved, after });
      setRevealed(false);
    },
    [pool, ms, rots],
  );

  useEffect(() => {
    if (round === null && pool.length > 0) newRound(v);
  }, [round, pool, v, newRound]);

  const next = () => {
    if (!round) return;
    setV(round.after);
    setCount((n) => n + 1);
    newRound(round.after);
  };

  const reset = () => {
    const s = solved();
    setV(s);
    setCount(0);
    newRound(s);
  };

  if (pool.length === 0) {
    return (
      <div className="trainer empty">
        <h1>No cases selected</h1>
        <p className="muted">
          Pick some cases on the <Link to="/">Cases</Link> page first.
        </p>
      </div>
    );
  }

  if (!round) return null;

  return (
    <div className="trainer">
      <div className="trainer-top">
        <h1>
          Apply <span className="case-id">{round.caseId}</span>
        </h1>
        <span className="muted">algs done this session: {count}</span>
      </div>

      <figure className="case-solved">
        <CaseDiagram state={round.caseSolved} size={230} />
        <figcaption>The case this alg solves</figcaption>
      </figure>

      <div className="trainer-cubes">
        <figure>
          <CaseDiagram state={v} size={150} />
          <figcaption>Your cube now</figcaption>
        </figure>
        <div className="arrow" aria-hidden>
          →
        </div>
        <figure className="result">
          <CaseDiagram state={round.after} size={150} />
          <figcaption>After this alg (no post-AUF)</figcaption>
        </figure>
      </div>

      <div className="trainer-alg">
        {revealed ? (
          <code className="alg">{round.alg || "(no alg set)"}</code>
        ) : (
          <button onClick={() => setRevealed(true)}>Reveal alg</button>
        )}
        <Link className="link-quiet" to={`/algs/${round.caseId}`}>
          change alg
        </Link>
      </div>

      <div className="trainer-actions">
        <button className="btn-primary" onClick={next}>
          Done → next
        </button>
        <button onClick={reset}>Reset to solved</button>
      </div>
    </div>
  );
}
