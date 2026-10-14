import type { AsyncStorageArea } from "../state/storage.ts";
import { type Consent, USAGE_COUNTS_OFF_KEY } from "./counts.ts";
import { DATA_COLLECTION } from "./messages.ts";

// Where each browser keeps the reader's choice about the anonymous counts (change:
// add-lingua-usage-counts, D6), read fresh each time the background needs it.

/** Chromium and Safari: the switch's key — `true` off, `false` on, absent unanswered. */
export async function switchConsent(area: AsyncStorageArea): Promise<Consent> {
  const got = await area.get(USAGE_COUNTS_OFF_KEY);
  const off = got[USAGE_COUNTS_OFF_KEY];
  return { kind: "switch", off: typeof off === "boolean" ? off : undefined };
}

/** What Firefox's `permissions.getAll()` answers: `data_collection` from Firefox 140 (Android 142). */
export interface AllPermissions {
  data_collection?: string[];
}

/** Firefox's permissions API, with its built-in data-collection consent. */
export interface FirefoxPermissions {
  getAll: () => Promise<AllPermissions>;
  request: (p: { data_collection: string[] }) => Promise<boolean>;
  remove: (p: { data_collection: string[] }) => Promise<boolean>;
}

/**
 * Firefox: its "technical and interaction data" consent, ticked by Firefox at installation and
 * changed by the reader there, in `about:addons` or with the switch. A Firefox whose answer has no
 * `data_collection` has no such consent, and counts nothing.
 */
export async function grantConsent(permissions: Pick<FirefoxPermissions, "getAll">): Promise<Consent> {
  try {
    const all = await permissions.getAll();
    if (!Array.isArray(all.data_collection)) return { kind: "grant", supported: false, granted: false };
    return { kind: "grant", supported: true, granted: all.data_collection.includes(DATA_COLLECTION) };
  } catch {
    return { kind: "grant", supported: false, granted: false };
  }
}

/** Asking and withdrawing Firefox's consent, for the background (usage/host.ts). */
export function dataCollectionOf(permissions: FirefoxPermissions): {
  request: () => Promise<boolean>;
  remove: () => Promise<boolean>;
} {
  return {
    request: () => permissions.request({ data_collection: [DATA_COLLECTION] }),
    remove: () => permissions.remove({ data_collection: [DATA_COLLECTION] }),
  };
}

/** Whether a `permissions.onAdded`/`onRemoved` event is about that consent. */
export function touchesDataCollection(change: unknown): boolean {
  const list = (change as AllPermissions | null)?.data_collection;
  return Array.isArray(list) && list.includes(DATA_COLLECTION);
}
