import type { DiagnosisRole, DiagnosisView } from "../api/types";
import { formatAge } from "./format";

export type DiagnosisTone = "info" | "warn" | "good";

export interface DiagnosisCopy {
  /** One sentence: the consequence for the player. The hedge is not in it; it lives in `HEDGE_TIP`. */
  headline: string;
  /** Shown inside Details. */
  detail?: string;
  /** A quiet hedged clause about the player's earlier searches, shown inside Details after the detail. Absent when there is none to say. */
  outlook?: string;
  /** Short chips for what the role row cannot show, e.g. "Asked 6 players to join, none replied". */
  chips: string[];
  tone: DiagnosisTone;
}

/**
 * Length limits for the player-facing sentences, for the story docs and for layout:
 * a headline stays within 90 characters (the longest is the ended no-tank line with a duration and a scope, about 70),
 * a detail within 110 (the longest, the ended no-tank detail, is about 80),
 * with the scope a count ("in 3 dungeons"), so the dungeon list never lengthens the sentence.
 */
export const HEADLINE_MAX_CHARS = 90;
export const DETAIL_MAX_CHARS = 110;

/** The outlook clause stays within 110 characters (the longest, rarely_found, is about 100). No numbers, never a promise. */
export const OUTLOOK_MAX_CHARS = 110;

/**
 * The hedge, said once, in the info tip beside the headline. It must stay within the info tip cap (160).
 * The label "Seen from here" and the `~` on rough counts stay visible beside it.
 */
export const HEDGE_TIP = "Counts are rough and only what this device sees from here. Not a promise that a group will form.";

const OUTLOOK_COPY: Record<string, string> = {
  usually_found: "In your earlier searches like this, a group was usually found.",
  sometimes_found: "In your earlier searches like this, a group was sometimes found.",
  rarely_found: "In your earlier searches like this, a group was rarely found, so trying another dungeon may be worth it.",
};

/** Situations that are not a live problem. Anything else, including an unknown code, counts as one: good news from history must not sit next to it. */
const NOT_PROBLEMS = new Set(["healthy", "others_grouping"]);

function isLiveProblem(situation: string): boolean {
  return !NOT_PROBLEMS.has(situation);
}

/** The outlook clause, or undefined: unknown values, an ended search, and `usually_found` next to a live problem are dropped. */
function outlookClause(d: DiagnosisView): string | undefined {
  if (d.ended || !d.outlook) return undefined;
  const clause = Object.prototype.hasOwnProperty.call(OUTLOOK_COPY, d.outlook) ? OUTLOOK_COPY[d.outlook] : undefined;
  if (!clause) return undefined;
  if (d.outlook === "usually_found" && isLiveProblem(d.situation)) return undefined;
  return clause;
}

/** "Asked 6 players to join, none replied" when the player's app asked anyone, else null. */
export function attemptsPhrase(d: DiagnosisView): string | null {
  const a = d.attempts;
  if (!a || a.invitesSent <= 0) return null;
  const asked = `Asked ${a.invitesSent} ${a.invitesSent === 1 ? "player" : "players"} to join`;
  return a.acceptsGot > 0 ? `${asked}, ${a.acceptsGot} replied` : `${asked}, none replied`;
}

/** The chips for a diagnosis: "Others are forming groups" for that situation, and the attempts. */
function chipsFor(d: DiagnosisView): string[] {
  const chips: string[] = [];
  if (d.situation === "others_grouping") chips.push("Others are forming groups");
  const tried = attemptsPhrase(d);
  if (tried) chips.push(tried);
  return chips;
}

/** " in 3 dungeons" for a combined search, else nothing. */
function inScope(dungeonCount: number): string {
  return dungeonCount > 1 ? ` in ${dungeonCount} dungeons` : "";
}

/**
 * Turns a diagnosis into the sentences shown to the player. Pure, no React.
 *
 * Wording rules: the headline is the consequence in one sentence; every claim is about what is seen from here
 * ("none seen", never "none exist"); numbers are rough; nothing promises a group will form; nothing reassures
 * while a role is missing or the app is not connected. Unknown situation codes get a neutral sentence.
 */
export function diagnosisCopy(d: DiagnosisView, elapsedS: number, dungeonCount: number): DiagnosisCopy {
  const copy = situationCopy(d, elapsedS, dungeonCount);
  const outlook = outlookClause(d);
  const withChips = { ...copy, chips: chipsFor(d) };
  return outlook ? { ...withChips, outlook } : withChips;
}

type Base = Omit<DiagnosisCopy, "chips" | "outlook">;

function situationCopy(d: DiagnosisView, elapsedS: number, dungeonCount: number): Base {
  const where = inScope(dungeonCount);
  const over = formatAge(elapsedS);

  switch (d.situation) {
    case "warming_up":
      return d.ended
        ? {
            headline: "The search ended before it finished looking around.",
            detail: "Search again, and check your connection if this repeats.",
            tone: "warn",
          }
        : { headline: "Looking around for other players.", tone: "info" };

    case "not_connected":
      return d.ended
        ? {
            headline: `Not connected to other players, so nobody could be seen${where}.`,
            detail: "Check your internet connection, firewall or VPN, then search again.",
            tone: "warn",
          }
        : {
            headline: "Not connected to other players yet.",
            detail: "Check your internet connection, firewall or VPN. The search keeps trying.",
            tone: "warn",
          };

    case "alone":
      return d.ended
        ? {
            headline: `No other players seen searching${where} in ${over}.`,
            detail: "Try another dungeon, or search again later.",
            tone: "warn",
          }
        : {
            headline: `No other players seen searching${where} yet.`,
            detail: "Another dungeon may have more players. You can also keep waiting.",
            tone: "warn",
          };

    case "no_tank":
    case "no_healer": {
      const role: DiagnosisRole = d.situation === "no_tank" ? "tank" : "healer";
      if (d.ownRole === role) {
        // The player is the role: a missing other one does not stop a party, so no consequence is claimed.
        return {
          headline: `No other ${role} seen${where}${d.ended ? ` in ${over}` : ""}.`,
          detail: d.ended ? "Try another dungeon, or search again later." : "Another dungeon may have more players. You can also keep waiting.",
          tone: "warn",
        };
      }
      return d.ended
        ? {
            headline: `No ${role} seen${where} in ${over}, so no party could form.`,
            detail: "Try another dungeon, a different role if you have one, or search again later.",
            tone: "warn",
          }
        : { headline: `No ${role} seen${where}, so a party can't form yet.`, tone: "warn" };
    }

    case "short_on_damage":
      return d.ended
        ? {
            headline: `Too few damage players seen${where} in ${over}, so no party could form.`,
            detail: "Try another dungeon, or search again later.",
            tone: "warn",
          }
        : { headline: `Too few damage players seen${where}, so a party can't form yet.`, tone: "warn" };

    case "others_grouping":
      return d.ended
        ? {
            headline: `Few players were left${where}, as others were forming groups.`,
            detail: "Wait and search again, or try another dungeon.",
            tone: "info",
          }
        : {
            headline: `Few players are left to group with${where} right now.`,
            detail: "Keep waiting, or try another dungeon.",
            tone: "info",
          };

    case "nobody_answered":
      return d.ended
        ? {
            headline: `Nobody replied when asked to join${where}, so no party formed.`,
            detail: "They may be offline. Search again, or try another dungeon.",
            tone: "warn",
          }
        : {
            headline: `Nobody has replied when asked to join${where} yet.`,
            detail: "They may be offline or not reaching you.",
            tone: "warn",
          };

    case "healthy":
      return d.ended
        ? {
            headline: `Enough players looked to be around${where}, but no party completed.`,
            detail: "Search again, or try another dungeon.",
            tone: "info",
          }
        : { headline: `A party looks possible${where}.`, tone: "good" };

    default:
      return d.ended
        ? { headline: `The search ended without a party${where}.`, detail: "Try another dungeon, or search again.", tone: "warn" }
        : { headline: `Searching${where}. Nothing more to report yet.`, tone: "info" };
  }
}
