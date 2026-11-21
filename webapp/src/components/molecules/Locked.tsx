import type { ReactNode } from "react";
import { SettingsIcon } from "../atoms/SettingsIcon";
import "../../styles/settings.css";

export interface LockedProps {
  /** True when this device may not change the control (a phone viewing a desktop-only setting). */
  locked?: boolean;
  /** Let the control take the row's width (a form) instead of its own size. */
  block?: boolean;
  children: ReactNode;
}

/**
 * The one shared "only the PC can change this" treatment: the caller disables the control, this adds the lock icon beside it.
 * Renders the control untouched when not locked.
 */
export function Locked({ locked, block, children }: LockedProps) {
  if (!locked) return <>{children}</>;
  return (
    <span className={["ft-locked", block && "ft-locked--block"].filter(Boolean).join(" ")}>
      {children}
      <SettingsIcon name="lock" label="Change this on your PC" />
    </span>
  );
}
