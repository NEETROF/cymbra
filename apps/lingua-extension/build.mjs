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
import { dirname, join, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { engineFiles, engineProblems } from "./tool/engine_pin.mjs";
import { bundledCatalogue, readCatalogue } from "./tool/model-catalogue.mjs";
import { LOCALES_DIR, readLocales } from "./tool/locales.mjs";
import { buildManifest } from "./tool/manifests.mjs";
import { assertPacksMatchEngine, packFile, shippedNatives, shippedPairs } from "./tool/packs.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const requested = process.argv.slice(2).filter((a) => !a.startsWith("-"));
const TARGETS = ["chromium", "firefox", "safari"];
const targets = requested.length ? requested : TARGETS;
for (const t of targets) {
  if (!TARGETS.includes(t)) throw new Error(`Unknown build target "${t}" — expected one of: ${TARGETS.join(", ")}.`);
}

// The pairs every package ships (packs.json, generalise-lingua-pack-build); the first gives the
// default studied language, the engine's first pack.
const PAIRS = shippedPairs();

// Guard: every listed pack is its pair's — its studied and its native language — and was built for
// its language's analyser version (tool/packs.mjs).
assertPacksMatchEngine({ appDir: root, pairs: PAIRS });

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
  // The glue's functions the extension calls beside the engine's methods (`WasmModule`).
  const glueText = readFileSync(gluePath, "utf8");
  for (const name of ["reprofileBackup"]) {
    if (!new RegExp(`^export function ${name}\\(`, "m").test(glueText)) missing.push(name);
  }
  if (missing.length > 0) {
    throw new Error(
      `src/wasm/pkg is older than the extension: LinguaEngine lacks ${missing.join(", ")}. ` +
        "Rebuild it: `yarn gen:wasm`.",
    );
  }
}
assertWasmMatchesEngine();

const baseManifest = JSON.parse(readFileSync(join(root, "manifest.json"), "utf8"));
// The committed `_locales` (localise-lingua-manifest D1): every language's messages, of which a
// package carries the shipped natives' once a pair glossed in another language than French ships
// (tool/manifests.mjs decides: fr and en since es-en ships, enable-lingua-english-speakers, and es
// since en-es ships, enable-lingua-spanish-speakers).
const MESSAGES = readLocales(root);
// The version has ONE home: package.json, which release-please bumps. It is stamped onto
// each built manifest here rather than mirrored into manifest.json, because a mirror is a
// second source that drifts — and release-please's JSON updater rewrites the whole file it
// touches (it expanded every array, which the Prettier gate then refused). manifest.json
// carries no `version` at all, so there is nothing to disagree with.
const { version: VERSION } = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));

// Backend gRPC-web origin + Google OAuth client id come from the environment so a
// dogfooding/release build points at the right backend without editing source. The
// origin is also granted in the manifest's host_permissions (tool/manifests.mjs), since an
// MV3 fetch to it needs the host permission.
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

// The translation engine (add-lingua-translation-delivery D1) is PART of every package — Chromium,
// Firefox and, since add-lingua-translation-safari, Safari: the stores count WebAssembly loaded from anywhere else as remote code, so only the
// model — data — is ever downloaded, and only once the reader turns « Traduction étendue » on.
// The engine comes from the lingua-engine-build workflow (`yarn fetch:engine`) or from source
// (`tool/build_engine.sh`), into engine/ unless LINGUA_TRANSLATION_ENGINE says otherwise, and a
// build refuses anything but the pinned bytes (engine-pin.json).
const ENGINE_DIR = process.env.LINGUA_TRANSLATION_ENGINE || join(root, "engine");
const ENGINE_FILES = engineFiles();

/** Where a variant hosts the translation engine. Never on a thread that paints. */
function translationHost(target) {
  // Safari's event page hosts it as Firefox's does (measured on iPhone, iPad and Mac:
  // add-lingua-translation-safari D1).
  return target === "chromium" ? "offscreen" : "event-page";
}

if (targets.some((t) => translationHost(t) !== "none")) {
  const problems = engineProblems(ENGINE_DIR);
  if (problems.length > 0) {
    throw new Error(
      `Every package carries the translation engine, and ${ENGINE_DIR} does not hold ` +
        `the pinned one: ${problems.join("; ")}. Fetch it with \`yarn fetch:engine\` (or build it on ` +
        "Linux with tool/build_engine.sh) — see apps/lingua-extension/TRANSLATION.md.",
    );
  }
}

// The models' catalogue (D3, generalise-lingua-translation-catalogue): where each file is, its size,
// and the sha256 its bytes must have, and the route of models for each studied language. It is
// bundled, so the reviewed package decides what is accepted — the host only serves bytes.
// LINGUA_MODEL_BASE_URL points a DEVELOPMENT build at another host serving the same paths (a local
// server, before the real one exists); check_variants.mjs refuses a package built that way.
const MODEL_CATALOGUE = readCatalogue();
const MODEL_BASE_URL = process.env.LINGUA_MODEL_BASE_URL ?? "";
if (MODEL_BASE_URL) console.warn(`[build] the models are fetched from ${MODEL_BASE_URL}: a development build.`);

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
    // Chromium only: the book reader's sections are served by the background service worker,
    // not from blob: URLs Chrome may isolate in another process (src/reader/section-server.ts).
    __SECTIONS_FROM_WORKER__: JSON.stringify(target === "chromium"),
    __TRANSLATION_HOST__: JSON.stringify(translationHost(target)),
  };
}

// The book reader's renderer, foliate-js, is vendored at a pinned commit (vendor/VENDOR.md) and
// imported as `foliate-js/…`. Its zip reader is the npm package foliate builds its own copy
// from, taken at the entry foliate bundles — `lib/zip-core.js`: no worker, no WebAssembly, the
// browser's DecompressionStream inflates. (The package's `exports` map would send that path to
// its WebAssembly variant, hence the explicit file.)
const ALIASES = {
  "foliate-js": "./vendor/foliate-js",
  "@zip.js/zip.js": "./node_modules/@zip.js/zip.js/lib/zip-core.js",
};

// view.js loads the other formats and features on demand — PDF, MOBI, FB2, CBZ, search, text
// to speech. The reader reads EPUB only, and none of those is vendored: each import resolves to
// a module that refuses, so esbuild bundles none of them.
const FOLIATE_UNSHIPPED = /^\.\/(pdf|mobi|fb2|comic-book|search|tts|vendor\/fflate)\.js$/;
const foliateEpubOnly = {
  name: "foliate-epub-only",
  setup(b) {
    b.onResolve({ filter: FOLIATE_UNSHIPPED }, (args) =>
      args.importer.includes(`${sep}vendor${sep}foliate-js${sep}`)
        ? { path: args.path, namespace: "foliate-unshipped" }
        : undefined,
    );
    b.onLoad({ filter: /.*/, namespace: "foliate-unshipped" }, (args) => ({
      contents: `throw new Error(${JSON.stringify(`Cymbra Lingua reads EPUB only: ${args.path} is not bundled`)});`,
      loader: "js",
    }));
  },
};

const staticCopies = [
  ["src/popup/popup.html", "popup.html"],
  ["src/popup/popup.css", "popup.css"],
  ["src/sidepanel/sidepanel.html", "sidepanel.html"],
  ["src/sidepanel/sidepanel.css", "sidepanel.css"],
  ["src/stats/stats.html", "stats.html"],
  ["src/stats/stats.css", "stats.css"],
  ["src/stats/stats-page.css", "stats-page.css"],
  ["src/onboarding/onboarding.html", "onboarding.html"],
  ["src/onboarding/onboarding.css", "onboarding.css"],
  ["src/account/account.html", "account.html"],
  ["src/account/account.css", "account.css"],
  ["src/reader/reader.html", "reader.html"],
  ["src/reader/reader.css", "reader.css"],
  ["src/styles/tokens.css", "tokens.css"],
  ["src/styles/review.css", "review.css"],
  ["src/styles/settings.css", "settings.css"],
  ["src/wasm/pkg/lingua_wasm.js", "wasm/lingua_wasm.js"],
  ["src/wasm/pkg/lingua_wasm_bg.wasm", "wasm/lingua_wasm_bg.wasm"],
  ...PAIRS.map((pair) => [packFile(pair), packFile(pair)]),
];

for (const target of targets) {
  const dist = join(root, `dist-${target}`);
  rmSync(dist, { recursive: true, force: true });
  mkdirSync(join(dist, "wasm"), { recursive: true });
  mkdirSync(join(dist, "assets", "packs"), { recursive: true });

  const common = {
    absWorkingDir: root,
    alias: ALIASES,
    plugins: [foliateEpubOnly],
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
      // A string, not an array: esbuild would put an array in a `<define:…>` module whose marker
      // comment keeps the name in the bundle, which check_variants rightly refuses.
      __LINGUA_PACKS__: JSON.stringify(PAIRS.join(",")),
      // Whether the reader chooses their native language (add-lingua-native-language-choice D1):
      // only once two native languages have a shipped pair. Until then the choice's views, its
      // preset and its copy fold out of every bundle, and the pages are `main`'s.
      __NATIVE_CHOICE__: JSON.stringify(shippedNatives(PAIRS).length > 1),
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

  // Popup + side panel + stats + the book reader → ES modules (loaded as <script type="module">).
  await build({
    ...common,
    entryPoints: {
      popup: join(root, "src/popup/popup.ts"),
      sidepanel: join(root, "src/sidepanel/sidepanel.ts"),
      stats: join(root, "src/stats/stats.ts"),
      onboarding: join(root, "src/onboarding/onboarding.ts"),
      account: join(root, "src/account/account.ts"),
      reader: join(root, "src/reader/reader.ts"),
    },
    outdir: dist,
    format: "esm",
  });

  const host = translationHost(target);
  if (host !== "none") {
    // The engine's own thread: a CLASSIC worker, so Mozilla's glue — which assumes sloppy mode —
    // runs unpatched under importScripts. Beside it, the worker that downloads the model: it
    // lives in the same host, and only for as long as a download does.
    await build({
      ...common,
      entryPoints: {
        "engine-worker": join(root, "src/translate/host/engine-worker.ts"),
        "model-worker": join(root, "src/translate/host/model-worker.ts"),
      },
      outdir: dist,
      format: "iife",
    });
    if (host === "offscreen") {
      await build({
        ...common,
        entryPoints: { offscreen: join(root, "src/translate/host/offscreen.ts") },
        outdir: dist,
        format: "iife",
      });
      cpSync(join(root, "src/translate/host/offscreen.html"), join(dist, "offscreen.html"));
    }
    // The engine's two files, never the model: whatever else ENGINE_DIR holds stays there.
    mkdirSync(join(dist, "engine"), { recursive: true });
    for (const f of ENGINE_FILES) cpSync(join(ENGINE_DIR, f), join(dist, "engine", f));
    const bundled = bundledCatalogue(MODEL_CATALOGUE, MODEL_BASE_URL || MODEL_CATALOGUE.base);
    writeFileSync(join(dist, "model-manifest.json"), `${JSON.stringify(bundled, null, 2)}\n`);
  }

  // The variant's manifest, and the languages it speaks (tool/manifests.mjs): the committed source
  // transformed for the target, package.json's version stamped on, the packs exposed, the backend
  // origin granted, the Chromium id pinned, the reader injected statically where the browser needs
  // it — and, once a pair glossed in another language than French ships, the description and the
  // commands read from `_locales`. While only French-native pairs ship, byte for byte today's.
  const { manifest, locales } = buildManifest({
    base: baseManifest,
    target,
    pairs: PAIRS,
    messages: MESSAGES,
    version: VERSION,
    grpcWebUrl: GRPC_WEB_URL,
    extKey: EXT_KEY,
    offscreen: host === "offscreen",
    allUrls: process.env.LINGUA_ALL_URLS === "1",
  });
  writeFileSync(join(dist, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  // The shipped natives' messages, and no other language's (localise-lingua-manifest D2).
  for (const language of locales) {
    cpSync(join(root, LOCALES_DIR, language), join(dist, LOCALES_DIR, language), { recursive: true });
  }
  for (const [from, to] of staticCopies) cpSync(join(root, from), join(dist, to));
  // The committed icon set (tool/gen_icons.sh), except the host app's 1024 px icon.
  cpSync(join(root, "icons"), join(dist, "icons"), {
    recursive: true,
    filter: (src) => !src.endsWith("icon-1024.png"),
  });

  console.log(`Built ${target} → dist-${target}/`);
}
