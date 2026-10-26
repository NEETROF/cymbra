import type { translation as fr } from "../fr/translation.ts";

// The translation setting's copy in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). A size comes formatted (« 25,8 ») and takes its unit here.

export const translation: typeof fr = {
  toggle: "Traduzione estesa",
  attribution: "Modello di traduzione: Firefox Translations (Mozilla), licenza MPL 2.0.",
  ready: "Pronta: le tue selezioni vengono tradotte su questo dispositivo.",
  removed: "Il browser ha eliminato il modello da questo dispositivo. Bisogna scaricarlo di nuovo.",
  missing: "Manca un modello per una delle tue lingue.",
  cancel: "Annulla",
  download: "Scarica",
  retry: "Riprova",
  resume: "Riprendi",
  again: "Scarica di nuovo",
  megabytes: (amount) => `${amount} MB`,
  theModel: "il modello",
  memoryPivot: "circa 340 MB",
  memorySingle: "circa 200 MB",
  cost: (download, memory) =>
    `Traduce le tue frasi su questo dispositivo, senza inviare nulla. Scarica ${download} una volta, poi usa ` +
    `${memory} di memoria durante la traduzione. Impostazione propria di questo dispositivo.`,
  failedNetwork: "Il download non è riuscito: nessuna connessione. Riprova quando sei online.",
  failedUnavailable: "Il download non è riuscito: il server non risponde. Riprova più tardi.",
  failedNotTheModel: "Il download non è riuscito: il file ricevuto non è il modello giusto. Riprova più tardi.",
  failedStorageSized: (size) => `Spazio insufficiente su questo dispositivo per il modello (${size}).`,
  failedStorage: "Spazio insufficiente su questo dispositivo per il modello.",
  failedUnknown: "Il download non è riuscito. Riprova più tardi.",
  progress: (received, total) => ` ${received} di ${total}`,
  downloading: (progress) => `Download del modello…${progress}`,
  interruptedAt: (progress) => `Download interrotto.${progress}`,
  missingSized: (size) => `Manca un modello per una delle tue lingue. ${size} da scaricare.`,
};
