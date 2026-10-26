import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import coverage from "../src/data/lingua-coverage.json";
import { type LinguaPair, linguaPairs, shippedPairs, type Coverage, type Routes } from "../src/lib/lingua-pairs";
import {
  type CardKey,
  englishHomeLinguaCard,
  esAnd,
  esOr,
  fill,
  frenchHomeLinguaCard,
  italianHomeLinguaCard,
  LINGUA_TEXT,
  type LinguaPageText,
  linguaPageText,
  SHOT_KEYS,
  spanishHomeLinguaCard,
  STUDIED,
  studiedLanguage,
} from "../src/lib/lingua-text";
import { EIGHT_PAIRS, MATRIX, NINE_PAIRS, ROUTES, SEVEN_PAIRS, SIX_PAIRS, TODAY, WITH_FR_EN } from "./support/lingua";
import { STUDIED as CAPTURED } from "../../lingua-extension/tool/capture/plan.mjs";

// The words around the data (change: add-site-lingua-matrix-pages, D2 and D4), built from
// committed pair lists (`test/support/lingua.ts`): today's two French-native pairs read as
// the committed pages did; the matrix (es-en and en-es beside them) leads each page with its
// readers' pairs. The French bytes themselves are pinned on the rendered page by
// `test/astro/lingua-page.spec.ts` and on the built one by `test/post-build/lingua.spec.ts`.
// The last block alone reads the live pairs, and the extension's card catalogues: the buttons
// each page quotes, and the English card change 34 handed over.

/** U+202F, the narrow no-break space: French and Spanish thousands, the Spanish per-cent sign. */
const NNBSP = "\u202F";
/** U+00A0, the no-break space `Intl` puts before the French per-cent sign. */
const NBSP = "\u00A0";

const shipped = (data: Coverage, routes: Routes = ROUTES) => shippedPairs(data, routes);
const today = shipped(TODAY);
const matrix = shipped(MATRIX);
const withFrEn = shipped(WITH_FR_EN);
const six = shipped(SIX_PAIRS);
const seven = shipped(SEVEN_PAIRS);
const eight = shipped(EIGHT_PAIRS);
const nine = shipped(NINE_PAIRS);
const discord = { discordUrl: "https://discord.gg/example" };
const noDiscord = { discordUrl: null };
const card = (t: LinguaPageText, key: CardKey) => t.cards.find((c) => c.key === key)!;

describe("today's pairs: the pages read as before", () => {
  it("French: the tagline names English and Spanish, the hero note has no audience sentence", () => {
    const t = linguaPageText("fr", today, discord);
    expect(t.tagline).toBe(
      "Une extension navigateur qui surligne les mots d'anglais ou d'espagnol que vous ne connaissez pas encore, directement sur la page. Pas d'appli à ouvrir, pas de session de révision à caler : vous lisez, vous apprenez.",
    );
    expect(t.heroNote).toBe(
      'Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur et Android) et sur Safari (iPhone, iPad, Mac). <a href="https://discord.gg/example" rel="noopener">Rejoignez la communauté</a> pour en parler.',
    );
    expect(linguaPageText("fr", today, noDiscord).heroNote).toBe(
      "Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur et Android) et sur Safari (iPhone, iPad, Mac).",
    );
    expect(t.cards.map((c) => c.key)).toEqual(["highlight", "percentage", "click", "deck", "offline", "level", "account", "languages"]);
    expect(card(t, "languages")).toEqual({
      key: "languages",
      title: "🧭 Anglais et espagnol → français",
      body: "Choisissez vos langues dans les Réglages : chaque page est lue dans la sienne.",
    });
    // The Spanish pack's card and levels sentences, since a listed pair studies Spanish — the
    // levels' as before but for the scale's name: « CEFR », as this page's level card and the
    // extension's French interface name it (change: add-lingua-french-listings, D7).
    expect(card(t, "click").body).toMatch(/« Ignorer »\. En espagnol, la carte nomme aussi le temps et le genre\.$/);
    expect(card(t, "level")).toEqual({
      key: "level",
      title: "📈 Votre niveau, en CEFR",
      body: "Une estimation de votre vocabulaire, adossée à l'échelle A1→C2 plutôt qu'à un badge maison. Pour l'espagnol, les niveaux sont estimés d'après la fréquence des mots, faute de liste CEFR libre de droits.",
    });
    expect(JSON.stringify(t)).not.toContain("CECR");
  });

  it("French: the coverage table names the studied languages and the note says what is translated", () => {
    const { coverage: table } = linguaPageText("fr", today, noDiscord);
    expect(table.caption).toBe(
      "La part des mots les plus courants qui ont une définition en français, mesurée de la même façon pour chaque langue, sur les dictionnaires livrés avec l'extension.",
    );
    expect(table.perPair).toBe(false);
    expect(table.head).toEqual(["Mots les plus courants", "Anglais", "Espagnol"]);
    expect(table.rows).toEqual([
      [`5${NNBSP}000`, `95${NBSP}%`, `88${NBSP}%`],
      [`10${NNBSP}000`, `90${NBSP}%`, `77${NBSP}%`],
      [`20${NNBSP}000`, `79${NBSP}%`, `64${NBSP}%`],
    ]);
    expect(table.note).toBe(
      "Les définitions viennent du Wiktionnaire et de traductions écrites par des personnes, jamais d'une traduction automatique. La traduction étendue, facultative, traduit l'anglais et l'espagnol (en passant par l'anglais).",
    );
  });

  it("English: today's sentence that the product is made for French speakers, not 'others'", () => {
    const t = linguaPageText("en", today, discord);
    expect(t.description).toBe(
      "A browser extension that highlights, on the page you are reading, the English or Spanish words you do not know yet. Offline analysis, no account required.",
    );
    expect(t.heroNote).toBe(
      'Available on Chrome, Edge and other Chromium browsers, on Firefox (desktop and Android) and on Safari (iPhone, iPad, Mac). Made for French speakers learning English or Spanish: the interface and the translations are in French. <a href="https://discord.gg/example" rel="noopener">Join the community</a> to talk about it.',
    );
    expect(card(t, "languages").title).toBe("🧭 English and Spanish → French");
    expect(card(t, "languages").body).toBe("Choose your languages in Settings: each page is read in its own.");
    expect(t.coverage.caption).toMatch(/^The share of the commonest words that have a French gloss, measured the same way for each language/);
    expect(t.coverage.head).toEqual(["Commonest words", "English", "Spanish"]);
    expect(t.coverage.rows[0]).toEqual(["5,000", "95%", "88%"]);
    expect(t.coverage.note).toMatch(/ Extended translation, optional, serves English and Spanish \(through English\)\.$/);
    expect(t.closing.body).toContain('<a href="/en/music">Cymbra Music</a>');
  });

  it("takes its rows from the tops it is given", () => {
    const two = shipped({ tops: [5000, 10000], glossed: { "en-fr": [95.1, 90.1], "es-fr": [87.6, 77.2] } });
    expect(linguaPageText("en", two, noDiscord).coverage.rows).toEqual([
      ["5,000", "95%", "88%"],
      ["10,000", "90%", "77%"],
    ]);
  });
});

describe("the matrix: each page leads with its readers' pairs (D2)", () => {
  it("English: Spanish for English speakers first, then the pairs for French speakers, then for Spanish speakers", () => {
    const t = linguaPageText("en", matrix, noDiscord);
    expect(t.tagline).toMatch(/^A browser extension that highlights the Spanish words you do not know yet/);
    expect(t.heroNote).toBe(
      "Available on Chrome, Edge and other Chromium browsers, on Firefox (desktop and Android) and on Safari (iPhone, iPad, Mac). Made for English speakers learning Spanish. Also for French speakers learning English or Spanish: the interface and the translations are in French. Also for Spanish speakers learning English: the interface and the translations are in Spanish.",
    );
    expect(card(t, "languages").title).toBe("🧭 Spanish → English · English and Spanish → French · English → Spanish");
    // Two native languages listed: one row per pair, the readers' first, one column per top
    // (change: add-lingua-french-listings, D8).
    expect(t.coverage.perPair).toBe(true);
    expect(t.coverage.head).toEqual(["Commonest words", "5,000", "10,000", "20,000"]);
    expect(t.coverage.rows).toEqual([
      ["Spanish → English", "90%", "80%", "70%"],
      ["English → French", "95%", "90%", "79%"],
      ["Spanish → French", "88%", "77%", "64%"],
      ["English → Spanish", "96%", "91%", "80%"],
    ]);
  });

  it("Spanish: English for Spanish speakers first, in Spanish (tú, RAE numbers)", () => {
    const t = linguaPageText("es", matrix, discord);
    expect(t.title).toBe("Cymbra Lingua — amplía tu vocabulario leyendo la web");
    expect(t.tagline).toBe(
      "Una extensión para el navegador que resalta las palabras en inglés que todavía no conoces, directamente en la página. Sin aplicación que abrir ni sesión de repaso que programar: lees y aprendes.",
    );
    expect(t.heroNote).toBe(
      'Disponible en Chrome, Edge y otros navegadores Chromium, en Firefox (escritorio y Android) y en Safari (iPhone, iPad, Mac). Pensada para hispanohablantes que aprenden inglés. También para francohablantes que aprenden inglés o español: la interfaz y las traducciones están en francés. También para anglohablantes que aprenden español: la interfaz y las traducciones están en inglés. <a href="https://discord.gg/example" rel="noopener">Únete a la comunidad</a> para hablar de ello.',
    );
    expect(card(t, "languages").title).toBe("🧭 Inglés → español · Inglés y español → francés · Español → inglés");
    expect(card(t, "languages").body).toBe("Elige tus idiomas en los Ajustes: cada página se lee en el suyo.");
    // es-fr and es-en study Spanish: the card's and the levels' sentences, in the extension's words.
    expect(card(t, "click").body).toMatch(/e «Ignorar»\. En español, la tarjeta también indica el tiempo verbal y el género\.$/);
    expect(card(t, "level").body).toMatch(/a falta de una lista MCER de uso libre\.$/);
    // RAE: 5000 solid, 10 000 and 20 000 with a narrow no-break space, and one before the
    // per-cent sign — the extension's `formatNumber` and `formatPercent`. One row per pair (D8).
    expect(t.coverage.head).toEqual(["Palabras más frecuentes", "5000", `10${NNBSP}000`, `20${NNBSP}000`]);
    expect(t.coverage.rows).toEqual([
      ["Inglés → español", `96${NNBSP}%`, `91${NNBSP}%`, `80${NNBSP}%`],
      ["Inglés → francés", `95${NNBSP}%`, `90${NNBSP}%`, `79${NNBSP}%`],
      ["Español → francés", `88${NNBSP}%`, `77${NNBSP}%`, `64${NNBSP}%`],
      ["Español → inglés", `90${NNBSP}%`, `80${NNBSP}%`, `70${NNBSP}%`],
    ]);
    // Its closing sends its readers to the Spanish Music page (change: extend-site-spanish-locale, D5).
    expect(t.closing.body).toContain('<a href="/es/music">Cymbra Music</a>');
  });

  it("French: the French-native pairs first, the others named for their readers", () => {
    const t = linguaPageText("fr", matrix, noDiscord);
    expect(t.heroNote).toBe(
      "Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur et Android) et sur Safari (iPhone, iPad, Mac). Conçue pour les francophones qui apprennent l'anglais ou l'espagnol. Aussi pour les anglophones qui apprennent l'espagnol : l'interface et les traductions sont en anglais. Aussi pour les hispanophones qui apprennent l'anglais : l'interface et les traductions sont en espagnol.",
    );
    expect(card(t, "languages").title).toBe("🧭 Anglais et espagnol → français · Espagnol → anglais · Anglais → espagnol");
    expect(t.coverage.head).toEqual(["Mots les plus courants", `5${NNBSP}000`, `10${NNBSP}000`, `20${NNBSP}000`]);
    expect(t.coverage.rows).toEqual([
      ["Anglais → français", `95${NBSP}%`, `90${NBSP}%`, `79${NBSP}%`],
      ["Espagnol → français", `88${NBSP}%`, `77${NBSP}%`, `64${NBSP}%`],
      ["Espagnol → anglais", `90${NBSP}%`, `80${NBSP}%`, `70${NBSP}%`],
      ["Anglais → espagnol", `96${NBSP}%`, `91${NBSP}%`, `80${NBSP}%`],
    ]);
  });

  it("a page whose only pairs are its readers' carries no audience sentence, like today's French page", () => {
    const enEs = shipped({ tops: MATRIX.tops, glossed: { "en-es": [96.0, 91.0, 80.0] } });
    const t = linguaPageText("es", enEs, noDiscord);
    expect(t.heroNote).toBe(
      "Disponible en Chrome, Edge y otros navegadores Chromium, en Firefox (escritorio y Android) y en Safari (iPhone, iPad, Mac).",
    );
    expect(card(t, "languages").title).toBe("🧭 Inglés → español");
    // No listed pair studies Spanish: the Spanish pack's sentences stay out.
    expect(card(t, "click").body).toMatch(/e «Ignorar»\.$/);
    expect(card(t, "level").body).toMatch(/insignia de la casa\.$/);
    expect(t.coverage.caption).toBe(
      "La proporción de las palabras más frecuentes que tienen una definición en español, medida de la misma manera para cada idioma, en los diccionarios que se entregan con la extensión.",
    );
    expect(t.coverage.perPair).toBe(false);
    expect(t.coverage.head).toEqual(["Palabras más frecuentes", "Inglés"]);
    expect(t.coverage.note).toBe(
      "Los significados proceden del Wikcionario y, para el italiano, también de Wikidata, escritos por personas, nunca por una traducción automática. La traducción ampliada, opcional, traduce el inglés.",
    );
  });

  it("a page none of whose pairs is its readers' says whom the product is made for", () => {
    // The French page is always built: with no French-glossed pair, it presents the first
    // other group as today's English page presents the French speakers.
    const noFrench = shipped({ tops: MATRIX.tops, glossed: { "es-en": [90, 80, 70], "en-es": [96, 91, 80] } });
    expect(linguaPageText("fr", noFrench, noDiscord).heroNote).toBe(
      "Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur et Android) et sur Safari (iPhone, iPad, Mac). Conçue pour les anglophones qui apprennent l'espagnol : l'interface et les traductions sont en anglais. Aussi pour les hispanophones qui apprennent l'anglais : l'interface et les traductions sont en espagnol.",
    );
    // The Spanish page is built only with a Spanish-glossed pair (D3): it has no such sentence.
    expect(() => linguaPageText("es", today, noDiscord)).toThrow(/the es page is built only with a pair glossed in its language/);
  });
});

describe("the Spanish home's Lingua card follows the shipped pairs (change: extend-site-spanish-locale, D3)", () => {
  // The sentence every state shares after the languages read; the card is inserted with
  // `set:html`, as the Lingua page's slots are.
  const REST =
    "con las palabras que aún no conoces resaltadas en la propia página. Un porcentaje honesto por página, un clic para la traducción y tu vocabulario, que se construye solo.";
  const words = (text: string) => text.split(/\s+/).length;

  it("today's pairs: every language read, for French speakers, and the English Lingua page", () => {
    const card = spanishHomeLinguaCard(today.pairs);
    expect(card.body).toBe(
      `Lee la web en inglés o en español ${REST} Pensada para francohablantes, con la interfaz y las traducciones en su idioma.`,
    );
    expect(words(card.body)).toBe(49);
    expect(card.href).toBe("/en/lingua");
  });

  it("today's and es-en (change 34): for French and English speakers, still the English Lingua page", () => {
    const withEsEn = shipped({ tops: MATRIX.tops, glossed: { ...TODAY.glossed, "es-en": MATRIX.glossed["es-en"] } });
    const card = spanishHomeLinguaCard(withEsEn.pairs);
    expect(card.body).toBe(
      `Lee la web en inglés o en español ${REST} Pensada para francohablantes y anglohablantes, con la interfaz y las traducciones en su idioma.`,
    );
    expect(words(card.body)).toBe(51);
    expect(card.href).toBe("/en/lingua");
  });

  it("the matrix (en-es, change 35): the language read with a Spanish gloss, no audience, the Spanish Lingua page", () => {
    const card = spanishHomeLinguaCard(matrix.pairs);
    expect(card.body).toBe(`Lee la web en inglés ${REST}`);
    expect(card.body).not.toContain("Pensada para");
    expect(words(card.body)).toBe(34);
    expect(card.href).toBe("/es/lingua");
  });

  it("the matrix with fr-en and fr-es (stage 3): English or French, read with a Spanish gloss", () => {
    const stage3 = shipped({ tops: MATRIX.tops, glossed: { ...MATRIX.glossed, "fr-en": [1, 1, 1], "fr-es": [1, 1, 1] } });
    const card = spanishHomeLinguaCard(stage3.pairs);
    expect(card.body).toBe(`Lee la web en inglés o en francés ${REST}`);
    expect(words(card.body)).toBe(37);
    expect(card.href).toBe("/es/lingua");
  });

  it("never says Lingua explains words in Spanish while no pair glossed in Spanish ships", () => {
    for (const pairs of [today.pairs, shipped({ tops: MATRIX.tops, glossed: { "es-en": [1, 1, 1] } }).pairs]) {
      const card = spanishHomeLinguaCard(pairs);
      expect(card.body).toMatch(/ Pensada para \S+, con la interfaz y las traducciones en su idioma\.$/);
      expect(card.href).toBe("/en/lingua");
    }
  });

  it("refuses a language the Spanish table does not name, and an empty list, at build time", () => {
    const german = shipped({ tops: MATRIX.tops, glossed: { "de-fr": [1, 1, 1] } }, {});
    expect(() => spanishHomeLinguaCard(german.pairs)).toThrow(/the es table has no name for the language "de"/);
    const forGermans = shipped({ tops: MATRIX.tops, glossed: { "en-de": [1, 1, 1] } }, {});
    expect(() => spanishHomeLinguaCard(forGermans.pairs)).toThrow(/the es table has no speakers for the language "de"/);
    expect(() => spanishHomeLinguaCard([])).toThrow(/no shipped pair to describe/);
  });
});

describe("Spanish lists", () => {
  it("« y » becomes « e » before an i- sound, but not before « hie »", () => {
    const swapped = shipped({ tops: MATRIX.tops, glossed: { "en-es": [1, 1, 1], "es-fr": [1, 1, 1], "en-fr": [1, 1, 1] } });
    expect(card(linguaPageText("es", swapped, noDiscord), "languages").title).toBe("🧭 Inglés → español · Español e inglés → francés");
    expect(esAnd(["español", "inglés"])).toBe("español e inglés");
    expect(esAnd(["agua", "hilo"])).toBe("agua e hilo");
    expect(esAnd(["agua", "hielo"])).toBe("agua y hielo");
    expect(esAnd(["francés", "inglés", "español"])).toBe("francés, inglés y español");
    expect(esAnd(["inglés"])).toBe("inglés");
    expect(esAnd([])).toBe("");
  });

  it("« o » becomes « u » before an o- sound", () => {
    expect(esOr(["inglés", "francés"])).toBe("inglés o francés");
    expect(esOr(["inglés", "occitano"])).toBe("inglés u occitano");
    expect(esOr(["inglés", "holandés"])).toBe("inglés u holandés");
    expect(esOr(["español", "francés", "occitano"])).toBe("español, francés u occitano");
    expect(esOr(["inglés"])).toBe("inglés");
  });
});

describe("the coverage table and the translation note per pair (D4)", () => {
  it("captions 'a gloss in the reader's language' once pairs of two native languages are listed", () => {
    expect(linguaPageText("en", matrix, noDiscord).coverage.caption).toBe(
      "The share of the commonest words that have a gloss in the reader's language, measured the same way for each pair, in the dictionaries the extension ships.",
    );
    expect(linguaPageText("fr", matrix, noDiscord).coverage.caption).toBe(
      "La part des mots les plus courants qui ont une définition dans la langue du lecteur, mesurée de la même façon pour chaque paire, sur les dictionnaires livrés avec l'extension.",
    );
    expect(linguaPageText("es", matrix, noDiscord).coverage.caption).toBe(
      "La proporción de las palabras más frecuentes que tienen una definición en la lengua del lector, medida de la misma manera para cada par, en los diccionarios que se entregan con la extensión.",
    );
  });

  it("says, per pair, whether extended translation serves it, directly or through English", () => {
    expect(linguaPageText("en", matrix, noDiscord).coverage.note).toBe(
      "Glosses come from Wiktionary and from translations written by people, never from machine translation. Extended translation, optional, serves Spanish to English, English to French, Spanish to French (through English) and English to Spanish.",
    );
    expect(linguaPageText("fr", matrix, noDiscord).coverage.note).toBe(
      "Les définitions viennent du Wiktionnaire et de traductions écrites par des personnes, jamais d'une traduction automatique. La traduction étendue, facultative, traduit l'anglais vers le français, l'espagnol vers le français (en passant par l'anglais), l'espagnol vers l'anglais et l'anglais vers l'espagnol.",
    );
    expect(linguaPageText("es", matrix, noDiscord).coverage.note).toBe(
      "Los significados proceden del Wikcionario y, para el italiano, también de Wikidata, escritos por personas, nunca por una traducción automática. La traducción ampliada, opcional, traduce del inglés al español, del inglés al francés, del español al francés (pasando por el inglés) y del español al inglés.",
    );
  });

  it("names the pairs extended translation does not serve yet", () => {
    const { "en-es": _, "es-en": __, ...routes } = ROUTES;
    const partly = shipped(MATRIX, routes);
    expect(linguaPageText("en", partly, noDiscord).coverage.note).toMatch(
      / Extended translation, optional, serves English to French and Spanish to French \(through English\)\. Not yet for Spanish to English and English to Spanish\.$/,
    );
    expect(linguaPageText("fr", partly, noDiscord).coverage.note).toMatch(/ Pas encore pour l'espagnol vers l'anglais et l'anglais vers l'espagnol\.$/);
    expect(linguaPageText("es", partly, noDiscord).coverage.note).toMatch(/ Pendiente: del inglés al español y del español al inglés\.$/);
    // One native language, one pair not served: the old wording, "Spanish follows".
    const onlyEn = shipped({ tops: MATRIX.tops, glossed: { "en-fr": [1, 1, 1], "es-fr": [1, 1, 1] } }, { "en-fr": ["x"] });
    expect(linguaPageText("fr", onlyEn, noDiscord).coverage.note).toMatch(/ traduit l'anglais\. Pas encore pour l'espagnol\.$/);
    expect(linguaPageText("en", onlyEn, noDiscord).coverage.note).toMatch(/ serves English\. Not yet for Spanish\.$/);
  });
});

describe("each studied language's card and levels, one sentence each (change: add-lingua-french-listings, D7)", () => {
  /** What the page appends to a card: its body after the table's own sentence. */
  const appended = (t: LinguaPageText, lang: "fr" | "en" | "es", key: CardKey) =>
    card(t, key).body.slice(LINGUA_TEXT[lang].cards.find((c) => c.key === key)!.body.length);

  it("fr-en beside es-en: the English page names Spanish and French in both sentences", () => {
    const t = linguaPageText("en", withFrEn, noDiscord);
    expect(appended(t, "en", "click")).toBe(" In Spanish and French, the card also names the tense and the gender.");
    expect(appended(t, "en", "level")).toBe(
      " For Spanish and French, the levels are estimated from word frequency, as no CEFR list can be shipped freely.",
    );
  });

  it("fr-en beside es-en: the French page, « en espagnol et en français », « l'espagnol et le français »", () => {
    const t = linguaPageText("fr", withFrEn, noDiscord);
    expect(appended(t, "fr", "click")).toBe(" En espagnol et en français, la carte nomme aussi le temps et le genre.");
    expect(appended(t, "fr", "level")).toBe(
      " Pour l'espagnol et le français, les niveaux sont estimés d'après la fréquence des mots, faute de liste CEFR libre de droits.",
    );
  });

  it("fr-en beside es-en: the Spanish page, « español y en francés », « el español y el francés »", () => {
    const t = linguaPageText("es", withFrEn, noDiscord);
    expect(appended(t, "es", "click")).toBe(" En español y en francés, la tarjeta también indica el tiempo verbal y el género.");
    expect(appended(t, "es", "level")).toBe(
      " Para el español y el francés, los niveles se estiman a partir de la frecuencia de las palabras, a falta de una lista MCER de uso libre.",
    );
  });

  it("fr-en and fr-es: the Spanish page names its readers' French first, the others keep their order", () => {
    const es = linguaPageText("es", six, noDiscord);
    expect(appended(es, "es", "click")).toBe(" En francés y en español, la tarjeta también indica el tiempo verbal y el género.");
    expect(appended(es, "es", "level")).toMatch(/^ Para el francés y el español, los niveles se estiman/);
    expect(appended(linguaPageText("en", six, noDiscord), "en", "click")).toBe(
      " In Spanish and French, the card also names the tense and the gender.",
    );
    expect(appended(linguaPageText("fr", six, noDiscord), "fr", "click")).toBe(
      " En espagnol et en français, la carte nomme aussi le temps et le genre.",
    );
  });

  it("never says English's card names a gender, nor that its levels are estimated", () => {
    // Only English studied (en-fr and en-es): neither sentence, on any page.
    const english = shipped({ tops: MATRIX.tops, glossed: { "en-fr": [1, 1, 1], "en-es": [1, 1, 1] } });
    for (const lang of ["fr", "en", "es"] as const) {
      const t = linguaPageText(lang, english, noDiscord);
      expect(appended(t, lang, "click")).toBe("");
      expect(appended(t, lang, "level")).toBe("");
    }
    // Every pair listed: English is never among the languages the sentences name.
    for (const lang of ["fr", "en", "es"] as const) {
      const t = linguaPageText(lang, six, noDiscord);
      const englishName = LINGUA_TEXT[lang].names.en;
      expect(appended(t, lang, "click")).not.toMatch(new RegExp(englishName, "i"));
      expect(appended(t, lang, "level")).not.toMatch(new RegExp(englishName, "i"));
    }
  });

  it("describes every language the tables can name, and nothing else", () => {
    for (const lang of ["fr", "en", "es"] as const) {
      expect(Object.keys(STUDIED).sort()).toEqual(Object.keys(LINGUA_TEXT[lang].names).sort());
    }
    expect(studiedLanguage("en")).toEqual({ namesTenseAndGender: false, levelsEstimated: false });
    expect(studiedLanguage("fr")).toEqual({ namesTenseAndGender: true, levelsEstimated: true });
    // Italian (change: add-lingua-italian-listings, D1): its card names the tense and the gender
    // (changes 13 and 18), its levels are estimated (change 14).
    expect(studiedLanguage("it")).toEqual({ namesTenseAndGender: true, levelsEstimated: true });
  });

  it("refuses a studied language the table does not describe, naming it, at build time", () => {
    expect(() => studiedLanguage("de")).toThrow(/STUDIED does not describe the studied language "de"/);
    expect(() => studiedLanguage("constructor")).toThrow(/does not describe the studied language "constructor"/);
    // A language the French table could name but `STUDIED` does not describe fails the page.
    const names = LINGUA_TEXT.fr.names;
    names.de = "allemand";
    try {
      const german = shipped({ tops: MATRIX.tops, glossed: { "de-fr": [1, 1, 1] } }, {});
      expect(() => linguaPageText("fr", german, noDiscord)).toThrow(/does not describe the studied language "de"/);
    } finally {
      delete names.de;
    }
  });
});

describe("the coverage table at a phone's width (change: add-lingua-french-listings, D8)", () => {
  it("today's pairs: one column per pair, as before", () => {
    const { coverage: table } = linguaPageText("en", today, noDiscord);
    expect(table.perPair).toBe(false);
    expect(table.head).toEqual(["Commonest words", "English", "Spanish"]);
  });

  it("six pairs: the English page has six rows, Spanish → English and French → English first", () => {
    const { coverage: table } = linguaPageText("en", six, noDiscord);
    expect(table.perPair).toBe(true);
    expect(table.head).toEqual(["Commonest words", "5,000", "10,000", "20,000"]);
    expect(table.rows).toEqual([
      ["Spanish → English", "90%", "80%", "70%"],
      ["French → English", "94%", "87%", "76%"],
      ["English → French", "95%", "90%", "79%"],
      ["Spanish → French", "88%", "77%", "64%"],
      ["English → Spanish", "96%", "91%", "80%"],
      ["French → Spanish", "83%", "71%", "57%"],
    ]);
  });

  it("six pairs: the French and Spanish pages lead with their readers' pairs", () => {
    expect(linguaPageText("fr", six, noDiscord).coverage.rows.map((row) => row[0])).toEqual([
      "Anglais → français",
      "Espagnol → français",
      "Espagnol → anglais",
      "Français → anglais",
      "Anglais → espagnol",
      "Français → espagnol",
    ]);
    expect(linguaPageText("es", six, noDiscord).coverage.rows.map((row) => row[0])).toEqual([
      "Inglés → español",
      "Francés → español",
      "Inglés → francés",
      "Español → francés",
      "Español → inglés",
      "Francés → inglés",
    ]);
  });
});

describe("Italian studied: it-fr beside the six pairs (change: add-lingua-italian-listings, D1, D2)", () => {
  /** What the page appends to a card: its body after the table's own sentence. */
  const appended = (t: LinguaPageText, lang: "fr" | "en" | "es", key: CardKey) =>
    card(t, key).body.slice(LINGUA_TEXT[lang].cards.find((c) => c.key === key)!.body.length);

  it("French: the audience, the card and the levels name Italian after English and Spanish", () => {
    const t = linguaPageText("fr", seven, noDiscord);
    expect(t.tagline).toMatch(/^Une extension navigateur qui surligne les mots d'anglais, d'espagnol ou d'italien que vous ne connaissez pas encore/);
    expect(t.description).toBe(
      "Une extension navigateur qui surligne, sur la page que vous lisez, les mots d'anglais, d'espagnol ou d'italien que vous ne connaissez pas encore. Analyse hors-ligne, aucun compte requis.",
    );
    expect(t.heroNote).toBe(
      "Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur et Android) et sur Safari (iPhone, iPad, Mac). Conçue pour les francophones qui apprennent l'anglais, l'espagnol ou l'italien. Aussi pour les anglophones qui apprennent l'espagnol ou le français : l'interface et les traductions sont en anglais. Aussi pour les hispanophones qui apprennent l'anglais ou le français : l'interface et les traductions sont en espagnol.",
    );
    expect(card(t, "languages").title).toBe("🧭 Anglais, espagnol et italien → français · Espagnol et français → anglais · Anglais et français → espagnol");
    expect(appended(t, "fr", "click")).toBe(" En espagnol, en italien et en français, la carte nomme aussi le temps et le genre.");
    expect(appended(t, "fr", "level")).toBe(
      " Pour l'espagnol, l'italien et le français, les niveaux sont estimés d'après la fréquence des mots, faute de liste CEFR libre de droits.",
    );
    // English is never among them (*English is never said to be estimated*).
    expect(appended(t, "fr", "click")).not.toMatch(/anglais/i);
    expect(appended(t, "fr", "level")).not.toMatch(/anglais/i);
  });

  it("French: seven rows, English → French, Spanish → French and Italian → French first", () => {
    const { coverage: table } = linguaPageText("fr", seven, noDiscord);
    expect(table.perPair).toBe(true);
    expect(table.head).toEqual(["Mots les plus courants", `5${NNBSP}000`, `10${NNBSP}000`, `20${NNBSP}000`]);
    expect(table.rows).toEqual([
      ["Anglais → français", `95${NBSP}%`, `90${NBSP}%`, `79${NBSP}%`],
      ["Espagnol → français", `88${NBSP}%`, `77${NBSP}%`, `64${NBSP}%`],
      ["Italien → français", `89${NBSP}%`, `82${NBSP}%`, `73${NBSP}%`],
      ["Espagnol → anglais", `90${NBSP}%`, `80${NBSP}%`, `70${NBSP}%`],
      ["Français → anglais", `94${NBSP}%`, `87${NBSP}%`, `76${NBSP}%`],
      ["Anglais → espagnol", `96${NBSP}%`, `91${NBSP}%`, `80${NBSP}%`],
      ["Français → espagnol", `83${NBSP}%`, `71${NBSP}%`, `57${NBSP}%`],
    ]);
    // it-fr's route goes through English (change 2): « (en passant par l'anglais) ».
    expect(table.note).toBe(
      "Les définitions viennent du Wiktionnaire et de traductions écrites par des personnes, jamais d'une traduction automatique. La traduction étendue, facultative, traduit l'anglais vers le français, l'espagnol vers le français (en passant par l'anglais), l'italien vers le français (en passant par l'anglais), l'espagnol vers l'anglais, le français vers l'anglais, l'anglais vers l'espagnol et le français vers l'espagnol (en passant par l'anglais).",
    );
  });

  it("French: without it-fr's route, the note says Italian is not translated yet", () => {
    const { "it-fr": _, ...routes } = ROUTES;
    expect(linguaPageText("fr", shipped(SEVEN_PAIRS, routes), noDiscord).coverage.note).toMatch(
      / Pas encore pour l'italien vers le français\.$/,
    );
  });

  // One order on every page (the requirement's « the page's readers' pairs first »): the readers'
  // studied languages, then the other readers'. Italian is French speakers' alone, so it follows
  // Spanish on the French page and French on the English one.
  it("English and Spanish: Italian named in their sentences, for French speakers, never offered to their readers", () => {
    const en = linguaPageText("en", seven, noDiscord);
    expect(en.heroNote).toContain(" Also for French speakers learning English, Spanish or Italian: the interface and the translations are in French.");
    expect(en.heroNote).toMatch(/Made for English speakers learning Spanish or French\./);
    expect(appended(en, "en", "click")).toBe(" In Spanish, French and Italian, the card also names the tense and the gender.");
    expect(appended(en, "en", "level")).toBe(
      " For Spanish, French and Italian, the levels are estimated from word frequency, as no CEFR list can be shipped freely.",
    );
    expect(en.coverage.rows.map((row) => row[0])).toContain("Italian → French");
    const es = linguaPageText("es", seven, noDiscord);
    expect(es.heroNote).toContain(" También para francohablantes que aprenden inglés, español o italiano: la interfaz y las traducciones están en francés.");
    expect(es.heroNote).toMatch(/Pensada para hispanohablantes que aprenden inglés o francés\./);
    expect(appended(es, "es", "click")).toBe(" En francés, en español y en italiano, la tarjeta también indica el tiempo verbal y el género.");
    expect(appended(es, "es", "level")).toMatch(/^ Para el francés, el español y el italiano, los niveles se estiman/);
    expect(es.coverage.rows.map((row) => row[0])).toContain("Italiano → francés");
    // The tagline names what the page's own readers study: no Italian for English or Spanish speakers.
    expect(en.tagline).not.toMatch(/Italian/);
    expect(es.tagline).not.toMatch(/italiano/);
  });

  it("the English and Spanish homes' cards do not move", () => {
    expect(englishHomeLinguaCard(seven.pairs)).toEqual(englishHomeLinguaCard(six.pairs));
    expect(spanishHomeLinguaCard(seven.pairs)).toEqual(spanishHomeLinguaCard(six.pairs));
  });
});

describe("Italian for English speakers: it-en beside the seven pairs (change: add-lingua-italian-english-listings, D1)", () => {
  // The Italian programme's change 25 lists it-en after it-fr; nothing in the site's code moves for
  // it: the English home card, the pages' audiences and the table are built from the shipped pairs.
  it("the English home card: Spanish, French or Italian, and the English Lingua page", () => {
    expect(englishHomeLinguaCard(eight.pairs)).toEqual({
      body: "Read the web in Spanish, French or Italian with the words you do not know yet highlighted in place. An honest per-page percentage, one click for the meaning, and a vocabulary that builds itself as you read.",
      href: "/en/lingua",
    });
  });

  it("English: the audience names Italian, and eight rows, Spanish → English, French → English and Italian → English first", () => {
    const t = linguaPageText("en", eight, noDiscord);
    expect(t.tagline).toMatch(/^A browser extension that highlights the Spanish, French or Italian words you do not know yet/);
    expect(t.heroNote).toMatch(/ Made for English speakers learning Spanish, French or Italian\. /);
    expect(card(t, "languages").title).toBe("🧭 Spanish, French and Italian → English · English, Spanish and Italian → French · English and French → Spanish");
    expect(t.coverage.rows.map((row) => row[0])).toEqual([
      "Spanish → English",
      "French → English",
      "Italian → English",
      "English → French",
      "Spanish → French",
      "Italian → French",
      "English → Spanish",
      "French → Spanish",
    ]);
    expect(t.coverage.rows[2]).toEqual(["Italian → English", "91%", "84%", "76%"]);
    // it-en's route is its own model (change 2): no « (through English) ».
    expect(t.coverage.note).toContain(" French to English, Italian to English, English to French,");
  });

  it("French and Spanish: Italian named among English speakers' languages, offered to no Spanish speaker", () => {
    const fr = linguaPageText("fr", eight, noDiscord);
    expect(fr.heroNote).toContain(" Aussi pour les anglophones qui apprennent l'espagnol, le français ou l'italien : l'interface et les traductions sont en anglais.");
    expect(fr.coverage.rows.map((row) => row[0])).toContain("Italien → anglais");
    const es = linguaPageText("es", eight, noDiscord);
    expect(es.heroNote).toContain(" También para anglohablantes que aprenden español, francés o italiano: la interfaz y las traducciones están en inglés.");
    expect(es.heroNote).toMatch(/Pensada para hispanohablantes que aprenden inglés o francés\./);
    expect(es.tagline).not.toMatch(/italiano/);
    expect(es.coverage.rows.map((row) => row[0])).toContain("Italiano → inglés");
  });

  it("the Spanish and French homes' cards do not move", () => {
    expect(spanishHomeLinguaCard(eight.pairs)).toEqual(spanishHomeLinguaCard(seven.pairs));
    expect(frenchHomeLinguaCard(eight.pairs)).toEqual(frenchHomeLinguaCard(seven.pairs));
  });
});

describe("Italian for Spanish speakers: it-es beside the eight pairs (change: add-lingua-italian-spanish-listings, D1)", () => {
  // The Italian programme's change 30 lists it-es after it-en; nothing in the site's code moves for
  // it: the Spanish home card, the pages' audiences and the table are built from the shipped pairs.
  it("the Spanish home card: English, French or Italian, no audience, and the Spanish Lingua page", () => {
    const card = spanishHomeLinguaCard(nine.pairs);
    expect(card).toEqual({
      body: "Lee la web en inglés, en francés o en italiano con las palabras que aún no conoces resaltadas en la propia página. Un porcentaje honesto por página, un clic para la traducción y tu vocabulario, que se construye solo.",
      href: "/es/lingua",
    });
    expect(card.body).not.toContain("Pensada para");
  });

  it("Spanish: the audience names Italian, and nine rows, English → Spanish, French → Spanish and Italian → Spanish first", () => {
    const t = linguaPageText("es", nine, noDiscord);
    expect(t.tagline).toMatch(/^Una extensión para el navegador que resalta las palabras en inglés, en francés o en italiano que todavía no conoces/);
    expect(t.heroNote).toMatch(/ Pensada para hispanohablantes que aprenden inglés, francés o italiano\. /);
    expect(card(t, "languages").title).toBe("🧭 Inglés, francés e italiano → español · Inglés, español e italiano → francés · Español, francés e italiano → inglés");
    // The readers' studied languages first, in `packs.json`'s order, then the others': Italian now
    // comes before Spanish on the Spanish page.
    expect(card(t, "click").body).toMatch(/ En francés, en italiano y en español, la tarjeta también indica el tiempo verbal y el género\.$/);
    expect(card(t, "level").body).toMatch(/ Para el francés, el italiano y el español, los niveles se estiman /);
    expect(t.coverage.rows.map((row) => row[0])).toEqual([
      "Inglés → español",
      "Francés → español",
      "Italiano → español",
      "Inglés → francés",
      "Español → francés",
      "Italiano → francés",
      "Español → inglés",
      "Francés → inglés",
      "Italiano → inglés",
    ]);
    expect(t.coverage.rows[2]).toEqual(["Italiano → español", `85${NNBSP}%`, `76${NNBSP}%`, `65${NNBSP}%`]);
    // it-es's route goes through English (change 2): « (pasando por el inglés) ».
    expect(t.coverage.note).toContain(" del francés al español (pasando por el inglés), del italiano al español (pasando por el inglés),");
  });

  it("French and English: Italian named among Spanish speakers' languages, and it-es's row", () => {
    const fr = linguaPageText("fr", nine, noDiscord);
    expect(fr.heroNote).toContain(" Aussi pour les hispanophones qui apprennent l'anglais, le français ou l'italien : l'interface et les traductions sont en espagnol.");
    expect(card(fr, "languages").title).toBe("🧭 Anglais, espagnol et italien → français · Espagnol, français et italien → anglais · Anglais, français et italien → espagnol");
    expect(fr.coverage.rows.map((row) => row[0])).toContain("Italien → espagnol");
    const en = linguaPageText("en", nine, noDiscord);
    expect(en.heroNote).toContain(" Also for Spanish speakers learning English, French or Italian: the interface and the translations are in Spanish.");
    expect(card(en, "languages").title).toBe("🧭 Spanish, French and Italian → English · English, Spanish and Italian → French · English, French and Italian → Spanish");
    expect(en.coverage.rows.map((row) => row[0])).toContain("Italian → Spanish");
  });

  it("Spanish: without it-es's route, the note says Italian into Spanish is not translated yet", () => {
    const { "it-es": _, ...routes } = ROUTES;
    expect(linguaPageText("es", shipped(NINE_PAIRS, routes), noDiscord).coverage.note).toMatch(/ Pendiente: del italiano al español\.$/);
  });

  it("the English and French homes' cards do not move", () => {
    expect(englishHomeLinguaCard(nine.pairs)).toEqual(englishHomeLinguaCard(eight.pairs));
    expect(frenchHomeLinguaCard(nine.pairs)).toEqual(frenchHomeLinguaCard(eight.pairs));
  });
});

describe("the English home's Lingua card follows the shipped pairs (change: add-lingua-french-listings, D9)", () => {
  const REST =
    "with the words you do not know yet highlighted in place. An honest per-page percentage, one click for the meaning, and a vocabulary that builds itself as you read.";

  it("today's pairs: today's paragraph, for French speakers, and the English Lingua page", () => {
    expect(englishHomeLinguaCard(today.pairs)).toEqual({ body: `Read the English web ${REST}`, href: "/en/lingua" });
  });

  it("es-en (change 34): the language read with an English gloss, Spanish", () => {
    const withEsEn = shipped({ tops: MATRIX.tops, glossed: { ...TODAY.glossed, "es-en": MATRIX.glossed["es-en"] } });
    expect(englishHomeLinguaCard(withEsEn.pairs)).toEqual({ body: `Read the web in Spanish ${REST}`, href: "/en/lingua" });
    expect(englishHomeLinguaCard(matrix.pairs).body).toBe(`Read the web in Spanish ${REST}`);
  });

  it("es-en and fr-en (change 52): Spanish or French", () => {
    expect(englishHomeLinguaCard(withFrEn.pairs)).toEqual({ body: `Read the web in Spanish or French ${REST}`, href: "/en/lingua" });
    expect(englishHomeLinguaCard(six.pairs).body).toBe(`Read the web in Spanish or French ${REST}`);
  });

  it("refuses a language the English table does not name, at build time", () => {
    const german = shipped({ tops: MATRIX.tops, glossed: { "de-en": [1, 1, 1] } }, {});
    expect(() => englishHomeLinguaCard(german.pairs)).toThrow(/the en table has no name for the language "de"/);
  });
});

describe("nothing but the site's own text reaches set:html", () => {
  it("refuses a language the table has no name for, at build time", () => {
    const german = shipped({ tops: MATRIX.tops, glossed: { "de-fr": [1, 1, 1] } }, {});
    expect(() => linguaPageText("fr", german, noDiscord)).toThrow(/no name for the language "de"/);
  });

  it("names a language from the table's own entries, never from the object's prototype", () => {
    // `linguaPairs` refuses such a key; built by hand, it still names nothing.
    const pair = (studied: string, native: string): LinguaPair => ({
      pair: `${studied}-${native}`,
      studied,
      native,
      glossed: [1, 1, 1],
      translation: "none",
    });
    const tops = MATRIX.tops;
    expect(() => linguaPageText("en", { tops, pairs: [pair("en", "constructor")] }, noDiscord)).toThrow(
      /no (name|speakers) for the language "constructor"/,
    );
    expect(() => linguaPageText("fr", { tops, pairs: [pair("toString", "fr")] }, noDiscord)).toThrow(/no name for the language "toString"/);
    expect(() => linguaPageText("fr", { tops, pairs: [pair("en", "fr"), pair("es", "__proto__")] }, noDiscord)).toThrow(
      /no (name|speakers) for the language "__proto__"/,
    );
  });

  it("escapes &, < and > in what it fills in, and leaves the apostrophe as today's page writes it", () => {
    expect(fill("a {x} b", { x: `<img src=x onerror="alert(1)"> & l'anglais` })).toBe(
      `a &lt;img src=x onerror="alert(1)"&gt; &amp; l'anglais b`,
    );
    expect(() => fill("{x}", {})).toThrow(/no value for \{x\}/);
    expect(() => fill("{constructor}", {})).toThrow(/no value for \{constructor\}/);
  });

  it("escapes the invite's address in the link", () => {
    const t = linguaPageText("fr", today, { discordUrl: 'https://x.test/?a=1&b="2"' });
    expect(t.heroNote).toContain('<a href="https://x.test/?a=1&amp;b=&quot;2&quot;" rel="noopener">');
  });
});

describe("the card's buttons, as the extension labels them (change 34, enable-lingua-english-speakers)", () => {
  // Each table quotes the word card's own labels (`apps/lingua-extension/src/i18n/<lang>/card.ts`)
  // in the marks its language writes them with (`apps/lingua-extension/src/i18n/README.md`):
  // « » with inner spaces in French, without in Spanish, “ ” in English.
  const catalogue = (lang: string) =>
    readFileSync(resolve(__dirname, `../../lingua-extension/src/i18n/${lang}/card.ts`), "utf8");
  const label = (lang: string, key: string) => catalogue(lang).match(new RegExp(`^  ${key}: "([^"]+)",$`, "m"))?.[1];
  const quoted: Record<"fr" | "en" | "es", (text: string) => string> = {
    fr: (text) => `« ${text} »`,
    en: (text) => `“${text}”`,
    es: (text) => `«${text}»`,
  };

  it.each(["fr", "en", "es"] as const)("%s: the click card names the three buttons as the word card labels them", (lang) => {
    const body = card(linguaPageText(lang, matrix, noDiscord), "click").body;
    for (const key of ["known", "addToDeck", "ignore"]) {
      const text = label(lang, key);
      expect(text, `${lang}/card.ts ${key}`).toBeTruthy();
      expect(body, `${lang}: ${key}`).toContain(quoted[lang](text!));
    }
  });

  // Until change 34 the English table quoted the French buttons and said "its French
  // translation": the interface was French for every reader. es-en ships glossed in English,
  // so an English speaker reads that card about their own pair, in their own interface.
  const FRENCH_WORDING = ["its French translation", "« Je connais »", "« + Deck »", "« Ignorer »"];
  const englishGlossed = linguaPairs(coverage, {})
    .filter((p) => p.native === "en")
    .map((p) => p.pair);
  const left = FRENCH_WORDING.filter((words) => JSON.stringify(LINGUA_TEXT.en).includes(words));

  it("the English table speaks of the English interface, an English-glossed pair shipping", () => {
    expect(
      englishGlossed,
      "the English table quotes the English interface's buttons: it ships only with a pair glossed in English",
    ).toContain("es-en");
    expect(left, `the English table of src/lib/lingua-text.ts still says ${left.join(", ")}`).toEqual([]);
  });
});

describe("the French home's Lingua card follows the shipped pairs (change: add-site-lingua-showcase, D15)", () => {
  const REST =
    "les mots que vous ne connaissez pas encore sont surlignés sur place. Un pourcentage honnête par page, un clic pour la traduction, et votre vocabulaire qui se construit tout seul.";

  it("today's pairs: English or Spanish, and the French Lingua page", () => {
    expect(frenchHomeLinguaCard(today.pairs)).toEqual({ body: `Lisez le web en anglais ou en espagnol, ${REST}`, href: "/lingua" });
    expect(frenchHomeLinguaCard(six.pairs).body).toBe(`Lisez le web en anglais ou en espagnol, ${REST}`);
  });

  it("it-fr beside the six pairs (the Italian programme's change 20): English, Spanish or Italian", () => {
    expect(frenchHomeLinguaCard(seven.pairs)).toEqual({
      body: `Lisez le web en anglais, en espagnol ou en italien, ${REST}`,
      href: "/lingua",
    });
  });

  it("en-fr alone: English", () => {
    const english = shipped({ tops: TODAY.tops, glossed: { "en-fr": TODAY.glossed["en-fr"] } });
    expect(frenchHomeLinguaCard(english.pairs)).toEqual({ body: `Lisez le web en anglais, ${REST}`, href: "/lingua" });
  });

  it("refuses a list with no pair glossed in French", () => {
    const spanish = shipped({ tops: TODAY.tops, glossed: { "en-es": MATRIX.glossed["en-es"] } });
    expect(() => frenchHomeLinguaCard(spanish.pairs)).toThrow(/no shipped pair is glossed in French/);
  });
});

describe("the captures' words (change: add-site-lingua-showcase, D10, D11)", () => {
  // French, English and Spanish: the languages `plan.mjs` captures. Italian's showcase strings are
  // type stubs until change 45 captures the extension in Italian and rewrites them (change:
  // add-site-italian-locale, D6), and joins them here then.
  const LANGS = ["fr", "en", "es"] as const;
  const extension = (path: string) => readFileSync(resolve(__dirname, `../../lingua-extension/src/i18n/${path}`), "utf8");
  const label = (lang: string, module: string, key: string) =>
    extension(`${lang}/${module}.ts`).match(new RegExp(`^  ${key}: "([^"]+)",$`, "m"))?.[1];
  const quoted: Record<(typeof LANGS)[number], (text: string) => string> = {
    fr: (text) => `« ${text} »`,
    en: (text) => `“${text}”`,
    es: (text) => `«${text}»`,
  };

  it.each(LANGS)("%s: a heading, the hero's alt text, the video's label and caption, and every capture's alt text and caption", (lang) => {
    const { showcase, hints } = linguaPageText(lang, six, noDiscord);
    expect(showcase.heading).toBeTruthy();
    expect(showcase.hero.alt.length).toBeGreaterThan(40);
    expect(showcase.video.label).toBeTruthy();
    expect(showcase.video.caption).toBeTruthy();
    expect(showcase.shareAlt).toBeTruthy();
    expect(Object.keys(showcase.shots).sort()).toEqual([...SHOT_KEYS].sort());
    for (const key of SHOT_KEYS) {
      expect(showcase.shots[key].alt.length, `${lang} ${key} alt`).toBeGreaterThan(30);
      expect(showcase.shots[key].caption, `${lang} ${key} caption`).toBeTruthy();
    }
    expect(Object.keys(hints).sort()).toEqual(["android", "ios"]);
    expect(hints.ios).toContain("Safari");
    expect(hints.android).toContain("Firefox");
  });

  it.each(LANGS)("%s: the alt texts quote no figure the seed could change, only the scale's levels", (lang) => {
    const { showcase } = LINGUA_TEXT[lang];
    const texts = [showcase.hero.alt, showcase.shareAlt, ...SHOT_KEYS.map((key) => showcase.shots[key].alt)];
    for (const text of texts) expect(text.replace(/\b[ABC][12]\b/g, ""), text).not.toMatch(/\d/);
  });

  it.each(LANGS)("%s: the alt texts name the languages the capture's reader studies (tool/capture/plan.mjs)", (lang) => {
    const t = LINGUA_TEXT[lang];
    const [first, second] = CAPTURED[lang];
    expect(t.showcase.hero.alt).toContain(t.names[first]);
    expect(t.showcase.shareAlt).toContain(t.names[first]);
    expect(t.showcase.shots.second.alt).toContain(t.names[second]);
    expect(t.showcase.shots.stats.alt).toContain(t.names[first]);
  });

  it.each(LANGS)("%s: the review's two answers, the « Je connais » and the e-ink preset are quoted as the extension labels them", (lang) => {
    const { showcase } = LINGUA_TEXT[lang];
    for (const key of ["missed", "recalled"]) {
      const text = label(lang, "review", key);
      expect(text, `${lang}/review.ts ${key}`).toBeTruthy();
      expect(showcase.shots.review.alt).toContain(quoted[lang](text!));
    }
    expect(showcase.video.caption).toContain(quoted[lang](label(lang, "card", "known")!));
    expect(showcase.shots["reader-eink"].alt).toContain(quoted[lang](label(lang, "colours", "presetEinkMono")!));
  });
});

describe("the Italian table (change: add-site-italian-locale, D6)", () => {
  // en-it, the pair change 45 lists (Italian speakers study English), beside the nine pairs: the
  // Italian page it builds, in Italian numbers and with each language's article.
  const withEnIt = shipped(
    { tops: NINE_PAIRS.tops, glossed: { ...NINE_PAIRS.glossed, "en-it": [91.4, 82.8, 70.3] } },
    { ...ROUTES, "en-it": ["en-it/base-memory/2.0"] },
  );

  it("leads with its readers, Italian speakers studying English, then the other readers", () => {
    const t = linguaPageText("it", withEnIt, noDiscord);
    expect(t.description).toContain("le parole in inglese che non conosci ancora");
    expect(t.heroNote).toContain("Pensata per italofoni che imparano l'inglese.");
    expect(t.heroNote).toContain(
      "Anche per francofoni che imparano l'inglese, lo spagnolo o l'italiano: l'interfaccia e le traduzioni sono in francese.",
    );
    expect(t.closing.body).toContain('<a href="/it/music">Cymbra Music</a>');
  });

  it("writes numbers as Italian does: 5000 solid, 20.000 grouped, 96% with no space", () => {
    const t = linguaPageText("it", withEnIt, noDiscord);
    expect(t.coverage.head).toEqual(["Parole più comuni", "5000", "10.000", "20.000"]);
    expect(t.coverage.rows[0]).toEqual(["Inglese → italiano", "91%", "83%", "70%"]);
  });

  it("contracts « da » and « a » with each language's article", () => {
    const { note } = linguaPageText("it", withEnIt, noDiscord).coverage;
    for (const words of ["dall'inglese all'italiano", "dallo spagnolo al francese", "dal francese allo spagnolo", "dall'italiano all'inglese"]) {
      expect(note).toContain(words);
    }
  });

  it("names the card's tense and gender and the estimated levels, never for English", () => {
    const t = linguaPageText("it", withEnIt, noDiscord);
    expect(card(t, "click").body).toContain(" In spagnolo, in italiano e in francese, la carta indica anche il tempo verbale e il genere.");
    expect(card(t, "level").body).toContain(" Per lo spagnolo, l'italiano e il francese, i livelli sono stimati");
    expect(card(t, "level").title).toContain("QCER");
  });

  it("quotes the word card's three buttons as the Italian card labels them, «» without inner spaces", () => {
    const catalogue = readFileSync(resolve(__dirname, "../../lingua-extension/src/i18n/it/card.ts"), "utf8");
    const body = card(linguaPageText("it", withEnIt, noDiscord), "click").body;
    for (const key of ["known", "addToDeck", "ignore"]) {
      const text = catalogue.match(new RegExp(`^  ${key}: "([^"]+)",$`, "m"))?.[1];
      expect(text, `it/card.ts ${key}`).toBeTruthy();
      expect(body).toContain(`«${text}»`);
    }
  });

  it("is built only with its readers' pair (D3, as Spanish's)", () => {
    expect(() => linguaPageText("it", nine, noDiscord)).toThrow("the it page is built only with a pair glossed in its language");
  });

  it("names Italian speakers in the French, English and Spanish tables", () => {
    expect(LINGUA_TEXT.fr.speakers.it).toBe("italophones");
    expect(LINGUA_TEXT.en.speakers.it).toBe("Italian speakers");
    expect(LINGUA_TEXT.es.speakers.it).toBe("italohablantes");
  });
});

describe("the Italian home's Lingua card follows the shipped pairs (change: add-site-italian-locale, D4)", () => {
  it("before a pair glossed in Italian: every language read, its readers, and the English Lingua page", () => {
    const { body, href } = italianHomeLinguaCard(nine.pairs);
    expect(body).toBe(
      "Leggi il web in inglese, in spagnolo, in francese o in italiano, con le parole che non conosci ancora evidenziate direttamente nella pagina. Una percentuale onesta per pagina, un clic per la traduzione e il tuo vocabolario, che si costruisce da solo. Pensata per francofoni, anglofoni e ispanofoni, con l'interfaccia e le traduzioni nella propria lingua.",
    );
    expect(href).toBe("/en/lingua");
  });

  it("with today's two French-glossed pairs: French speakers alone", () => {
    expect(italianHomeLinguaCard(today.pairs).body).toMatch(/^Leggi il web in inglese o in spagnolo, .* Pensata per francofoni, con l'interfaccia/);
  });

  it("once en-it ships: English, read with an Italian gloss, no audience, and /it/lingua", () => {
    const pairs = shipped({ tops: NINE_PAIRS.tops, glossed: { ...NINE_PAIRS.glossed, "en-it": [91.4, 82.8, 70.3] } }).pairs;
    const { body, href } = italianHomeLinguaCard(pairs);
    expect(body).toMatch(/^Leggi il web in inglese, con le parole/);
    expect(body).not.toContain("Pensata per");
    expect(href).toBe("/it/lingua");
  });

  it("refuses a language its table cannot name", () => {
    const pairs = shipped({ tops: [1], glossed: { "de-fr": [1] } }, {}).pairs;
    expect(() => italianHomeLinguaCard(pairs)).toThrow('the it table has no name for the language "de"');
  });
});

describe("the Italian note on pairs extended translation does not serve yet (review of change 41)", () => {
  it("reads whole whether a pair is named by its studied language or by both", () => {
    const one = shipped({ tops: [5000], glossed: { "en-it": [90] } }, {});
    expect(linguaPageText("it", one, noDiscord).coverage.note).toContain(" Non ancora disponibile: l'inglese.");
    const two = shipped({ tops: [5000], glossed: { "en-it": [90], "en-fr": [95] } }, {});
    expect(linguaPageText("it", two, noDiscord).coverage.note).toContain(
      " Non ancora disponibile: dall'inglese all'italiano e dall'inglese al francese.",
    );
  });
});
