import { useMemo } from "react";
import { Link } from "react-router-dom";

import { CaseDiagram } from "../components/CaseDiagram";
import { CASES, SUBSETS, useSelectedCases } from "../store";

/** Home page: choose which ZBLL cases to practice, grouped by OCLL subset. */
export function HomePage() {
  const { selected, toggle, setMany } = useSelectedCases();

  const bySubset = useMemo(() => {
    const groups = new Map<string, typeof CASES>();
    for (const s of SUBSETS) groups.set(s, []);
    for (const c of CASES) {
      if (!groups.has(c.subset)) groups.set(c.subset, []);
      groups.get(c.subset)!.push(c);
    }
    return groups;
  }, []);

  const allIds = CASES.map((c) => c.id);

  return (
    <div>
      <div className="page-head">
        <div>
          <h1>Choose cases</h1>
          <p className="muted">
            {selected.size} of {CASES.length} cases selected.
          </p>
        </div>
        <div className="head-actions">
          <button onClick={() => setMany(allIds, true)}>Select all</button>
          <button onClick={() => setMany(allIds, false)}>Clear</button>
          <Link className="btn-primary" to="/train">
            Train →
          </Link>
        </div>
      </div>

      {[...bySubset.entries()].map(([subset, cases]) =>
        cases.length === 0 ? null : (
          <section key={subset} className="subset">
            <div className="subset-head">
              <h2>
                {subset} <span className="muted">({cases.length})</span>
              </h2>
              <div className="head-actions">
                <button
                  onClick={() =>
                    setMany(
                      cases.map((c) => c.id),
                      true,
                    )
                  }
                >
                  All
                </button>
                <button
                  onClick={() =>
                    setMany(
                      cases.map((c) => c.id),
                      false,
                    )
                  }
                >
                  None
                </button>
              </div>
            </div>
            <div className="case-grid">
              {cases.map((c) => {
                const on = selected.has(c.id);
                return (
                  <button
                    key={c.id}
                    className={`case-card ${on ? "on" : "off"}`}
                    onClick={() => toggle(c.id)}
                    aria-pressed={on}
                  >
                    <CaseDiagram state={c.state} size={104} />
                    <span className="case-label">{c.id}</span>
                  </button>
                );
              })}
            </div>
          </section>
        ),
      )}
    </div>
  );
}
