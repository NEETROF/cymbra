import type { languages as fr } from "../fr/languages.ts";

// How the interface names the studied languages, in English — after the French
// (src/i18n/README.md; add-lingua-native-language-labels D1, the owner reviews it, M9). English has
// no elision and no agreement: a language's forms are all its bare name, and the titles put it
// before "level" ("English level"), where the French puts « d'anglais » after « niveau ». The
// Windows menu path names the language as an English Windows lists it.

/** The level scale's name in English (D3): the CEFR, as in French. */
const levelScale = "CEFR";

export const languages: typeof fr = {
  english: {
    name: "English",
    of: "English",
    the: "English",
    masculine: "English",
    feminine: "English",
    windowsVoice: "English (United States)",
    preview: "This is how your pages will sound when Lingua reads them aloud.",
  },
  spanish: {
    name: "Spanish",
    of: "Spanish",
    the: "Spanish",
    masculine: "Spanish",
    feminine: "Spanish",
    windowsVoice: "Spanish (Spain)",
    preview: "Así sonarán tus páginas cuando Lingua las lea en voz alta.",
  },
  levelScale,
  ownNames: { french: "Français", english: "English", spanish: "Español" },
  levelTitle: (of) => `${of} level`,
  levelTitleEstimated: (of) => `Estimated ${of} level`,
  myLevelTitle: (of) => `My ${of} level`,
  myLevelTitleEstimated: (of) => `My estimated ${of} level`,
  estimatedLevelsNote: (the) =>
    `Levels estimated from word frequency, as no freely licensed ${levelScale} list exists for ${the}.`,
  borrowedTypicalNote: (from, the) => `taken from ${from}, whose level sizes ${the} borrows.`,
  levelNameEstimated: (level) => `${level} (estimated)`,
  chooseLevelPrompt: (of) => `Choose your ${of} level`,
  noTextDetected: (masculine) => `No ${masculine} text found on this page.`,
  noTextInYourLanguages: "No text in your languages found on this page.",
  noVoiceInstalled: (feminine) => `No ${feminine} voice is installed on this device. `,
  levelQuestion: (of) => `What's your ${of} level?`,
};
