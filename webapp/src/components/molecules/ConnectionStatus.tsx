import type { ConnectionStatus as Status } from "../../api/types";
import { StatusDot } from "../atoms/StatusDot";

const COPY: Record<Exclude<Status, "live">, { tone: "warn" | "bad"; text: string; detail: string }> = {
  connecting: { tone: "warn", text: "Connecting...", detail: "Connecting to the app" },
  reconnecting: { tone: "bad", text: "Reconnecting...", detail: "The connection to the app was lost. Showing the last known state." },
  unauthorized: { tone: "bad", text: "Not linked", detail: "This device is not linked to the app. Scan the QR code in the desktop app's Settings." },
};

/**
 * A chip shown only when something is wrong: a dot and a short phrase. A healthy connection renders nothing,
 * because there is nothing to act on. The longer meaning is the tooltip.
 */
export function ConnectionStatus({ status }: { status: Status }) {
  if (status === "live") return null;
  const { tone, text, detail } = COPY[status];
  return (
    <span className="ft-conn" role="status" title={detail}>
      <StatusDot tone={tone} />
      {text}
    </span>
  );
}
