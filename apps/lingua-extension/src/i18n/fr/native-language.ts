// The native language's choice (reading/native-language-view.ts), in French — the source module
// (add-lingua-native-language-choice D6). A module of its own, like `studied-languages`: Réglages,
// the onboarding page and the popup's first run mount the choice. The native languages are named
// in their own language by the `languages` module (`ownNames`), the same in every interface language;
// Réglages' block title is `settings.nativeLanguage`.

export const nativeLanguage = {
  /** The question heading the choice in the onboarding and in the popup's first run. */
  question: "Je lis en…",
  /** What the choice sets, under the native languages. */
  note: "La langue de l'interface et des traductions de mots.",
  /** The consequence, said before confirming: `languages`, the studied languages after the change. */
  studiesAfter: (languages: string) => `Tu étudieras ensuite : ${languages}.`,
  confirm: "Confirmer",
  failed: "Le changement n'a pas pu être fait. Réessaie.",
};
