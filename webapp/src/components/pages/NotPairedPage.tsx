import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Text } from "../atoms/Typography";
import { AppHeader } from "../organisms/AppHeader";
import { SettingsTemplate } from "../templates/SettingsTemplate";

/** Shown when the server answers 401: the token is missing, wrong or was reset. */
export function NotPairedPage() {
  return (
    <SettingsTemplate
      header={<AppHeader view="finder" onNavigate={() => {}} status="unauthorized" />}
      sections={
        <Card title="This device is not paired">
          <Notice tone="warn">The app refused the request because this browser has no valid token.</Notice>
          <Text>
            On the desktop app open Settings, turn on LAN mode and scan the pairing QR with this phone's camera app. If you reset
            the token on the desktop, every phone must pair again.
          </Text>
        </Card>
      }
    />
  );
}
