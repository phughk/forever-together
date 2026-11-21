import type { Meta, StoryObj } from "@storybook/react-vite";
import { useMemo } from "react";
import { App } from "../../App";
import { expect, userEvent, within } from "storybook/test";
import { MAC_ADDONS, NOW, makeAddon, makeDiagnosis, makeParty, makeState, searchingFor } from "../../api/fixtures";
import { createMockApi, type MockApiOptions } from "../../api/mockApi";
import { NotPairedPage } from "./NotPairedPage";

function Mocked(opts: MockApiOptions) {
  const api = useMemo(() => createMockApi(opts), []);
  return <App api={api} />;
}

const meta = { title: "Pages/App (mock API)", component: Mocked, parameters: { layout: "fullscreen" } } satisfies Meta<typeof Mocked>;
export default meta;
type Story = StoryObj<typeof meta>;

const phone = { globals: { viewport: { value: "mobile1", isRotated: false } } } as const;

/** Fully wired against the in-memory API. Paste `WFT1:x` to read a code, `WFT2:x` for an unknown version. */
export const InteractiveFirstRun: Story = { args: { initial: makeState({ character: null }), latencyMs: 200 } };
export const InteractiveFirstRunOnPhone: Story = { ...phone, args: { initial: makeState({ character: null }), latencyMs: 200 } };

/** Idle: tap a dungeon to search. */
export const InteractiveIdle: Story = { args: { initial: makeState(), latencyMs: 200 } };
export const InteractiveIdleOnPhone: Story = { ...phone, args: { initial: makeState(), latencyMs: 200 } };

/** Searching, with Stop. */
export const InteractiveSearching: Story = { args: { initial: makeState({ search: searchingFor(30) }) } };
export const InteractiveSearchingOnPhone: Story = { ...phone, args: { initial: makeState({ search: searchingFor(30) }) } };

/** The ended card: "Search again" restarts the same dungeons, "Change dungeon" returns to the list. */
export const InteractiveEnded: Story = {
  args: {
    initial: makeState({
      search: searchingFor(600, { status: "limit_reached", stage: 4, diagnosis: makeDiagnosis("no_tank", { ended: true }) }),
    }),
    latencyMs: 200,
  },
};

/** A party was found: Accept, Decline. */
export const InteractivePartyFound: Story = { args: { initial: makeState({ search: searchingFor(75), party: makeParty() }), latencyMs: 200 } };
export const InteractivePartyFoundOnPhone: Story = { ...phone, args: { initial: makeState({ search: searchingFor(75), party: makeParty() }), latencyMs: 200 } };

/** A phone that opened the app: the settings that only the desktop may change are locked. */
export const InteractiveOnPhone: Story = {
  ...phone,
  args: {
    initial: makeState({ search: searchingFor(30), settings: { lan_mode: true, lan_address: null, known_peers_size: 100, trusted_peers: [] } }),
    remote: true,
  },
};

const openSettings = async (canvasElement: HTMLElement) => {
  const c = within(canvasElement);
  await userEvent.click(await c.findByRole("button", { name: "Settings" }));
  return c;
};

/** Settings, addon update: one click installs and shows the /reload line. */
export const InteractiveAddonUpdate: Story = {
  args: { initial: makeState(), addon: makeAddon("update_available"), latencyMs: 100 },
  play: async ({ canvasElement }) => {
    const c = await openSettings(canvasElement);
    await userEvent.click(await c.findByRole("button", { name: "Update addon" }));
    await expect(await c.findByText("Installed")).toBeVisible();
    await expect(c.getByRole("button", { name: /\/reload/ })).toBeVisible();
  },
};

/** A detected folder is saved by the same click that installs. */
export const InteractiveAddonDetectedFolder: Story = {
  args: { initial: makeState(), addon: makeAddon("not_installed", { folder_source: "detected" }), latencyMs: 100 },
  play: async ({ canvasElement }) => {
    const c = await openSettings(canvasElement);
    await expect(await c.findByText(/Use this folder\?/)).toBeVisible();
    await userEvent.click(c.getByRole("button", { name: "Install addon" }));
    await expect(await c.findByText("Installed")).toBeVisible();
  },
};

/** The server refuses to write: the message names the folder, "Try again" and "Change folder" follow. */
export const InteractiveAddonInstallFails: Story = {
  args: {
    initial: makeState(),
    addon: makeAddon("not_installed"),
    addonInstallError: { status: 403, code: "addon_permission_denied", message: `This app is not allowed to write to ${MAC_ADDONS}. Close the game and try again, or pick another folder.` },
  },
  play: async ({ canvasElement }) => {
    const c = await openSettings(canvasElement);
    await userEvent.click(await c.findByRole("button", { name: "Install addon" }));
    await expect(await c.findByText("Could not install")).toBeVisible();
    await expect(c.getByRole("button", { name: "Try again" })).toBeVisible();
  },
};

/** No folder found: paste one the server accepts, then install. */
export const InteractiveAddonNoFolder: Story = {
  args: { initial: makeState(), addon: makeAddon("no_folder"), latencyMs: 100 },
  play: async ({ canvasElement }) => {
    const c = await openSettings(canvasElement);
    await userEvent.click(await c.findByRole("button", { name: "Use a folder" }));
    await userEvent.type(c.getByLabelText("AddOns folder"), "/wrong/place");
    await userEvent.click(c.getByRole("button", { name: "Use folder" }));
    await expect(await c.findByText(/not a World of Warcraft AddOns folder/)).toBeVisible();
    await userEvent.clear(c.getByLabelText("AddOns folder"));
    await userEvent.type(c.getByLabelText("AddOns folder"), MAC_ADDONS);
    await userEvent.click(c.getByRole("button", { name: "Use folder" }));
    await userEvent.click(await c.findByRole("button", { name: "Install addon" }));
    await expect(await c.findByText("Installed")).toBeVisible();
  },
};

/** On a phone the addon call is refused with 403: the card is locked. */
export const InteractiveAddonOnPhone: Story = {
  ...phone,
  args: { initial: makeState(), remote: true },
  play: async ({ canvasElement }) => {
    const c = await openSettings(canvasElement);
    await expect(await c.findByRole("button", { name: "Install addon" })).toBeDisabled();
  },
};

export const NotPaired: StoryObj = { render: () => <NotPairedPage /> };
export const NotPairedOnPhone: StoryObj = { ...phone, render: () => <NotPairedPage /> };
export const NotPairedViaStream: Story = { args: { initial: makeState({ serverNow: NOW }), status: "unauthorized" } };
