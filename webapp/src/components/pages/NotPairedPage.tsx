import { Card } from "../atoms/Card";
import { SettingsIcon } from "../atoms/SettingsIcon";
import { NumberedStep } from "../molecules/NumberedStep";
import { SettingsTemplate } from "../templates/SettingsTemplate";
import "../../styles/settings.css";

/** Shown when the server answers 401: this phone has no valid link. It has no navigation, since nothing else works yet. */
export function NotPairedPage() {
  return (
    <SettingsTemplate
      narrow
      sections={
        <Card title="This phone isn't linked yet">
          <ol className="ft-steps">
            <NumberedStep n={1} icon={<SettingsIcon name="monitor" size={24} />}>
              Open Settings on your PC
            </NumberedStep>
            <NumberedStep n={2} icon={<SettingsIcon name="switch" size={24} />}>
              Switch on Phone link
            </NumberedStep>
            <NumberedStep n={3} icon={<SettingsIcon name="qr" size={24} />}>
              Scan the QR with this phone
            </NumberedStep>
          </ol>
        </Card>
      }
    />
  );
}
