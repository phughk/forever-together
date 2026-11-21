import { formatClock } from "../../lib/format";
import { ProgressBar } from "../atoms/ProgressBar";
import "../../styles/search.css";

export interface SearchClockProps {
  /** Seconds searched so far. */
  elapsedS: number;
  /** The search limit in seconds. */
  limitS: number;
}

/** The last minute turns the bar red, nothing else changes. */
const LAST_MINUTE_S = 60;

/**
 * The time of a running search: elapsed time as a large tabular number, the limit beside it, and a thick bar.
 * The number is the main thing on the screen, so it is set large; the limit stays muted.
 */
export function SearchClock({ elapsedS, limitS }: SearchClockProps) {
  const shown = Math.min(Math.max(0, Math.floor(elapsedS)), limitS);
  const urgent = limitS - shown <= LAST_MINUTE_S;
  return (
    <div className="ft-clock">
      <div className="ft-clock__row">
        <span className="ft-clock__elapsed" aria-label={`Searched ${shown} seconds`}>
          {formatClock(shown)}
        </span>
        <span className="ft-clock__limit">of {formatClock(limitS)}</span>
      </div>
      <ProgressBar value={elapsedS / limitS} label="Time until the search limit" size="thick" urgent={urgent} />
    </div>
  );
}
