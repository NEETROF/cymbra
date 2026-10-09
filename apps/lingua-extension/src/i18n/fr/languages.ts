// How the interface names the studied languages (analyzer/language-labels.ts), in French — the
// source module (add-lingua-interface-language, settled by add-lingua-native-language-labels D1).
// Each language's words come in the forms the sentences need; the sentences take them. The one
// place a language's name is written in French: test/lint-language-labels.spec.ts refuses it
// anywhere else.
//
// `french` is there for French's voices (add-lingua-french-read-aloud D4): the read-aloud block
// names a speaker's language before French is a studied language. A French-native reader cannot
// study French, so no French interface shows these words today; this module carries them because
// the English and Spanish ones are typed after it.

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
  /** The language with the region its voices default to, as Windows lists it in this interface language. */
  windowsVoice: string;
  /** What a voice preview reads, in the language itself — the same in every interface language. */
  preview: string;
}

/** The level scale's name in this interface language (D3): « CEFR » in French. */
const levelScale = "CEFR";

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
  french: {
    name: "Français",
    of: "de français",
    the: "le français",
    masculine: "français",
    feminine: "française",
    windowsVoice: "Français (France)",
    preview: "Voici comment sonneront tes pages quand Lingua les lira à voix haute.",
  } satisfies LanguageWords,
  levelScale,
  /**
   * The native languages the reader may choose, each named in its own language
   * (add-lingua-native-language-choice D4): the same in every interface language.
   */
  ownNames: { french: "Français", english: "English", spanish: "Español" },
  /** « Niveau d'anglais » — `of` is the language's. */
  levelTitle: (of: string) => `Niveau ${of}`,
  /** « Niveau d'espagnol estimé », when the pack's levels are estimated from word frequency. */
  levelTitleEstimated: (of: string) => `Niveau ${of} estimé`,
  myLevelTitle: (of: string) => `Mon niveau ${of}`,
  myLevelTitleEstimated: (of: string) => `Mon niveau ${of} estimé`,
  /** Why a language's levels read « estimé » — `the` is the language's; the scale is `levelScale`. */
  estimatedLevelsNote: (the: string) =>
    `Niveaux estimés d'après la fréquence des mots, faute de liste ${levelScale} libre de droits pour ${the}.`,
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
