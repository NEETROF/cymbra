import { describe, expect, it } from "vitest";
import coverage from "../src/data/lingua-coverage.json";
import { type LinguaPair, linguaPairs, shippedPairs, type Coverage, type Routes } from "../src/lib/lingua-pairs";
import { type CardKey, esAnd, esOr, fill, LINGUA_TEXT, type LinguaPageText, linguaPageText } from "../src/lib/lingua-text";
import { MATRIX, ROUTES, TODAY } from "./support/lingua";

// The words around the data (change: add-site-lingua-matrix-pages, D2 and D4), built from
// committed pair lists (`test/support/lingua.ts`): today's two French-native pairs read as
// the committed pages did; the matrix (es-en and en-es beside them) leads each page with its
// readers' pairs. The French bytes themselves are pinned on the rendered page by
// `test/astro/lingua-page.spec.ts` and on the built one by `test/post-build/lingua.spec.ts`.
// The last block alone reads the live pairs: the handover to change 34.

/** U+202F, the narrow no-break space: French and Spanish thousands, the Spanish per-cent sign. */
const NNBSP = "\u202F";
/** U+00A0, the no-break space `Intl` puts before the French per-cent sign. */
const NBSP = "\u00A0";

const shipped = (data: Coverage, routes: Routes = ROUTES) => shippedPairs(data, routes);
const today = shipped(TODAY);
const matrix = shipped(MATRIX);
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
      'Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur) et sur Safari (iPhone, iPad, Mac). <a href="https://discord.gg/example" rel="noopener">Rejoignez la communauté</a> pour en parler.',
    );
    expect(linguaPageText("fr", today, noDiscord).heroNote).toBe(
      "Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur) et sur Safari (iPhone, iPad, Mac).",
    );
    expect(t.cards.map((c) => c.key)).toEqual(["highlight", "percentage", "click", "deck", "offline", "level", "account", "languages"]);
    expect(card(t, "languages")).toEqual({
      key: "languages",
      title: "🧭 Anglais et espagnol → français",
      body: "Choisissez vos langues dans les Réglages : chaque page est lue dans la sienne. Plusieurs langues à la fois : gratuit pour l'instant.",
    });
    // The Spanish pack's card and levels sentences, since a listed pair studies Spanish.
    expect(card(t, "click").body).toMatch(/« Ignorer »\. En espagnol, la carte nomme aussi le temps et le genre\.$/);
    expect(card(t, "level").body).toMatch(/badge maison\. Pour l'espagnol, les niveaux sont estimés/);
  });

  it("French: the coverage table names the studied languages and the note says what is translated", () => {
    const { coverage: table } = linguaPageText("fr", today, noDiscord);
    expect(table.caption).toBe(
      "La part des mots les plus courants qui ont une définition en français, mesurée de la même façon pour chaque langue, sur les dictionnaires livrés avec l'extension.",
    );
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
      'Available on Chrome, Edge and other Chromium browsers, on Firefox (desktop) and on Safari (iPhone, iPad, Mac). Made for French speakers learning English or Spanish: the interface and the translations are in French. <a href="https://discord.gg/example" rel="noopener">Join the community</a> to talk about it.',
    );
    expect(card(t, "languages").title).toBe("🧭 English and Spanish → French");
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
      "Available on Chrome, Edge and other Chromium browsers, on Firefox (desktop) and on Safari (iPhone, iPad, Mac). Made for English speakers learning Spanish. Also for French speakers learning English or Spanish: the interface and the translations are in French. Also for Spanish speakers learning English: the interface and the translations are in Spanish.",
    );
    expect(card(t, "languages").title).toBe("🧭 Spanish → English · English and Spanish → French · English → Spanish");
    expect(t.coverage.head).toEqual([
      "Commonest words",
      "Spanish → English",
      "English → French",
      "Spanish → French",
      "English → Spanish",
    ]);
    expect(t.coverage.rows[0]).toEqual(["5,000", "90%", "95%", "88%", "96%"]);
  });

  it("Spanish: English for Spanish speakers first, in Spanish (tú, RAE numbers)", () => {
    const t = linguaPageText("es", matrix, discord);
    expect(t.title).toBe("Cymbra Lingua — amplía tu vocabulario leyendo la web");
    expect(t.tagline).toBe(
      "Una extensión para el navegador que resalta las palabras en inglés que todavía no conoces, directamente en la página. Sin aplicación que abrir ni sesión de repaso que programar: lees y aprendes.",
    );
    expect(t.heroNote).toBe(
      'Disponible en Chrome, Edge y otros navegadores Chromium, en Firefox (escritorio) y en Safari (iPhone, iPad, Mac). Pensada para hispanohablantes que aprenden inglés. También para francohablantes que aprenden inglés o español: la interfaz y las traducciones están en francés. También para anglohablantes que aprenden español: la interfaz y las traducciones están en inglés. <a href="https://discord.gg/example" rel="noopener">Únete a la comunidad</a> para hablar de ello.',
    );
    expect(card(t, "languages").title).toBe("🧭 Inglés → español · Inglés y español → francés · Español → inglés");
    // es-fr and es-en study Spanish: the card's and the levels' sentences, in the extension's words.
    expect(card(t, "click").body).toMatch(/e «Ignorar»\. En español, la tarjeta también indica el tiempo verbal y el género\.$/);
    expect(card(t, "level").body).toMatch(/a falta de una lista MCER de uso libre\.$/);
    expect(t.coverage.head).toEqual([
      "Palabras más frecuentes",
      "Inglés → español",
      "Inglés → francés",
      "Español → francés",
      "Español → inglés",
    ]);
    // RAE: 5000 solid, 10 000 and 20 000 with a narrow no-break space, and one before the
    // per-cent sign — the extension's `formatNumber` and `formatPercent`.
    expect(t.coverage.rows).toEqual([
      ["5000", `96${NNBSP}%`, `95${NNBSP}%`, `88${NNBSP}%`, `90${NNBSP}%`],
      [`10${NNBSP}000`, `91${NNBSP}%`, `90${NNBSP}%`, `77${NNBSP}%`, `80${NNBSP}%`],
      [`20${NNBSP}000`, `80${NNBSP}%`, `79${NNBSP}%`, `64${NNBSP}%`, `70${NNBSP}%`],
    ]);
    expect(t.closing.body).toContain('<a href="/en/music">Cymbra Music</a>');
  });

  it("French: the French-native pairs first, the others named for their readers", () => {
    const t = linguaPageText("fr", matrix, noDiscord);
    expect(t.heroNote).toBe(
      "Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur) et sur Safari (iPhone, iPad, Mac). Conçue pour les francophones qui apprennent l'anglais ou l'espagnol. Aussi pour les anglophones qui apprennent l'espagnol : l'interface et les traductions sont en anglais. Aussi pour les hispanophones qui apprennent l'anglais : l'interface et les traductions sont en espagnol.",
    );
    expect(card(t, "languages").title).toBe("🧭 Anglais et espagnol → français · Espagnol → anglais · Anglais → espagnol");
    expect(t.coverage.head).toEqual([
      "Mots les plus courants",
      "Anglais → français",
      "Espagnol → français",
      "Espagnol → anglais",
      "Anglais → espagnol",
    ]);
  });

  it("a page whose only pairs are its readers' carries no audience sentence, like today's French page", () => {
    const enEs = shipped({ tops: MATRIX.tops, glossed: { "en-es": [96.0, 91.0, 80.0] } });
    const t = linguaPageText("es", enEs, noDiscord);
    expect(t.heroNote).toBe(
      "Disponible en Chrome, Edge y otros navegadores Chromium, en Firefox (escritorio) y en Safari (iPhone, iPad, Mac).",
    );
    expect(card(t, "languages").title).toBe("🧭 Inglés → español");
    // No listed pair studies Spanish: the Spanish pack's sentences stay out.
    expect(card(t, "click").body).toMatch(/e «Ignorar»\.$/);
    expect(card(t, "level").body).toMatch(/insignia de la casa\.$/);
    expect(t.coverage.caption).toBe(
      "La proporción de las palabras más frecuentes que tienen una definición en español, medida de la misma manera para cada idioma, en los diccionarios que se entregan con la extensión.",
    );
    expect(t.coverage.head).toEqual(["Palabras más frecuentes", "Inglés"]);
    expect(t.coverage.note).toBe(
      "Las definiciones proceden del Wikcionario y de traducciones escritas por personas, nunca de una traducción automática. La traducción ampliada, opcional, traduce el inglés.",
    );
  });

  it("a page none of whose pairs is its readers' says whom the product is made for", () => {
    // The French page is always built: with no French-glossed pair, it presents the first
    // other group as today's English page presents the French speakers.
    const noFrench = shipped({ tops: MATRIX.tops, glossed: { "es-en": [90, 80, 70], "en-es": [96, 91, 80] } });
    expect(linguaPageText("fr", noFrench, noDiscord).heroNote).toBe(
      "Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur) et sur Safari (iPhone, iPad, Mac). Conçue pour les anglophones qui apprennent l'espagnol : l'interface et les traductions sont en anglais. Aussi pour les hispanophones qui apprennent l'anglais : l'interface et les traductions sont en espagnol.",
    );
    // The Spanish page is built only with a Spanish-glossed pair (D3): it has no such sentence.
    expect(() => linguaPageText("es", today, noDiscord)).toThrow(/the es page is built only with a pair glossed in its language/);
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
      "Las definiciones proceden del Wikcionario y de traducciones escritas por personas, nunca de una traducción automática. La traducción ampliada, opcional, traduce del inglés al español, del inglés al francés, del español al francés (pasando por el inglés) y del español al inglés.",
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

describe("nothing but the site's own text reaches set:html", () => {
  it("refuses a language the table has no name for, at build time", () => {
    const italian = shipped({ tops: MATRIX.tops, glossed: { "it-fr": [1, 1, 1] } }, {});
    expect(() => linguaPageText("fr", italian, noDiscord)).toThrow(/no name for the language "it"/);
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

describe("the handover to change 34 (enable-lingua-english-speakers)", () => {
  // The English table keeps today's page while every pair is glossed in French: the card
  // quotes the French buttons and says "its French translation". Once a pair glossed in
  // English ships (es-en, change 34), an English speaker reads that card about their own
  // pair: change 34's task 1.2 replaces these words in the same pull request.
  const FRENCH_WORDING = ["its French translation", "« Je connais »", "« + Deck »", "« Ignorer »"];
  const englishGlossed = linguaPairs(coverage, {})
    .filter((p) => p.native === "en")
    .map((p) => p.pair);
  const left = FRENCH_WORDING.filter((words) => JSON.stringify(LINGUA_TEXT.en).includes(words));

  it("no English-glossed pair ships while the English table speaks of the French interface", () => {
    if (englishGlossed.length === 0) {
      // Today: the English page is today's page, French buttons and all (D1).
      expect(left).toEqual(FRENCH_WORDING);
      return;
    }
    expect(
      left,
      `${englishGlossed.join(", ")} ships glossed in English, but the English table of src/lib/lingua-text.ts still says ${left.join(", ")}: ` +
        "change 34 (enable-lingua-english-speakers) task 1.2 replaces them with the English card's words",
    ).toEqual([]);
  });
});
