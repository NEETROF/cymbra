// Build the loadable MV3 extension into dist-<target>/ for each browser variant. One
// source; the differences are confined to the manifest and, at runtime, behind the
// AnalyzerPort (Chromium: WASM in the content script; Firefox: WASM in the event page)
// and the panel surface (Side Panel API vs sidebar_action) — selected by the esbuild
// `__TARGET__` define. The content script is a classic IIFE (content scripts are not
// modules); the popup and side panel are ES modules; the background is an ES-module
// service worker on Chromium and a classic event-page script on Firefox.
import { build } from "esbuild";
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const requested = process.argv.slice(2).filter((a) => !a.startsWith("-"));
const targets = requested.length ? requested : ["chromium", "firefox"];

const baseManifest = JSON.parse(readFileSync(join(root, "manifest.json"), "utf8"));

// Backend gRPC-web origin + Google OAuth client id come from the environment so a
// dogfooding/release build points at the right backend without editing source. The
// origin is also granted in the manifest's host_permissions (below), since an MV3
// fetch to it needs the host permission.
const GRPC_WEB_URL = process.env.LINGUA_GRPC_WEB_URL ?? "http://localhost:50051";
const GOOGLE_CLIENT_ID = process.env.LINGUA_GOOGLE_CLIENT_ID ?? "";

/** `https://host/*` match pattern for the backend origin (host_permissions). */
function hostPattern(url) {
  const u = new URL(url);
  return `${u.protocol}//${u.host}/*`;
}

/** Transform the (Chromium) base manifest into the Firefox variant. */
function firefoxManifest(base) {
  const m = structuredClone(base);
  delete m.minimum_chrome_version;
  // Firefox MV3 background is an event page (scripts), not a service worker.
  m.background = { scripts: ["background.js"] };
  // No Side Panel API on Firefox; the same page is a sidebar.
  delete m.side_panel;
  m.sidebar_action = { default_panel: "sidepanel.html", default_title: "Cymbra Lingua" };
  // Firefox requires an add-on id; it has no "sidePanel" permission.
  m.browser_specific_settings = { gecko: { id: "lingua@cymbra.app", strict_min_version: "128.0" } };
  m.permissions = (m.permissions ?? []).filter((p) => p !== "sidePanel");
  return m;
}

const staticCopies = [
  ["src/popup/popup.html", "popup.html"],
  ["src/popup/popup.css", "popup.css"],
  ["src/sidepanel/sidepanel.html", "sidepanel.html"],
  ["src/sidepanel/sidepanel.css", "sidepanel.css"],
  ["src/styles/tokens.css", "tokens.css"],
  ["src/styles/review.css", "review.css"],
  ["src/wasm/pkg/lingua_wasm.js", "wasm/lingua_wasm.js"],
  ["src/wasm/pkg/lingua_wasm_bg.wasm", "wasm/lingua_wasm_bg.wasm"],
  ["assets/pack.lingua", "assets/pack.lingua"],
];

for (const target of targets) {
  const dist = join(root, `dist-${target}`);
  rmSync(dist, { recursive: true, force: true });
  mkdirSync(join(dist, "wasm"), { recursive: true });
  mkdirSync(join(dist, "assets"), { recursive: true });

  const common = {
    bundle: true,
    sourcemap: false,
    target: ["chrome116", "firefox128"],
    logLevel: "info",
    loader: { ".css": "text" },
    define: {
      __TARGET__: JSON.stringify(target),
      __GRPC_WEB_URL__: JSON.stringify(GRPC_WEB_URL),
      __GOOGLE_CLIENT_ID__: JSON.stringify(GOOGLE_CLIENT_ID),
    },
  };

  // Content script → classic IIFE (dynamic import of the wasm glue stays a runtime import).
  await build({ ...common, entryPoints: { content: join(root, "src/content.ts") }, outdir: dist, format: "iife" });

  // Background → ES-module service worker (Chromium) or classic event-page script (Firefox).
  await build({
    ...common,
    entryPoints: { background: join(root, "src/background.ts") },
    outdir: dist,
    format: target === "firefox" ? "iife" : "esm",
  });

  // Popup + side panel → ES modules (loaded as <script type="module">).
  await build({
    ...common,
    entryPoints: { popup: join(root, "src/popup/popup.ts"), sidepanel: join(root, "src/sidepanel/sidepanel.ts") },
    outdir: dist,
    format: "esm",
  });

  const manifest = target === "firefox" ? firefoxManifest(baseManifest) : structuredClone(baseManifest);
  // Grant the configured backend origin so the sync transport's gRPC-web fetch is
  // allowed (the server must also allow the extension origin via CYMBRA_ALLOWED_WEB_ORIGINS).
  manifest.host_permissions = [...new Set([...(manifest.host_permissions ?? []), hostPattern(GRPC_WEB_URL)])];
  writeFileSync(join(dist, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  for (const [from, to] of staticCopies) cpSync(join(root, from), join(dist, to));

  console.log(`Built ${target} → dist-${target}/`);
}
