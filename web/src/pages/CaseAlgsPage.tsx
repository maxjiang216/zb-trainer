import { Link, useParams } from "react-router-dom";

import { CaseDiagram } from "../components/CaseDiagram";
import { CASES_BY_ID, useChosenAlgs } from "../store";
import type { CaseAlg } from "../types";

/** Per-case alg chooser: lists candidate algs with source + cuber attribution. */
export function CaseAlgsPage() {
  const { id } = useParams<{ id: string }>();
  const { chosenAlg, setChosen } = useChosenAlgs();

  const caseData = id ? CASES_BY_ID.get(id) : undefined;
  if (!caseData) {
    return (
      <div>
        <p>
          Unknown case. <Link to="/algs">Back to algs</Link>
        </p>
      </div>
    );
  }

  const current = chosenAlg(caseData.id);

  return (
    <div className="case-detail">
      <div className="page-head">
        <Link className="link-quiet" to="/algs">
          ← all algs
        </Link>
      </div>

      <div className="case-detail-head">
        <CaseDiagram state={caseData.state} size={180} />
        <div>
          <h1>
            {caseData.subset} {caseData.n}
          </h1>
          <p className="muted">
            {caseData.algs.length} algorithm
            {caseData.algs.length === 1 ? "" : "s"}
          </p>
        </div>
      </div>

      <ul className="alg-list">
        {caseData.algs.map((a, i) => (
          <AlgRow
            key={i}
            alg={a}
            selected={a.alg === current}
            onSelect={() => setChosen(caseData.id, a.alg)}
          />
        ))}
      </ul>
    </div>
  );
}

function AlgRow({
  alg,
  selected,
  onSelect,
}: {
  alg: CaseAlg;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <li className={`alg-row ${selected ? "selected" : ""}`}>
      <button className="alg-pick" onClick={onSelect} aria-pressed={selected}>
        <span className={`radio ${selected ? "on" : ""}`} aria-hidden />
        <div className="alg-body">
          <code className="alg">{alg.alg}</code>
          <div className="alg-meta">
            <span className={`badge ${alg.source === "reco.nz" ? "reco" : "scdb"}`}>
              {alg.source}
            </span>
            {alg.uses > 0 && (
              <span className="muted">{alg.uses} solves</span>
            )}
            {alg.solvers.length > 0 && (
              <span className="solvers" title={alg.solvers.join(", ")}>
                {alg.solvers.slice(0, 6).join(", ")}
                {alg.solvers.length > 6 ? ` +${alg.solvers.length - 6} more` : ""}
              </span>
            )}
          </div>
        </div>
      </button>
    </li>
  );
}
