import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { createHttpApi } from "./api/httpApi";
import { adoptToken, localStorageTokenStore } from "./api/token";
import type { Api } from "./api/types";
import "./index.css";

const root = document.getElementById("root");
if (!root) throw new Error("#root not found");

async function pickApi(): Promise<Api> {
  // `pnpm web?mock`: run the frontend alone against in-memory data.
  if (import.meta.env.DEV && new URLSearchParams(window.location.search).has("mock")) {
    const { createMockApi } = await import("./api/mockApi");
    return createMockApi({ latencyMs: 150 });
  }
  // Take the token from #token=... (pairing QR or desktop window), keep it, and clear the address bar.
  adoptToken(localStorageTokenStore);
  return createHttpApi({ tokens: localStorageTokenStore });
}

void pickApi().then((api) =>
  createRoot(root).render(
    <StrictMode>
      <App api={api} />
    </StrictMode>,
  ),
);
