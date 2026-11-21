import type { DiagnosisRole, DiagnosisView } from "../../api/types";
import { attemptsPhrase, diagnosisCopy } from "../../lib/diagnosisCopy";
import { Notice } from "../atoms/Notice";
import { RoleIcon } from "../atoms/RoleIcon";
import { Text } from "../atoms/Typography";

export interface DiagnosisSummaryProps {
  diagnosis: DiagnosisView;
  /** Seconds searched so far, or the full length of an ended search. */
  elapsedS: number;
  /** How many dungeons the search covers. More than one adds the scope to the sentence. */
  dungeonCount: number;
}

const ROLES: DiagnosisRole[] = ["tank", "healer", "damage"];
const KEY = { tank: "tanks", healer: "healers", damage: "damage" } as const;

/** The counts mean nothing before the app has looked around or while it is offline. */
function showsCounts(d: DiagnosisView): boolean {
  return d.situation !== "warming_up" && d.situation !== "not_connected";
}

/** What this computer sees, in one or two sentences and a compact row of counts. */
export function DiagnosisSummary({ diagnosis, elapsedS, dungeonCount }: DiagnosisSummaryProps) {
  const copy = diagnosisCopy(diagnosis, elapsedS, dungeonCount);
  const prefix = diagnosis.sampleFull ? "about " : "";
  const n = diagnosis.seen.players;
  const tried = attemptsPhrase(diagnosis);
  return (
    <div className="ft-diagnosis">
      <Notice tone={copy.tone}>
        <div>{copy.headline}</div>
        {copy.detail && <div className="ft-diagnosis__detail">{copy.detail}</div>}
      </Notice>
      {copy.outlook && (
        <Text size="small" muted>
          {copy.outlook}
        </Text>
      )}
      {showsCounts(diagnosis) && (
        <div>
          <ul className="ft-diagnosis__seen" aria-label="Other players this computer sees">
            <li>
              {n > 0 ? prefix : ""}
              {n} other {n === 1 ? "player" : "players"}:
            </li>
            {ROLES.map((r) => (
              <li key={r}>
                <RoleIcon role={r} /> {diagnosis.seen[KEY[r]]}
              </li>
            ))}
            {tried && <li>{tried}</li>}
          </ul>
          <Text size="small" muted>
            Rough counts from this computer, not including you.
          </Text>
        </div>
      )}
    </div>
  );
}
