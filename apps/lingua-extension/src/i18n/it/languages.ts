import type { languages as fr } from "../fr/languages.ts";

// How the interface names the studied languages, in Italian — after the French
// (src/i18n/README.md; add-lingua-italian-interface D1, D4), reviewed by the Italian reviewer agent
// (I2). Italian agrees in gender where the adjective does (« voce spagnola », « voce inglese »),
// gives each language its article (« l'inglese », « lo spagnolo », « il francese », « l'italiano »)
// and contracts a preposition with it (« dall'inglese », « dallo spagnolo », « dal francese »). The
// Windows menu path names the language as an Italian Windows lists it (unverified, see settings.ts).

/** The level scale's name in Italian (D3, M19, I6): the QCER, where French and English say CEFR. */
const levelScale = "QCER";

/**
 * « da » before a word carrying its article: « da l'inglese » is « dall'inglese », « da lo spagnolo »
 * « dallo spagnolo », « da il francese » « dal francese ». The module's own grammar, as Spanish's
 * `de()`, for the one message that puts « da » before a language's `the`.
 */
function da(the: string): string {
  if (the.startsWith("l'")) return `dall'${the.slice(2)}`;
  if (the.startsWith("lo ")) return `dallo ${the.slice(3)}`;
  if (the.startsWith("il ")) return `dal ${the.slice(3)}`;
  if (the.startsWith("la ")) return `dalla ${the.slice(3)}`;
  return `da ${the}`;
}

export const languages: typeof fr = {
  english: {
    name: "Inglese",
    of: "di inglese",
    the: "l'inglese",
    masculine: "inglese",
    feminine: "inglese",
    windowsVoice: "Inglese (Stati Uniti)",
    preview: "This is how your pages will sound when Lingua reads them aloud.",
  },
  spanish: {
    name: "Spagnolo",
    of: "di spagnolo",
    the: "lo spagnolo",
    masculine: "spagnolo",
    feminine: "spagnola",
    windowsVoice: "Spagnolo (Spagna)",
    preview: "Así sonarán tus páginas cuando Lingua las lea en voz alta.",
  },
  french: {
    name: "Francese",
    of: "di francese",
    the: "il francese",
    masculine: "francese",
    feminine: "francese",
    windowsVoice: "Francese (Francia)",
    preview: "Voici comment sonneront tes pages quand Lingua les lira à voix haute.",
  },
  italian: {
    name: "Italiano",
    of: "di italiano",
    the: "l'italiano",
    masculine: "italiano",
    feminine: "italiana",
    windowsVoice: "Italiano (Italia)",
    preview: "Ecco come suoneranno le tue pagine quando Lingua le leggerà ad alta voce.",
  },
  levelScale,
  ownNames: { french: "Français", english: "English", spanish: "Español", italian: "Italiano" },
  levelTitle: (of) => `Livello ${of}`,
  levelTitleEstimated: (of) => `Livello ${of} stimato`,
  myLevelTitle: (of) => `Il mio livello ${of}`,
  myLevelTitleEstimated: (of) => `Il mio livello ${of} stimato`,
  estimatedLevelsNote: (the) =>
    `Livelli stimati in base alla frequenza delle parole, in mancanza di una lista ${levelScale} libera da diritti per ${the}.`,
  borrowedTypicalNote: (from, the) => `ripreso ${da(from)}, di cui ${the} adotta il numero di parole per livello.`,
  levelNameEstimated: (level) => `${level} (stimato)`,
  chooseLevelPrompt: (of) => `Scegli il tuo livello ${of}`,
  noTextDetected: (masculine) => `Nessun testo in ${masculine} rilevato in questa pagina.`,
  noTextInYourLanguages: "Nessun testo nelle tue lingue rilevato in questa pagina.",
  noVoiceInstalled: (feminine) => `Nessuna voce ${feminine} è installata su questo dispositivo. `,
  levelQuestion: (of) => `Qual è il tuo livello ${of}?`,
};
