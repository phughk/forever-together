import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Set by `tauri dev` when targeting a physical mobile device.
const host = process.env.TAURI_DEV_HOST;
// Optional: `WOWFT_API=http://127.0.0.1:<port> pnpm dev` forwards /api to a running desktop app.
// Without it, open http://localhost:1420/?mock to run against in-memory data.
const api = process.env.WOWFT_API;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    proxy: api ? { "/api": { target: api, changeOrigin: false } } : undefined,
    watch: { ignored: ["**/app/**"] },
  },
});
