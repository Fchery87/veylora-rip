import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vite-plus/test";

// The dev server and these tests accept a file that is not valid UTF-8, but the production build
// refuses it, so it would only show up when packaging. Catch it here instead.
const root = join(import.meta.dirname, "..");
const TEXT = /\.(svelte|ts|js|mjs|css|html|json)$/;

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? files(path) : TEXT.test(name) ? [path] : [];
  });
}

describe("source encoding", () => {
  it("keeps every frontend source file valid UTF-8", () => {
    const decoder = new TextDecoder("utf-8", { fatal: true });
    const bad = [...files(join(root, "src")), ...files(join(root, "tests"))].filter((path) => {
      try {
        decoder.decode(readFileSync(path));
        return false;
      } catch {
        return true;
      }
    });
    expect(bad).toEqual([]);
  });
});
