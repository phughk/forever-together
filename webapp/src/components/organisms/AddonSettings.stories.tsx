import type { Meta, StoryObj } from "@storybook/react-vite";
import { useMemo, useState } from "react";
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import { installAddonFlow, type AddonResult } from "../../api/addonFlow";
import { MAC_ADDONS, WIN_ADDONS, makeAddon } from "../../api/fixtures";
import { createMockApi } from "../../api/mockApi";
import type { AddonInfo } from "../../api/types";
import { AddonSettings } from "./AddonSettings";

const meta = {
  title: "Organisms/AddonSettings (Addon)",
  component: AddonSettings,
  args: { info: makeAddon("not_installed"), onInstall: fn(), onSaveFolder: fn() },
} satisfies Meta<typeof AddonSettings>;
export default meta;
type Story = StoryObj<typeof meta>;

// One story per row of the state table in the addon installer brief.
export const NoFolderFound: Story = { args: { info: makeAddon("no_folder") } };
export const FolderInvalid: Story = { args: { info: makeAddon("folder_invalid") } };
export const NotInstalled: Story = { args: { info: makeAddon("not_installed") } };
export const UpToDate: Story = { args: { info: makeAddon("up_to_date") } };
export const UpdateAvailable: Story = { args: { info: makeAddon("update_available") } };
/** Same version, different files: both versions show so "Reinstall" is understood. */
export const SameVersionFilesDiffer: Story = { args: { info: makeAddon("files_differ") } };
export const InstalledNewer: Story = { args: { info: makeAddon("installed_newer") } };
export const VersionUnknown: Story = { args: { info: makeAddon("unknown_version") } };
export const Success: Story = {
  args: { info: makeAddon("up_to_date"), result: { kind: "installed", version: "0.2.0" } },
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await expect(c.getByText("Installed")).toBeVisible();
    await expect(c.getByText("Now:")).toBeVisible();
    await expect(c.getByRole("button", { name: /\/reload/ })).toBeVisible();
  },
};
export const Failed: Story = {
  args: {
    info: makeAddon("update_available"),
    result: { kind: "failed", message: `This app is not allowed to write to ${MAC_ADDONS}. Close the game and try again, or pick another folder.` },
  },
};
export const FailedOnWindows: Story = {
  args: {
    info: makeAddon("update_available", { folder: WIN_ADDONS, candidates: [WIN_ADDONS] }),
    result: { kind: "failed", message: `A file in ${WIN_ADDONS}\\ForeverTogether is in use. Close the game and try again.` },
  },
};

// Folder choice.
/** One valid folder was detected: nothing is written until the install click. */
export const DetectedFolderConfirm: Story = { args: { info: makeAddon("not_installed", { folder_source: "detected" }) } };
export const DetectedFolderWithUpdate: Story = { args: { info: makeAddon("update_available", { folder_source: "detected" }) } };
export const SeveralCandidates: Story = {
  args: { info: makeAddon("no_folder", { candidates: [MAC_ADDONS, "/Applications/World of Warcraft/_forever_beta_/Interface/AddOns"] }) },
};
export const PasteFieldOpen: Story = { args: { info: makeAddon("no_folder"), defaultFolderOpen: true } };
export const PasteFieldWithError: Story = {
  args: {
    info: makeAddon("folder_invalid"),
    defaultFolderOpen: true,
    folderError: "/Applications/World of Warcraft/_classic_/Interface is not a World of Warcraft AddOns folder. It should end in Interface/AddOns.",
  },
};
export const ChangeFolderOpen: Story = { args: { info: makeAddon("up_to_date"), defaultFolderOpen: true } };

// Other conditions.
export const Installing: Story = { args: { info: makeAddon("update_available"), busy: true } };
export const Loading: Story = { args: { info: null } };
/** A phone: the buttons are disabled with a lock icon. */
export const ViewedOnPhone: Story = { args: { info: makeAddon("update_available"), remote: true } };
export const ViewedOnPhoneNoData: Story = { args: { info: null, remote: true } };
export const LongPathOnPhoneWidth: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { info: makeAddon("not_installed", { folder_source: "detected", folder: WIN_ADDONS, candidates: [WIN_ADDONS] }) },
};

// Behaviour against the in-memory API, with the same flow the Settings page uses.
function Live({ info, error }: { info: AddonInfo; error?: { status: number; code: string; message: string } }) {
  const api = useMemo(() => createMockApi({ addon: info, addonInstallError: error }), [info, error]);
  const [current, setCurrent] = useState(info);
  const [result, setResult] = useState<AddonResult | null>(null);
  const [busy, setBusy] = useState(false);
  return (
    <>
      <AddonSettings
        info={current}
        result={result}
        busy={busy}
        onInstall={async () => {
          setBusy(true);
          setResult(null);
          const out = await installAddonFlow(api, current);
          if (out.ok) setCurrent(out.info);
          setResult(out.result);
          setBusy(false);
          (window as unknown as { __addonLog?: string[] }).__addonLog = api.addonLog;
        }}
        onSaveFolder={async (f) => setCurrent(await api.setAddonFolder(f))}
      />
    </>
  );
}

export const InstallShowsInstalledAndReload: StoryObj = {
  render: () => <Live info={makeAddon("update_available")} />,
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await userEvent.click(c.getByRole("button", { name: "Update addon" }));
    await waitFor(() => expect(c.getByText("Installed")).toBeVisible());
    await expect(c.getByText("Now:")).toBeVisible();
    await expect(c.getByText(/In the game, type/)).toBeVisible();
    await expect(c.getByRole("button", { name: /\/reload/ })).toBeVisible();
    await expect(c.queryByRole("button", { name: "Update addon" })).toBeNull();
  },
};

export const DetectedFolderIsSavedBeforeInstall: StoryObj = {
  render: () => <Live info={makeAddon("not_installed", { folder_source: "detected" })} />,
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await expect(c.getByText(/Use this folder\?/)).toBeVisible();
    await userEvent.click(c.getByRole("button", { name: "Install addon" }));
    await waitFor(() => expect(c.getByText("Installed")).toBeVisible());
    await expect((window as unknown as { __addonLog: string[] }).__addonLog).toEqual([`folder:${MAC_ADDONS}`, "install"]);
  },
};

export const FailedInstallShowsMessageAndTryAgain: StoryObj = {
  render: () => (
    <Live
      info={makeAddon("not_installed")}
      error={{ status: 403, code: "addon_permission_denied", message: `This app is not allowed to write to ${MAC_ADDONS}.` }}
    />
  ),
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await userEvent.click(c.getByRole("button", { name: "Install addon" }));
    await waitFor(() => expect(c.getByText("Could not install")).toBeVisible());
    await expect(c.getByText(/not allowed to write to/)).toBeVisible();
    await expect(c.getByRole("button", { name: "Try again" })).toBeVisible();
    await expect(c.getByRole("button", { name: "Change folder" })).toBeVisible();
  },
};

export const UseAFolderOpensPasteField: Story = {
  args: { info: makeAddon("no_folder") },
  play: async ({ canvasElement, args }) => {
    const c = within(canvasElement);
    await userEvent.click(c.getByRole("button", { name: "Use a folder" }));
    await userEvent.type(c.getByLabelText("AddOns folder"), MAC_ADDONS);
    await userEvent.click(c.getByRole("button", { name: "Use folder" }));
    await expect(args.onSaveFolder).toHaveBeenCalledWith(MAC_ADDONS);
  },
};
