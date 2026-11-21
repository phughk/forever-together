import type { PartyMember } from "../../api/types";
import { fullName, ROLE_LABEL } from "../../lib/format";
import { Badge } from "../atoms/Badge";
import { RoleIcon } from "../atoms/RoleIcon";

export interface MemberRowProps {
  member: PartyMember;
  /** Names are shown only when the party is final. */
  revealName: boolean;
}

export function MemberRow({ member, revealName }: MemberRowProps) {
  const name = fullName(member);
  return (
    <div className={["ft-member", member.self && "ft-member--self"].filter(Boolean).join(" ")}>
      <RoleIcon role={member.role} />
      <div className="ft-member__main">
        <span className="ft-member__name">
          {member.self ? "You" : revealName && name ? name : ROLE_LABEL[member.role]}
          {revealName && name && !member.self && <span className="ft-member__claimed"> (claimed by the player)</span>}
        </span>
        <span className="ft-member__meta">
          Level {member.level}
          {member.itemLevel > 0 ? `, item level ${member.itemLevel}` : ""}
        </span>
      </div>
      <div className="ft-member__status">
        {member.declined ? (
          <Badge tone="bad">Declined</Badge>
        ) : member.confirmed ? (
          <Badge tone="good">Confirmed</Badge>
        ) : member.ready ? (
          <Badge tone="warn">Ready, not confirmed</Badge>
        ) : (
          <Badge>Waiting</Badge>
        )}
      </div>
    </div>
  );
}
