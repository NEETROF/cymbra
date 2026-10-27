import type { sync as fr } from "../fr/sync.ts";

// The sync block's copy in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). The date is written without an article (« Ultima
// sincronizzazione: 04/10/2026 »), which would have to elide before some days (D4).

export const sync: typeof fr = {
  notSynced: "Non ancora sincronizzato su questo dispositivo.",
  syncedJustNow: "Sincronizzato proprio ora.",
  syncedMinutesAgo: (n) => `Sincronizzato ${n} min fa.`,
  syncedHoursAgo: (n) => `Sincronizzato ${n} h fa.`,
  lastSyncOn: (date) => `Ultima sincronizzazione: ${date}.`,
  storageFull:
    "La memoria dell'estensione è piena su questo dispositivo: reimposta i tuoi dati locali in Impostazioni.",
  unavailable: "Server irraggiungibile: riprova più tardi.",
  sessionExpired: "Sessione scaduta: accedi di nuovo qui sopra, in Account.",
  conflict: "C'è già un'operazione in corso: riprova tra un istante.",
  failed: "La sincronizzazione non è riuscita: riprova più tardi.",
};
