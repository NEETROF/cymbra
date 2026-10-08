import type { LinguaPort } from "../analyzer/port.ts";
import { fillPageInLanguage, type InterfaceLanguageArea } from "../i18n/index.ts";
import { type AsyncStorageArea, hydrateEngine } from "../state/storage.ts";
import { mountStats, statsCopy } from "./view.ts";

// The standalone statistics tab's start (localise-lingua-review-stats D1): the interface language
// read and the page's static copy filled in it beside the engine's hydration, as content.ts reads
// it beside its port, then the view mounted in that language. stats.ts calls it.

/** How long the tab waits for the interface language before it shows in French. */
export const LANGUAGE_READ_BOUND_MS = 500;

export interface StatsPageDeps {
  /** Preferences (chrome.storage.local), where the interface language is kept. */
  prefs: InterfaceLanguageArea;
  /** The reader's data, owned by the background. */
  store: AsyncStorageArea;
  port: LinguaPort;
  /** The bound on the interface language's read; `LANGUAGE_READ_BOUND_MS` when not given. */
  languageReadBoundMs?: number;
}

/**
 * `area`, its read bounded: a read that has not answered within `ms` fails, and a failed read is
 * French, said with a warning (`interfaceLanguage`) — so a storage that hangs never keeps the tab
 * hidden behind its pending mark until the reveal's fallback, nor its view from mounting.
 */
function bounded(area: InterfaceLanguageArea, ms: number): InterfaceLanguageArea {
  return {
    get: (keys) =>
      new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error(`no answer within ${ms} ms`)), ms);
        area.get(keys).then(
          (got) => {
            clearTimeout(timer);
            resolve(got);
          },
          (e: unknown) => {
            clearTimeout(timer);
            reject(e instanceof Error ? e : new Error(String(e)));
          },
        );
      }),
  };
}

/** Start the statistics tab in `document`: its copy, its engine, then its view, in the interface language. */
export async function start(document: Document, deps: StatsPageDeps): Promise<void> {
  const [{ language }] = await Promise.all([
    fillPageInLanguage(document, bounded(deps.prefs, deps.languageReadBoundMs ?? LANGUAGE_READ_BOUND_MS), statsCopy),
    hydrateEngine(deps.port, deps.store),
  ]);
  const root = document.getElementById("stats-root");
  if (!root) return;
  await mountStats(root, deps.port, deps.store, undefined, language);
}
