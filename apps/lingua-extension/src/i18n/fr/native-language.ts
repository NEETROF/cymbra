// The native language's choice (reading/native-language-view.ts), in French — the source module
// (add-lingua-native-language-choice D6). A module of its own, like `studied-languages`: Réglages,
// the onboarding page and the popup's first run mount the choice. The native languages are named
// in their own language by the `languages` module (`ownNames`), the same in every interface language;
// Réglages' block title is `settings.nativeLanguage`.

export const nativeLanguage = {
  /** The question heading the choice in the onboarding and in the popup's first run. */
  question: "Je lis en…",
  /** What changing it changes, under the native languages — the owner's wording (2026-10-11). */
  note: "La langue de l'interface et des traductions de mots sera modifiée.",
  confirm: "Confirmer",
  failed: "Le changement n'a pas pu être fait. Réessaie.",
};
