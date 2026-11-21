import type { Meta, StoryObj } from "@storybook/react-vite";
import { useMemo } from "react";
import { App } from "../../App";
import { NOW, makeDiagnosis, makeParty, makeState, searchingFor } from "../../api/fixtures";
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

export const NotPaired: StoryObj = { render: () => <NotPairedPage /> };
export const NotPairedOnPhone: StoryObj = { ...phone, render: () => <NotPairedPage /> };
export const NotPairedViaStream: Story = { args: { initial: makeState({ serverNow: NOW }), status: "unauthorized" } };
