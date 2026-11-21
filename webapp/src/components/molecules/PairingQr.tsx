import { QrCode } from "../atoms/QrCode";
import { Text } from "../atoms/Typography";
import { Details } from "./Details";
import "../../styles/settings.css";

/** The QR that links a phone. The link text sits behind "Show link" and is shown only on the desktop (it carries the secret). */
export function PairingQr({ url }: { url: string }) {
  return (
    <div className="ft-pairing">
      <QrCode value={url} label="QR code. Scan it with the phone camera." />
      <Text size="small" muted>
        Scan with your phone's camera.
      </Text>
      <Details summary="Show link">
        <Text size="small" mono className="ft-pairing__url">
          {url}
        </Text>
      </Details>
    </div>
  );
}
