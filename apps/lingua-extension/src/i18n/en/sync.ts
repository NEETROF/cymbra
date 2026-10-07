import type { sync as fr } from "../fr/sync.ts";

// The sync block's copy in English — a draft after the French (src/i18n/README.md).

export const sync: typeof fr = {
  notSynced: "Not synced on this device yet.",
  syncedJustNow: "Synced just now.",
  syncedMinutesAgo: (n) => `Synced ${n} min. ago.`,
  syncedHoursAgo: (n) => `Synced ${n} h ago.`,
  lastSyncOn: (date) => `Last synced on ${date}.`,
  storageFull: "The extension's storage is full on this device — reset your local data in Settings.",
  unavailable: "Server unreachable — try again later.",
  sessionExpired: "Session expired — sign in again above, under Account.",
  conflict: "An operation is already running — try again in a moment.",
  failed: "Sync failed — try again later.",
};
