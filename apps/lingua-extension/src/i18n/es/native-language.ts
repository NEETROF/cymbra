import type { nativeLanguage as fr } from "../fr/native-language.ts";

// The native language's choice in Spanish — a draft after the French (src/i18n/README.md).

export const nativeLanguage: typeof fr = {
  question: "Leo en…",
  note: "El idioma de la interfaz y de las traducciones de palabras.",
  studiesAfter: (languages) => `Luego estudiarás ${languages}.`,
  confirm: "Confirmar",
  failed: "No se pudo hacer el cambio. Vuelve a intentarlo.",
};
