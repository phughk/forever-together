import { Badge } from "../atoms/Badge";
import { InfoTip } from "../atoms/InfoTip";
import "../../styles/search.css";

/** The one sentence on what widening means, in the info tip. Within the info tip cap (160). */
export const SEARCHING_WIDER_TIP =
  "After a while the search accepts a wider item level and level range, then other recommended dungeons.";

/** A small chip shown once the search has widened. No countdown to the next step. */
export function SearchingWiderChip() {
  return (
    <span className="ft-wider">
      <Badge tone="info">Searching wider</Badge>
      <InfoTip label="About searching wider">{SEARCHING_WIDER_TIP}</InfoTip>
    </span>
  );
}
