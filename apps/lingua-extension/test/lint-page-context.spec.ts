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
