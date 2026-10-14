import { localDayStart } from "../review/session.ts";
import { type InterfaceLanguage } from "../i18n/index.ts";
import { BADGE_TEXT_COLOUR, einkColours, globalBadge, type GlobalBadge, type PageReport, tabBadge } from "./badge.ts";
import {
  countsAt,
  loadOutlook,
  OUTLOOK_KEY,
  OUTLOOK_VERSION,
  parseOutlook,
  type ReviewOutlook,
  usedUp,
} from "./review-outlook.ts";
import {
  type AsyncStorageArea,
  loadBadgeCount,
  loadColourPreference,
  loadNewWordsPerDay,
  loadStored,
} from "./storage.ts";

// The toolbar icon's count, as the background runs it (add-lingua-comeback-hints D2–D4): kept
// apart from background.ts so its wiring is tested on fakes. Two halves:
//  - after each change of the reader's data, the outlook is worked out again — the standalone WASM
//    function over the stored backup, no engine, no pack — and kept under OUTLOOK_KEY. A change
//    arriving while one runs marks it dirty, and one more run follows: no timer.
//  - on every event (a tab activated, a window focused, the browser starting, a setting changed, a
//    page's report…), the outlook is read, the count worked out for now, and the action API
//    touched only when the text, colour or title differ from what was last applied.
// Nothing here sends anything anywhere, and nothing runs between two events: the count can be
// behind, by design (D3). It does not run while the reader turned the count off, nor where the
// browser cannot show it beside page percentages (Safari before 18), where the badge keeps its
// behaviour from before.

/** The action API's badge calls, as the background has them (`chrome.action`). */
export interface BadgeAction {
  /** `null` with a tab: the tab's own text is cleared and it shows the global one. */
  setBadgeText(details: { text: string | null; tabId?: number }): Promise<unknown>;
  setBadgeBackgroundColor(details: { color: string; tabId?: number }): Promise<unknown>;
  /** Absent in Safari, which draws its own badge colours. */
  setBadgeTextColor?(details: { color: string; tabId?: number }): Promise<unknown> | void;
  setTitle(details: { title: string; tabId?: number }): Promise<unknown>;
}

export interface BadgeHostDeps {
  action: BadgeAction;
  /** The preferences (chrome.storage.local): the setting, the allowance, the colours — and the outlook. */
  prefs: AsyncStorageArea;
  /** Where the last applied global values are kept across the worker's stops (chrome.storage.session). */
  applied: AsyncStorageArea | null;
  /** The reader's data, whose root key holds the backup (the background's own store). */
  store: AsyncStorageArea;
  /** lingua-wasm's `reviewOutlook` over a backup: JSON. */
  computeOutlook: (backup: string, now: number, dayStart: number) => Promise<string>;
  /** Epoch seconds. */
  now: () => number;
  /** Whether this browser can show the count beside page percentages (`countSupported`). */
  supported: boolean;
  interfaceLanguage: () => Promise<InterfaceLanguage>;
}

export interface BadgeHost {
  /** The reader's data changed: the outlook is worked out again, then shown. */
  dataChanged(): Promise<void>;
  /** An event: the count shown as of now, the outlook worked out first when missing or used up. */
  refresh(): Promise<void>;
  /** A page reported its figure for its tab. */
  tabReport(tabId: number, report: PageReport): Promise<void>;
  /** The reader turned the count on or off. */
  settingChanged(): Promise<void>;
}

/** Where the last applied global values are kept. */
export const BADGE_APPLIED_KEY = "cymbra-lingua-badge-applied";

const sameLook = (a: unknown, b: GlobalBadge): boolean => {
  const v = (a ?? {}) as Partial<GlobalBadge>;
  return v.text === b.text && v.colour === b.colour && v.title === b.title;
};

export function badgeHost(deps: BadgeHostDeps): BadgeHost {
  let running: Promise<void> | null = null;
  let dirty = false;

  /** One run: the outlook of the stored backup as of now, kept. */
  async function workOut(): Promise<void> {
    const now = deps.now();
    const dayStart = localDayStart(now);
    const stored = await loadStored(deps.store);
    let outlook: ReviewOutlook | null;
    if (stored.kind === "v2") {
      outlook = parseOutlook(await deps.computeOutlook(stored.backup, now, dayStart));
    } else {
      // No backup yet, or a reading-only state that holds no deck: nothing waits.
      outlook = { version: OUTLOOK_VERSION, computedAt: now, dayStart, languages: [] };
    }
    if (outlook) await deps.prefs.set({ [OUTLOOK_KEY]: outlook });
  }

  /** Work the outlook out again, one run at a time; a change during a run adds one more. */
  function recompute(): Promise<void> {
    if (running) {
      dirty = true;
      return running;
    }
    running = (async () => {
      try {
        do {
          dirty = false;
          try {
            await workOut();
          } catch (e) {
            // A backup the function cannot read keeps the outlook there was: the next change retries.
            console.warn("[Cymbra Lingua] could not work out the review outlook:", e);
          }
        } while (dirty);
      } finally {
        // In the same turn as the last check of `dirty`: no change can slip in between.
        running = null;
      }
    })();
    return running;
  }

  async function apply(look: GlobalBadge): Promise<void> {
    const last = deps.applied ? (await deps.applied.get(BADGE_APPLIED_KEY))[BADGE_APPLIED_KEY] : undefined;
    if (sameLook(last, look)) return;
    await deps.action.setBadgeText({ text: look.text });
    await deps.action.setBadgeBackgroundColor({ color: look.colour });
    await deps.action.setBadgeTextColor?.({ color: BADGE_TEXT_COLOUR });
    await deps.action.setTitle({ title: look.title });
    await deps.applied?.set({ [BADGE_APPLIED_KEY]: look });
  }

  /** Show the count as of now; `afterRun` says an outlook was just worked out, so none is asked again. */
  async function show(afterRun: boolean): Promise<void> {
    if (!deps.supported) return;
    const [shown, colours, interfaceLanguage] = await Promise.all([
      loadBadgeCount(deps.prefs),
      loadColourPreference(deps.prefs),
      deps.interfaceLanguage(),
    ]);
    const opts = { shown, eink: einkColours(colours), interfaceLanguage };
    if (!shown) {
      await apply(globalBadge([], opts));
      return;
    }
    const now = deps.now();
    const outlook = await loadOutlook(deps.prefs);
    // Missing, of another version, or its upcoming times used up: only a new one knows.
    if (!afterRun && (!outlook || usedUp(outlook, now))) {
      await recompute();
      await show(true);
      return;
    }
    const perDay = await loadNewWordsPerDay(deps.prefs);
    const counts = outlook ? countsAt(outlook, now, localDayStart(now), perDay) : [];
    await apply(globalBadge(counts, opts));
  }

  return {
    async dataChanged() {
      if (!deps.supported || !(await loadBadgeCount(deps.prefs))) return;
      await recompute();
      await show(true);
    },
    refresh: () => show(false),
    async tabReport(tabId, report) {
      const eink = einkColours(await loadColourPreference(deps.prefs));
      const look = tabBadge(report, { eink, counting: deps.supported });
      await deps.action.setBadgeText({ tabId, text: look.text });
      await deps.action.setBadgeBackgroundColor({ tabId, color: look.colour });
      await deps.action.setBadgeTextColor?.({ tabId, color: BADGE_TEXT_COLOUR });
    },
    async settingChanged() {
      // Off: no outlook is kept, so none can go stale; on again, it is worked out at once.
      if (!(await loadBadgeCount(deps.prefs))) await deps.prefs.set({ [OUTLOOK_KEY]: null });
      await show(false);
    },
  };
}
