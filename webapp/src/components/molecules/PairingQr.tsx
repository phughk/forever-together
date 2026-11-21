import { QrCode } from "../atoms/QrCode";
import { Text } from "../atoms/Typography";

/** The pairing QR with the link as text. The URL fragment holds the token, so the link is shown only on the desktop. */
export function PairingQr({ url }: { url: string }) {
  return (
    <div className="ft-pairing">
      <QrCode value={url} label="Pairing QR code. Scan it with the phone camera." />
      <Text size="small" muted>
        Scan with the phone camera app and open the link.
      </Text>
      <Text size="small" mono className="ft-pairing__url">
        {url}
      </Text>
    </div>
  );
}
