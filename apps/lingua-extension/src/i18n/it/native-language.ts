import type { nativeLanguage as fr } from "../fr/native-language.ts";

// The native language's choice in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1).

export const nativeLanguage: typeof fr = {
  question: "Leggo in…",
  note: "Verrà modificata la lingua dell'interfaccia e delle traduzioni delle parole.",
  confirm: "Conferma",
  failed: "Non è stato possibile effettuare la modifica. Riprova.",
};
