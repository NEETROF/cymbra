// How the interface names the studied languages (analyzer/language-labels.ts), in French — the
// source module (add-lingua-interface-language). Each language's words come in the forms the
// sentences need; the sentences take them. The English and Spanish entries are drafted here and
// settled by add-lingua-language-labels (change 19).

/** One studied language, in the forms the sentences take. */
export interface LanguageWords {
  /** On its own, in a list: « Anglais ». */
  name: string;
  /** After « niveau »: « d'anglais ». */
  of: string;
  /** With its article: « l'anglais ». */
  the: string;
  /** After a masculine noun: « texte anglais ». */
  masculine: string;
  /** After a feminine noun: « voix anglaise ». */
  feminine: string;
  /** The language with the region its voices default to, as Windows lists it. */
  windowsVoice: string;
  /** What a voice preview reads, in the language itself — the same in every interface language. */
  preview: string;
}

export const languages = {
  english: {
    name: "Anglais",
    of: "d'anglais",
    the: "l'anglais",
    masculine: "anglais",
    feminine: "anglaise",
    windowsVoice: "Anglais (États-Unis)",
    preview: "This is how your pages will sound when Lingua reads them aloud.",
  } satisfies LanguageWords,
  spanish: {
    name: "Espagnol",
    of: "d'espagnol",
    the: "l'espagnol",
    masculine: "espagnol",
    feminine: "espagnole",
    windowsVoice: "Espagnol (Espagne)",
    preview: "Así sonarán tus páginas cuando Lingua las lea en voz alta.",
  } satisfies LanguageWords,
  /** « Niveau d'anglais » — `of` is the language's. */
  levelTitle: (of: string) => `Niveau ${of}`,
  /** « Niveau d'espagnol estimé », when the pack's levels are estimated from word frequency. */
  levelTitleEstimated: (of: string) => `Niveau ${of} estimé`,
  myLevelTitle: (of: string) => `Mon niveau ${of}`,
  myLevelTitleEstimated: (of: string) => `Mon niveau ${of} estimé`,
  /** Why a language's levels read « estimé » — `the` is the language's. */
  estimatedLevelsNote: (the: string) =>
    `Niveaux estimés d'après la fréquence des mots, faute de liste CEFR libre de droits pour ${the}.`,
  /** « repris de l'anglais, dont l'espagnol reprend les tailles de niveaux. » */
  borrowedTypicalNote: (from: string, the: string) => `repris de ${from}, dont ${the} reprend les tailles de niveaux.`,
  /** « B1 (estimé) ». */
  levelNameEstimated: (level: string) => `${level} (estimé)`,
  chooseLevelPrompt: (of: string) => `Choisis ton niveau ${of}`,
  /** « Pas de texte anglais détecté sur cette page. » — `masculine` is the language's. */
  noTextDetected: (masculine: string) => `Pas de texte ${masculine} détecté sur cette page.`,
  noTextInYourLanguages: "Pas de texte dans tes langues détecté sur cette page.",
  /** « Aucune voix anglaise n'est installée sur cet appareil. » — `feminine` is the language's. */
  noVoiceInstalled: (feminine: string) => `Aucune voix ${feminine} n'est installée sur cet appareil. `,
  levelQuestion: (of: string) => `Quel est ton niveau ${of} ?`,
};
