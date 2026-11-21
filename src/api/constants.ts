import type { Stage } from "./types";

// Mirrors the implementation plan's shared definitions. Display only: the
// schedule itself lives in the Rust engine and the API cannot change it.
export const WIDENING_STEP_S = 60;
export const SEARCH_LIMIT_S = 600;
export const STALE_CODE_AFTER_S = 30 * 60;
export const CONFIRM_WINDOW_MS = 15_000;
export const PARTY_SIZE = 5;
export const LAST_STAGE: Stage = 4;

export const STAGES: ReadonlyArray<{ stage: Stage; startsAtS: number; title: string; rule: string }> = [
  { stage: 0, startsAtS: 0, title: "Strict", rule: "Item level within 15%, level spread up to 3, exact roles" },
  { stage: 1, startsAtS: 60, title: "Secondary roles", rule: "Secondary roles allowed (no change in v1)" },
  { stage: 2, startsAtS: 120, title: "Wider item level", rule: "Item level within 30%" },
  { stage: 3, startsAtS: 180, title: "Wider levels", rule: "Level spread up to 6" },
  { stage: 4, startsAtS: 240, title: "Widest", rule: "Level spread up to 10, plus the other recommended dungeons" },
];
