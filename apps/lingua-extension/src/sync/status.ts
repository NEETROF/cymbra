import type { AuthErrorKind } from "../state/auth-errors.ts";

// Réglages copy for synchronisation: when this device last synced, and why « Synchroniser
// maintenant » failed — always from a category, never from an error string.

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/** « Synchronisé il y a 3 min. », relative to `now`; a date past a day. */
export function lastSyncLabel(at: number | null, now: number): string {
  if (at === null) return "Pas encore synchronisé sur cet appareil.";
  const elapsed = Math.max(0, now - at);
  if (elapsed < MINUTE) return "Synchronisé à l'instant.";
  if (elapsed < HOUR) return `Synchronisé il y a ${Math.floor(elapsed / MINUTE)} min.`;
  if (elapsed < DAY) return `Synchronisé il y a ${Math.floor(elapsed / HOUR)} h.`;
  return `Dernière synchronisation le ${new Date(at).toLocaleDateString("fr-FR")}.`;
}

export function syncErrorCopy(kind: AuthErrorKind | undefined): string {
  switch (kind) {
    case "storageFull":
      return "La mémoire de l’extension est pleine sur cet appareil — réinitialise tes données locales dans Réglages.";
    case "unavailable":
      return "Serveur injoignable — réessaie plus tard.";
    case "unauthenticated":
      return "Session expirée — reconnecte-toi depuis le menu de l'extension.";
    case "conflict":
      return "Une opération est déjà en cours — réessaie dans un instant.";
    default:
      return "La synchronisation a échoué — réessaie plus tard.";
  }
}
