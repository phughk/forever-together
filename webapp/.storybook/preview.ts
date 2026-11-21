import type { Preview } from "@storybook/react-vite";
import "../src/index.css";

// The app is dark only: the canvas uses the app's own --bg (set on body by index.css), not a light Storybook backdrop.
const preview: Preview = {
  parameters: {
    backgrounds: { options: { app: { name: "App", value: "#08041a" } } },
  },
  initialGlobals: { backgrounds: { value: "app" } },
};

export default preview;
