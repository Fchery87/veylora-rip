import { mount } from "svelte";
import "@fontsource-variable/geist";
import "@fontsource-variable/geist-mono";
import "./app.css";
import App from "./App.svelte";
import { createMockServices } from "./lib/services/mock";
import { createTauriServices, isTauriRuntime } from "./lib/services/tauri";

// UI-first workflow: mocked application services with fixture data by default.
// Inside the Tauri shell, capabilities come from Rust; the rest stays mocked
// until the matching Rust commands exist.
async function createServices() {
  const mocks = createMockServices("working-day");
  if (!isTauriRuntime()) return mocks;
  const { invoke } = await import("@tauri-apps/api/core");
  return createTauriServices(invoke, mocks);
}

mount(App, {
  target: document.getElementById("app")!,
  props: { services: await createServices() },
});
