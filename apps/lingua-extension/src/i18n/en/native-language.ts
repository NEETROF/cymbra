import type { nativeLanguage as fr } from "../fr/native-language.ts";

// The native language's choice in English — a draft after the French (src/i18n/README.md).

export const nativeLanguage: typeof fr = {
  question: "I read in…",
  note: "The language of the interface and of word translations.",
  studiesAfter: (languages) => `You'll then study: ${languages}.`,
  confirm: "Confirm",
  failed: "The change couldn't be made. Try again.",
};
