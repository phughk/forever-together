import type { ConnectionStatus as Status } from "../../api/types";
import { StatusDot } from "../atoms/StatusDot";

const COPY: Record<Status, { tone: "good" | "warn" | "bad"; text: string }> = {
  live: { tone: "good", text: "Connected" },
  connecting: { tone: "warn", text: "Connecting..." },
  reconnecting: { tone: "warn", text: "Connection lost, retrying" },
  unauthorized: { tone: "bad", text: "Not paired" },
};

export function ConnectionStatus({ status }: { status: Status }) {
  const { tone, text } = COPY[status];
  return (
    <span className="ft-conn" role="status">
      <StatusDot tone={tone} />
      {text}
    </span>
  );
}
