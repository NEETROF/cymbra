import type { languages as fr } from "../fr/languages.ts";

// How the interface names the studied languages, in English — a draft after the French
// (src/i18n/README.md), settled by add-lingua-language-labels (change 19). English has no
// elision and no agreement: a language's forms are its name, except the one after « niveau ».

export const languages: typeof fr = {
  english: {
    name: "English",
    of: "of English",
    the: "English",
    masculine: "English",
    feminine: "English",
    windowsVoice: "English (United States)",
    preview: "This is how your pages will sound when Lingua reads them aloud.",
  },
  spanish: {
    name: "Spanish",
    of: "of Spanish",
    the: "Spanish",
    masculine: "Spanish",
    feminine: "Spanish",
    windowsVoice: "Spanish (Spain)",
    preview: "Así sonarán tus páginas cuando Lingua las lea en voz alta.",
  },
  levelTitle: (of) => `Level ${of}`,
  levelTitleEstimated: (of) => `Estimated level ${of}`,
  myLevelTitle: (of) => `My level ${of}`,
  myLevelTitleEstimated: (of) => `My estimated level ${of}`,
  estimatedLevelsNote: (the) =>
    `Levels estimated from word frequency, as no freely licensed CEFR list exists for ${the}.`,
  borrowedTypicalNote: (from, the) => `taken from ${from}, whose level sizes ${the} borrows.`,
  levelNameEstimated: (level) => `${level} (estimated)`,
  chooseLevelPrompt: (of) => `Choose your level ${of}`,
  noTextDetected: (masculine) => `No ${masculine} text detected on this page.`,
  noTextInYourLanguages: "No text in your languages detected on this page.",
  noVoiceInstalled: (feminine) => `No ${feminine} voice is installed on this device. `,
  levelQuestion: (of) => `What is your level ${of}?`,
};
