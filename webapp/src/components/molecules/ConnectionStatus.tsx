import type { ConnectionStatus as Status } from "../../api/types";
import { StatusDot } from "../atoms/StatusDot";

const COPY: Record<Status, { tone: "good" | "warn" | "bad"; text: string; detail: string }> = {
  live: { tone: "good", text: "Online", detail: "Connected" },
  connecting: { tone: "warn", text: "Connecting", detail: "Connecting" },
  reconnecting: { tone: "bad", text: "Offline", detail: "Connection lost, retrying" },
  unauthorized: { tone: "bad", text: "Not paired", detail: "Not paired" },
};

/** A dot and one word. The longer meaning is the tooltip. */
export function ConnectionStatus({ status }: { status: Status }) {
  const { tone, text, detail } = COPY[status];
  return (
    <span className="ft-conn" role="status" title={detail}>
      <StatusDot tone={tone} />
      {text}
    </span>
  );
}
