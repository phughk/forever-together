import type { DiagnosisRole, DiagnosisView } from "../api/types";
import { formatAge } from "./format";

export type DiagnosisTone = "info" | "warn" | "good";

export interface DiagnosisCopy {
  headline: string;
  detail?: string;
  tone: DiagnosisTone;
}

/**
 * Length limits for the player-facing sentences, for the story docs and for layout:
 * a headline stays within 220 characters and a detail within 140, with up to 3 dungeons named in the scope
 * (the scope is a count, "across 3 dungeons", so the dungeon list never lengthens the sentence).
 */
export const HEADLINE_MAX_CHARS = 220;
export const DETAIL_MAX_CHARS = 140;

const PARTY_NEED = "A party needs 1 tank, 1 healer and 3 damage players.";

const ROLE_NOUN: Record<DiagnosisRole, [string, string]> = {
  tank: ["tank", "tanks"],
  healer: ["healer", "healers"],
  damage: ["damage player", "damage players"],
};

function noun(n: number, role: DiagnosisRole): string {
  return ROLE_NOUN[role][n === 1 ? 0 : 1];
}

/** "about 8 other players"; "no other players" for zero. A full sample is a slice, so counts are rough. */
function playersPhrase(d: DiagnosisView): string {
  const n = d.seen.players;
  if (n === 0) return "no other players";
  return `${d.sampleFull ? "about " : ""}${n} other ${n === 1 ? "player" : "players"}`;
}

/** "0 tanks, 2 healers, 6 damage players". Counts are of other players, so the sentence around it says so. */
function mixPhrase(d: DiagnosisView): string {
  const p = (n: number) => (d.sampleFull && n > 0 ? "about " : "");
  const part = (n: number, r: DiagnosisRole) => `${p(n)}${n} ${noun(n, r)}`;
  return `${part(d.seen.tanks, "tank")}, ${part(d.seen.healers, "healer")}, ${part(d.seen.damage, "damage")}`;
}

function scope(dungeonCount: number): string {
  return dungeonCount > 1 ? ` across ${dungeonCount} dungeons` : "";
}

/** "no tank", or "no other tank besides you" when the missing role is the player's own. */
function missing(d: DiagnosisView, role: DiagnosisRole): string {
  return d.ownRole === role ? `no other ${ROLE_NOUN[role][0]} besides you` : `no ${ROLE_NOUN[role][0]}`;
}

/**
 * Turns a diagnosis into the sentences shown to the player. Pure, no React.
 *
 * Wording rules: every claim is about what this computer sees ("none seen", never "none exist"), numbers are
 * rough, nothing promises a group will form, and nothing reassures while a role is missing or the app is not
 * connected. Unknown situation codes get a neutral sentence.
 */
export function diagnosisCopy(d: DiagnosisView, elapsedS: number, dungeonCount: number): DiagnosisCopy {
  const where = scope(dungeonCount);
  const over = formatAge(elapsedS);

  switch (d.situation) {
    case "warming_up":
      return d.ended
        ? {
            headline: `The search ended before this computer finished looking around${where}.`,
            detail: "Search again, and check your connection if this repeats.",
            tone: "warn",
          }
        : { headline: "Still looking around for other players. What this computer can see will show up here shortly.", tone: "info" };

    case "not_connected":
      return d.ended
        ? {
            headline: `In ${over} this app did not reach any other players, so it could not see who is searching${where}.`,
            detail: "Check your internet connection, firewall or VPN, then search again.",
            tone: "warn",
          }
        : {
            headline: "This app is not reaching other players yet.",
            detail: "Check your internet connection, firewall or VPN. The search keeps trying in the meantime.",
            tone: "warn",
          };

    case "alone":
      return d.ended
        ? {
            headline: `In ${over} this computer saw no other compatible players searching${where}.`,
            detail: "Try another dungeon, or search again later. Others may be searching where this computer cannot see.",
            tone: "warn",
          }
        : {
            headline: `Connected, but this computer sees no other compatible players searching${where} so far.`,
            detail: "Another dungeon may have more players. You can also keep waiting.",
            tone: "warn",
          };

    case "no_tank":
    case "no_healer": {
      const role: DiagnosisRole = d.situation === "no_tank" ? "tank" : "healer";
      const gap = missing(d, role);
      return d.ended
        ? {
            headline: `Over ${over} this computer saw ${playersPhrase(d)}${where}: ${mixPhrase(d)}. It saw ${gap}.`,
            detail: `${PARTY_NEED} Try another dungeon, a different role if you have one, or search again later.`,
            tone: "warn",
          }
        : {
            headline: `This computer sees ${playersPhrase(d)}${where}, but ${gap} so far.`,
            detail: PARTY_NEED,
            tone: "warn",
          };
    }

    case "short_on_damage":
      return d.ended
        ? {
            headline: `Over ${over} this computer saw ${playersPhrase(d)}${where}: ${mixPhrase(d)}. That is short on damage players.`,
            detail: `${PARTY_NEED} Try another dungeon, or search again later.`,
            tone: "warn",
          }
        : {
            headline: `This computer sees a tank and a healer${where}, but fewer than 3 other damage players so far.`,
            detail: PARTY_NEED,
            tone: "warn",
          };

    case "healthy":
      return d.ended
        ? {
            headline: `Over ${over} this computer saw ${playersPhrase(d)}${where}: ${mixPhrase(d)}. That looked like enough, but no party was completed.`,
            detail: "Search again, or try another dungeon.",
            tone: "info",
          }
        : {
            headline: `From what this computer sees${where}, a group looks possible: ${playersPhrase(d)}, ${mixPhrase(d)}.`,
            detail: "This is only what is visible from here, not a promise that a group will form.",
            tone: "good",
          };

    default:
      return d.ended
        ? { headline: `The search ended without forming a party${where}.`, detail: "Try another dungeon, or search again.", tone: "warn" }
        : { headline: `Searching${where}. This computer cannot say more about what it sees yet.`, tone: "info" };
  }
}
