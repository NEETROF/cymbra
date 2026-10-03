import type { StudiedLanguage } from "./types.ts";

// How the interface names a studied language, in French (add-lingua-language-choice D1): the one
// place a language's name is written. test/lint-language-labels.spec.ts refuses it anywhere else,
// so a surface always names the language it speaks of.

interface LanguageWords {
  /** On its own, in a list: « Anglais ». */
  readonly name: string;
  /** After « niveau »: « d'anglais ». */
  readonly of: string;
  /** After a masculine noun: « texte anglais ». */
  readonly masculine: string;
  /** After a feminine noun: « voix anglaise ». */
  readonly feminine: string;
  /** The language with the region its voices default to, as Windows lists it (D5 of the programme for Spanish). */
  readonly windowsVoice: string;
  /** What a voice preview reads, in the language itself. */
  readonly preview: string;
}

const WORDS: Record<StudiedLanguage, LanguageWords> = {
  en: {
    name: "Anglais",
    of: "d'anglais",
    masculine: "anglais",
    feminine: "anglaise",
    windowsVoice: "Anglais (États-Unis)",
    preview: "This is how your pages will sound when Lingua reads them aloud.",
  },
  es: {
    name: "Espagnol",
    of: "d'espagnol",
    masculine: "espagnol",
    feminine: "espagnole",
    windowsVoice: "Espagnol (Espagne)",
    preview: "Así sonarán tus páginas cuando Lingua las lea en voz alta.",
  },
};

/** « Anglais ». */
export function languageName(language: StudiedLanguage): string {
  return WORDS[language].name;
}

/** « Niveau d'anglais ». */
export function levelTitle(language: StudiedLanguage): string {
  return `Niveau ${WORDS[language].of}`;
}

/** « Mon niveau d'anglais ». */
export function myLevelTitle(language: StudiedLanguage): string {
  return `Mon niveau ${WORDS[language].of}`;
}

/** « Choisis ton niveau d'anglais ». */
export function chooseLevelPrompt(language: StudiedLanguage): string {
  return `Choisis ton niveau ${WORDS[language].of}`;
}

/** « Pas de texte anglais détecté sur cette page. » — or neutral for a reader of several languages. */
export function noTextDetected(languages: readonly StudiedLanguage[]): string {
  return languages.length === 1
    ? `Pas de texte ${WORDS[languages[0]].masculine} détecté sur cette page.`
    : "Pas de texte dans tes langues détecté sur cette page.";
}

/** « Aucune voix anglaise n'est installée sur cet appareil. » */
export function noVoiceInstalled(language: StudiedLanguage): string {
  return `Aucune voix ${WORDS[language].feminine} n'est installée sur cet appareil. `;
}

/** The Windows language to add for a voice on the device: « Anglais (États-Unis) ». */
export function windowsVoiceLanguage(language: StudiedLanguage): string {
  return WORDS[language].windowsVoice;
}

/** A sentence in the language, for a voice preview. */
export function previewSentence(language: StudiedLanguage): string {
  return WORDS[language].preview;
}

/** « Quel est ton niveau d'anglais ? » */
export function levelQuestion(language: StudiedLanguage): string {
  return `Quel est ton niveau ${WORDS[language].of} ?`;
}
