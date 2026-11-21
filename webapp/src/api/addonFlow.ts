import { describeAddonError, isApiError, isLoopbackOnly } from "./errors";
import type { AddonInfo, Api } from "./types";

/** What the last install attempt did. It overrides the status shown until the next action. */
export type AddonResult = { kind: "installed"; version: string } | { kind: "failed"; message: string };

export type InstallOutcome =
  | { ok: true; info: AddonInfo; result: AddonResult }
  | { ok: false; remote: boolean; result: AddonResult };

/**
 * One click on "Install addon": a detected folder is saved first (that click is the confirmation), then the app's copy is installed.
 * The server message is shown as given; the fallback sentence is used only when it is empty.
 */
export async function installAddonFlow(api: Api, info: AddonInfo): Promise<InstallOutcome> {
  try {
    if (info.folder_source === "detected" && info.folder) await api.setAddonFolder(info.folder);
    const after = await api.installAddon();
    return { ok: true, info: after, result: { kind: "installed", version: after.bundled_version } };
  } catch (e) {
    if (isLoopbackOnly(e)) return { ok: false, remote: true, result: { kind: "failed", message: "That can only be changed in the desktop app." } };
    const message = isApiError(e) ? e.message || describeAddonError(e.code) : describeAddonError("");
    return { ok: false, remote: false, result: { kind: "failed", message } };
  }
}
