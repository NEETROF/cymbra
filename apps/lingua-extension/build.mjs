// Build the loadable MV3 extension into dist-<target>/ for each browser variant. One
// source; the differences are confined to the manifest and, at runtime, behind the
// AnalyzerPort (Chromium: WASM in the content script; Firefox and Safari: WASM in the event
// page) and the panel surface (Side Panel API vs the in-page drawer) — selected by the
// esbuild `__TARGET__` and capability defines. The content script is a classic IIFE
// (content scripts are not modules); the popup and side panel are ES modules; the
// background is an ES-module service worker on Chromium and a classic event-page script on
// Firefox and Safari.
import { build } from "esbuild";
import { existsSync, cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const requested = process.argv.slice(2).filter((a) => !a.startsWith("-"));
const TARGETS = ["chromium", "firefox", "safari"];
const targets = requested.length ? requested : TARGETS;
for (const t of targets) {
  if (!TARGETS.includes(t)) throw new Error(`Unknown build target "${t}" — expected one of: ${TARGETS.join(", ")}.`);
}

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

// Guard: src/wasm/pkg is gitignored and only refreshed by `yarn gen:wasm`, so a build can
// bundle an engine older than the code calling it — the stats view then stops on its first
// missing method (vocabularyEstimate, #439) with nothing on screen. Every method the
// extension calls is declared on `WasmEngine` (src/analyzer/engine.ts); the wasm-bindgen
// glue's `LinguaEngine` class must provide each of them.
function assertWasmMatchesEngine() {
  const gluePath = join(root, "src/wasm/pkg/lingua_wasm.js");
  if (!existsSync(gluePath)) {
    throw new Error("src/wasm/pkg is missing — run `yarn gen:wasm` before building.");
  }
  const declared = readFileSync(join(root, "src/analyzer/engine.ts"), "utf8").match(
    /interface WasmEngine \{([\s\S]*?)\n\}/,
  )?.[1];
  if (!declared) throw new Error("`interface WasmEngine` not found in src/analyzer/engine.ts.");
  const called = [...declared.matchAll(/^ {2}(\w+)\(/gm)].map((m) => m[1]);
  const glue = readFileSync(gluePath, "utf8").match(/export class LinguaEngine \{([\s\S]*?)\n\}/)?.[1] ?? "";
  const provided = new Set([...glue.matchAll(/^ {4}(\w+)\(/gm)].map((m) => m[1]));
  const missing = called.filter((name) => !provided.has(name));
  if (missing.length > 0) {
    throw new Error(
      `src/wasm/pkg is older than the extension: LinguaEngine lacks ${missing.join(", ")}. ` +
        "Rebuild it: `yarn gen:wasm`.",
    );
  }
}
assertWasmMatchesEngine();

const baseManifest = JSON.parse(readFileSync(join(root, "manifest.json"), "utf8"));
// The version has ONE home: package.json, which release-please bumps. It is stamped onto
// each built manifest here rather than mirrored into manifest.json, because a mirror is a
// second source that drifts — and release-please's JSON updater rewrites the whole file it
// touches (it expanded every array, which the Prettier gate then refused). manifest.json
// carries no `version` at all, so there is nothing to disagree with.
const { version: VERSION } = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));

// Backend gRPC-web origin + Google OAuth client id come from the environment so a
// dogfooding/release build points at the right backend without editing source. The
// origin is also granted in the manifest's host_permissions (below), since an MV3
// fetch to it needs the host permission.
const GRPC_WEB_URL = process.env.LINGUA_GRPC_WEB_URL ?? "http://localhost:50051";
const GOOGLE_CLIENT_ID = process.env.LINGUA_GOOGLE_CLIENT_ID ?? "";
// Apple Services ID (the site's web client id). Empty hides "Continuer avec Apple".
const APPLE_CLIENT_ID = process.env.LINGUA_APPLE_CLIENT_ID ?? "";
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
  // No lateral panel on Firefox: the reader stays in the page via the injected drawer, so
  // there is no side_panel AND no sidebar_action. A sidebar_action would make Firefox
  // auto-open its native sidebar on install/reload (unwanted) and duplicate the drawer.
  delete m.side_panel;
  // Firefox requires an add-on id; it has no "sidePanel" permission.
  //
  // data_collection_permissions mirrors the privacy policy's Annex B (cymbra.app/confidentialite):
  // everything here is OPTIONAL because signing in is optional (offline/anonymous use sends
  // nothing at all — see REVIEWERS.md, "Where the add-on reaches the network"). Once signed in
  // and syncing: authenticationInfo (the Cymbra account/session), personallyIdentifyingInfo (its
  // email), websiteContent (the deck's stored source sentence — no URL, but still page text), and
  // technicalAndInteraction (daily stats, the random install id; Mozilla requires this one to be
  // optional regardless). No location/health/financial/communications/search/bookmarks data, and
  // no browsingActivity — the page address never leaves the device (Annex B). Firefox only shows
  // this consent UI from version 140; strict_min_version stays 128.0 for now (AMO currently only
  // warns, not blocks, on a missing key), so this key alone isn't yet a full compliance story for
  // Firefox 128–139 — see https://mzl.la/firefox-builtin-data-consent, "older Firefox versions".
  m.browser_specific_settings = {
    gecko: {
      id: "lingua@cymbra.app",
      strict_min_version: "128.0",
      data_collection_permissions: {
        optional: ["authenticationInfo", "personallyIdentifyingInfo", "websiteContent", "technicalAndInteraction"],
      },
    },
  };
  m.permissions = (m.permissions ?? []).filter((p) => p !== "sidePanel");
  return m;
}

/** Transform the base manifest into the Safari variant (macOS + iOS), hosted by apps/lingua-apple. */
function safariManifest(base) {
  // Safari takes the Firefox path (event-page engine, static reader, in-page drawer) — the
  // add-lingua-apple spike ran it unchanged on macOS, the iOS simulator and an iPhone.
  const m = firefoxManifest(base);
  // Safari needs no add-on id (the host app's bundle identifies it).
  delete m.browser_specific_settings;
  // iOS refuses a persistent background page; the event page is suspended and woken on demand.
  m.background = { ...m.background, persistent: false };
  // Safari does not support the identity API; Apple and Google come from the host app instead,
  // collected through the extension's native handler (add-lingua-connected-clients D6).
  m.permissions = [...m.permissions.filter((p) => p !== "identity"), "nativeMessaging"];
  return m;
}

/**
 * Build-time capabilities, injected as esbuild defines so each variant's bundle folds its
 * branches (and a grep of dist-<target>/ shows only that variant's code). Chromium runs the
 * engine in the content script and opens review in its Side Panel; Firefox and Safari run the
 * engine in the event page, review in the in-page drawer, and inject the reader statically.
 */
function capabilities(target) {
  const eventPageFamily = target !== "chromium";
  return {
    __ENGINE_IN_EVENT_PAGE__: JSON.stringify(eventPageFamily),
    __REVIEW_IN_PAGE__: JSON.stringify(eventPageFamily),
    __STATIC_READER__: JSON.stringify(eventPageFamily),
    // Safari only: Apple and Google come from the host app over native messaging.
    __NATIVE_PROVIDERS__: JSON.stringify(target === "safari"),
  };
}

const manifestFor = { chromium: structuredClone, firefox: firefoxManifest, safari: safariManifest };

const staticCopies = [
  ["src/popup/popup.html", "popup.html"],
  ["src/popup/popup.css", "popup.css"],
  ["src/sidepanel/sidepanel.html", "sidepanel.html"],
  ["src/sidepanel/sidepanel.css", "sidepanel.css"],
  ["src/stats/stats.html", "stats.html"],
  ["src/stats/stats.css", "stats.css"],
  ["src/onboarding/onboarding.html", "onboarding.html"],
  ["src/onboarding/onboarding.css", "onboarding.css"],
  ["src/account/account.html", "account.html"],
  ["src/account/account.css", "account.css"],
  ["src/styles/tokens.css", "tokens.css"],
  ["src/styles/review.css", "review.css"],
  ["src/styles/settings.css", "settings.css"],
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
    // Fold the define'd constants and drop the branches they disable, so each variant ships
    // only its own code (identifiers and whitespace stay readable; this is syntax-only).
    minifySyntax: true,
    target: ["chrome116", "firefox128"],
    logLevel: "info",
    loader: { ".css": "text" },
    define: {
      __TARGET__: JSON.stringify(target),
      ...capabilities(target),
      __GRPC_WEB_URL__: JSON.stringify(GRPC_WEB_URL),
      __GOOGLE_CLIENT_ID__: JSON.stringify(GOOGLE_CLIENT_ID),
      __APPLE_CLIENT_ID__: JSON.stringify(APPLE_CLIENT_ID),
    },
  };

  // Content script → classic IIFE (dynamic import of the wasm glue stays a runtime import).
  await build({ ...common, entryPoints: { content: join(root, "src/content.ts") }, outdir: dist, format: "iife" });

  // Background → ES-module service worker (Chromium) or classic event-page script (Firefox, Safari).
  await build({
    ...common,
    entryPoints: { background: join(root, "src/background.ts") },
    outdir: dist,
    format: target === "chromium" ? "esm" : "iife",
  });

  // Popup + side panel + stats → ES modules (loaded as <script type="module">).
  await build({
    ...common,
    entryPoints: {
      popup: join(root, "src/popup/popup.ts"),
      sidepanel: join(root, "src/sidepanel/sidepanel.ts"),
      stats: join(root, "src/stats/stats.ts"),
      onboarding: join(root, "src/onboarding/onboarding.ts"),
      account: join(root, "src/account/account.ts"),
    },
    outdir: dist,
    format: "esm",
  });

  const manifest = manifestFor[target](baseManifest);
  manifest.version = VERSION;
  // Grant the configured backend origin so the sync transport's gRPC-web fetch is
  // allowed (the server must also allow the extension origin via CYMBRA_ALLOWED_WEB_ORIGINS).
  manifest.host_permissions = [...new Set([...(manifest.host_permissions ?? []), hostPattern(GRPC_WEB_URL)])];
  // Pin the unpacked Chromium id (stable chrome.identity redirect URL); Firefox uses its
  // gecko id and Safari its host app's bundle, so neither must carry `key`.
  if (target === "chromium" && EXT_KEY) manifest.key = EXT_KEY;
  // Reader injection strategy differs by browser:
  //  - Firefox (incl. Android): a STATIC content script on every page, ALWAYS. MV3 dynamic
  //    registration (scripting.registerContentScripts) does not reliably fire on GeckoView,
  //    and browser.contentScripts.register() dies with the non-persistent event page — so the
  //    browser-level static injection is the only thing that runs on every load AND reload.
  //    The global "Surlignage activé" toggle (default on) is the off switch; the reader is
  //    entirely local (no network), so always-on is an acceptable trade for reliability.
  //  - Chromium: activeTab-first by design — no static script; scripting.registerContentScripts
  //    (which works there) powers "Toujours surligner". LINGUA_ALL_URLS=1 forces the static
  //    script for dev testing of the always-on path on Chrome.
  //  - Safari (macOS + iOS) follows Firefox: a static content script, like the event-page engine.
  if (target !== "chromium" || process.env.LINGUA_ALL_URLS === "1") {
    manifest.content_scripts = [{ matches: ["<all_urls>"], js: ["content.js"], run_at: "document_idle" }];
  }
  writeFileSync(join(dist, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  for (const [from, to] of staticCopies) cpSync(join(root, from), join(dist, to));
  // The committed icon set (tool/gen_icons.sh), except the host app's 1024 px icon.
  cpSync(join(root, "icons"), join(dist, "icons"), {
    recursive: true,
    filter: (src) => !src.endsWith("icon-1024.png"),
  });

  console.log(`Built ${target} → dist-${target}/`);
}
