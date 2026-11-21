import type { AppState } from "../api/types";

/** The one thing the finder shows. Every state maps to exactly one screen. */
export type FinderScreen =
  | "first_run" // no character yet: the first-run steps
  | "idle" // character, no search, no party: the dungeon list
  | "searching" // a search is running
  | "ended" // the search hit its time limit
  | "party_found" // a party formed and is waiting for answers
  | "party_final" // everyone accepted
  | "party_dropped"; // the party fell apart

/** Pure: derives the screen from the state. A party wins over a search, a search over the character. */
export function finderScreen({ character, search, party }: Pick<AppState, "character" | "search" | "party">): FinderScreen {
  if (party) {
    if (party.phase === "final") return "party_final";
    if (party.phase === "dropped") return "party_dropped";
    return "party_found";
  }
  if (search.status === "searching") return "searching";
  if (search.status === "limit_reached") return "ended";
  return character ? "idle" : "first_run";
}
