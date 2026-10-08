import { describe, expect, it } from "vitest";
import manifest from "../../lingua-extension/model-manifest.json";
import { linguaPairs, type Coverage } from "../src/lib/lingua-pairs";
import { linguaPageText } from "../src/lib/lingua-text";

// The words around the data (change: add-site-lingua-matrix-pages, D2 and D4), built from
// the pairs: today's two French-native pairs read as the committed pages did; the matrix
// (es-en and en-es beside them) leads each page with its readers' pairs. The French bytes
// themselves are pinned by `test/post-build/lingua.spec.ts` on the built page.

const today = linguaPairs();
const matrix: Coverage = {
  tops: [5000, 10000, 20000],
  glossed: {
    "en-fr": [95.1, 90.1, 78.9],
    "es-fr": [87.6, 77.2, 63.7],
    "es-en": [90.0, 80.0, 70.0],
    "en-es": [96.0, 91.0, 80.0],
  },
};
const pairs = linguaPairs(matrix, manifest.routes);
const discord = { discordUrl: "https://discord.gg/example" };
const noDiscord = { discordUrl: null };

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
    expect(t.cards.at(-1)).toEqual({
      title: "🧭 Anglais et espagnol → français",
      body: "Choisissez vos langues dans les Réglages : chaque page est lue dans la sienne. Plusieurs langues à la fois : gratuit pour l'instant.",
    });
    // The Spanish pack's card and levels sentences, since a listed pair studies Spanish.
    expect(t.cards[2].body).toMatch(/« Ignorer »\. En espagnol, la carte nomme aussi le temps et le genre\.$/);
    expect(t.cards[5].body).toMatch(/badge maison\. Pour l'espagnol, les niveaux sont estimés/);
  });

  it("French: the coverage table names the studied languages and the note says what is translated", () => {
    const { coverage } = linguaPageText("fr", today, noDiscord);
    expect(coverage.caption).toBe(
      "La part des mots les plus courants qui ont une définition en français, mesurée de la même façon pour chaque langue, sur les dictionnaires livrés avec l'extension.",
    );
    expect(coverage.head).toEqual(["Mots les plus courants", "Anglais", "Espagnol"]);
    expect(coverage.rows).toEqual([
      ["5 000", "95 %", "88 %"],
      ["10 000", "90 %", "77 %"],
      ["20 000", "79 %", "64 %"],
    ]);
    expect(coverage.note).toBe(
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
    expect(t.cards.at(-1)!.title).toBe("🧭 English and Spanish → French");
    expect(t.coverage.caption).toMatch(/^The share of the commonest words that have a French gloss, measured the same way for each language/);
    expect(t.coverage.head).toEqual(["Commonest words", "English", "Spanish"]);
    expect(t.coverage.rows[0]).toEqual(["5,000", "95%", "88%"]);
    expect(t.coverage.note).toMatch(/ Extended translation, optional, serves English and Spanish \(through English\)\.$/);
    expect(t.closing.body).toContain('<a href="/en/music">Cymbra Music</a>');
  });
});

describe("the matrix: each page leads with its readers' pairs (D2)", () => {
  it("English: Spanish for English speakers first, then the pairs for French speakers, then for Spanish speakers", () => {
    const t = linguaPageText("en", pairs, noDiscord);
    expect(t.tagline).toMatch(/^A browser extension that highlights the Spanish words you do not know yet/);
    expect(t.heroNote).toBe(
      "Available on Chrome, Edge and other Chromium browsers, on Firefox (desktop) and on Safari (iPhone, iPad, Mac). Made for English speakers learning Spanish. Also for French speakers learning English or Spanish: the interface and the translations are in French. Also for Spanish speakers learning English: the interface and the translations are in Spanish.",
    );
    expect(t.cards.at(-1)!.title).toBe("🧭 Spanish → English · English and Spanish → French · English → Spanish");
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
    const t = linguaPageText("es", pairs, discord);
    expect(t.title).toBe("Cymbra Lingua — amplía tu vocabulario leyendo la web");
    expect(t.tagline).toBe(
      "Una extensión para el navegador que resalta las palabras en inglés que todavía no conoces, directamente en la página. Sin aplicación que abrir ni sesión de repaso que programar: lees y aprendes.",
    );
    expect(t.heroNote).toBe(
      'Disponible en Chrome, Edge y otros navegadores Chromium, en Firefox (ordenador) y en Safari (iPhone, iPad, Mac). Pensada para hispanohablantes que aprenden inglés. También para francohablantes que aprenden inglés o español: la interfaz y las traducciones están en francés. También para anglohablantes que aprenden español: la interfaz y las traducciones están en inglés. <a href="https://discord.gg/example" rel="noopener">Únete a la comunidad</a> para hablar de ello.',
    );
    expect(t.cards.at(-1)!.title).toBe("🧭 Inglés → español · Inglés y español → francés · Español → inglés");
    expect(t.coverage.head).toEqual([
      "Palabras más frecuentes",
      "Inglés → español",
      "Inglés → francés",
      "Español → francés",
      "Español → inglés",
    ]);
    // RAE: 5000 solid, 10 000 and 20 000 with a thin space, a space before the per-cent sign.
    expect(t.coverage.rows).toEqual([
      ["5000", "96 %", "95 %", "88 %", "90 %"],
      ["10 000", "91 %", "90 %", "77 %", "80 %"],
      ["20 000", "80 %", "79 %", "64 %", "70 %"],
    ]);
    expect(t.closing.body).toContain('<a href="/en/music">Cymbra Music</a>');
  });

  it("French: the French-native pairs first, the others named for their readers", () => {
    const t = linguaPageText("fr", pairs, noDiscord);
    expect(t.heroNote).toBe(
      "Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur) et sur Safari (iPhone, iPad, Mac). Conçue pour les francophones qui apprennent l'anglais ou l'espagnol. Aussi pour les anglophones qui apprennent l'espagnol : l'interface et les traductions sont en anglais. Aussi pour les hispanophones qui apprennent l'anglais : l'interface et les traductions sont en espagnol.",
    );
    expect(t.cards.at(-1)!.title).toBe("🧭 Anglais et espagnol → français · Espagnol → anglais · Anglais → espagnol");
    expect(t.coverage.head).toEqual([
      "Mots les plus courants",
      "Anglais → français",
      "Espagnol → français",
      "Espagnol → anglais",
      "Anglais → espagnol",
    ]);
  });

  it("a page whose only pairs are its readers' carries no audience sentence, like today's French page", () => {
    const enEs = linguaPairs({ tops: matrix.tops, glossed: { "en-es": [96.0, 91.0, 80.0] } }, manifest.routes);
    const t = linguaPageText("es", enEs, noDiscord);
    expect(t.heroNote).toBe(
      "Disponible en Chrome, Edge y otros navegadores Chromium, en Firefox (ordenador) y en Safari (iPhone, iPad, Mac).",
    );
    expect(t.cards.at(-1)!.title).toBe("🧭 Inglés → español");
    // No listed pair studies Spanish: the Spanish pack's sentences stay out.
    expect(t.cards[2].body).toMatch(/e «Ignorar»\.$/);
    expect(t.cards[5].body).toMatch(/insignia de la casa\.$/);
    expect(t.coverage.caption).toBe(
      "La proporción de las palabras más frecuentes que tienen una definición en español, medida de la misma manera para cada idioma, en los diccionarios que se entregan con la extensión.",
    );
    expect(t.coverage.head).toEqual(["Palabras más frecuentes", "Inglés"]);
    expect(t.coverage.note).toBe(
      "Las definiciones proceden del Wikcionario y de traducciones escritas por personas, nunca de una traducción automática. La traducción ampliada, opcional, traduce el inglés.",
    );
  });

  it("Spanish lists: « y » becomes « e » before an i- sound", () => {
    const swapped = linguaPairs({ tops: matrix.tops, glossed: { "es-fr": [1, 1, 1], "en-fr": [1, 1, 1] } }, manifest.routes);
    expect(linguaPageText("es", swapped, noDiscord).cards.at(-1)!.title).toBe("🧭 Español e inglés → francés");
  });
});

describe("the coverage table and the translation note per pair (D4)", () => {
  it("captions 'a gloss in the reader's language' once pairs of two native languages are listed", () => {
    expect(linguaPageText("en", pairs, noDiscord).coverage.caption).toBe(
      "The share of the commonest words that have a gloss in the reader's language, measured the same way for each pair, in the dictionaries the extension ships.",
    );
    expect(linguaPageText("fr", pairs, noDiscord).coverage.caption).toBe(
      "La part des mots les plus courants qui ont une définition dans la langue du lecteur, mesurée de la même façon pour chaque paire, sur les dictionnaires livrés avec l'extension.",
    );
    expect(linguaPageText("es", pairs, noDiscord).coverage.caption).toBe(
      "La proporción de las palabras más frecuentes que tienen una definición en la lengua del lector, medida de la misma manera para cada par, en los diccionarios que se entregan con la extensión.",
    );
  });

  it("says, per pair, whether extended translation serves it, directly or through English", () => {
    expect(linguaPageText("en", pairs, noDiscord).coverage.note).toBe(
      "Glosses come from Wiktionary and from translations written by people, never from machine translation. Extended translation, optional, serves Spanish to English, English to French, Spanish to French (through English) and English to Spanish.",
    );
    expect(linguaPageText("fr", pairs, noDiscord).coverage.note).toBe(
      "Les définitions viennent du Wiktionnaire et de traductions écrites par des personnes, jamais d'une traduction automatique. La traduction étendue, facultative, traduit l'anglais vers le français, l'espagnol vers le français (en passant par l'anglais), l'espagnol vers l'anglais et l'anglais vers l'espagnol.",
    );
    expect(linguaPageText("es", pairs, noDiscord).coverage.note).toBe(
      "Las definiciones proceden del Wikcionario y de traducciones escritas por personas, nunca de una traducción automática. La traducción ampliada, opcional, traduce del inglés al español, del inglés al francés, del español al francés (pasando por el inglés) y del español al inglés.",
    );
  });

  it("names the pairs extended translation does not serve yet", () => {
    const { "en-es": _, "es-en": __, ...routes } = manifest.routes;
    const partly = linguaPairs(matrix, routes);
    expect(linguaPageText("en", partly, noDiscord).coverage.note).toMatch(
      / Extended translation, optional, serves English to French and Spanish to French \(through English\)\. Not yet for Spanish to English and English to Spanish\.$/,
    );
    expect(linguaPageText("fr", partly, noDiscord).coverage.note).toMatch(/ Pas encore pour l'espagnol vers l'anglais et l'anglais vers l'espagnol\.$/);
    expect(linguaPageText("es", partly, noDiscord).coverage.note).toMatch(/ Pendiente: del inglés al español y del español al inglés\.$/);
    // One native language, one pair not served: the old wording, "Spanish follows".
    const onlyEn = linguaPairs({ tops: matrix.tops, glossed: { "en-fr": [1, 1, 1], "es-fr": [1, 1, 1] } }, { "en-fr": ["x"] });
    expect(linguaPageText("fr", onlyEn, noDiscord).coverage.note).toMatch(/ traduit l'anglais\. Pas encore pour l'espagnol\.$/);
    expect(linguaPageText("en", onlyEn, noDiscord).coverage.note).toMatch(/ serves English\. Not yet for Spanish\.$/);
  });

  it("refuses a language the table has no name for, at build time", () => {
    const italian = linguaPairs({ tops: matrix.tops, glossed: { "it-fr": [1, 1, 1] } }, {});
    expect(() => linguaPageText("fr", italian, noDiscord)).toThrow(/no name for the language "it"/);
  });

  it("escapes the invite's address in the link", () => {
    const t = linguaPageText("fr", today, { discordUrl: 'https://x.test/?a=1&b="2"' });
    expect(t.heroNote).toContain('<a href="https://x.test/?a=1&amp;b=&quot;2&quot;" rel="noopener">');
  });
});
