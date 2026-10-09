import type { languages as fr } from "../fr/languages.ts";

// How the interface names the studied languages, in Spanish — after the French
// (src/i18n/README.md; add-lingua-native-language-labels D1, the owner reviews it, M9). Spanish
// agrees in gender («voz inglesa») and elides nothing, but contracts («del inglés»); a text is
// «en inglés». The Windows menu path names the language as a Spanish Windows lists it.

/** The level scale's name in Spanish (D3, M19): the MCER, where French and English say CEFR. */
const levelScale = "MCER";

/**
 * « de » before a word carrying its article: « de el inglés » is « del inglés ». The module's own
 * grammar, for the one message that puts « de » before a language's `the`.
 */
function de(the: string): string {
  return the.startsWith("el ") ? `del ${the.slice(3)}` : `de ${the}`;
}

export const languages: typeof fr = {
  english: {
    name: "Inglés",
    of: "de inglés",
    the: "el inglés",
    masculine: "inglés",
    feminine: "inglesa",
    windowsVoice: "Inglés (Estados Unidos)",
    preview: "This is how your pages will sound when Lingua reads them aloud.",
  },
  spanish: {
    name: "Español",
    of: "de español",
    the: "el español",
    masculine: "español",
    feminine: "española",
    windowsVoice: "Español (España)",
    preview: "Así sonarán tus páginas cuando Lingua las lea en voz alta.",
  },
  french: {
    name: "Francés",
    of: "de francés",
    the: "el francés",
    masculine: "francés",
    feminine: "francesa",
    windowsVoice: "Francés (Francia)",
    preview: "Voici comment sonneront tes pages quand Lingua les lira à voix haute.",
  },
  levelScale,
  ownNames: { french: "Français", english: "English", spanish: "Español" },
  levelTitle: (of) => `Nivel ${of}`,
  levelTitleEstimated: (of) => `Nivel ${of} estimado`,
  myLevelTitle: (of) => `Mi nivel ${of}`,
  myLevelTitleEstimated: (of) => `Mi nivel ${of} estimado`,
  estimatedLevelsNote: (the) =>
    `Niveles estimados según la frecuencia de las palabras, a falta de una lista ${levelScale} de uso libre para ${the}.`,
  borrowedTypicalNote: (from, the) => `tomado ${de(from)}, cuyos tamaños de nivel retoma ${the}.`,
  levelNameEstimated: (level) => `${level} (estimado)`,
  chooseLevelPrompt: (of) => `Elige tu nivel ${of}`,
  noTextDetected: (masculine) => `No se detectó texto en ${masculine} en esta página.`,
  noTextInYourLanguages: "No se detectó texto en tus idiomas en esta página.",
  noVoiceInstalled: (feminine) => `No hay ninguna voz ${feminine} instalada en este dispositivo. `,
  levelQuestion: (of) => `¿Cuál es tu nivel ${of}?`,
};
