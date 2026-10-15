// What the captures show, per interface language (add-site-lingua-showcase D4, D5): the reader's
// two studied languages, the page and the book of each, and the words the seed and the shots click.
// A word is a whole word of the fixture page; when a listed word is not highlighted (the pack moved
// and it is now under the reader's level), the next one is used, and the log says so.

/** The interface languages, in the order the tool runs them. */
export const INTERFACE_LANGUAGES = ["fr", "en", "es"];

/** The browser locale each run starts with: a new install presets its native language from it. */
export const LOCALE = { fr: "fr-FR", en: "en-GB", es: "es-ES" };

/**
 * The time zone every run reads its clock in — the tool's own process and the browser — so that a
 * day of statistics, a card's due day and the review's « today » are the same whoever runs it.
 */
export const TIME_ZONE = "Europe/Paris";

/** The two studied languages of each interface language's reader, the first the one the shots lead with. */
export const STUDIED = { fr: ["en", "es"], en: ["es", "fr"], es: ["en", "fr"] };

/** The reader's level in each studied language. */
export const LEVEL = "B1";

/**
 * The level some interface language's reader declares in its first studied language, in Réglages,
 * before the EPUB reader's captures (`reader`, `reader-eink`, the last ones): the English set reads
 * Galdós, whose first chapter a B1 reader of Spanish knows 82 % of — a page of highlights rather
 * than a page read; at C1, 92 %, as the other sets' Austen at B1.
 */
export const READER_LEVELS = { en: "C1" };

/**
 * Per studied language, the words of its fixture page:
 * - `card`: a verb form whose card names its tense — the hero's card (D5 `card`);
 * - `second`: the card the second studied language's capture opens (a noun, with its gender, or a
 *   verb), with a translation in the reader's language: a card without one fails the capture;
 * - `known`: the words the video marks « Je connais » (D8), one after the other until the pill's
 *   share rises — its « + Deck » goes to the `card` verb — left unknown by the seed, and on the
 *   page's first screen;
 * - `seedKnown` and `seedDeck`: what `--record-seed` marks known and adds to the deck (D4). The deck's
 *   first four are answered in the seed's short review (`SEED_REVIEW`), so the fifth is the first new
 *   card the review capture reveals: its gloss must pass `glossProblems` in every interface language
 *   that studies the page's language;
 * - `names`: the book's names and interjections the seed marks « Ignorer » in the EPUB reader, as a
 *   reader does, so the reader's capture does not highlight a character's name.
 */
export const WORDS = {
  en: {
    card: ["abandoned", "negotiated", "vanished"],
    second: ["footprint", "subsidies", "carriages"],
    known: ["carriages", "sleeper", "maintain", "bunk", "revival"],
    seedKnown: [
      "twentieth",
      "ordinary",
      "borders",
      "airlines",
      "railway",
      "services",
      "routes",
      "decades",
      "operators",
      "security",
      "distant",
      "climate",
      "sometimes",
      "season",
      "whether",
      "investment",
      "patience",
      "remove",
      "taxes",
      "rail",
    ],
    // `emissions` fifth: « Émission » and « Emisión », the review capture's card in fr and es.
    seedDeck: ["narrow", "queues", "charm", "cabins", "emissions", "overnight"],
    names: ["Bennet", "Bingley", "Morris", "Lizzy", "Oh", "Ah"],
  },
  es: {
    card: ["alargaban", "cruzaban", "prometieron", "despertarán"],
    second: ["huella", "litera", "subvenciones"],
    known: ["nocturno", "despacio", "madrugada", "renacimiento", "lejano"],
    seedKnown: [
      "silencioso",
      "viajeros",
      "billetes",
      "coste",
      "panorama",
      "operadores",
      "emisiones",
      "afirman",
      "notado",
      "suprimir",
      "fabricar",
      "tardan",
      "circulan",
      "pasajeros",
      "quejan",
      "retrasos",
      "suelen",
      "describen",
      "convierte",
      "dependerá",
    ],
    // `aerolíneas` fifth: "airline", the English set's review card.
    seedDeck: ["estrecha", "autopistas", "mercancías", "defecto", "aerolíneas", "amanecer", "compartimentos", "vagón"],
    names: ["Golfín", "Teodoro", "Socartes", "Choto", "Villafangosa", "Ah"],
  },
  fr: {
    card: ["allongeaient", "traversaient", "préféraient", "réveilleront"],
    // `empreinte` first: « huella », a noun with its gender and its translation in the Spanish set.
    second: ["empreinte", "couchette", "navette"],
    known: ["lenteur", "volontiers", "lointain", "renouveau", "matinal"],
    seedKnown: [
      "voyageurs",
      "dîner",
      "décennies",
      "opérateurs",
      "horaires",
      "passagers",
      "photographies",
      "patience",
      "ferroviaires",
      "trajets",
      "inquiétudes",
      "affirment",
      "livrer",
      "climatisation",
      "compartiments",
      "roues",
      "décrivent",
      "existé",
      "file",
      "liées",
    ],
    seedDeck: [
      "étroite",
      "marchandises",
      "panne",
      "aube",
      "constructeurs",
      "wagon-restaurant",
      "parenthèse",
      "plaignent",
    ],
    names: ["Phileas", "Fogg", "Passepartout", "Forster", "Byron", "Saville-row", "Burlington"],
  },
};

/**
 * Where one interface language's shots take other words than `WORDS`' (interface, then studied
 * language): the Spanish set's hero card on `abandoned` opens on a noun sense, « Despreocupación »;
 * the English set's « empreinte » explains itself in a parenthesis, where « couchette » reads
 * "sleeping berth".
 */
export const OVERRIDES = {
  es: { en: { card: ["vanished", "negotiated"] } },
  en: { fr: { second: ["couchette", "navette"] } },
};

/** The words the shots of `interfaceLanguage` use on `studied`'s page: `WORDS`, then `OVERRIDES`. */
export function wordsFor(interfaceLanguage, studied) {
  return { ...WORDS[studied], ...(OVERRIDES[interfaceLanguage]?.[studied] ?? {}) };
}

/** The book each studied language's reader shot opens (fixtures/books/<language>.txt). */
export const BOOK = { en: "en", es: "es", fr: "fr" };

/** How many cards `--record-seed` answers in its short review: « Su » but for one « Pas su ». */
export const SEED_REVIEW = ["recalled", "missed", "recalled", "recalled"];

/** The store captures, in their order: five, the Chrome Web Store's cap (D5). */
export const STORE_SHOTS = ["card", "second", "review", "stats", "reader"];

/** The site's shots: the store's five and the e-ink reader (open question 3: site only). */
export const SITE_SHOTS = [...STORE_SHOTS, "reader-eink"];

/** The longest gloss a capture shows on a card: a longer one is a dictionary's list, not a translation. */
export const GLOSS_LIMIT = 60;

/**
 * What is wrong with a gloss a capture would show (the revealed review card, the second page's
 * card): empty, longer than `GLOSS_LIMIT`, a parenthesis (the dictionary's own explanations, often
 * in English), or a first name's sense. Empty when it reads as a translation.
 * @param {string | null | undefined} gloss
 * @returns {string[]}
 */
export function glossProblems(gloss) {
  const text = (gloss ?? "").trim();
  if (!text) return ["no translation"];
  const problems = [];
  if (text.length > GLOSS_LIMIT) problems.push(`${text.length} characters, more than ${GLOSS_LIMIT}`);
  if (/[()]/.test(text)) problems.push("a parenthesis");
  if (/pr[ée]nom|first name|given name|nombre de pila|nombre propio/i.test(text)) problems.push("a first name");
  return problems;
}
