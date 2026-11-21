import type { Role } from "../../api/types";
import { ROLE_LABEL } from "../../lib/format";

const GLYPH: Record<Role, string> = { tank: "T", healer: "H", damage: "D", unknown: "?" };

export function RoleIcon({ role }: { role: Role }) {
  return (
    <span className={`ft-role ft-role--${role}`} role="img" aria-label={ROLE_LABEL[role]} title={ROLE_LABEL[role]}>
      {GLYPH[role]}
    </span>
  );
}
