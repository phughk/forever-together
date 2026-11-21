import { Badge } from "../atoms/Badge";

export interface StageStepProps {
  number: number;
  title: string;
  rule: string;
  state: "done" | "current" | "upcoming";
  /** Seconds until this stage begins, for an upcoming one. */
  startsInS?: number;
}

export function StageStep({ number, title, rule, state, startsInS }: StageStepProps) {
  return (
    <li className={`ft-step ft-step--${state}`} aria-current={state === "current" ? "step" : undefined}>
      <span className="ft-step__num">{number}</span>
      <span>
        <span className="ft-step__title">{title}</span>
        <br />
        <span className="ft-text ft-text--sm ft-text--muted">{rule}</span>
      </span>
      {state === "current" && <Badge tone="info">Now</Badge>}
      {state === "done" && <Badge tone="good">Passed</Badge>}
      {state === "upcoming" && startsInS !== undefined && <Badge>in {startsInS} s</Badge>}
    </li>
  );
}
