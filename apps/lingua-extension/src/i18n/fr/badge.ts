import { pluralForms } from "../index.ts";

// The toolbar icon's title (state/badge.ts), in French — the source module
// (add-lingua-comeback-hints D5). The badge itself carries a figure only; the title says it in
// words, naming each language when several hold cards. A count keeps « carte(s) » in every form,
// as the review's does.

export const badge = {
  /** Nothing to review, or the count turned off: the extension's name alone. */
  name: "Cymbra Lingua",
  /** « Cymbra Lingua — 12 carte(s) à revoir »: one language holds cards. */
  cards: pluralForms({
    one: (n) => `Cymbra Lingua — ${n} carte(s) à revoir`,
    other: (n) => `Cymbra Lingua — ${n} carte(s) à revoir`,
  }),
  /** « Cymbra Lingua — à revoir : anglais 8, espagnol 4 »: several languages hold cards. */
  perLanguage: (list: string) => `Cymbra Lingua — à revoir : ${list}`,
  /** One language of that list: « anglais 8 ». */
  languageCount: (language: string, n: string) => `${language} ${n}`,
  /** Between two languages of the list. */
  listSeparator: ", ",
};
