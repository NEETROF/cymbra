import type { popup as fr } from "../fr/popup.ts";

// The toolbar popup's copy in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). « Connecté » is worded without a gender (D4).

export const popup: typeof fr = {
  title: "Cymbra Lingua",
  settings: "Impostazioni",
  back: "Indietro",
  sessionLost: "Sessione scaduta: accedi di nuovo per sincronizzare. Le tue parole restano su questo dispositivo.",
  highlightingOn: "Evidenziazione attiva",
  highlightingOff: "Evidenziazione disattivata",
  highlightingOffNote: "Evidenziazione disattivata su tutte le pagine.",
  notAnalysed: "Questa pagina non è ancora stata analizzata.",
  analyse: "Analizza questa pagina",
  alwaysHighlight: "Evidenzia sempre (tutte le pagine)",
  noPercent: "—",
  knownOnPage: "di parole conosciute in questa pagina",
  knownInChapter: "di parole conosciute in questo capitolo",
  wordsAnalysed: "Parole analizzate",
  wordsUnknown: "Parole sconosciute",
  distinctUnknown: "di cui parole diverse",
  wordsTracked: "Parole seguite",
  chooseLevel: "Scegli il tuo livello",
  levelLine: (title, level) => `${title}: ${level}`,
  edit: "Modifica",
  deck: "Mazzo (in apprendimento)",
  review: (due) => `Ripassa (${due})`,
  signInToSync: "Accedi per sincronizzare",
  signedIn: "Accesso effettuato",
  accountAndSync: "Account e sincronizzazione",
  library: "Biblioteca (libri EPUB)",
  statistics: "Statistiche di apprendimento",
  openBookForFigures: "Apri un libro della tua biblioteca per vederne i numeri.",
  handleToChoose: "Nome utente da scegliere",
  syncOn: "Sincronizzazione attiva",
  beginner: "Principiante",
  handle: (handle) => `@${handle}`,
  // — The anonymous counts' first-run sentence (add-lingua-usage-counts D7) —
  usageInform:
    "Lingua conta in forma anonima alcuni passaggi (installazione, primi passi, ritorni), senza alcun identificativo; nulla di ciò che leggi lascia il tuo dispositivo. Puoi disattivare questi conteggi in Impostazioni → Dati.",
  usageAsk:
    "Permetti a Lingua di contare in forma anonima alcuni passaggi (installazione, primi passi, ritorni), senza alcun identificativo? Nulla di ciò che leggi lascia il tuo dispositivo, e potrai cambiare idea in Impostazioni → Dati.",
  usageYes: "Sì, conta",
  usageNo: "No, grazie",
  usageYesDone: "Grazie! Puoi disattivare questi conteggi in qualsiasi momento in Impostazioni → Dati.",
  usageNoDone: "Fatto: non verrà contato nulla.",
  usagePrivacy: "Informativa sulla privacy",
};
