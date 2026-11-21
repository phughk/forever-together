import { formatClock } from "../../lib/format";
import { ProgressBar } from "../atoms/ProgressBar";

export interface CountdownProps {
  label: string;
  remainingMs: number;
  totalMs: number;
}

/** A label, the time left and a bar that drains. Turns urgent in the last quarter. */
export function Countdown({ label, remainingMs, totalMs }: CountdownProps) {
  const ratio = totalMs > 0 ? remainingMs / totalMs : 0;
  return (
    <div className="ft-countdown">
      <div className="ft-countdown__head">
        <span>{label}</span>
        <span className="ft-countdown__time">{formatClock(remainingMs / 1000)}</span>
      </div>
      <ProgressBar value={ratio} label={label} urgent={ratio <= 0.25} />
    </div>
  );
}
