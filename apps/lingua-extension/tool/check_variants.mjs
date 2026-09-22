// Verify the built browser variants (run after `yarn build`): each manifest carries what its
// browser needs, and each bundle kept only its own branches of the capability defines
// (build.mjs `capabilities` + esbuild minifySyntax). A stray `__TARGET__ === "firefox"` left
// where a capability belongs builds fine and type-checks — only the bundle shows it.
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const failures = [];
// `--engine` checks a DEVELOPMENT build made with LINGUA_TRANSLATION_ENGINE; without it, this
// checks what ships — and what ships must carry no trace of the translation engine.
const ENGINE_BUILD = process.argv.includes("--engine");

function read(target, file) {
  return readFileSync(join(root, `dist-${target}`, file), "utf8");
}

function expect(ok, message) {
  if (!ok) failures.push(message);
}

const manifests = Object.fromEntries(
  ["chromium", "firefox", "safari"].map((t) => [t, JSON.parse(read(t, "manifest.json"))]),
);

// Manifests.
const { chromium, firefox, safari } = manifests;
expect(chromium.background?.service_worker, "chromium: background must be a service worker");
expect(!chromium.content_scripts, "chromium: must stay activeTab-first (no static content script)");
expect(chromium.permissions.includes("sidePanel"), "chromium: needs the sidePanel permission");

expect(firefox.background?.scripts, "firefox: background must be an event page");
expect(firefox.browser_specific_settings?.gecko?.id, "firefox: needs a gecko add-on id");
expect(firefox.content_scripts?.length, "firefox: needs the static content script");
expect(!firefox.key, "firefox: must not carry the Chromium key");

expect(
  safari.background?.scripts && safari.background.persistent === false,
  "safari: event page must be non-persistent",
);
expect(!safari.background?.service_worker, "safari: must not declare a service worker");
expect(safari.content_scripts?.length, "safari: needs the static content script");
expect(!safari.side_panel, "safari: must not declare a side panel");
for (const p of ["sidePanel", "identity"]) {
  expect(!safari.permissions.includes(p), `safari: must not request the unsupported "${p}" permission`);
}
expect(!safari.key && !safari.browser_specific_settings, "safari: must carry neither the Chromium key nor a gecko id");
expect(
  safari.permissions.includes("nativeMessaging"),
  "safari: needs nativeMessaging to collect the host app's id_token",
);
for (const [target, m] of [
  ["chromium", chromium],
  ["firefox", firefox],
]) {
  expect(!m.permissions.includes("nativeMessaging"), `${target}: must not request nativeMessaging`);
}

for (const [target, m] of Object.entries(manifests)) {
  expect(m.icons && Object.keys(m.icons).length > 0, `${target}: manifest must declare icons`);
}

// Folded branches: markers that exist in exactly one family of variants.
const markers = [
  // The in-content WASM probe of resolveContentPort — Chromium only.
  { file: "content.js", text: "inContent.calibration", chromium: true },
  // The in-page drawer as the review surface — event-page family only.
  { file: "content.js", text: "drawer.openOn(view)", chromium: false },
  // Dynamic reader registration — Chromium only.
  { file: "background.js", text: "registerContentScripts(", chromium: true },
];
for (const { file, text, chromium: inChromium } of markers) {
  for (const target of ["chromium", "firefox", "safari"]) {
    const present = read(target, file).includes(text);
    const wanted = (target === "chromium") === inChromium;
    expect(present === wanted, `${target}/${file}: "${text}" should be ${wanted ? "present" : "folded away"}`);
  }
}

// Safari only: the host app hand-off (add-lingua-connected-clients D6).
const safariOnly = [
  { file: "background.js", text: "sendNativeMessage(" },
  { file: "popup.js", text: "account:collectHandedToken" },
];
for (const { file, text } of safariOnly) {
  for (const target of ["chromium", "firefox", "safari"]) {
    const wanted = target === "safari";
    expect(
      read(target, file).includes(text) === wanted,
      `${target}/${file}: "${text}" should be ${wanted ? "present" : "folded away"}`,
    );
  }
}

// A define missing from build.mjs builds fine and type-checks (env.d.ts declares it), then
// throws a ReferenceError at runtime: no bundle may keep one of env.d.ts's defines.
const defines = [...readFileSync(join(root, "env.d.ts"), "utf8").matchAll(/declare const (__[A-Z_]+__)/g)].map(
  (m) => m[1],
);
expect(defines.length > 0, "env.d.ts: no build defines found");
for (const target of ["chromium", "firefox", "safari"]) {
  for (const file of ["background.js", "content.js", "popup.js", "account.js"]) {
    const bundle = read(target, file);
    for (const name of defines) {
      expect(!bundle.includes(name), `${target}/${file}: build define "${name}" was not replaced`);
    }
  }
}

// The translation engine (add-lingua-translation-engine). Shipped: absent, everywhere — no
// permission, no file, no code. Development build: hosted off every thread that paints, by an
// offscreen document on Chromium (whose service worker cannot construct a Worker) and by the
// event page on Firefox; Safari is out of scope and carries none of it.
const has = (target, file) => existsSync(join(root, `dist-${target}`, file));
const ENGINE_MARKERS = ["lingua-translate", "engine-worker.js", "offscreen.html", "loadBergamot"];
const hosting = ENGINE_BUILD ? ["chromium", "firefox"] : [];
for (const target of ["chromium", "firefox", "safari"]) {
  const hosts = hosting.includes(target);
  const offscreen = hosts && target === "chromium";
  expect(
    manifests[target].permissions.includes("offscreen") === offscreen,
    `${target}: the "offscreen" permission should be ${offscreen ? "requested" : "absent"}`,
  );
  expect(
    has(target, "engine-worker.js") === hosts,
    `${target}: engine-worker.js should be ${hosts ? "built" : "absent"}`,
  );
  expect(
    has(target, "offscreen.html") === offscreen,
    `${target}: offscreen.html should be ${offscreen ? "built" : "absent"}`,
  );
  expect(
    has(target, "engine/bergamot-translator.wasm") === hosts,
    `${target}: the engine artefact should be ${hosts ? "bundled" : "absent"}`,
  );
  if (!hosts) {
    for (const file of ["background.js", "content.js", "popup.js", "sidepanel.js", "stats.js", "account.js"]) {
      if (!has(target, file)) continue;
      const bundle = read(target, file);
      for (const marker of ENGINE_MARKERS) {
        expect(!bundle.includes(marker), `${target}/${file}: "${marker}" must not ship without the engine`);
      }
    }
  } else {
    // The background relays; which host it relays to is the variant's.
    const bg = read(target, "background.js");
    expect(bg.includes("lingua-translate"), `${target}/background.js: should relay translations`);
    expect(
      bg.includes("offscreen.html") === offscreen,
      `${target}/background.js: offscreen host should be ${offscreen ? "present" : "folded away"}`,
    );
    // Whoever constructs the worker names its script: the event page on Firefox, the offscreen
    // document on Chromium — never Chromium's service worker, where `Worker` does not exist and
    // the call would throw at runtime.
    const owner = offscreen ? "offscreen.js" : "background.js";
    expect(read(target, owner).includes("engine-worker.js"), `${target}/${owner}: should construct the engine worker`);
    if (offscreen) {
      expect(
        !bg.includes("engine-worker.js"),
        `${target}/background.js: a service worker cannot construct a Worker — the offscreen document must own it`,
      );
    }
  }
}

if (failures.length > 0) {
  console.error(`Variant check failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log(
  `Variant check passed: chromium, firefox, safari${ENGINE_BUILD ? " (development build with the engine)" : ""}.`,
);
