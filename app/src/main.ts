import { mount } from "svelte";
import "@fontsource-variable/geist";
import "@fontsource-variable/geist-mono";
import "./app.css";
import App from "./App.svelte";
import { createMockServices } from "./lib/services/mock";

// UI-first workflow: mocked application services with fixture data.
// The Tauri adapter replaces this once backend integration is unblocked.
const services = createMockServices("working-day");

mount(App, { target: document.getElementById("app")!, props: { services } });
