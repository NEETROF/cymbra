// Verify the built browser variants (run after `yarn build`): each manifest carries what its
// browser needs, and each bundle kept only its own branches of the capability defines
// (build.mjs `capabilities` + esbuild minifySyntax). A stray `__TARGET__ === "firefox"` left
// where a capability belongs builds fine and type-checks — only the bundle shows it.
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { engineProblems } from "./engine_pin.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const failures = [];

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
  // The book reader's sections served by the service worker — Chromium only (section-server.ts).
  { file: "background.js", text: "reader-section/", chromium: true },
  { file: "reader.js", text: "reader-section/", chromium: true },
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

// The translation engine (add-lingua-translation-delivery D9). Chromium and Firefox CARRY it —
// the pinned build, hosted off every thread that paints: by an offscreen document on Chromium
// (whose service worker cannot construct a Worker), by the event page on Firefox — together with
// the manifest of the model it may download. Safari carries none of it. No package carries a
// model file, and nothing in any package fetches code: the only thing downloaded is data.
const has = (target, file) => existsSync(join(root, `dist-${target}`, file));
const ENGINE_MARKERS = ["lingua-translate", "engine-worker.js", "model-worker.js", "offscreen.html", "loadBergamot"];
const committedModel = JSON.parse(readFileSync(join(root, "model-manifest.json"), "utf8"));

/** Every file under dist-<target>, relative. */
function filesOf(target) {
  const base = join(root, `dist-${target}`);
  const out = [];
  const walk = (dir) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) walk(path);
      else out.push(relative(base, path));
    }
  };
  walk(base);
  return out;
}

for (const target of ["chromium", "firefox", "safari"]) {
  const hosts = target !== "safari";
  const offscreen = target === "chromium";
  expect(
    manifests[target].permissions.includes("offscreen") === offscreen,
    `${target}: the "offscreen" permission should be ${offscreen ? "requested" : "absent"}`,
  );
  for (const [file, wanted] of [
    ["engine-worker.js", hosts],
    ["model-worker.js", hosts],
    ["model-manifest.json", hosts],
    ["offscreen.html", offscreen],
  ]) {
    expect(has(target, file) === wanted, `${target}: ${file} should be ${wanted ? "built" : "absent"}`);
  }
  expect(
    has(target, "engine/bergamot-translator.wasm") === hosts,
    `${target}: the engine should be ${hosts ? "packaged" : "absent"}`,
  );

  // No model file, whatever it is called: the model is fetched once the reader asks, never shipped.
  for (const file of filesOf(target)) {
    const size = statSync(join(root, `dist-${target}`, file)).size;
    expect(
      !/\.(bin|spm)(\.gz)?$/.test(file),
      `${target}: ${file} looks like a model file — the model is never packaged`,
    );
    expect(size < 8_000_000, `${target}: ${file} is ${size} bytes — no packaged file is that large but a model`);
  }

  // No code fetched at run time: no bundle holds the address of a remote script or WebAssembly
  // module as a string (an address in a library's comment is not one).
  for (const file of filesOf(target).filter((f) => f.endsWith(".js"))) {
    const remote = read(target, file).match(/["'`]https?:\/\/[^"'`\s]+\.(?:m?js|wasm)(?:[?#][^"'`]*)?["'`]/);
    expect(!remote, `${target}/${file}: names remote code (${remote?.[0]}) — only the model may be fetched`);
  }

  if (!hosts) {
    for (const file of ["background.js", "content.js", "popup.js", "sidepanel.js", "stats.js", "account.js"]) {
      if (!has(target, file)) continue;
      const bundle = read(target, file);
      for (const marker of ENGINE_MARKERS) {
        expect(!bundle.includes(marker), `${target}/${file}: "${marker}" must not ship without the engine`);
      }
    }
    continue;
  }

  // The pinned engine, byte for byte.
  for (const problem of engineProblems(join(root, `dist-${target}`, "engine"))) {
    expect(false, `${target}: packaged engine — ${problem}`);
  }

  // The bundled manifest is the committed one, at the production host, and names only data.
  const model = JSON.parse(read(target, "model-manifest.json"));
  expect(
    model.base === committedModel.base,
    `${target}: the model is fetched from ${model.base}, not ${committedModel.base} — a development build (LINGUA_MODEL_BASE_URL)`,
  );
  expect(model.version === committedModel.version, `${target}: model-manifest.json is not the committed model`);
  for (const [role, file] of Object.entries(committedModel.files)) {
    const got = model.files?.[role];
    expect(
      got?.path === file.path && got.sha256 === file.sha256 && got.size === file.size,
      `${target}: model-manifest.json's ${role} differs from the committed one`,
    );
    expect(/\.gz$/.test(file.path) && /^[0-9a-f]{64}$/.test(file.sha256), `${target}: ${role} must be pinned data`);
  }
  expect(/^https:\/\//.test(model.base), `${target}: the model must be fetched over https`);

  // The background relays; which host it relays to is the variant's.
  const bg = read(target, "background.js");
  expect(bg.includes("lingua-translate"), `${target}/background.js: should relay translations`);
  expect(
    bg.includes("offscreen.html") === offscreen,
    `${target}/background.js: offscreen host should be ${offscreen ? "present" : "folded away"}`,
  );
  // Whoever constructs the workers names their scripts: the event page on Firefox, the offscreen
  // document on Chromium — never Chromium's service worker, where `Worker` does not exist and
  // the call would throw at runtime.
  const owner = offscreen ? "offscreen.js" : "background.js";
  for (const worker of ["engine-worker.js", "model-worker.js"]) {
    expect(read(target, owner).includes(worker), `${target}/${owner}: should construct ${worker}`);
    if (offscreen) {
      expect(
        !bg.includes(worker),
        `${target}/background.js: a service worker cannot construct a Worker — the offscreen document must own ${worker}`,
      );
    }
  }
}

// The book reader (add-lingua-reader): the same page in every variant, with the EPUB renderer
// only — view.js's other formats are refused at build time (build.mjs) and must not ship — and
// no new permission: the file picker, IndexedDB and an extension page need none. The zip reader
// is foliate's own entry, with no WebAssembly variant pulled in through the package's exports.
const READER_FILES = ["reader.html", "reader.js", "reader.css"];
const READER_PERMISSIONS = { chromium: ["activeTab", "storage", "scripting", "sidePanel", "identity"] };
READER_PERMISSIONS.firefox = READER_PERMISSIONS.chromium.filter((p) => p !== "sidePanel");
READER_PERMISSIONS.safari = [...READER_PERMISSIONS.firefox.filter((p) => p !== "identity"), "nativeMessaging"];
for (const target of ["chromium", "firefox", "safari"]) {
  for (const file of READER_FILES) expect(has(target, file), `${target}: ${file} should be built`);
  if (!has(target, "reader.js")) continue;
  const reader = read(target, "reader.js");
  expect(reader.includes("foliate-view"), `${target}/reader.js: should carry the foliate-js renderer`);
  expect(reader.includes("Cymbra Lingua reads EPUB only"), `${target}/reader.js: other formats should be refused`);
  // The modules themselves, by the paths esbuild notes above each: pdf.js's engine, and the
  // zip package's own codecs (JavaScript or WebAssembly) and inline workers.
  for (const marker of [
    "pdfjs",
    "GlobalWorkerOptions",
    "lib/core/streams/zlib-js/",
    "lib/core/streams/zlib-wasm/",
    "lib/core/web-worker-inline",
  ]) {
    expect(!reader.includes(marker), `${target}/reader.js: "${marker}" must not ship (EPUB only, no zip codec)`);
  }
  const extra = manifests[target].permissions.filter(
    (p) => !READER_PERMISSIONS[target].includes(p) && p !== "offscreen",
  );
  expect(
    extra.length === 0,
    `${target}: the reader added no permission, but the manifest asks for ${extra.join(", ")}`,
  );
}

if (failures.length > 0) {
  console.error(`Variant check failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log("Variant check passed: chromium, firefox, safari.");
