// The synchronisation block's copy (sync/status.ts), in French — the source module
// (add-lingua-interface-language). The elapsed counts are raw, with the full stop after « min »
// as the surface writes it (D4); the date as `formatDate` writes it.

export const sync = {
  notSynced: "Pas encore synchronisé sur cet appareil.",
  syncedJustNow: "Synchronisé à l'instant.",
  syncedMinutesAgo: (n: string) => `Synchronisé il y a ${n} min.`,
  syncedHoursAgo: (n: string) => `Synchronisé il y a ${n} h.`,
  lastSyncOn: (date: string) => `Dernière synchronisation le ${date}.`,
  storageFull:
    "La mémoire de l’extension est pleine sur cet appareil — réinitialise tes données locales dans Réglages.",
  unavailable: "Serveur injoignable — réessaie plus tard.",
  sessionExpired: "Session expirée — reconnecte-toi ci-dessus, dans Compte.",
  conflict: "Une opération est déjà en cours — réessaie dans un instant.",
  failed: "La synchronisation a échoué — réessaie plus tard.",
};
