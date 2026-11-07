import type { AppServices, CapabilityState } from "./types";

export type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

/** True inside the Tauri webview. Browsers and tests never have the internals object. */
export function isTauriRuntime(host: object = globalThis): boolean {
  return "__TAURI_INTERNALS__" in host;
}

/**
 * Thin Tauri adapter. Only `get_capabilities` is connected to the Rust shell so
 * far; every other service stays on the fallback (mocked fixture data) until the
 * matching Rust command exists. It owns no behavior of its own (ADR 0001).
 */
export function createTauriServices(invoke: Invoke, fallback: AppServices): AppServices {
  return {
    ...fallback,
    getCapabilities: () => invoke<CapabilityState>("get_capabilities"),
  };
}
