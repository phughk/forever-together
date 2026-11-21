import type { DiagnosisView } from "../../api/types";
import { HEDGE_TIP, diagnosisCopy } from "../../lib/diagnosisCopy";
import { Badge } from "../atoms/Badge";
import { InfoTip } from "../atoms/InfoTip";
import { Notice } from "../atoms/Notice";
import { Text } from "../atoms/Typography";
import { Details } from "./Details";
import { PartyNeedRow } from "./PartyNeedRow";
import "../../styles/search.css";

export interface DiagnosisSummaryProps {
  diagnosis: DiagnosisView;
  /** Seconds searched so far, or the full length of an ended search. */
  elapsedS: number;
  /** How many dungeons the search covers. More than one adds the scope to the sentence. */
  dungeonCount: number;
}

/** The counts mean nothing before the app has looked around or while it is offline. */
function showsCounts(d: DiagnosisView): boolean {
  return d.situation !== "warming_up" && d.situation !== "not_connected";
}

/**
 * What is seen from here: a one-sentence consequence with the hedge in an info tip, the three roles against what a
 * party needs, short chips, and the detail and outlook sentences inside Details.
 */
export function DiagnosisSummary({ diagnosis, elapsedS, dungeonCount }: DiagnosisSummaryProps) {
  const copy = diagnosisCopy(diagnosis, elapsedS, dungeonCount);
  const counts = showsCounts(diagnosis);
  const hasMore = Boolean(copy.detail || copy.outlook);
  return (
    <div className="ft-diagnosis">
      <Notice tone={copy.tone}>
        <div className="ft-diagnosis__headline">
          <span>{copy.headline}</span>
          {counts && <InfoTip label="About these counts">{HEDGE_TIP}</InfoTip>}
        </div>
      </Notice>
      {counts && <PartyNeedRow seen={diagnosis.seen} ownRole={diagnosis.ownRole} rough={diagnosis.sampleFull} />}
      {copy.chips.length > 0 && (
        <div className="ft-diagnosis__chips">
          {copy.chips.map((c) => (
            <Badge key={c}>{c}</Badge>
          ))}
        </div>
      )}
      {hasMore && (
        // The connection advice is the one action the player can take, so it starts open.
        <Details summary="More" defaultOpen={diagnosis.situation === "not_connected"}>
          {copy.detail && <Text size="small">{copy.detail}</Text>}
          {copy.outlook && (
            <Text size="small" muted>
              {copy.outlook}
            </Text>
          )}
        </Details>
      )}
    </div>
  );
}
