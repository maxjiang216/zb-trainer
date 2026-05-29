import type { CubeState } from "./cube/cube";

export type AlgSource = "speedcubedb" | "reco.nz";

export interface CaseAlg {
  alg: string;
  source: AlgSource;
  /** Top cubers observed using this alg (reco.nz reconstructions). */
  solvers: string[];
  /** Number of reconstructions using this alg. */
  uses: number;
}

export interface ZbllCaseData {
  /** Stable id, e.g. "U12". */
  id: string;
  /** OCLL subset: H, T, U, L, Pi, S, AS. */
  subset: string;
  n: number;
  /** Canonical last-layer state for rendering the case diagram. */
  state: CubeState;
  algs: CaseAlg[];
}
