import { CONFIRM_WINDOW_MS } from "../../api/constants";
import type { DropReason, PartyView } from "../../api/types";
import { fullName } from "../../lib/format";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Text } from "../atoms/Typography";
import { Countdown } from "../molecules/Countdown";
import { MemberRow } from "../molecules/MemberRow";

export interface PartyPanelProps {
  party: PartyView;
  now: number;
  onConfirm: () => void | Promise<void>;
  onDecline: () => void | Promise<void>;
  onDismiss?: () => void;
}

const DROP_COPY: Record<DropReason, string> = {
  declined: "A member declined. The party was dropped and everyone is searching again. Nobody is penalised.",
  timeout: "A member did not confirm in time. The party was dropped and everyone is searching again. Nobody is penalised.",
  unreachable: "A member's app stopped answering. The party was dropped and that peer is skipped for about a minute.",
};

export function PartyPanel({ party, now, onConfirm, onDecline, onDismiss }: PartyPanelProps) {
  const me = party.members.find((m) => m.self);
  const waitingOn = party.members.filter((m) => !m.confirmed && !m.declined).length;

  if (party.phase === "dropped") {
    return (
      <Card title="Party dropped">
        <Notice tone="warn">{DROP_COPY[party.dropReason ?? "timeout"]}</Notice>
        {onDismiss && (
          <div className="ft-actions">
            <Button onClick={onDismiss}>OK</Button>
          </div>
        )}
      </Card>
    );
  }

  if (party.phase === "final") {
    const others = party.members.filter((m) => !m.self);
    return (
      <Card title="Your party is ready">
        <Notice tone="good">
          All five confirmed. Invite each other in game, typing the name exactly as shown, first name then surname.
        </Notice>
        <ul className="ft-list">
          {party.members.map((m) => (
            <li key={m.peerId}>
              <MemberRow member={m} revealName />
            </li>
          ))}
        </ul>
        <Text size="small" muted>
          Names are claimed by the other players and not verified. To invite: {others.map((m) => fullName(m)).filter(Boolean).join(", ")}.
        </Text>
      </Card>
    );
  }

  const remainingMs = party.confirmDeadline ? Math.max(0, party.confirmDeadline - now) : 0;
  return (
    <Card title="A party was found">
      <Text>
        Five players fit together. Press Confirm if you want to play with them. Waiting on {waitingOn}{" "}
        {waitingOn === 1 ? "player" : "players"}.
      </Text>
      <ul className="ft-list">
        {party.members.map((m) => (
          <li key={m.peerId}>
            <MemberRow member={m} revealName={false} />
          </li>
        ))}
      </ul>
      {party.confirmDeadline !== null && (
        <Countdown label="Time to confirm" remainingMs={remainingMs} totalMs={CONFIRM_WINDOW_MS} />
      )}
      <div className="ft-actions">
        <Button variant="primary" disabled={!me || me.confirmed} onClick={() => void onConfirm()}>
          {me?.confirmed ? "Confirmed, waiting for the others" : "Confirm"}
        </Button>
        <Button variant="ghost" disabled={!me || me.confirmed} onClick={() => void onDecline()}>
          Decline
        </Button>
      </div>
      <Text size="small" muted>
        Names appear after all five confirm.
      </Text>
    </Card>
  );
}
