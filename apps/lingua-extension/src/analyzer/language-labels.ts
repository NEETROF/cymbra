import { languages as en } from "../i18n/en/languages.ts";
import { languages as es } from "../i18n/es/languages.ts";
import { languages as fr, type LanguageWords } from "../i18n/fr/languages.ts";
import type { InterfaceLanguage } from "../i18n/index.ts";
import type { NativeLanguage, StudiedLanguage } from "./types.ts";

// How the interface names a studied language, in the interface language (add-lingua-language-choice
// D1, add-lingua-native-language-labels D2): the module every surface calls, which holds no name of
// its own — each is the catalogue's `languages` module for the interface language
// (src/i18n/<language>/languages.ts), in that language's grammar. The surfaces pass the language
// they read with their preferences, first; test/lint-language-labels.spec.ts refuses a language's
// name anywhere but the catalogue's modules, so a surface always names the language it speaks of,
// in the language it speaks.

/** The catalogue's `languages` module by interface language: the one place that holds all three. */
const MODULES: Record<InterfaceLanguage, typeof fr> = { fr, en, es };

/**
 * The words of a studied language, in the interface language: the module's entry for it, keyed by
 * code so that a studied language the type gains and this map lacks fails to compile — and written
 * without a literal, which test/lint-language-labels.spec.ts would refuse here.
 */
function words(language: InterfaceLanguage, studied: StudiedLanguage): LanguageWords {
  const module = MODULES[language];
  const entries: Record<StudiedLanguage, LanguageWords> = { en: module.english, es: module.spanish };
  return entries[studied];
}

/** « Anglais » — "English", « Inglés ». */
export function languageName(language: InterfaceLanguage, studied: StudiedLanguage): string {
  return words(language, studied).name;
}

/** « l'anglais » — "English", « el inglés »: the language with its article, where a sentence takes it. */
export function languageWithArticle(language: InterfaceLanguage, studied: StudiedLanguage): string {
  return words(language, studied).the;
}

/**
 * A native language named in its own language — « Français », "English", « Español » — the same in
 * every interface language (add-lingua-native-language-choice D4): a reader finds their own language
 * whatever the interface speaks.
 */
export function nativeLanguageName(language: InterfaceLanguage, native: NativeLanguage): string {
  const names = MODULES[language].ownNames;
  const entries: Record<NativeLanguage, string> = { fr: names.french, en: names.english, es: names.spanish };
  return entries[native];
}

/** The level scale's name: « CEFR » in French and English, « MCER » in Spanish (D3, M19). */
export function levelScale(language: InterfaceLanguage): string {
  return MODULES[language].levelScale;
}

/**
 * « Niveau d'anglais » — « Niveau d'espagnol estimé » when the pack's levels are estimated from word
 * frequency (add-lingua-spanish-levels).
 */
export function levelTitle(language: InterfaceLanguage, studied: StudiedLanguage, estimated = false): string {
  const module = MODULES[language];
  const { of } = words(language, studied);
  return estimated ? module.levelTitleEstimated(of) : module.levelTitle(of);
}

/** « Mon niveau d'anglais » — « Mon niveau d'espagnol estimé » when the levels are estimated. */
export function myLevelTitle(language: InterfaceLanguage, studied: StudiedLanguage, estimated = false): string {
  const module = MODULES[language];
  const { of } = words(language, studied);
  return estimated ? module.myLevelTitleEstimated(of) : module.myLevelTitle(of);
}

/** Why a language's levels read « estimé »: no list of the scale can be shipped for it. */
export function estimatedLevelsNote(language: InterfaceLanguage, studied: StudiedLanguage): string {
  return MODULES[language].estimatedLevelsNote(words(language, studied).the);
}

/**
 * Where a ladder's « estimés » come from when another language's pack gave them: « repris de
 * l'anglais, dont l'espagnol reprend les tailles de niveaux. » (fix-lingua-spanish-ladder-estimates).
 */
export function borrowedTypicalNote(
  language: InterfaceLanguage,
  studied: StudiedLanguage,
  from: StudiedLanguage,
): string {
  return MODULES[language].borrowedTypicalNote(words(language, from).the, words(language, studied).the);
}

/** A level as a surface names it: « B1 », or « B1 (estimé) » when the levels are estimated. */
export function levelName(language: InterfaceLanguage, level: string, estimated: boolean): string {
  return estimated ? MODULES[language].levelNameEstimated(level) : level;
}

/** « Choisis ton niveau d'anglais ». */
export function chooseLevelPrompt(language: InterfaceLanguage, studied: StudiedLanguage): string {
  return MODULES[language].chooseLevelPrompt(words(language, studied).of);
}

/** « Pas de texte anglais détecté sur cette page. » — or neutral for a reader of several languages. */
export function noTextDetected(language: InterfaceLanguage, studied: readonly StudiedLanguage[]): string {
  const module = MODULES[language];
  return studied.length === 1
    ? module.noTextDetected(words(language, studied[0]).masculine)
    : module.noTextInYourLanguages;
}

/** « Aucune voix anglaise n'est installée sur cet appareil. » */
export function noVoiceInstalled(language: InterfaceLanguage, studied: StudiedLanguage): string {
  return MODULES[language].noVoiceInstalled(words(language, studied).feminine);
}

/** The Windows language to add for a voice on the device: « Anglais (États-Unis) », in the interface language. */
export function windowsVoiceLanguage(language: InterfaceLanguage, studied: StudiedLanguage): string {
  return words(language, studied).windowsVoice;
}

/** A sentence in the studied language, for a voice preview — the same whatever the interface language. */
export function previewSentence(language: InterfaceLanguage, studied: StudiedLanguage): string {
  return words(language, studied).preview;
}

/** « Quel est ton niveau d'anglais ? » */
export function levelQuestion(language: InterfaceLanguage, studied: StudiedLanguage): string {
  return MODULES[language].levelQuestion(words(language, studied).of);
}
