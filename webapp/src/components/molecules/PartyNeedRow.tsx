import type { DiagnosisRole } from "../../api/types";
import { ROLE_LABEL } from "../../lib/format";
import { RoleIcon } from "../atoms/RoleIcon";
import "../../styles/search.css";

/** What a party needs: 1 tank, 1 healer, 3 damage players. */
export const PARTY_NEED: Record<DiagnosisRole, number> = { tank: 1, healer: 1, damage: 3 };
const ROLES: DiagnosisRole[] = ["tank", "healer", "damage"];

export interface PartyNeedRowProps {
  /** Other players seen per role, not including the player. */
  seen: { tanks: number; healers: number; damage: number };
  /** The player's own role is pre-filled, so it counts one toward its slot. */
  ownRole: DiagnosisRole | null;
  /** The counts are a slice of a larger sample: a number above zero gets a `~`. */
  rough?: boolean;
}

const KEY = { tank: "tanks", healer: "healers", damage: "damage" } as const;

/**
 * The three roles against what a party needs, as "seen / needed". Grey means not seen from here, never red:
 * "none seen" must not read as "none exist". The label "Seen from here" and the `~` carry the hedge.
 */
export function PartyNeedRow({ seen, ownRole, rough }: PartyNeedRowProps) {
  return (
    <div className="ft-need">
      <span className="ft-need__label">Seen from here</span>
      <ul className="ft-need__roles" aria-label="Roles seen from here, against what a party needs">
        {ROLES.map((r) => {
          const n = seen[KEY[r]] + (ownRole === r ? 1 : 0);
          const need = PARTY_NEED[r];
          const state = n === 0 ? "none" : n >= need ? "met" : "some";
          return (
            <li
              key={r}
              className={`ft-need__role ft-need__role--${state}`}
              aria-label={`${ROLE_LABEL[r]}: ${n} seen of ${need} needed${ownRole === r ? ", including you" : ""}`}
            >
              <RoleIcon role={r} />
              <span aria-hidden="true">
                {rough && n > 0 ? "~" : ""}
                {n}/{need}
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
