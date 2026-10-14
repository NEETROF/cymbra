import { localDayStart } from "../review/session.ts";
import { type InterfaceLanguage } from "../i18n/index.ts";
import { BADGE_TEXT_COLOUR, einkColours, globalBadge, type GlobalBadge, type PageReport, tabBadge } from "./badge.ts";
import {
  countsAt,
  loadOutlook,
  OUTLOOK_KEY,
  OUTLOOK_VERSION,
  outlookFingerprint,
  parseOutlook,
  type ReviewOutlook,
  staleAt,
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
//    function over the stored backup, no engine, no pack — and kept under OUTLOOK_KEY, with the
//    digest of what it read in the backup (the deck and the studied languages). A write whose
//    digest is the same — a page's reading exposures, every two seconds while reading — works
//    nothing out again. A change arriving while one runs marks it dirty, and one more run follows:
//    no timer. A run that started before the reader turned the count off, or off and on again,
//    writes nothing: the switch moves an epoch.
//  - on every event (a tab activated, a window focused, the browser starting, a setting changed, a
//    page's report…), the outlook is read, the count worked out for now, and the action API
//    touched only when the text, colour or title differ from what was last applied. Every touch of
//    the action API — the global badge, a tab's own — runs on one chain, in the order the events
//    came, so what is shown and what is remembered as shown never part.
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
  /** The digest of a backup's outlook inputs (`outlookFingerprint`), unless given. */
  fingerprint?: (backup: string) => Promise<string | null>;
  /**
   * Ask a tab's page to say its figure again (`badge:again`) once its load is over: a page that
   * stayed — a link within it — puts back the percentage its navigation's start cleared. Rejects
   * where no reader runs.
   */
  askReport?: (tabId: number) => Promise<unknown>;
}

export interface BadgeHost {
  /** The reader's data changed: the outlook is worked out again when what it reads changed, then shown. */
  dataChanged(): Promise<void>;
  /** An event: the count shown as of now, the outlook worked out first when missing, used up or stale. */
  refresh(): Promise<void>;
  /** A page reported its figure for its tab. */
  tabReport(tabId: number, report: PageReport): Promise<void>;
  /** The reader turned the count on or off. */
  settingChanged(): Promise<void>;
  /**
   * A tab's load moved (`tabs.onUpdated`'s `status`): at its start the tab's own percentage goes, so
   * it never outlives its page whatever the browser does; at its end the page is asked to say its
   * figure again (`askReport`).
   */
  tabUpdated(tabId: number, status: string | undefined): Promise<void>;
}

/** Where the last applied global values are kept. */
export const BADGE_APPLIED_KEY = "cymbra-lingua-badge-applied";

const sameLook = (a: unknown, b: GlobalBadge): boolean => {
  const v = (a ?? {}) as Partial<GlobalBadge>;
  return v.text === b.text && v.colour === b.colour && v.title === b.title;
};

export function badgeHost(deps: BadgeHostDeps): BadgeHost {
  const fingerprint = deps.fingerprint ?? outlookFingerprint;
  let running: Promise<void> | null = null;
  let dirty = false;
  /** Moved by every turn of the switch: a run started before it writes nothing (D7). */
  let epoch = 0;
  /** The action API's chain: one touch at a time, in the order asked. */
  let chain: Promise<void> = Promise.resolve();

  /** Run `task` after every touch of the action API asked before it; a failure does not stop the chain. */
  function onChain(task: () => Promise<void>): Promise<void> {
    const run = chain.then(task);
    chain = run.catch(() => {});
    return run;
  }

  /**
   * One run: the outlook of the stored backup as of now, kept — unless the one kept was worked out
   * from the same deck and languages and can still count now, or the switch turned since the run
   * started.
   */
  async function workOut(): Promise<void> {
    const started = epoch;
    const now = deps.now();
    const dayStart = localDayStart(now);
    const stored = await loadStored(deps.store);
    let outlook: ReviewOutlook | null;
    if (stored.kind === "v2") {
      const [inputs, kept] = await Promise.all([fingerprint(stored.backup), loadOutlook(deps.prefs)]);
      if (inputs && kept?.inputs === inputs && !staleAt(kept, now, dayStart) && !usedUp(kept, now)) return;
      outlook = parseOutlook(await deps.computeOutlook(stored.backup, now, dayStart));
      if (outlook && inputs) outlook.inputs = inputs;
    } else {
      // No backup yet, or a reading-only state that holds no deck: nothing waits.
      outlook = { version: OUTLOOK_VERSION, computedAt: now, dayStart, languages: [] };
    }
    // Turned off, or off and on again, while it ran: what it worked out is not this switch's.
    if (!outlook || started !== epoch || !(await loadBadgeCount(deps.prefs))) return;
    await deps.prefs.set({ [OUTLOOK_KEY]: outlook });
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

  /** The global badge as of now, from what is kept; null when the outlook must be worked out first. */
  async function lookNow(afterRun: boolean): Promise<GlobalBadge | null> {
    const [shown, colours, interfaceLanguage] = await Promise.all([
      loadBadgeCount(deps.prefs),
      loadColourPreference(deps.prefs),
      deps.interfaceLanguage(),
    ]);
    const opts = { shown, eink: einkColours(colours), interfaceLanguage };
    if (!shown) return globalBadge([], opts);
    const now = deps.now();
    const dayStart = localDayStart(now);
    const outlook = await loadOutlook(deps.prefs);
    // Missing, of another version, its upcoming times used up, or the clock gone back: only a new one knows.
    if (!afterRun && (!outlook || usedUp(outlook, now) || staleAt(outlook, now, dayStart))) return null;
    const perDay = await loadNewWordsPerDay(deps.prefs);
    // Still stale after a run (a backup that could not be read): nothing rather than too much.
    return globalBadge((outlook && countsAt(outlook, now, dayStart, perDay)) ?? [], opts);
  }

  /** Apply `look` unless it is what was last applied: always on the chain, so the record and the icon agree. */
  async function apply(look: GlobalBadge): Promise<void> {
    const last = deps.applied ? (await deps.applied.get(BADGE_APPLIED_KEY))[BADGE_APPLIED_KEY] : undefined;
    if (sameLook(last, look)) return;
    await deps.action.setBadgeText({ text: look.text });
    await deps.action.setBadgeBackgroundColor({ color: look.colour });
    await deps.action.setBadgeTextColor?.({ color: BADGE_TEXT_COLOUR });
    await deps.action.setTitle({ title: look.title });
    await deps.applied?.set({ [BADGE_APPLIED_KEY]: look });
  }

  /**
   * Show the count as of now; `afterRun` says an outlook was just worked out, so none is asked
   * again. The look is worked out on the chain itself, so the last event shows the latest figure.
   */
  async function show(afterRun: boolean): Promise<void> {
    if (!deps.supported) return;
    let missing = false;
    await onChain(async () => {
      const look = await lookNow(afterRun);
      if (look) await apply(look);
      else missing = true;
    });
    if (!missing) return;
    await recompute();
    await show(true);
  }

  /** A tab's own badge, on the chain. */
  function setTab(tabId: number, report: PageReport): Promise<void> {
    return onChain(async () => {
      const eink = einkColours(await loadColourPreference(deps.prefs));
      const look = tabBadge(report, { eink, counting: deps.supported });
      await deps.action.setBadgeText({ tabId, text: look.text });
      await deps.action.setBadgeBackgroundColor({ tabId, color: look.colour });
      await deps.action.setBadgeTextColor?.({ tabId, color: BADGE_TEXT_COLOUR });
    });
  }

  return {
    async dataChanged() {
      if (!deps.supported || !(await loadBadgeCount(deps.prefs))) return;
      await recompute();
      await show(true);
    },
    refresh: () => show(false),
    tabReport: (tabId, report) => setTab(tabId, report),
    async settingChanged() {
      epoch += 1;
      if (await loadBadgeCount(deps.prefs)) {
        // On again: worked out at once, whatever a run started before it went off had in hand.
        if (deps.supported) await recompute();
        await show(true);
        return;
      }
      // Off: no outlook is kept, so none can go stale.
      await deps.prefs.set({ [OUTLOOK_KEY]: null });
      await show(false);
    },
    async tabUpdated(tabId, status) {
      // Safari before 18 keeps one badge for every tab: clearing one would clear them all (D4).
      if (!deps.supported) return;
      if (status === "loading") await setTab(tabId, { pct: null });
      else if (status === "complete") await deps.askReport?.(tabId).catch(() => {});
    },
  };
}
