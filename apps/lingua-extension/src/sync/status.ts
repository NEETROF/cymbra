import { sync as frSync } from "../i18n/fr/sync.ts";
import { DEFAULT_INTERFACE_LANGUAGE, formatDate, type InterfaceLanguage } from "../i18n/index.ts";
import { settingsCopy, type SyncCopy } from "../reading/settings-copy.ts";
import type { AuthErrorKind } from "../state/auth-errors.ts";

// Réglages copy for synchronisation: when this device last synced, and why « Synchroniser
// maintenant » failed — always from a category, never from an error string. The words are the
// catalogue's `sync` module, handed by the settings view in the interface language, and the date
// past a day is that language's (localise-lingua-settings D4); French when neither is given.

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/**
 * « Synchronisé il y a 3 min. », relative to `now` — the raw count, as every language's message takes
 * it; a date past a day, as `language` writes it.
 */
export function lastSyncLabel(
  at: number | null,
  now: number,
  language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE,
  copy: SyncCopy = settingsCopy(language).sync,
): string {
  if (at === null) return copy.notSynced;
  const elapsed = Math.max(0, now - at);
  if (elapsed < MINUTE) return copy.syncedJustNow;
  if (elapsed < HOUR) return copy.syncedMinutesAgo(String(Math.floor(elapsed / MINUTE)));
  if (elapsed < DAY) return copy.syncedHoursAgo(String(Math.floor(elapsed / HOUR)));
  return copy.lastSyncOn(formatDate(language, new Date(at)));
}

/** Why « Synchroniser maintenant » failed, from its category, in `copy`'s language. */
export function syncErrorCopy(kind: AuthErrorKind | undefined, copy: SyncCopy = frSync): string {
  switch (kind) {
    case "storageFull":
      return copy.storageFull;
    case "unavailable":
      return copy.unavailable;
    case "unauthenticated":
      return copy.sessionExpired;
    case "conflict":
      return copy.conflict;
    default:
      return copy.failed;
  }
}
