import type { CharacterView } from "../../api/types";
import { FACTION_LABEL } from "../../lib/format";
import "../../styles/finder.css";
import { Badge } from "../atoms/Badge";
import { InfoTip } from "../atoms/InfoTip";
import { RoleIcon } from "../atoms/RoleIcon";

export interface CharacterSummaryProps {
  character: CharacterView;
  /** The code is old: shows the "Out of date" badge. */
  stale?: boolean;
}

/** The character on one line: name, class, level, role, item level, and the badges that matter. */
export function CharacterSummary({ character, stale }: CharacterSummaryProps) {
  return (
    <div className="ft-charline">
      <span className="ft-charline__name" title={`${character.firstName} ${character.surname}`}>
        {character.firstName} {character.surname}
      </span>
      <Badge tone={character.faction}>{FACTION_LABEL[character.faction]}</Badge>
      <span className="ft-charline__fact">
        {[character.className, `level ${character.level}`].filter(Boolean).join(", ")}
      </span>
      <RoleIcon role={character.role} />
      {character.itemLevel > 0 && <span className="ft-charline__fact">item level {character.itemLevel}</span>}
      {stale && <Badge tone="warn">Out of date</Badge>}
      {!character.zoneKnown && <Badge tone="info">Zone not recognised</Badge>}
      <InfoTip label="About this information">Read from your code in the addon. It is not verified.</InfoTip>
    </div>
  );
}
