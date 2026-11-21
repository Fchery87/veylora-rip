// Lists what the built frontend redistributes: the npm packages compiled into its JavaScript and
// every other file it emits (fonts, CSS). It builds with source maps into a temporary directory and
// reads the package of each source module from the map, so the list is derived from the real bundle,
// never kept by hand. Prints JSON on stdout.
//
// usage: node scripts/frontend-bundle.mjs <app-dir>
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readdirSync, readFileSync, rmSync, statSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative, resolve, sep } from "node:path";

const appDir = process.argv[2] && resolve(process.argv[2]);
if (!appDir) throw new Error("usage: node frontend-bundle.mjs <app-dir>");
const out = mkdtempSync(join(tmpdir(), "veylora-frontend-"));
try {
  const vp = join(appDir, "node_modules", ".bin", process.platform === "win32" ? "vp.cmd" : "vp");
  execFileSync(vp, ["build", "--sourcemap", "--outDir", out], {
    cwd: appDir,
    stdio: ["ignore", "ignore", "inherit"],
    shell: process.platform === "win32",
  });

  const walk = (dir) =>
    readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
      e.isDirectory() ? walk(join(dir, e.name)) : [join(dir, e.name)],
    );
  const files = walk(out).sort();
  const sha = (p) => createHash("sha256").update(readFileSync(p)).digest("hex");

  const packages = new Map();
  for (const f of files.filter((p) => p.endsWith(".map"))) {
    const map = JSON.parse(readFileSync(f, "utf8"));
    for (const source of map.sources) {
      const path = source.split(sep).join("/").replace(/\\/g, "/");
      if (!path.includes("node_modules")) continue;
      // pnpm layout: node_modules/.pnpm/<store-dir>/node_modules/<name>/...
      const m = path.match(/node_modules\/\.pnpm\/[^/]+\/node_modules\/((?:@[^/]+\/)?[^/]+)\//);
      if (!m) throw new Error(`module outside the pnpm layout: ${source}`);
      const name = m[1];
      const root = path.slice(0, path.indexOf(`/node_modules/${name}/`) + `/node_modules/${name}`.length);
      const dir = join(appDir, root.replace(/^.*?(?=node_modules\/\.pnpm)/, ""));
      const pkgFile = join(dir, "package.json");
      if (!existsSync(pkgFile)) throw new Error(`no package.json for ${name} at ${dir}`);
      const pkg = JSON.parse(readFileSync(pkgFile, "utf8"));
      packages.set(`${pkg.name}@${pkg.version}`, {
        name: pkg.name,
        version: pkg.version,
        license: pkg.license ?? null,
        dir: relative(appDir, dir).split(sep).join("/"),
      });
    }
  }

  const emitted = files
    .filter((p) => !p.endsWith(".map"))
    .map((p) => ({
      file: relative(out, p).split(sep).join("/"),
      bytes: statSync(p).size,
      sha256: sha(p),
    }));
  process.stdout.write(
    JSON.stringify({ packages: [...packages.values()].sort((a, b) => a.name.localeCompare(b.name)), emitted }, null, 2),
  );
} finally {
  rmSync(out, { recursive: true, force: true });
}
