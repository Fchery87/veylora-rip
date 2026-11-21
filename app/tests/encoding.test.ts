/// <reference types="vite-plus/client" />
import { describe, expect, it } from "vite-plus/test";

// The dev server and these tests accept a file that is not valid UTF-8, but the production build
// refuses it, so it would only show up when packaging. Reading a file as text turns an invalid
// byte into U+FFFD, so that character is the sign of a damaged file.
const sources = import.meta.glob(["../src/**/*.{svelte,ts,js,css}", "./**/*.ts"], {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

describe("source encoding", () => {
  it("keeps every frontend source file valid UTF-8", () => {
    expect(Object.keys(sources).length).toBeGreaterThan(20);
    const bad = Object.entries(sources)
      .filter(([, text]) => text.includes("\uFFFD"))
      .map(([path]) => path);
    expect(bad).toEqual([]);
  });
});
