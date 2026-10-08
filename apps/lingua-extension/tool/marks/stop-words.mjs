// The stop words of the gloss experiment (../measure_marks.mjs, release-lingua-spanish-translation
// D5): the function words of the pair's NATIVE language — the language the pack's gloss and the
// translated sentence are both written in — skipped when the gloss's words are looked for in the
// sentence, since an article, a preposition or an auxiliary is in every sentence and would mark the
// wrong place. One set per native language (measure-lingua-translation-matrix-marks D1): the
// French set as en-fr and es-fr were measured with it, an English set and a Spanish set, each of
// articles, prepositions, conjunctions, pronouns and auxiliaries. The harness picks the set of the
// language its route translates into.
//
// Authored by category, so that test/translate-measure.spec.ts can hold each word to the committed
// tables (scripts/lingua-data/tables): a function word has a function-class reading in a senses
// table of its language, an auxiliary a verb's. The lookup folds case and accents, as glossMark
// folds the gloss's tokens, so « está » and « esta » are one entry once folded.

/** Case and accents removed: the form glossMark compares. */
export const fold = (s) => s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

/** The French set, word for word the one en-fr and es-fr were measured with (2026-10-05). */
export const FRENCH = Object.freeze({
  articles: "le la les un une des l",
  prepositions: "de du d à a en au aux pour par sur dans avec sans",
  conjunctions: "et ou que",
  pronouns: "qui que se sa son ses leur leurs ce cet cette ces",
  negation: "ne pas plus",
  auxiliaries: "est être avoir faire",
});

/** The English set: the native language of es-en. `s` is the possessive ’s, as the fold leaves it. */
export const ENGLISH = Object.freeze({
  articles: "the a an no s",
  prepositions:
    "of to in on at by for from with without into onto over under about as than between through during before after against among",
  conjunctions: "and or but nor so if that whether because while",
  pronouns:
    "it its he him she her they them his their we us our you your i me my this these those which who whom whose what",
  negation: "not",
  auxiliaries:
    "be am is are was were been being have has had having do does did done will would shall should can could may might must",
});

/** The Spanish set: the native language of en-es. */
export const SPANISH = Object.freeze({
  articles: "el la los las un una unos unas lo",
  prepositions: "de del a al en con sin por para sobre entre hacia desde hasta ante bajo tras según",
  conjunctions: "y e o u ni pero sino que si como cuando porque aunque",
  pronouns:
    "yo tú él ella ellos ellas nosotros usted ustedes me te se nos os le les mi mis tu tus su sus nuestro nuestra nuestros nuestras este esta estos estas ese esa esos esas esto eso aquel aquella aquello cuyo cuya",
  negation: "no",
  auxiliaries:
    "ser es son era eran fue fueron sido estar está están estaba estaban haber ha han había habían hay he hemos",
});

/** The sets by native language, as the catalogue names one (a model's `to`). */
export const STOP_WORDS = Object.freeze({ fr: FRENCH, en: ENGLISH, es: SPANISH });

/** Every word of a set, by category, as authored. */
export function wordsOf(set) {
  return Object.entries(set).flatMap(([category, words]) => words.split(" ").map((word) => ({ category, word })));
}

/** The stop words of `native`, folded for the lookup; a language without a set is refused. */
export function stopWords(native) {
  const set = STOP_WORDS[native];
  if (!set) {
    throw new Error(
      `no stop words for the native language "${native}": tool/marks/stop-words.mjs holds ${Object.keys(STOP_WORDS).join(", ")}`,
    );
  }
  return new Set(wordsOf(set).map(({ word }) => fold(word)));
}
