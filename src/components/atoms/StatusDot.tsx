export function StatusDot({ tone = "neutral" }: { tone?: "neutral" | "good" | "warn" | "bad" }) {
  return <span aria-hidden="true" className={["ft-dot", tone !== "neutral" && `ft-dot--${tone}`].filter(Boolean).join(" ")} />;
}
