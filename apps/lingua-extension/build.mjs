// Build the loadable MV3 extension into dist-<target>/ for each browser variant. One
// source; the differences are confined to the manifest and, at runtime, behind the
// AnalyzerPort (Chromium: WASM in the content script; Firefox: WASM in the event page)
// and the panel surface (Side Panel API vs sidebar_action) — selected by the esbuild
// `__TARGET__` define. The content script is a classic IIFE (content scripts are not
// modules); the popup and side panel are ES modules; the background is an ES-module
// service worker on Chromium and a classic event-page script on Firefox.
import { build } from "esbuild";
import { existsSync, cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const requested = process.argv.slice(2).filter((a) => !a.startsWith("-"));
const targets = requested.length ? requested : ["chromium", "firefox"];

// Guard: the bundled pack and the engine must share an analyzer version. The engine
// refuses a mismatched pack at RUNTIME ("pack built for analyzer X but this core is
// Y"), which reads as "the extension is broken" during dogfooding. A bump of
// lingua-core's ANALYZER_VERSION leaves the gitignored real pack (gen:pack:real) stale,
// so fail the build early with an actionable message instead. Read the core's constant
// as the source of truth; scan the pack container for its meta version (ASCII in the
// binary). CI runs gen:pack (testdata, current version) before build, so it always matches.
function coreAnalyzerVersion() {
  const src = readFileSync(join(root, "../../crates/lingua-core/src/analysis/mod.rs"), "utf8");
  return src.match(/ANALYZER_VERSION:\s*&str\s*=\s*"([^"]+)"/)?.[1] ?? null;
}
function packAnalyzerVersion(packPath) {
  // latin1 keeps the binary intact while the ASCII meta JSON stays matchable.
  return (
    readFileSync(packPath)
      .toString("latin1")
      .match(/"analyzer_version"\s*:\s*"([^"]+)"/)?.[1] ?? null
  );
}
function assertPackMatchesEngine() {
  const packPath = join(root, "assets/pack.lingua");
  if (!existsSync(packPath)) {
    throw new Error(
      "assets/pack.lingua is missing — run `yarn gen:pack` (testdata) or `yarn gen:pack:real` (full en-fr) before building.",
    );
  }
  const core = coreAnalyzerVersion();
  const pack = packAnalyzerVersion(packPath);
  if (core && pack && core !== pack) {
    throw new Error(
      `Analyzer version mismatch: assets/pack.lingua is ${pack} but lingua-core is ${core}. ` +
        "The engine refuses a mismatched pack at runtime. Rebuild the pack: " +
        "`yarn gen:pack:real` (your full en-fr pack) or `yarn gen:pack` (testdata).",
    );
  }
}
assertPackMatchesEngine();

const baseManifest = JSON.parse(readFileSync(join(root, "manifest.json"), "utf8"));

// Backend gRPC-web origin + Google OAuth client id come from the environment so a
// dogfooding/release build points at the right backend without editing source. The
// origin is also granted in the manifest's host_permissions (below), since an MV3
// fetch to it needs the host permission.
const GRPC_WEB_URL = process.env.LINGUA_GRPC_WEB_URL ?? "http://localhost:50051";
const GOOGLE_CLIENT_ID = process.env.LINGUA_GOOGLE_CLIENT_ID ?? "";
// A committed DEV public key pins the UNPACKED Chromium extension id. Without it Chromium
// assigns a random per-profile id, so chrome.identity.getRedirectURL() —
// https://<id>.chromiumapp.org/ — changes per machine and never matches the redirect URI
// registered on the Google OAuth client (redirect_uri_mismatch). This throwaway PUBLIC key
// yields the stable dev id `figfjglfdiffocldficbimecjnhnkhkh`, i.e. redirect
// https://figfjglfdiffocldficbimecjnhnkhkh.chromiumapp.org/ and origin
// chrome-extension://figfjglfdiffocldficbimecjnhnkhkh. Override with LINGUA_EXT_KEY for a
// release (or set it to "" to let the Web Store assign the id). Firefox ignores `key` — it
// pins its id via browser_specific_settings.gecko instead.
const EXT_KEY =
  process.env.LINGUA_EXT_KEY ??
  "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAmO6fyVYfGiE/aY3LTZ5RIZnwHslFqU5VEVPwehSSs7ah1g3L3LQby7Sg/UubpfrAiKzn/Y97la+j//5nLHGIR0R7+Mu4wuWJjqlb1JglazkjMgIKALmJehrPCb+0n5l+9WNerFSV3YCC76mm9XYeHlgrvrQsmAMq1hI5b264lL45akxRF3fR7QoPh/pzBVyociD4BOYCB0DsjDql8fghaH4hxoeQFwkdhcePO0I4S5KbuehMgIazk9DCh7eTVGGSUs9p0pRMYGydFkzTc9YBG3gL2OoBQ+p5lQMM5B3hNyDPni5BpJ02XW8ZhLdm009avnk3rbm1DBLWF5GcHsuUnQIDAQAB";

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
  ["src/stats/stats.html", "stats.html"],
  ["src/stats/stats.css", "stats.css"],
  ["src/onboarding/onboarding.html", "onboarding.html"],
  ["src/onboarding/onboarding.css", "onboarding.css"],
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

  // Popup + side panel + stats → ES modules (loaded as <script type="module">).
  await build({
    ...common,
    entryPoints: {
      popup: join(root, "src/popup/popup.ts"),
      sidepanel: join(root, "src/sidepanel/sidepanel.ts"),
      stats: join(root, "src/stats/stats.ts"),
      onboarding: join(root, "src/onboarding/onboarding.ts"),
    },
    outdir: dist,
    format: "esm",
  });

  const manifest = target === "firefox" ? firefoxManifest(baseManifest) : structuredClone(baseManifest);
  // Grant the configured backend origin so the sync transport's gRPC-web fetch is
  // allowed (the server must also allow the extension origin via CYMBRA_ALLOWED_WEB_ORIGINS).
  manifest.host_permissions = [...new Set([...(manifest.host_permissions ?? []), hostPattern(GRPC_WEB_URL)])];
  // Pin the unpacked Chromium id (stable chrome.identity redirect URL); Firefox uses its
  // gecko id instead, so it must never carry `key`.
  if (target !== "firefox" && EXT_KEY) manifest.key = EXT_KEY;
  writeFileSync(join(dist, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  for (const [from, to] of staticCopies) cpSync(join(root, from), join(dist, to));

  console.log(`Built ${target} → dist-${target}/`);
}
