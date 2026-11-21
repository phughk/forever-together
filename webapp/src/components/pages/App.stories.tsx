import type { Meta, StoryObj } from "@storybook/react-vite";
import { useMemo } from "react";
import { App } from "../../App";
import { NOW, makeState, searchingFor } from "../../api/fixtures";
import { createMockApi, type MockApiOptions } from "../../api/mockApi";
import { NotPairedPage } from "./NotPairedPage";

function Mocked(opts: MockApiOptions) {
  const api = useMemo(() => createMockApi(opts), []);
  return <App api={api} />;
}

const meta = { title: "Pages/App (mock API)", component: Mocked, parameters: { layout: "fullscreen" } } satisfies Meta<typeof Mocked>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Fully wired against the in-memory API. Paste `WFT1:x` to read a code, `WFT2:x` for an unknown version. */
export const InteractiveFirstRun: Story = { args: { initial: makeState({ character: null }), latencyMs: 200 } };
export const InteractiveWithCharacter: Story = { args: { initial: makeState(), latencyMs: 200 } };
export const InteractiveSearching: Story = { args: { initial: makeState({ search: searchingFor(30) }) } };
export const InteractiveOnPhone: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { initial: makeState({ search: searchingFor(30), settings: { lan_mode: true, known_peers_size: 100, trusted_peers: [] } }), remote: true },
};

export const NotPaired: StoryObj = { render: () => <NotPairedPage /> };
export const NotPairedViaStream: Story = { args: { initial: makeState({ serverNow: NOW }), status: "unauthorized" } };
