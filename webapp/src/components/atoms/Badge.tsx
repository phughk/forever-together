import type { HTMLAttributes } from "react";

export type Tone = "neutral" | "good" | "warn" | "bad" | "info" | "alliance" | "horde";

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  tone?: Tone;
}

export function Badge({ tone = "neutral", className, ...rest }: BadgeProps) {
  const cls = ["ft-badge", tone !== "neutral" && `ft-badge--${tone}`, className].filter(Boolean).join(" ");
  return <span className={cls} {...rest} />;
}
