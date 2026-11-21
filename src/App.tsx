import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { Button } from "./components/Button";

export function App() {
  const [greeting, setGreeting] = useState("");

  async function greet() {
    setGreeting(await invoke<string>("greet", { name: "world" }));
  }

  return (
    <main>
      <h1>template-rust-app</h1>
      <Button onClick={greet}>Greet from Rust</Button>
      {greeting && <p>{greeting}</p>}
    </main>
  );
}
