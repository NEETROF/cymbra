import type { languages as fr } from "../fr/languages.ts";

// How the interface names the studied languages, in Spanish — a draft after the French
// (src/i18n/README.md), settled by add-lingua-language-labels (change 19). Spanish agrees in
// gender («texto inglés», «voz inglesa») and elides nothing, but contracts.

/**
 * « de » before a word carrying its article: « de el inglés » is « del inglés ». The module's own
 * grammar, for the one message that puts « de » before a language's `the`; change 19 settles the
 * words table.
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
  levelTitle: (of) => `Nivel ${of}`,
  levelTitleEstimated: (of) => `Nivel ${of} estimado`,
  myLevelTitle: (of) => `Mi nivel ${of}`,
  myLevelTitleEstimated: (of) => `Mi nivel ${of} estimado`,
  estimatedLevelsNote: (the) =>
    `Niveles estimados según la frecuencia de las palabras, a falta de una lista MCER de uso libre para ${the}.`,
  borrowedTypicalNote: (from, the) => `tomado ${de(from)}, cuyos tamaños de nivel retoma ${the}.`,
  levelNameEstimated: (level) => `${level} (estimado)`,
  chooseLevelPrompt: (of) => `Elige tu nivel ${of}`,
  noTextDetected: (masculine) => `No se ha detectado texto ${masculine} en esta página.`,
  noTextInYourLanguages: "No se ha detectado texto en tus idiomas en esta página.",
  noVoiceInstalled: (feminine) => `No hay ninguna voz ${feminine} instalada en este dispositivo. `,
  levelQuestion: (of) => `¿Cuál es tu nivel ${of}?`,
};
