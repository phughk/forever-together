import { CONFIRM_WINDOW_MS } from "../../api/constants";
import type { DropReason, PartyView } from "../../api/types";
import "../../styles/party.css";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Countdown } from "../molecules/Countdown";
import { MemberRow } from "../molecules/MemberRow";

export interface PartyPanelProps {
  party: PartyView;
  now: number;
  /** Called by the Accept button (the visible text changed, the name did not). */
  onConfirm: () => void | Promise<void>;
  onDecline: () => void | Promise<void>;
  onDismiss?: () => void;
}

/** One sentence of cause and consequence. Only an unreachable player is skipped; the other two say nothing about penalties. */
const DROP_COPY: Record<DropReason, string> = {
  declined: "Someone declined. Searching again.",
  timeout: "Someone did not answer in time. Searching again.",
  unreachable: "Someone went offline. Searching again and skipping them for about a minute.",
};

export function PartyPanel({ party, now, onConfirm, onDecline, onDismiss }: PartyPanelProps) {
  const me = party.members.find((m) => m.self);

  if (party.phase === "dropped") {
    return (
      <Card title="Party dropped">
        <div className="ft-party">
          <Notice tone="warn">{DROP_COPY[party.dropReason ?? "timeout"]}</Notice>
          {onDismiss && (
            <div className="ft-actions">
              <Button onClick={onDismiss}>OK</Button>
            </div>
          )}
        </div>
      </Card>
    );
  }

  if (party.phase === "final") {
    return (
      <Card title="Your party is ready">
        <div className="ft-party">
          <Notice tone="good">Invite each other in game.</Notice>
          <ul className="ft-list">
            {party.members.map((m) => (
              <li key={m.peerId}>
                <MemberRow member={m} revealName />
              </li>
            ))}
          </ul>
        </div>
      </Card>
    );
  }

  const remainingMs = party.confirmDeadline ? Math.max(0, party.confirmDeadline - now) : 0;
  const accepted = !!me?.confirmed;
  return (
    <Card title="A party was found">
      <div className="ft-party ft-party--confirming">
        {party.confirmDeadline !== null && (
          <Countdown label="Time to accept" remainingMs={remainingMs} totalMs={CONFIRM_WINDOW_MS} />
        )}
        <div className="ft-party__actions">
          <Button variant="primary" disabled={!me || accepted} onClick={() => void onConfirm()}>
            {accepted ? "Accepted" : "Accept"}
          </Button>
          <Button variant="ghost" disabled={!me || accepted} onClick={() => void onDecline()}>
            Decline
          </Button>
        </div>
        <ul className="ft-list">
          {party.members.map((m) => (
            <li key={m.peerId}>
              <MemberRow member={m} revealName={false} />
            </li>
          ))}
        </ul>
      </div>
    </Card>
  );
}
