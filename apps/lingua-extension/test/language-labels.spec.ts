import { describe, expect, it } from "vitest";
import {
  borrowedTypicalNote,
  chooseLevelPrompt,
  estimatedLevelsNote,
  isNamedLanguage,
  languageName,
  levelName,
  levelQuestion,
  levelScale,
  levelTitle,
  listedLanguageName,
  myLevelTitle,
  noTextDetected,
  noVoiceInstalled,
  previewSentence,
  readAsForm,
  windowsVoiceLanguage,
} from "@/analyzer/language-labels.ts";
import type { StudiedLanguage } from "@/analyzer/types.ts";
import { languages as enLanguages } from "@/i18n/en/languages.ts";
import { languages as esLanguages } from "@/i18n/es/languages.ts";
import { languages as frLanguages } from "@/i18n/fr/languages.ts";
import { INTERFACE_LANGUAGES, type InterfaceLanguage } from "@/i18n/index.ts";

// The labels module names a studied language in the interface language it is handed
// (add-lingua-native-language-labels D2): every message, in the three languages, for both studied
// languages. The French is what the module wrote before the catalogue, byte for byte (M23): the
// strings below are taken from that code, not from the French module.

const STUDIED: readonly StudiedLanguage[] = ["en", "es"];

/** Every message of the module, called for one interface language and one studied language. */
function every(language: InterfaceLanguage, studied: StudiedLanguage): Record<string, string> {
  const other: StudiedLanguage = studied === "en" ? "es" : "en";
  return {
    languageName: languageName(language, studied),
    levelScale: levelScale(language),
    levelTitle: levelTitle(language, studied),
    levelTitleEstimated: levelTitle(language, studied, true),
    myLevelTitle: myLevelTitle(language, studied),
    myLevelTitleEstimated: myLevelTitle(language, studied, true),
    estimatedLevelsNote: estimatedLevelsNote(language, studied),
    borrowedTypicalNote: borrowedTypicalNote(language, studied, other),
    levelName: levelName(language, "B1", false),
    levelNameEstimated: levelName(language, "B1", true),
    chooseLevelPrompt: chooseLevelPrompt(language, studied),
    noTextDetected: noTextDetected(language, [studied]),
    noTextInYourLanguages: noTextDetected(language, [studied, other]),
    noVoiceInstalled: noVoiceInstalled(language, studied),
    windowsVoiceLanguage: windowsVoiceLanguage(language, studied),
    previewSentence: previewSentence(language, studied),
    levelQuestion: levelQuestion(language, studied),
  };
}

describe("Every reader today: the French messages, byte for byte as the module wrote them", () => {
  it("names English and Spanish, with the grammar around them", () => {
    expect(every("fr", "en")).toEqual({
      languageName: "Anglais",
      levelScale: "CEFR",
      levelTitle: "Niveau d'anglais",
      levelTitleEstimated: "Niveau d'anglais estimé",
      myLevelTitle: "Mon niveau d'anglais",
      myLevelTitleEstimated: "Mon niveau d'anglais estimé",
      estimatedLevelsNote:
        "Niveaux estimés d'après la fréquence des mots, faute de liste CEFR libre de droits pour l'anglais.",
      borrowedTypicalNote: "repris de l'espagnol, dont l'anglais reprend les tailles de niveaux.",
      levelName: "B1",
      levelNameEstimated: "B1 (estimé)",
      chooseLevelPrompt: "Choisis ton niveau d'anglais",
      noTextDetected: "Pas de texte anglais détecté sur cette page.",
      noTextInYourLanguages: "Pas de texte dans tes langues détecté sur cette page.",
      noVoiceInstalled: "Aucune voix anglaise n'est installée sur cet appareil. ",
      windowsVoiceLanguage: "Anglais (États-Unis)",
      previewSentence: "This is how your pages will sound when Lingua reads them aloud.",
      levelQuestion: "Quel est ton niveau d'anglais ?",
    });
    expect(every("fr", "es")).toEqual({
      languageName: "Espagnol",
      levelScale: "CEFR",
      levelTitle: "Niveau d'espagnol",
      levelTitleEstimated: "Niveau d'espagnol estimé",
      myLevelTitle: "Mon niveau d'espagnol",
      myLevelTitleEstimated: "Mon niveau d'espagnol estimé",
      estimatedLevelsNote:
        "Niveaux estimés d'après la fréquence des mots, faute de liste CEFR libre de droits pour l'espagnol.",
      borrowedTypicalNote: "repris de l'anglais, dont l'espagnol reprend les tailles de niveaux.",
      levelName: "B1",
      levelNameEstimated: "B1 (estimé)",
      chooseLevelPrompt: "Choisis ton niveau d'espagnol",
      noTextDetected: "Pas de texte espagnol détecté sur cette page.",
      noTextInYourLanguages: "Pas de texte dans tes langues détecté sur cette page.",
      noVoiceInstalled: "Aucune voix espagnole n'est installée sur cet appareil. ",
      windowsVoiceLanguage: "Espagnol (Espagne)",
      previewSentence: "Así sonarán tus páginas cuando Lingua las lea en voz alta.",
      levelQuestion: "Quel est ton niveau d'espagnol ?",
    });
  });
});

describe("An English-native reader", () => {
  it("reads every message in English: no article, no agreement, the CEFR, the Windows voice with its region", () => {
    expect(every("en", "es")).toEqual({
      languageName: "Spanish",
      levelScale: "CEFR",
      levelTitle: "Spanish level",
      levelTitleEstimated: "Estimated Spanish level",
      myLevelTitle: "My Spanish level",
      myLevelTitleEstimated: "My estimated Spanish level",
      estimatedLevelsNote: "Levels estimated from word frequency, as no freely licensed CEFR list exists for Spanish.",
      borrowedTypicalNote: "taken from English, whose level sizes Spanish borrows.",
      levelName: "B1",
      levelNameEstimated: "B1 (estimated)",
      chooseLevelPrompt: "Choose your Spanish level",
      noTextDetected: "No Spanish text found on this page.",
      noTextInYourLanguages: "No text in your languages found on this page.",
      noVoiceInstalled: "No Spanish voice is installed on this device. ",
      windowsVoiceLanguage: "Spanish (Spain)",
      previewSentence: "Así sonarán tus páginas cuando Lingua las lea en voz alta.",
      levelQuestion: "What's your Spanish level?",
    });
    expect(every("en", "en")).toMatchObject({
      languageName: "English",
      levelTitle: "English level",
      noTextDetected: "No English text found on this page.",
      noVoiceInstalled: "No English voice is installed on this device. ",
      windowsVoiceLanguage: "English (United States)",
      levelQuestion: "What's your English level?",
    });
  });
});

describe("A Spanish-native reader", () => {
  it("reads every message in Spanish: « de inglés », « el inglés », « voz inglesa », the MCER", () => {
    expect(every("es", "en")).toEqual({
      languageName: "Inglés",
      levelScale: "MCER",
      levelTitle: "Nivel de inglés",
      levelTitleEstimated: "Nivel de inglés estimado",
      myLevelTitle: "Mi nivel de inglés",
      myLevelTitleEstimated: "Mi nivel de inglés estimado",
      estimatedLevelsNote:
        "Niveles estimados según la frecuencia de las palabras, a falta de una lista MCER de uso libre para el inglés.",
      borrowedTypicalNote: "tomado del español, cuyos tamaños de nivel retoma el inglés.",
      levelName: "B1",
      levelNameEstimated: "B1 (estimado)",
      chooseLevelPrompt: "Elige tu nivel de inglés",
      noTextDetected: "No se detectó texto en inglés en esta página.",
      noTextInYourLanguages: "No se detectó texto en tus idiomas en esta página.",
      noVoiceInstalled: "No hay ninguna voz inglesa instalada en este dispositivo. ",
      windowsVoiceLanguage: "Inglés (Estados Unidos)",
      previewSentence: "This is how your pages will sound when Lingua reads them aloud.",
      levelQuestion: "¿Cuál es tu nivel de inglés?",
    });
    expect(every("es", "es")).toMatchObject({
      languageName: "Español",
      levelTitleEstimated: "Nivel de español estimado",
      estimatedLevelsNote:
        "Niveles estimados según la frecuencia de las palabras, a falta de una lista MCER de uso libre para el español.",
      borrowedTypicalNote: "tomado del inglés, cuyos tamaños de nivel retoma el español.",
      noTextDetected: "No se detectó texto en español en esta página.",
      noVoiceInstalled: "No hay ninguna voz española instalada en este dispositivo. ",
      windowsVoiceLanguage: "Español (España)",
    });
  });
});

describe("every message, in every interface language, for every studied language", () => {
  for (const language of INTERFACE_LANGUAGES) {
    for (const studied of STUDIED) {
      it(`${language}, studying ${studied}: nothing empty, and nothing French outside French`, () => {
        const messages = every(language, studied);
        const french = every("fr", studied);
        for (const [key, text] of Object.entries(messages)) {
          expect(text.trim(), `${language}/${studied}: ${key}`).not.toBe("");
          // The preview is spoken in the studied language, the same whatever the interface; the
          // level alone is a bare « B1 »; the CEFR is the CEFR in English too.
          if (language === "fr" || ["previewSentence", "levelName", "levelScale"].includes(key)) continue;
          expect(text, `${language}/${studied}: ${key} is still the French`).not.toBe(french[key]);
        }
      });
    }
  }

  it("the preview sentence is the studied language's, whatever the interface language", () => {
    for (const language of INTERFACE_LANGUAGES) {
      expect(previewSentence(language, "en")).toBe(previewSentence("fr", "en"));
      expect(previewSentence(language, "es")).toBe(previewSentence("fr", "es"));
    }
  });
});

describe("French studied by readers of English and Spanish (enable-lingua-french D3)", () => {
  // Every message of a surface that names French, asks or shows its level or says its levels are
  // estimated — the labels of the change's design (*Measured*), which the catalogue held before it.
  const PREVIEW = "Voici comment sonneront tes pages quand Lingua les lira à voix haute.";

  it("an English-native reader reads every message in English: the estimated level, the note, the borrowed sizes", () => {
    expect(every("en", "fr")).toEqual({
      languageName: "French",
      levelScale: "CEFR",
      levelTitle: "French level",
      levelTitleEstimated: "Estimated French level",
      myLevelTitle: "My French level",
      myLevelTitleEstimated: "My estimated French level",
      estimatedLevelsNote: "Levels estimated from word frequency, as no freely licensed CEFR list exists for French.",
      borrowedTypicalNote: "taken from English, whose level sizes French borrows.",
      levelName: "B1",
      levelNameEstimated: "B1 (estimated)",
      chooseLevelPrompt: "Choose your French level",
      noTextDetected: "No French text found on this page.",
      noTextInYourLanguages: "No text in your languages found on this page.",
      noVoiceInstalled: "No French voice is installed on this device. ",
      windowsVoiceLanguage: "French (France)",
      previewSentence: PREVIEW,
      levelQuestion: "What's your French level?",
    });
  });

  it("a Spanish-native reader reads every message in Spanish: « de francés », « el francés », the MCER", () => {
    expect(every("es", "fr")).toEqual({
      languageName: "Francés",
      levelScale: "MCER",
      levelTitle: "Nivel de francés",
      levelTitleEstimated: "Nivel de francés estimado",
      myLevelTitle: "Mi nivel de francés",
      myLevelTitleEstimated: "Mi nivel de francés estimado",
      estimatedLevelsNote:
        "Niveles estimados según la frecuencia de las palabras, a falta de una lista MCER de uso libre para el francés.",
      borrowedTypicalNote: "tomado del inglés, cuyos tamaños de nivel retoma el francés.",
      levelName: "B1",
      levelNameEstimated: "B1 (estimado)",
      chooseLevelPrompt: "Elige tu nivel de francés",
      noTextDetected: "No se detectó texto en francés en esta página.",
      noTextInYourLanguages: "No se detectó texto en tus idiomas en esta página.",
      noVoiceInstalled: "No hay ninguna voz francesa instalada en este dispositivo. ",
      windowsVoiceLanguage: "Francés (Francia)",
      previewSentence: PREVIEW,
      levelQuestion: "¿Cuál es tu nivel de francés?",
    });
  });

  it("a reader of Spanish and French, or English and French, is told of their languages, neutral", () => {
    expect(noTextDetected("en", ["es", "fr"])).toBe("No text in your languages found on this page.");
    expect(noTextDetected("es", ["en", "fr"])).toBe("No se detectó texto en tus idiomas en esta página.");
  });
});

describe("French, for its voices (add-lingua-french-read-aloud D4)", () => {
  /** The four messages the read-aloud block calls, for a speaker reading French. */
  const voiceMessages = (language: InterfaceLanguage): Record<string, string> => ({
    languageName: languageName(language, "fr"),
    noVoiceInstalled: noVoiceInstalled(language, "fr"),
    windowsVoiceLanguage: windowsVoiceLanguage(language, "fr"),
    previewSentence: previewSentence(language, "fr"),
  });
  const PREVIEW = "Voici comment sonneront tes pages quand Lingua les lira à voix haute.";

  it("an English-native reader reads them in English", () => {
    expect(voiceMessages("en")).toEqual({
      languageName: "French",
      noVoiceInstalled: "No French voice is installed on this device. ",
      windowsVoiceLanguage: "French (France)",
      previewSentence: PREVIEW,
    });
  });

  it("a Spanish-native reader reads them in Spanish: « voz francesa », « Francés (Francia) »", () => {
    expect(voiceMessages("es")).toEqual({
      languageName: "Francés",
      noVoiceInstalled: "No hay ninguna voz francesa instalada en este dispositivo. ",
      windowsVoiceLanguage: "Francés (Francia)",
      previewSentence: PREVIEW,
    });
  });

  it("the French module holds them too, in French", () => {
    expect(voiceMessages("fr")).toEqual({
      languageName: "Français",
      noVoiceInstalled: "Aucune voix française n'est installée sur cet appareil. ",
      windowsVoiceLanguage: "Français (France)",
      previewSentence: PREVIEW,
    });
  });

  it("none of the English and Spanish ones is the French one, but the preview, the same in all three", () => {
    const french = voiceMessages("fr");
    for (const language of ["en", "es"] as const) {
      for (const [key, text] of Object.entries(voiceMessages(language))) {
        if (key === "previewSentence") expect(text).toBe(french[key]);
        else expect(text, `${language}: ${key} is still the French`).not.toBe(french[key]);
      }
    }
  });

  it("names the languages the catalogue holds, and no other tag", () => {
    expect(["en", "es", "fr"].every(isNamedLanguage)).toBe(true);
    expect(["de", "", "EN", "fra", "constructor", "toString"].some(isNamedLanguage)).toBe(false);
  });
});

describe("Italian, for its voices (add-lingua-italian-read-aloud D3)", () => {
  /** The four messages the read-aloud block calls, for a speaker reading Italian. */
  const voiceMessages = (language: InterfaceLanguage): Record<string, string> => ({
    languageName: languageName(language, "it"),
    noVoiceInstalled: noVoiceInstalled(language, "it"),
    windowsVoiceLanguage: windowsVoiceLanguage(language, "it"),
    previewSentence: previewSentence(language, "it"),
  });
  const PREVIEW = "Ecco come suoneranno le tue pagine quando Lingua le leggerà ad alta voce.";

  it("a French-native reader reads them in French: « voix italienne », « Italien (Italie) »", () => {
    expect(voiceMessages("fr")).toEqual({
      languageName: "Italien",
      noVoiceInstalled: "Aucune voix italienne n'est installée sur cet appareil. ",
      windowsVoiceLanguage: "Italien (Italie)",
      previewSentence: PREVIEW,
    });
  });

  it("an English-native reader reads them in English", () => {
    expect(voiceMessages("en")).toEqual({
      languageName: "Italian",
      noVoiceInstalled: "No Italian voice is installed on this device. ",
      windowsVoiceLanguage: "Italian (Italy)",
      previewSentence: PREVIEW,
    });
  });

  it("a Spanish-native reader reads them in Spanish: « voz italiana », « Italiano (Italia) »", () => {
    expect(voiceMessages("es")).toEqual({
      languageName: "Italiano",
      noVoiceInstalled: "No hay ninguna voz italiana instalada en este dispositivo. ",
      windowsVoiceLanguage: "Italiano (Italia)",
      previewSentence: PREVIEW,
    });
  });

  it("none of the English and Spanish ones is the French one, but the preview, the same in all three", () => {
    const french = voiceMessages("fr");
    for (const language of ["en", "es"] as const) {
      for (const [key, text] of Object.entries(voiceMessages(language))) {
        if (key === "previewSentence") expect(text).toBe(french[key]);
        else expect(text, `${language}: ${key} is still the French`).not.toBe(french[key]);
      }
    }
  });

  it("names Italian, a plain tag, and no other spelling of it", () => {
    expect(isNamedLanguage("it")).toBe(true);
    expect(["IT", "ita", "it-IT", "italian"].some(isNamedLanguage)).toBe(false);
    expect([readAsForm("fr", "it"), readAsForm("en", "it"), readAsForm("es", "it")]).toEqual([
      "italien",
      "Italian",
      "italiana",
    ]);
    expect([listedLanguageName("fr", "it"), listedLanguageName("en", "it"), listedLanguageName("es", "it")]).toEqual([
      "italien",
      "Italian",
      "italiano",
    ]);
  });

  it("holds the forms the level titles and the sentences take once Italian is studied", () => {
    // Keyed by the studied language, the titles name Italian only from enable-lingua-italian
    // (change 20): the templates are read here with the entry they will take.
    const fr = frLanguages;
    const { italian: frIt } = fr;
    expect([
      fr.levelTitle(frIt.of),
      fr.levelTitleEstimated(frIt.of),
      fr.myLevelTitle(frIt.of),
      fr.chooseLevelPrompt(frIt.of),
      fr.levelQuestion(frIt.of),
      fr.noTextDetected(frIt.masculine),
      fr.estimatedLevelsNote(frIt.the),
      fr.borrowedTypicalNote(fr.english.the, frIt.the),
    ]).toEqual([
      "Niveau d'italien",
      "Niveau d'italien estimé",
      "Mon niveau d'italien",
      "Choisis ton niveau d'italien",
      "Quel est ton niveau d'italien ?",
      "Pas de texte italien détecté sur cette page.",
      "Niveaux estimés d'après la fréquence des mots, faute de liste CEFR libre de droits pour l'italien.",
      "repris de l'anglais, dont l'italien reprend les tailles de niveaux.",
    ]);
    const en = enLanguages;
    expect([
      en.levelTitle(en.italian.of),
      en.noTextDetected(en.italian.masculine),
      en.estimatedLevelsNote(en.italian.the),
    ]).toEqual([
      "Italian level",
      "No Italian text found on this page.",
      "Levels estimated from word frequency, as no freely licensed CEFR list exists for Italian.",
    ]);
    const es = esLanguages;
    expect([
      es.levelTitle(es.italian.of),
      es.levelQuestion(es.italian.of),
      es.noTextDetected(es.italian.masculine),
      es.estimatedLevelsNote(es.italian.the),
      es.borrowedTypicalNote(es.italian.the, es.english.the),
    ]).toEqual([
      "Nivel de italiano",
      "¿Cuál es tu nivel de italiano?",
      "No se detectó texto en italiano en esta página.",
      "Niveles estimados según la frecuencia de las palabras, a falta de una lista MCER de uso libre para el italiano.",
      "tomado del italiano, cuyos tamaños de nivel retoma el inglés.",
    ]);
  });
});
