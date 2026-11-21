import { Notice } from "../atoms/Notice";
import { SettingsIcon } from "../atoms/SettingsIcon";
import "../../styles/settings.css";

/** The single banner shown at the top of Settings on a phone. Individual controls carry the lock icon instead of repeating this. */
export function RemoteBanner() {
  return (
    <Notice tone="info" className="ft-remote-banner">
      <SettingsIcon name="lock" />
      Change settings on your PC.
    </Notice>
  );
}
