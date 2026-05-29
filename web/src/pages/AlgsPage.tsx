import { Link } from "react-router-dom";

import { CaseDiagram } from "../components/CaseDiagram";
import { CASES, SUBSETS, useChosenAlgs } from "../store";

/** Index of all cases; click through to choose the alg for each. */
export function AlgsPage() {
  const { chosenAlg } = useChosenAlgs();

  return (
    <div>
      <div className="page-head">
        <h1>Algorithms</h1>
        <p className="muted">Pick the alg you use for each case.</p>
      </div>

      {SUBSETS.map((subset) => {
        const cases = CASES.filter((c) => c.subset === subset);
        if (cases.length === 0) return null;
        return (
          <section key={subset} className="subset">
            <h2>
              {subset} <span className="muted">({cases.length})</span>
            </h2>
            <div className="case-grid">
              {cases.map((c) => (
                <Link key={c.id} to={`/algs/${c.id}`} className="case-card link">
                  <CaseDiagram state={c.state} size={104} />
                  <span className="case-label">{c.id}</span>
                  <code className="alg-mini">{chosenAlg(c.id)}</code>
                  <span className="muted alg-count">
                    {c.algs.length} alg{c.algs.length === 1 ? "" : "s"}
                  </span>
                </Link>
              ))}
            </div>
          </section>
        );
      })}
    </div>
  );
}
