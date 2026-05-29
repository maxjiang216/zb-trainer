import { useCallback, useEffect, useState } from "react";

import casesJson from "./data/cases.json";
import type { ZbllCaseData } from "./types";

export const CASES = casesJson as unknown as ZbllCaseData[];

export const CASES_BY_ID = new Map(CASES.map((c) => [c.id, c]));

export const SUBSETS = ["H", "T", "U", "L", "Pi", "S", "AS"] as const;

/** Generic localStorage-backed state hook (JSON-serialized). */
export function usePersistentState<T>(
  key: string,
  initial: T,
): [T, (next: T | ((prev: T) => T)) => void] {
  const [value, setValue] = useState<T>(() => {
    try {
      const raw = localStorage.getItem(key);
      return raw === null ? initial : (JSON.parse(raw) as T);
    } catch {
      return initial;
    }
  });

  useEffect(() => {
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch {
      // Storage may be unavailable (private mode); selection just won't persist.
    }
  }, [key, value]);

  return [value, setValue];
}

const SELECTED_KEY = "zbll.selectedCases";
const CHOSEN_ALG_KEY = "zbll.chosenAlgs";

/**
 * Which case ids are selected for practice. Stored as an explicit id list so a
 * growing case set defaults sanely; absent = all selected.
 */
export function useSelectedCases() {
  const [ids, setIds] = usePersistentState<string[] | null>(SELECTED_KEY, null);
  const selected = new Set(ids ?? CASES.map((c) => c.id));

  const toggle = useCallback(
    (id: string) => {
      setIds((prev) => {
        const set = new Set(prev ?? CASES.map((c) => c.id));
        if (set.has(id)) set.delete(id);
        else set.add(id);
        return [...set];
      });
    },
    [setIds],
  );

  const setMany = useCallback(
    (idsToSet: string[], on: boolean) => {
      setIds((prev) => {
        const set = new Set(prev ?? CASES.map((c) => c.id));
        for (const id of idsToSet) {
          if (on) set.add(id);
          else set.delete(id);
        }
        return [...set];
      });
    },
    [setIds],
  );

  return { selected, toggle, setMany };
}

/** The user's chosen alg per case (defaults to the first / speedcubedb alg). */
export function useChosenAlgs() {
  const [map, setMap] = usePersistentState<Record<string, string>>(
    CHOSEN_ALG_KEY,
    {},
  );

  const chosenAlg = useCallback(
    (id: string): string => {
      const c = CASES_BY_ID.get(id);
      if (!c) return "";
      return map[id] ?? c.algs[0]?.alg ?? "";
    },
    [map],
  );

  const setChosen = useCallback(
    (id: string, alg: string) => setMap((prev) => ({ ...prev, [id]: alg })),
    [setMap],
  );

  return { chosenAlg, setChosen };
}
