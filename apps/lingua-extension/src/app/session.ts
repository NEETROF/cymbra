import type { CardOp, LinguaPort, StatusOp } from "../analyzer/port.ts";
import {
  DEFAULT_INTERFACE_LANGUAGE,
  INTERFACE_LANGUAGE_KEY,
  type InterfaceLanguage,
  isInterfaceLanguage,
} from "../i18n/language.ts";
import {
  applyAppOps,
  appOpsSince,
  encodeAppOps,
  parseAppOps,
  parseSharedState,
  type SharedState,
  reviewCountsBetween,
  sharedDailyStats,
  toSecond,
  withReviewCounts,
} from "../state/app-relay.ts";
import { DAILY_KEY, loadDailyStats } from "../state/dailystats.ts";
import { type AsyncStorageArea, ROOT_KEY, STORAGE_VERSION } from "../state/storage.ts";
import type { AppSend } from "./bridge.ts";

// The Cymbra Lingua app's session over the extension's state (add-lingua-apple-review D3): the
// engine restored from the state the extension shared, the app's changes not yet taken laid back on
// it, and two storage areas the extension's own pages are mounted on — the store, whose every save of
// the backup writes the review's changes to the App Group at once, and the preferences, the
// extension's as it shared them under what the app chose itself.

/** Where the app keeps what it chose itself (the review's language): this page's own origin. */
export const APP_PREFS_KEY = "cymbra-lingua-app-prefs";

export interface AppSessionDeps {
  send: AppSend;
  /** The engine for the shared backup: built once it is known, so it loads that reader's packs. */
  port: (backup: string) => LinguaPort;
  /** Epoch millis. */
  now: () => number;
  /** The page's own storage (localStorage), or null where it cannot be used. */
  local?: Pick<Storage, "getItem" | "setItem"> | null;
  /** The interface language the app was opened in, when the shared state names none. */
  fallbackLanguage?: string;
  /** This opening's id, under which the extension counts its reviews once; a random one unless given. */
  openingId?: string;
  /**
   * Called with the shared state and the interface language as soon as they are read — before the
   * engine, which takes seconds to load the deck: the page shows its loading copy in the reader's
   * language and colours meanwhile.
   */
  onState?: (state: SharedState, interfaceLanguage: InterfaceLanguage) => void;
  /**
   * The backup or the day's counts were saved, and the pending changes written (add-lingua-app-deck-sync
   * D5): the app's synchronisation pushes them, debounced.
   */
  onSaved?: () => void;
}

export interface AppSession {
  port: LinguaPort;
  /** The reader's data: the backup and the daily counts, in memory. */
  store: AsyncStorageArea;
  /** The preferences the review reads. */
  prefs: AsyncStorageArea;
  /** When the extension saved the state shown (epoch millis; 0 when it did not say). */
  savedAt: number;
  interfaceLanguage: InterfaceLanguage;
  /**
   * The interface language the reader chose in the extension, as the shared state names it; null when
   * it names none (the page then follows the app's language). The locale of the email sign-in's
   * requests (add-lingua-app-email-signin D3).
   */
  sharedLanguage: InterfaceLanguage | null;
  /** The extension's last snapshot of the account's online-translation access (add-lingua-review-translation-fallback D9). */
  onlineAccess: boolean;
  /** The extension's account digest, null signed out, undefined from an older extension (add-lingua-app-deck-sync D7). */
  sharedAccount: string | null | undefined;
  /** The erasure mark the extension's state acted on (D8). */
  sharedErasedAt: number;
  /** Every change dated from here on (epoch millis) is the app's to hand over — and to push (D3). */
  since: number;
  /** What the app hands over now: the changes since `since` (D3). */
  pendingOps: () => Promise<{ cards: CardOp[]; statuses: StatusOp[] }>;
  /**
   * The account was erased after the state this opening started from (add-lingua-app-deck-sync D8),
   * and the engine reset: keep of the pending changes only those made after the mark `erasedAt`, lay
   * them on the reset engine, forget the earlier openings' counts, and write what is left — nothing,
   * as a rule — so the extension finds no change from before the erasure.
   */
  discardPending: (erasedAt: number) => Promise<void>;
}

function pick(items: Record<string, unknown>, keys: string | string[] | null): Record<string, unknown> {
  if (keys === null) return { ...items };
  const out: Record<string, unknown> = {};
  for (const key of typeof keys === "string" ? [keys] : keys) if (key in items) out[key] = items[key];
  return out;
}

function readLocal(local: AppSessionDeps["local"]): Record<string, unknown> {
  try {
    const parsed: unknown = JSON.parse(local?.getItem(APP_PREFS_KEY) ?? "{}");
    return parsed !== null && typeof parsed === "object" ? (parsed as Record<string, unknown>) : {};
  } catch {
    return {};
  }
}

/** The interface language the shared state names, or null when it names none. */
export function sharedInterfaceLanguage(state: SharedState): InterfaceLanguage | null {
  const shared = state.prefs[INTERFACE_LANGUAGE_KEY];
  return isInterfaceLanguage(shared) ? shared : null;
}

/** The extension's interface language as the shared state names it, else the app's, else French. */
export function sessionLanguage(state: SharedState, fallback: string | undefined): InterfaceLanguage {
  const shared = sharedInterfaceLanguage(state);
  if (shared) return shared;
  return isInterfaceLanguage(fallback) ? fallback : DEFAULT_INTERFACE_LANGUAGE;
}

/**
 * Open the review on the state the extension shared, or resolve null when there is none the app can
 * read — the app then says how to turn the extension on.
 */
export async function openAppSession(deps: AppSessionDeps): Promise<AppSession | null> {
  const state = parseSharedState(((await deps.send({ type: "review.state" })) as { state?: unknown } | null)?.state);
  if (!state) return null;
  const interfaceLanguage = sessionLanguage(state, deps.fallbackLanguage);
  deps.onState?.(state, interfaceLanguage);
  const port = deps.port(state.backup);
  await port.restore(state.backup);
  // Changes this app made that the extension has not taken yet: on top of an older state they are
  // the reader's latest; on a state that already holds them, applying them again changes nothing.
  const pending = parseAppOps(((await deps.send({ type: "review.pendingOps" })) as { ops?: unknown } | null)?.ops);
  if (pending) await applyAppOps(port, pending);
  const since = Math.min(pending?.since ?? Infinity, toSecond(deps.now()));

  // The day's counts as the extension shared them, with what earlier openings counted and the
  // extension has not taken yet: the statistics show them, and this opening counts from there.
  let earlier = pending?.daily ?? {};
  let opened = withReviewCounts(await sharedDailyStats(state.storeItems), ...Object.values(earlier));
  const openingId = deps.openingId ?? crypto.randomUUID();

  let items: Record<string, unknown> = {
    ...state.storeItems,
    // A copy: the counts are updated in place, and `opened` is what this opening counts from.
    [DAILY_KEY]: structuredClone(opened),
    [ROOT_KEY]: { v: STORAGE_VERSION, backup: await port.backup() },
  };
  const store: AsyncStorageArea = {
    get: async (keys) => pick(items, keys),
    async set(next) {
      items = { ...items, ...next };
      // After every answer (D3): the app may be closed at any moment, and nothing else would carry it.
      // The answer's count lands just after its backup, so a change of the counts is written too.
      if (ROOT_KEY in next || DAILY_KEY in next) {
        const counted = reviewCountsBetween(opened, await loadDailyStats(store));
        const ops = await appOpsSince(port, since);
        await deps.send({
          type: "review.writeOps",
          ops: encodeAppOps({ ...ops, daily: { ...earlier, [openingId]: counted } }),
        });
        deps.onSaved?.();
      }
    },
  };

  async function discardPending(erasedAt: number): Promise<void> {
    const kept = {
      v: pending?.v ?? 1,
      since,
      cardOps: (pending?.cardOps ?? []).filter((op) => op.client_ts > erasedAt),
      statusOps: (pending?.statusOps ?? []).filter((op) => op.updated_at > erasedAt),
    } as const;
    if (kept.cardOps.length > 0 || kept.statusOps.length > 0) await applyAppOps(port, kept);
    earlier = {};
    opened = await loadDailyStats(store);
    await store.set({ [ROOT_KEY]: { v: STORAGE_VERSION, backup: await port.backup() } });
  }

  // The interface language is the extension's, never one the app chose: it follows the reader's
  // native language, which only the extension changes.
  let own = readLocal(deps.local);
  const prefs: AsyncStorageArea = {
    get: async (keys) => pick({ ...state.prefs, ...own, [INTERFACE_LANGUAGE_KEY]: interfaceLanguage }, keys),
    async set(next) {
      own = { ...own, ...next };
      try {
        deps.local?.setItem(APP_PREFS_KEY, JSON.stringify(own));
      } catch {
        /* kept for this launch only */
      }
    },
  };
  return {
    port,
    store,
    prefs,
    savedAt: state.savedAt,
    interfaceLanguage,
    sharedLanguage: sharedInterfaceLanguage(state),
    onlineAccess: state.onlineAccess,
    sharedAccount: state.account,
    sharedErasedAt: state.erasedAt,
    since,
    pendingOps: async () => {
      const ops = await appOpsSince(port, since);
      return { cards: ops.cardOps, statuses: ops.statusOps };
    },
    discardPending,
  };
}
