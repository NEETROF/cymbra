import { readdirSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { describe, expect, it } from "vitest";

// A content script runs in the VISITED page's origin, where the extension APIs it may call
// are a short list: runtime messaging, storage, i18n. `chrome.tabs` is not one of them, and
// calling it does nothing at all — no error the reader sees, just a dead button (dogfooding,
// build 100/101: « Gérer mes données » in the in-page drawer). The modules the content
// script pulls in must therefore go through the background (`state/open-page.ts`).
//
// The extension's own pages — popup, side panel, account, onboarding — are not concerned:
// they run on the extension origin and may call it directly.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const IN_THE_PAGE = ["src/content.ts", "src/reading", "src/review", "src/stats", "src/analyzer"];

function tsFiles(path: string): string[] {
  if (statSync(path).isFile()) return path.endsWith(".ts") ? [path] : [];
  return readdirSync(path).flatMap((name) => tsFiles(join(path, name)));
}

const FORBIDDEN = [
  { api: /\bchrome\.tabs\b/, why: "send `openPage` to the background instead (state/open-page.ts)" },
  { api: /\bchrome\.action\b/, why: "the background owns the toolbar icon and badge" },
  { api: /\bchrome\.identity\b/, why: "sign-in runs in the background" },
  { api: /\bchrome\.sidePanel\b/, why: "the background opens the panel, with the click's activation" },
];

describe("what the page's own code may call", () => {
  const files = IN_THE_PAGE.flatMap((p) => tsFiles(join(root, p))).filter((f) => !f.includes("/wasm/"));

  it("finds the modules that run in the page", () => {
    expect(files.length).toBeGreaterThan(5);
  });

  for (const file of files) {
    const rel = file.slice(root.length + 1);
    it(`no extension-page-only API: ${rel}`, () => {
      const text = readFileSync(file, "utf8");
      for (const { api, why } of FORBIDDEN) {
        const offending = text
          .split("\n")
          .filter((line) => api.test(line) && !line.trimStart().startsWith("//") && !line.trimStart().startsWith("*"));
        expect(offending, `${rel} calls ${api.source} — ${why}`).toEqual([]);
      }
    });
  }
});

// The second half of the same family: a key that MOVED. The reader's data left
// chrome.storage.local for the store the background owns, and the sync trigger kept
// listening for it there — compiling, silent, and never firing again (dogfooding: a level
// chosen on the phone never left it). Nothing may watch a moved key through
// `chrome.storage.onChanged`; follow the store instead (`watchStore` / `watchBackup`).

const MOVED_KEYS = [
  { key: "ROOT_KEY", also: /"lingua"/ },
  { key: "cymbra-lingua-daily" },
  { key: "cymbra-lingua-status-cursor" },
  { key: "cymbra-lingua-card-cursor" },
  { key: "cymbra-lingua-erased-at" },
];

describe("nothing watches a key that moved to the store", () => {
  const files = tsFiles(join(root, "src")).filter(
    (f) => !f.includes("/wasm/") && !f.endsWith("state/store.ts") && !f.endsWith("state/storage.ts"),
  );

  for (const file of files) {
    const rel = file.slice(root.length + 1);
    const text = readFileSync(file, "utf8");
    if (!text.includes("storage.onChanged")) continue;
    it(`follows the store, not a departed key: ${rel}`, () => {
      // Only the lines inside a storage.onChanged listener matter; a plain read of the key
      // through the store's area is fine.
      const watched = text
        .split("\n")
        .filter((line) => /changes\[/.test(line))
        .join("\n");
      for (const { key, also } of MOVED_KEYS) {
        expect(watched, `${rel} watches ${key} in chrome.storage — use watchStore/watchBackup`).not.toContain(key);
        if (also) expect(also.test(watched)).toBe(false);
      }
    });
  }
});

// The third of the same family, and the one that bit twice: a surface that builds its OWN
// `chrome.storage.local` area and hands it to the reader's data. The store left that area
// (change: move-lingua-store-to-indexeddb), so such a surface hydrates an EMPTY engine and
// writes back where nothing reads — compiling, silent, and losing what the reader did. It
// cost the welcome tab's level choice, and the standalone stats tab's whole view.
//
// Preferences and marks (the HUD toggle, the last sync, the lost-session mark) still live in
// chrome.storage.local and are none of this rule's business: only the calls below are.

/** Areas a file builds out of chrome.storage.local — fine for preferences, not for the store. */
function settingsAreas(text: string): string[] {
  const out: string[] = [];
  const re = /const\s+(\w+)\s*:\s*AsyncStorageArea\s*=\s*\{([\s\S]*?)\}/g;
  for (let m = re.exec(text); m; m = re.exec(text)) {
    if (m[2].includes("chrome.storage.local")) out.push(m[1]);
  }
  return out;
}

/** Every call that reaches the reader's data, and where its area sits in the arguments. */
const READER_DATA = [
  { name: "hydrateEngine", re: /\bhydrateEngine\(\s*[^,()]+,\s*([\w.]+)/g },
  { name: "saveBackup", re: /\bsaveBackup\(\s*([\w.]+)/g },
  { name: "mountStats", re: /\bmountStats\(\s*[^,]+,\s*[^,]+,\s*([\w.]+)/g },
  { name: "mountReview", re: /\bmountReview\(\s*[^,]+,\s*[^,]+,\s*([\w.]+)/g },
];

describe("the reader's data is asked of its owner, never of chrome.storage.local", () => {
  const files = tsFiles(join(root, "src")).filter(
    (f) => !f.includes("/wasm/") && !f.endsWith("state/store.ts") && !f.endsWith("state/storage.ts"),
  );

  it("still recognises a settings area where one exists", () => {
    // Positive control: the side panel builds one for its preferences, and must keep passing
    // `store` to the calls below. Without this, the rule could pass by seeing nothing at all.
    const sidepanel = readFileSync(join(root, "src/sidepanel/sidepanel.ts"), "utf8");
    expect(settingsAreas(sidepanel)).toContain("area");
  });

  for (const file of files) {
    const rel = file.slice(root.length + 1);
    const text = readFileSync(file, "utf8");
    const areas = settingsAreas(text);
    if (areas.length === 0) continue;
    it(`hands the store, not its settings area: ${rel}`, () => {
      for (const { name, re } of READER_DATA) {
        for (let m = new RegExp(re).exec(text); m; m = re.exec(text)) {
          expect(
            areas,
            `${rel} passes its chrome.storage.local area to ${name}() — the store moved; use messagedArea()`,
          ).not.toContain(m[1]);
        }
      }
    });
  }
});

// The fourth of the same family, the one the book reader brought (add-lingua-reader D3): code
// reading the WRONG DOCUMENT. The reading module reads a web page in the content script, and
// each section of a book in the reader page — a document in foliate-js's iframe, whose window
// has its own selection, highlight registry and observers. A module that reaches for the global
// `document` or `window` reads the reader page's chrome instead of the book: it compiles, throws
// nothing, and paints or hears nothing. Same trap with `instanceof Element`: a node of the
// iframe belongs to another realm and is never an instance of this window's classes.
//
// So a module under src/reading reaches the read document through a parameter or a node
// (`docOf(node)`, `host.doc`, `host.win`), and a default of the global (`= document.body`,
// `= window`) keeps the content script's call sites unchanged. Only the modules that BUILD the
// surfaces — which mount in the reader page's own document, as they mount in the web page —
// may use the global document, and they are named here with that reason.

/** The modules that build a surface (popup, drawer, HUD, settings) in this context's own document. */
const SURFACE_MODULES = ["drawer.ts", "hud.ts", "settings-view.ts", "wordpopup.ts"];

const GLOBAL_DOM = [
  { re: /\bdocument\./, why: "read the document through a parameter or a node (docOf), not the global" },
  { re: /\bwindow\./, why: "read the window through a parameter (host.win), not the global" },
  {
    re: /\binstanceof\s+(Element|HTMLElement|Node|Text|Range|Document)\b/,
    why: "test the node type (isElement): a node of a book section's iframe is of another realm",
  },
];

/** A default parameter falling back on the global is how the content script's calls stay as they were. */
const DEFAULT_TO_GLOBAL = /=\s*(document\.body|document|window)\b(?=\s*[,)])/g;

describe("the reading module reads the document it was given", () => {
  const dir = join(root, "src/reading");
  const readSide = readdirSync(dir).filter((f) => f.endsWith(".ts") && !SURFACE_MODULES.includes(f));

  it("still classifies the surface modules it names", () => {
    // Positive control: every named surface module exists and does use the global document,
    // so the exemption is still needed — and a rename cannot silently widen the rule's hole.
    for (const f of SURFACE_MODULES) expect(readFileSync(join(dir, f), "utf8")).toMatch(/\bdocument\./);
    expect(readSide.length).toBeGreaterThan(5);
  });

  for (const file of readSide) {
    it(`no global document, window or cross-realm instanceof: src/reading/${file}`, () => {
      const offending = readFileSync(join(dir, file), "utf8")
        .split("\n")
        .filter((line) => !/^\s*(\/\/|\/\*|\*)/.test(line))
        .map((line) => line.replace(DEFAULT_TO_GLOBAL, ""))
        .flatMap((line) => GLOBAL_DOM.filter(({ re }) => re.test(line)).map(({ why }) => `${line.trim()} — ${why}`));
      expect(offending, `src/reading/${file}`).toEqual([]);
    });
  }
});
