import { useState } from "react";
import type { AddonResult } from "../../api/addonFlow";
import type { AddonInfo, AddonStatus } from "../../api/types";
import { Badge, type Tone } from "../atoms/Badge";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { CopyChip } from "../atoms/CopyChip";
import { InfoTip } from "../atoms/InfoTip";
import { Notice } from "../atoms/Notice";
import { Text } from "../atoms/Typography";
import { AddonPathField } from "../molecules/AddonPathField";
import { AddonVersionPair } from "../molecules/AddonVersionPair";
import { Locked } from "../molecules/Locked";
import "../../styles/settings.css";

export interface AddonSettingsProps {
  /** Null while loading, or when this device may not see it. */
  info: AddonInfo | null;
  result?: AddonResult | null;
  /** An install or a folder save is running. */
  busy?: boolean;
  /** Plain-language reason the last pasted folder was refused. */
  folderError?: string | null;
  /** The page runs on a phone or another device: the buttons are locked. */
  remote?: boolean;
  /** Install, or update, or reinstall: the page saves a detected folder first. */
  onInstall: () => void | Promise<void>;
  /** Resolve `false` when the folder was refused: the field then stays open. */
  onSaveFolder: (folder: string) => void | boolean | Promise<void | boolean>;
  /** Open the folder field at first render (stories). */
  defaultFolderOpen?: boolean;
}

const BADGE: Record<AddonStatus, { tone: Tone; text: string }> = {
  no_folder: { tone: "warn", text: "Not found" },
  folder_invalid: { tone: "bad", text: "Wrong folder" },
  not_installed: { tone: "neutral", text: "Not installed" },
  up_to_date: { tone: "good", text: "Up to date" },
  update_available: { tone: "info", text: "Update available" },
  files_differ: { tone: "warn", text: "Files differ" },
  installed_newer: { tone: "warn", text: "Newer than this app" },
  unknown_version: { tone: "warn", text: "Unknown version" },
};

const TIP = "The addon folder is replaced as a whole. After installing, type /reload in the game or restart it.";

function buttonLabel(info: AddonInfo): string {
  switch (info.status) {
    case "no_folder":
      return "Use a folder";
    case "folder_invalid":
      return "Change folder";
    case "not_installed":
      return "Install addon";
    case "up_to_date":
    case "files_differ":
      return "Reinstall";
    case "update_available":
    case "unknown_version":
      return "Update addon";
    case "installed_newer":
      return `Install ${info.bundled_version} anyway`;
  }
}

function Values({ info }: { info: AddonInfo }) {
  const v = info.installed.version;
  switch (info.status) {
    case "no_folder":
      return null;
    case "folder_invalid":
      return (
        <p className="ft-addon__folder">
          <code>{info.folder}</code>
        </p>
      );
    case "not_installed":
    case "unknown_version":
      return <AddonVersionPair thisApp={info.bundled_version} thisAppLabel="In this app" />;
    case "update_available":
      return <AddonVersionPair inGame={v} thisApp={info.bundled_version} thisAppLabel="New" />;
    default:
      return <AddonVersionPair inGame={v} thisApp={info.bundled_version} />;
  }
}

/** The "Addon" card: a status badge, a values line and one button. The folder field shows only when it is needed or after "Change folder". */
export function AddonSettings({ info, result, busy, folderError, remote, onInstall, onSaveFolder, defaultFolderOpen }: AddonSettingsProps) {
  const [folderOpen, setFolderOpen] = useState(defaultFolderOpen ?? false);
  const badge = result ? (result.kind === "installed" ? { tone: "good" as Tone, text: "Installed" } : { tone: "bad" as Tone, text: "Could not install" }) : info ? BADGE[info.status] : null;
  const pickFolder = !!info && info.status === "no_folder" && info.candidates.length > 0;
  const showField = !remote && !!info && (folderOpen || pickFolder) && result?.kind !== "installed";
  const detected = !!info && info.folder_source === "detected" && info.folder !== null && info.status !== "folder_invalid";

  return (
    <Card>
      <div className="ft-addon__head">
        <span className="ft-setting__titlerow">
          <span className="ft-setting__title">Addon</span>
          <InfoTip label="About the addon">{TIP}</InfoTip>
        </span>
        {badge && <Badge tone={pickFolder && !result ? "warn" : badge.tone}>{pickFolder && !result ? "Choose a folder" : badge.text}</Badge>}
      </div>
      {!info && !remote && <Text muted>Checking the AddOns folder...</Text>}
      {!info && remote && (
        <Locked locked>
          <Button disabled>Install addon</Button>
        </Locked>
      )}
      {info && (
        <div className="ft-addon__body">
          {result?.kind === "installed" && (
            <>
              <AddonVersionPair thisApp={result.version} thisAppLabel="Now" />
              <p className="ft-addon__line">
                In the game, type <CopyChip text="/reload" />
              </p>
            </>
          )}
          {result?.kind === "failed" && <Notice tone="bad">{result.message}</Notice>}
          {!result && <Values info={info} />}
          {!result && detected && !showField && (
            <p className="ft-addon__folder">
              Use this folder? <code>{info.folder}</code>
            </p>
          )}
          {showField && (
            <AddonPathField
              candidates={info.candidates}
              busy={busy}
              error={folderError}
              onSubmit={async (f) => {
                if ((await onSaveFolder(f)) !== false) setFolderOpen(false);
              }}
              onCancel={pickFolder ? undefined : () => setFolderOpen(false)}
            />
          )}
          {!showField && result?.kind !== "installed" && (
            <div className="ft-addon__actions">
              <Locked locked={remote}>
                {result?.kind === "failed" ? (
                  <Button variant="primary" disabled={remote || busy} onClick={() => void onInstall()}>
                    {busy ? "Installing..." : "Try again"}
                  </Button>
                ) : info.status === "no_folder" || info.status === "folder_invalid" ? (
                  <Button variant="primary" disabled={remote || busy} onClick={() => setFolderOpen(true)}>
                    {buttonLabel(info)}
                  </Button>
                ) : (
                  <Button variant="primary" disabled={remote || busy} onClick={() => void onInstall()}>
                    {busy ? "Installing..." : buttonLabel(info)}
                  </Button>
                )}
              </Locked>
              {(result?.kind === "failed" || (info.folder !== null && info.status !== "folder_invalid")) && (
                <Button variant="ghost" size="small" disabled={remote || busy} onClick={() => setFolderOpen(true)}>
                  Change folder
                </Button>
              )}
            </div>
          )}
        </div>
      )}
    </Card>
  );
}
