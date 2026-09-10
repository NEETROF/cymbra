// Build the loadable MV3 extension into dist/. The content script is bundled as a
// classic IIFE (content scripts are not modules); the service worker and the popup
// are ES modules. CSS imported from TS is inlined as text (injected into the page and
// the closed shadow root); the popup's own stylesheets are copied as files it <link>s.
// The wasm-pack output (yarn gen:wasm) and the data pack (yarn gen:pack) are copied in.
import { build } from "esbuild";
import { cpSync, mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const dist = join(root, "dist");

rmSync(dist, { recursive: true, force: true });
mkdirSync(join(dist, "wasm"), { recursive: true });
mkdirSync(join(dist, "assets"), { recursive: true });

const common = {
  bundle: true,
  sourcemap: false,
  target: ["chrome116"],
  logLevel: "info",
  loader: { ".css": "text" },
};

// Content script → classic IIFE (dynamic import of the wasm glue stays a runtime import).
await build({
  ...common,
  entryPoints: { content: join(root, "src/content.ts") },
  outdir: dist,
  format: "iife",
});

// Service worker + popup → ES modules.
await build({
  ...common,
  entryPoints: {
    background: join(root, "src/background.ts"),
    popup: join(root, "src/popup/popup.ts"),
  },
  outdir: dist,
  format: "esm",
});

// Static assets the popup and manifest reference by name.
const copies = [
  ["manifest.json", "manifest.json"],
  ["src/popup/popup.html", "popup.html"],
  ["src/popup/popup.css", "popup.css"],
  ["src/styles/tokens.css", "tokens.css"],
  ["src/wasm/pkg/lingua_wasm.js", "wasm/lingua_wasm.js"],
  ["src/wasm/pkg/lingua_wasm_bg.wasm", "wasm/lingua_wasm_bg.wasm"],
  ["assets/pack.lingua", "assets/pack.lingua"],
];
for (const [from, to] of copies) cpSync(join(root, from), join(dist, to));

console.log("Built extension → dist/ (load unpacked)");
