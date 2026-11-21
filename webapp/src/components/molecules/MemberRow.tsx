import { useState } from "react";
import type { PartyMember } from "../../api/types";
import { fullName, ROLE_LABEL } from "../../lib/format";
import "../../styles/party.css";
import { Badge } from "../atoms/Badge";
import { Button } from "../atoms/Button";
import { RoleIcon } from "../atoms/RoleIcon";

export interface MemberRowProps {
  member: PartyMember;
  /** Names are shown only when the party is final. */
  revealName: boolean;
  /** Called with the name when Copy name is pressed. Defaults to the browser clipboard. */
  onCopyName?: (name: string) => void;
}

async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    /* no clipboard access: the name is still on screen */
  }
}

/** One of the five role slots: a tick once the player accepted, the role, and Waiting or Accepted. */
export function MemberRow({ member, revealName, onCopyName }: MemberRowProps) {
  const [copied, setCopied] = useState(false);
  const name = fullName(member);
  const canCopy = revealName && !!name && !member.self;
  const copy = () => {
    if (onCopyName) onCopyName(name);
    else void copyToClipboard(name);
    setCopied(true);
  };
  return (
    <div className={["ft-member", "ft-member--slot", member.self && "ft-member--self"].filter(Boolean).join(" ")}>
      <span
        className={["ft-member__tick", member.confirmed && "ft-member__tick--on"].filter(Boolean).join(" ")}
        role="img"
        aria-label={member.confirmed ? "Accepted" : "Waiting"}
      >
        {"✓"}
      </span>
      <RoleIcon role={member.role} />
      <div className="ft-member__main">
        <span className="ft-member__name" title={revealName && name && !member.self ? name : undefined}>
          {member.self ? "You" : revealName && name ? name : ROLE_LABEL[member.role]}
        </span>
        <span className="ft-member__meta">
          Level {member.level}
          {member.itemLevel > 0 ? `, item level ${member.itemLevel}` : ""}
        </span>
      </div>
      <div className="ft-member__status">
        {canCopy ? (
          <Button size="small" onClick={copy} aria-label={`Copy name ${name}`}>
            {copied ? "Copied" : "Copy name"}
          </Button>
        ) : (
          !revealName && (member.confirmed ? <Badge tone="good">Accepted</Badge> : <Badge>Waiting</Badge>)
        )}
      </div>
    </div>
  );
}
