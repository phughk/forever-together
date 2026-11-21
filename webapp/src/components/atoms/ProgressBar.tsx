export interface ProgressBarProps {
  /** 0 to 1. */
  value: number;
  /** Accessible name. */
  label: string;
  urgent?: boolean;
  /** `thick` is the 16px bar for a screen where the bar is the main thing. */
  size?: "default" | "thick";
}

export function ProgressBar({ value, label, urgent, size = "default" }: ProgressBarProps) {
  const pct = Math.round(Math.min(1, Math.max(0, value)) * 100);
  return (
    <div
      className={["ft-progress", size === "thick" && "ft-progress--thick", urgent && "ft-progress--warn"].filter(Boolean).join(" ")}
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={pct}
    >
      <div className="ft-progress__bar" style={{ width: `${pct}%` }} />
    </div>
  );
}
