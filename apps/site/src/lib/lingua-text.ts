// The words of the Lingua page, one table per site language, and the builders that fill
// them from the shipped pairs (change: add-site-lingua-matrix-pages, D1, D2, D4). Read by
// `src/components/LinguaPage.astro` — and, for the Lingua card of the Spanish home page,
// by `src/pages/es/index.astro` (`spanishHomeLinguaCard`, change: extend-site-spanish-locale, D3).
//
// Every string is the INNER HTML of its slot — inserted with `set:html`, never through an
// `{expression}`: an expression escapes apostrophes (`'` → `&#39;`) and `&nbsp;`, and the
// French table is today's page byte for byte (this change's D1). The text holds no reader
// input, and what comes from data cannot carry markup: `lingua-pairs.ts` refuses a pair key
// that is not two language codes, a language is named only from these tables
// (`Object.hasOwn`), and `fill` escapes `&`, `<` and `>` in every value it inserts — not
// `'`, so the bytes stay today's.
//
// Placeholders (`{name}`) are filled by `linguaPageText`:
//   {ofStudied}     the lead group's studied languages, as the tagline needs them
//                   (fr « d'anglais ou d'espagnol », en "English or Spanish", es « en inglés o en francés »)
//   {learnStudied}  a group's studied languages after "learning" (fr « l'anglais ou l'espagnol »)
//   {speakers}      a group's readers (fr « francophones », en "French speakers")
//   {native}        a language name, as the sentence needs it (fr « français », en "French")
//   {served}        the pairs extended translation serves, a list; {unserved} the others
//   {music}         the Music page's address in this language
// The English table keeps today's page as it is, « Je connais », « + Deck » and "its French
// translation" included: change 34 (enable-lingua-english-speakers, task 1.2) replaces them
// when it ships the English interface, and `test/lingua-text.spec.ts` fails if an
// English-glossed pair ships first. The Spanish table follows the extension's conventions
// (`apps/lingua-extension/src/i18n/README.md`: tú, neutral, no vosotros; RAE numbers, « 20 000 »
// and « 96 % » with a narrow no-break space) and quotes its Spanish labels
// (`apps/lingua-extension/src/i18n/es/card.ts`).

import type { Lang } from "./i18n";
import { type LinguaPair, linguaHref, type NativeGroup, pairsByNative, type ShippedPairs } from "./lingua-pairs";

/** The feature cards, named so that a sentence is appended to the right one. */
export type CardKey = "highlight" | "percentage" | "click" | "deck" | "offline" | "level" | "account" | "languages";

interface LinguaTable {
  /** ISO 639-1 → the language's name, as this language writes it in a sentence. */
  names: Record<string, string>;
  /** ISO 639-1 → its speakers, as this language names them after "for". */
  speakers: Record<string, string>;
  title: string;
  description: string;
  kicker: string;
  headline: string;
  tagline: string;
  /** Where the extension runs; the audience sentences follow it in the hero note. */
  platforms: string;
  /** The readers' pairs, when pairs for other readers are listed too. */
  forReaders: string;
  /**
   * Another group's pairs, when no readers' pair precedes them (today's English page).
   * `null` for a language whose page is built only with its readers' pairs (D3: Spanish).
   */
  madeFor: string | null;
  /** Another group's pairs, after the readers'. */
  alsoFor: string;
  joinCommunity: string;
  toTalk: string;
  featuresId: string;
  cards: { key: CardKey; title: string; body: string }[];
  /** Appended to the `click` and `level` cards when a listed pair studies Spanish. */
  spanishCard: string;
  spanishLevels: string;
  /** The 🧭 `languages` card: its title is the groups, « <studied> → <native> »; its body is this. */
  languagesCard: string;
  coverageHeading: string;
  /** The caption while every listed pair is glossed in one language, and once in two (D4). */
  captionOneNative: string;
  captionPerPair: string;
  commonestWords: string;
  glossesOrigin: string;
  translationServes: string;
  translationNotYet: string;
  throughEnglish: string;
  closingHeading: string;
  closingBody: string;
}

const fr: LinguaTable = {
  names: { en: "anglais", es: "espagnol", fr: "français" },
  speakers: { fr: "francophones", en: "anglophones", es: "hispanophones" },
  title: "Cymbra Lingua — enrichissez votre vocabulaire en lisant le web",
  description:
    "Une extension navigateur qui surligne, sur la page que vous lisez, les mots {ofStudied} que vous ne connaissez pas encore. Analyse hors-ligne, aucun compte requis.",
  kicker: 'Cymbra Lingua · <span class="badge">bêta</span>',
  headline: 'Votre vocabulaire, <span class="gradient">là où vous lisez déjà</span>',
  tagline:
    "Une extension navigateur qui surligne les mots {ofStudied} que vous ne connaissez pas encore, directement sur la page. Pas d'appli à ouvrir, pas de session de révision à caler : vous lisez, vous apprenez.",
  platforms:
    "Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur) et sur Safari (iPhone, iPad, Mac).",
  forReaders: "Conçue pour les {speakers} qui apprennent {learnStudied}.",
  madeFor: "Conçue pour les {speakers} qui apprennent {learnStudied} : l'interface et les traductions sont en {native}.",
  alsoFor: "Aussi pour les {speakers} qui apprennent {learnStudied} : l'interface et les traductions sont en {native}.",
  joinCommunity: "Rejoignez la communauté",
  toTalk: " pour en parler.",
  featuresId: "fonctionnalites",
  cards: [
    {
      key: "highlight",
      title: "🖍️ Surligné sur place",
      body: "Les mots inconnus apparaissent surlignés sur la page elle-même — sans rien casser du site que vous lisez.",
    },
    {
      key: "percentage",
      title: "📊 Un pourcentage honnête",
      body: "L'icône affiche la part de mots que vous connaissez sur la page courante. Un chiffre par page, pas un score gonflé.",
    },
    {
      key: "click",
      title: "👆 Un clic pour comprendre",
      body: "Cliquez un mot surligné : sa forme du dictionnaire, sa traduction, sa fréquence en français clair, et trois boutons — « Je connais », « + Deck », « Ignorer ».",
    },
    {
      key: "deck",
      title: "🗂️ Deck et révisions",
      body: "Les mots et expressions capturés deviennent des cartes, avec la phrase où vous les avez rencontrés. Révision dans le panneau latéral.",
    },
    {
      key: "offline",
      title: "📴 Hors-ligne par construction",
      body: "L'analyse tourne dans votre navigateur, sur un dictionnaire embarqué. Aucune page lue n'est envoyée nulle part.",
    },
    {
      key: "level",
      title: "📈 Votre niveau, en CEFR",
      body: "Une estimation de votre vocabulaire, adossée à l'échelle A1→C2 plutôt qu'à un badge maison.",
    },
    {
      key: "account",
      title: "🪪 Compte optionnel",
      body: "Sans compte, tout reste local. Connecté à Cymbra ID, votre deck et vos statuts suivent d'un appareil à l'autre.",
    },
  ],
  spanishCard: " En espagnol, la carte nomme aussi le temps et le genre.",
  spanishLevels: " Pour l'espagnol, les niveaux sont estimés d'après la fréquence des mots, faute de liste CECR libre de droits.",
  languagesCard: "Choisissez vos langues dans les Réglages : chaque page est lue dans la sienne.",
  coverageHeading: "Ce que couvre le dictionnaire",
  captionOneNative:
    "La part des mots les plus courants qui ont une définition en {native}, mesurée de la même façon pour chaque langue, sur les dictionnaires livrés avec l'extension.",
  captionPerPair:
    "La part des mots les plus courants qui ont une définition dans la langue du lecteur, mesurée de la même façon pour chaque paire, sur les dictionnaires livrés avec l'extension.",
  commonestWords: "Mots les plus courants",
  glossesOrigin:
    "Les définitions viennent du Wiktionnaire et de traductions écrites par des personnes, jamais d'une traduction automatique.",
  translationServes: " La traduction étendue, facultative, traduit {served}.",
  translationNotYet: " Pas encore pour {unserved}.",
  throughEnglish: " (en passant par l'anglais)",
  closingHeading: "Et la musique&nbsp;?",
  closingBody:
    'Lingua partage son compte, sa philosophie et son respect de vos données avec <a href="{music}">Cymbra Music</a>, disponible sur l\'App Store et Google Play.',
};

const en: LinguaTable = {
  names: { en: "English", es: "Spanish", fr: "French" },
  speakers: { fr: "French speakers", en: "English speakers", es: "Spanish speakers" },
  title: "Cymbra Lingua — grow your vocabulary while you read the web",
  description:
    "A browser extension that highlights, on the page you are reading, the {ofStudied} words you do not know yet. Offline analysis, no account required.",
  kicker: 'Cymbra Lingua · <span class="badge">beta</span>',
  headline: 'Your vocabulary, <span class="gradient">where you already read</span>',
  tagline:
    "A browser extension that highlights the {ofStudied} words you do not know yet, right on the page. No app to open, no revision session to schedule: you read, you learn.",
  platforms:
    "Available on Chrome, Edge and other Chromium browsers, on Firefox (desktop) and on Safari (iPhone, iPad, Mac).",
  forReaders: "Made for {speakers} learning {learnStudied}.",
  madeFor: "Made for {speakers} learning {learnStudied}: the interface and the translations are in {native}.",
  alsoFor: "Also for {speakers} learning {learnStudied}: the interface and the translations are in {native}.",
  joinCommunity: "Join the community",
  toTalk: " to talk about it.",
  featuresId: "features",
  cards: [
    {
      key: "highlight",
      title: "🖍️ Highlighted in place",
      body: "Unknown words appear highlighted on the page itself — without breaking the site you are reading.",
    },
    {
      key: "percentage",
      title: "📊 An honest percentage",
      body: "The icon shows how much of the current page you actually know. One number per page, not an inflated score.",
    },
    {
      key: "click",
      title: "👆 One click to understand",
      body: "Click a highlighted word: its dictionary form, its French translation, its frequency in plain language, and three buttons to mark it known, add it to your deck, or ignore it — « Je connais », « + Deck », « Ignorer ».",
    },
    {
      key: "deck",
      title: "🗂️ Deck and review",
      body: "Captured words and phrases become cards, carrying the sentence you met them in. Review them from the side panel.",
    },
    {
      key: "offline",
      title: "📴 Offline by construction",
      body: "The analysis runs inside your browser, against a bundled dictionary. No page you read is ever sent anywhere.",
    },
    {
      key: "level",
      title: "📈 Your level, on the CEFR scale",
      body: "An estimate of your vocabulary, anchored to A1→C2 rather than to a homemade badge.",
    },
    {
      key: "account",
      title: "🪪 Optional account",
      body: "Signed out, everything stays local. Signed in to Cymbra ID, your deck and word statuses follow you across devices.",
    },
  ],
  spanishCard: " In Spanish, the card also names the tense and the gender.",
  spanishLevels: " For Spanish, the levels are estimated from word frequency, as no CEFR list can be shipped freely.",
  languagesCard: "Choose your languages in Settings: each page is read in its own.",
  coverageHeading: "What the dictionary covers",
  captionOneNative:
    "The share of the commonest words that have a {native} gloss, measured the same way for each language, in the dictionaries the extension ships.",
  captionPerPair:
    "The share of the commonest words that have a gloss in the reader's language, measured the same way for each pair, in the dictionaries the extension ships.",
  commonestWords: "Commonest words",
  glossesOrigin: "Glosses come from Wiktionary and from translations written by people, never from machine translation.",
  translationServes: " Extended translation, optional, serves {served}.",
  translationNotYet: " Not yet for {unserved}.",
  throughEnglish: " (through English)",
  closingHeading: "What about music?",
  closingBody:
    'Lingua shares its account, its philosophy and its respect for your data with <a href="{music}">Cymbra Music</a>, available on the App Store and Google Play.',
};

const es: LinguaTable = {
  names: { en: "inglés", es: "español", fr: "francés" },
  speakers: { es: "hispanohablantes", en: "anglohablantes", fr: "francohablantes" },
  title: "Cymbra Lingua — amplía tu vocabulario leyendo la web",
  description:
    "Una extensión para el navegador que resalta, en la página que lees, las palabras {ofStudied} que todavía no conoces. Análisis sin conexión, sin necesidad de cuenta.",
  kicker: 'Cymbra Lingua · <span class="badge">beta</span>',
  headline: 'Tu vocabulario, <span class="gradient">donde ya lees</span>',
  tagline:
    "Una extensión para el navegador que resalta las palabras {ofStudied} que todavía no conoces, directamente en la página. Sin aplicación que abrir ni sesión de repaso que programar: lees y aprendes.",
  platforms:
    "Disponible en Chrome, Edge y otros navegadores Chromium, en Firefox (escritorio) y en Safari (iPhone, iPad, Mac).",
  forReaders: "Pensada para {speakers} que aprenden {learnStudied}.",
  // The Spanish page is built only once a pair glossed in Spanish ships (D3): it always has readers.
  madeFor: null,
  alsoFor: "También para {speakers} que aprenden {learnStudied}: la interfaz y las traducciones están en {native}.",
  joinCommunity: "Únete a la comunidad",
  toTalk: " para hablar de ello.",
  featuresId: "funciones",
  cards: [
    {
      key: "highlight",
      title: "🖍️ Resaltado en la propia página",
      body: "Las palabras desconocidas aparecen resaltadas en la propia página, sin romper nada del sitio que estás leyendo.",
    },
    {
      key: "percentage",
      title: "📊 Un porcentaje honesto",
      body: "El icono muestra la proporción de palabras que conoces en la página actual. Una cifra por página, no una puntuación inflada.",
    },
    {
      key: "click",
      title: "👆 Un clic para entender",
      body: "Haz clic en una palabra resaltada: su forma de diccionario, su traducción, su frecuencia explicada con claridad y tres botones: «La conozco», «+ Mazo» e «Ignorar».",
    },
    {
      key: "deck",
      title: "🗂️ Mazo y repaso",
      body: "Las palabras y expresiones capturadas se convierten en tarjetas, con la frase en la que las encontraste. Repaso en el panel lateral.",
    },
    {
      key: "offline",
      title: "📴 Sin conexión por diseño",
      body: "El análisis se ejecuta en tu navegador, con un diccionario integrado. Ninguna página leída se envía a ningún sitio.",
    },
    {
      key: "level",
      title: "📈 Tu nivel, en la escala MCER",
      body: "Una estimación de tu vocabulario, referida a la escala A1→C2 y no a una insignia de la casa.",
    },
    {
      key: "account",
      title: "🪪 Cuenta opcional",
      body: "Sin cuenta, todo se queda en tu dispositivo. Con la sesión iniciada en Cymbra ID, tu mazo y tus estados te siguen de un dispositivo a otro.",
    },
  ],
  spanishCard: " En español, la tarjeta también indica el tiempo verbal y el género.",
  spanishLevels:
    " Para el español, los niveles se estiman a partir de la frecuencia de las palabras, a falta de una lista MCER de uso libre.",
  languagesCard: "Elige tus idiomas en los Ajustes: cada página se lee en el suyo.",
  coverageHeading: "Qué cubre el diccionario",
  captionOneNative:
    "La proporción de las palabras más frecuentes que tienen una definición en {native}, medida de la misma manera para cada idioma, en los diccionarios que se entregan con la extensión.",
  captionPerPair:
    "La proporción de las palabras más frecuentes que tienen una definición en la lengua del lector, medida de la misma manera para cada par, en los diccionarios que se entregan con la extensión.",
  commonestWords: "Palabras más frecuentes",
  glossesOrigin:
    "Las definiciones proceden del Wikcionario y de traducciones escritas por personas, nunca de una traducción automática.",
  translationServes: " La traducción ampliada, opcional, traduce {served}.",
  translationNotYet: " Pendiente: {unserved}.",
  throughEnglish: " (pasando por el inglés)",
  closingHeading: "¿Y la música?",
  closingBody:
    'Lingua comparte su cuenta, su filosofía y su respeto por tus datos con <a href="{music}">Cymbra Music</a>, disponible en el App Store y en Google Play.',
};

export const LINGUA_TEXT: Record<Lang, LinguaTable> = { fr, en, es };

/** Where each language's page sends its readers for Cymbra Music: its Music page in that language. */
const MUSIC_HREF: Record<Lang, string> = { fr: "/music", en: "/en/music", es: "/es/music" };

// Each language's grammar around a language name: the forms the sentences need, the list
// conjunctions and the number formats. Code, not copy: nothing here reaches the page on its
// own.
interface Grammar {
  /** « les mots d'anglais » / "the English words" / « las palabras en inglés ». */
  of: (name: string) => string;
  /** « apprennent l'anglais » / "learning English" / « aprenden inglés ». */
  learn: (name: string) => string;
  /** A pair named by its studied language alone: « l'anglais » / "English" / « el inglés ». */
  the: (name: string) => string;
  /** A pair named by both languages: « l'anglais vers le français » / "English to French" / « del inglés al francés ». */
  to: (studied: string, native: string) => string;
  and: (items: string[]) => string;
  or: (items: string[]) => string;
  count: (n: number) => string;
  percent: (share: number) => string;
}

const frElides = (name: string): boolean => /^[aeiouyàâäéèêëîïôöùûüh]/i.test(name);
const joinList = (items: string[], last: string): string =>
  items.length <= 1 ? (items[0] ?? "") : `${items.slice(0, -1).join(", ")} ${last} ${items[items.length - 1]}`;
// The extension's Spanish numbers (`apps/lingua-extension/src/i18n/index.ts`, `formatNumber`
// and `formatPercent`): as the RAE writes them, four digits solid (5000), longer ones in
// groups of three parted by a narrow no-break space (20 000); a narrow no-break space before
// the per-cent sign (96 %), where `Intl` would put a no-break space.
const NARROW_NO_BREAK_SPACE = "\u202F";
const esCount = (n: number): string => {
  const digits = String(n);
  if (digits.length <= 4) return digits;
  return digits.replace(/\B(?=(\d{3})+(?!\d))/g, NARROW_NO_BREAK_SPACE);
};
const esPercent = (share: number): string => `${Math.round(share)}${NARROW_NO_BREAK_SPACE}%`;
// Spanish: « y » reads « e » before an i- sound (not before « hie », a diphthong: « agua y
// hielo »), « o » reads « u » before an o- sound.
export const esAnd = (items: string[]): string => {
  if (items.length <= 1) return items[0] ?? "";
  const last = items[items.length - 1];
  return joinList(items, /^h?i(?!e)/i.test(last) ? "e" : "y");
};
export const esOr = (items: string[]): string => {
  if (items.length <= 1) return items[0] ?? "";
  const last = items[items.length - 1];
  return joinList(items, /^h?o/i.test(last) ? "u" : "o");
};

const GRAMMAR: Record<Lang, Grammar> = {
  fr: {
    of: (name) => (frElides(name) ? `d'${name}` : `de ${name}`),
    learn: (name) => (frElides(name) ? `l'${name}` : `le ${name}`),
    the: (name) => (frElides(name) ? `l'${name}` : `le ${name}`),
    to: (studied, native) => `${GRAMMAR.fr.the(studied)} vers ${GRAMMAR.fr.the(native)}`,
    and: (items) => joinList(items, "et"),
    or: (items) => joinList(items, "ou"),
    count: (n) => new Intl.NumberFormat("fr-FR").format(n),
    percent: (share) => new Intl.NumberFormat("fr-FR", { style: "percent", maximumFractionDigits: 0 }).format(share / 100),
  },
  en: {
    of: (name) => name,
    learn: (name) => name,
    the: (name) => name,
    to: (studied, native) => `${studied} to ${native}`,
    and: (items) => joinList(items, "and"),
    or: (items) => joinList(items, "or"),
    count: (n) => new Intl.NumberFormat("en-US").format(n),
    percent: (share) => new Intl.NumberFormat("en-US", { style: "percent", maximumFractionDigits: 0 }).format(share / 100),
  },
  es: {
    of: (name) => `en ${name}`,
    learn: (name) => name,
    the: (name) => `el ${name}`,
    to: (studied, native) => `del ${studied} al ${native}`,
    and: esAnd,
    or: esOr,
    count: esCount,
    percent: esPercent,
  },
};

/**
 * `template` with each `{name}` replaced by its value, the value's `&`, `<` and `>` escaped:
 * the result goes through `set:html`, so a value is text, never markup. `'` is left as it is,
 * as today's page writes it (an `{expression}` would turn it into `&#39;`).
 */
export function fill(template: string, values: Record<string, string>): string {
  return template.replace(/\{(\w+)\}/g, (whole, key: string) => {
    if (!Object.hasOwn(values, key)) throw new Error(`lingua-text.ts: no value for ${whole}`);
    return values[key].replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  });
}

function capitalize(text: string, lang: Lang): string {
  return text.charAt(0).toLocaleUpperCase(lang) + text.slice(1);
}

function escapeAttribute(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/"/g, "&quot;");
}

/** The text of the whole page in `lang`, every slot ready for `set:html`. */
export interface LinguaPageText {
  title: string;
  description: string;
  kicker: string;
  headline: string;
  tagline: string;
  heroNote: string;
  featuresId: string;
  cards: { key: CardKey; title: string; body: string }[];
  coverage: { heading: string; caption: string; head: string[]; rows: string[][]; note: string };
  closing: { heading: string; body: string };
}

export interface LinguaPageOptions {
  /** The community invite; `null` renders no link, like every other Discord link. */
  discordUrl: string | null;
}

/** The page in `lang`, from the shipped pairs and the tops their figures are measured on. */
export function linguaPageText(lang: Lang, shipped: ShippedPairs, options: LinguaPageOptions): LinguaPageText {
  const { tops, pairs } = shipped;
  const t = LINGUA_TEXT[lang];
  const g = GRAMMAR[lang];
  // `Object.hasOwn`: a code such as « constructor » must not name a language through the
  // object's prototype.
  const name = (code: string): string => {
    if (!Object.hasOwn(t.names, code)) throw new Error(`lingua-text.ts: the ${lang} table has no name for the language "${code}"`);
    return t.names[code];
  };
  const speakers = (code: string): string => {
    if (!Object.hasOwn(t.speakers, code)) {
      throw new Error(`lingua-text.ts: the ${lang} table has no speakers for the language "${code}"`);
    }
    return t.speakers[code];
  };
  const studiedOf = (group: NativeGroup): string[] => group.pairs.map((p) => name(p.studied));

  const groups = pairsByNative(lang, pairs);
  if (groups.length === 0) throw new Error("lingua-text.ts: no shipped pair to describe");
  const lead = groups[0];
  const readers = lead.native === lang ? lead : null;
  const others = readers ? groups.slice(1) : groups;
  const oneNative = groups.length === 1;
  const studiesSpanish = pairs.some((p) => p.studied === "es");

  // The hero note: the platforms, one audience sentence per group of other readers — and
  // the readers' own group, named only beside others (alone, the whole page is theirs, as
  // today's French page) — then the community invite. With no readers' pair, the first
  // other group is the one the product is "made for" (today's English page).
  const sentences = [t.platforms];
  if (readers && others.length) {
    sentences.push(fill(t.forReaders, { speakers: speakers(readers.native), learnStudied: g.or(studiedOf(readers).map(g.learn)) }));
  }
  others.forEach((group, i) => {
    const template = i === 0 && !readers ? t.madeFor : t.alsoFor;
    if (template === null) {
      throw new Error(`lingua-text.ts: the ${lang} page is built only with a pair glossed in its language (D3)`);
    }
    sentences.push(
      fill(template, {
        speakers: speakers(group.native),
        learnStudied: g.or(studiedOf(group).map(g.learn)),
        native: name(group.native),
      }),
    );
  });
  let heroNote = sentences.join(" ");
  if (options.discordUrl) {
    heroNote += ` <a href="${escapeAttribute(options.discordUrl)}" rel="noopener">${t.joinCommunity}</a>${t.toTalk}`;
  }

  const ofStudied = g.or(studiedOf(lead).map(g.of));
  const appended: Partial<Record<CardKey, string>> = studiesSpanish ? { click: t.spanishCard, level: t.spanishLevels } : {};
  const cards = t.cards.map((card) => ({ ...card, body: card.body + (appended[card.key] ?? "") }));
  cards.push({
    key: "languages",
    title: `🧭 ${groups.map((group) => `${capitalize(g.and(studiedOf(group)), lang)} → ${name(group.native)}`).join(" · ")}`,
    body: t.languagesCard,
  });

  // The coverage table: one column per pair, the readers' first; headers name the studied
  // language alone while every pair is glossed in one language, both once in two (D4).
  const ordered = groups.flatMap((group) => group.pairs);
  const head = [
    t.commonestWords,
    ...ordered.map((p) => (oneNative ? capitalize(name(p.studied), lang) : `${capitalize(name(p.studied), lang)} → ${name(p.native)}`)),
  ];
  const rows = tops.map((top, i) => [g.count(top), ...ordered.map((p) => g.percent(p.glossed[i]))]);
  const named = (p: LinguaPair): string => (oneNative ? g.the(name(p.studied)) : g.to(name(p.studied), name(p.native)));
  const served = ordered
    .filter((p) => p.translation !== "none")
    .map((p) => `${named(p)}${p.translation === "through-english" ? t.throughEnglish : ""}`);
  const unserved = ordered.filter((p) => p.translation === "none").map(named);
  let note = t.glossesOrigin;
  if (served.length) note += fill(t.translationServes, { served: g.and(served) });
  if (unserved.length) note += fill(t.translationNotYet, { unserved: g.and(unserved) });

  return {
    title: t.title,
    description: fill(t.description, { ofStudied }),
    kicker: t.kicker,
    headline: t.headline,
    tagline: fill(t.tagline, { ofStudied }),
    heroNote,
    featuresId: t.featuresId,
    cards,
    coverage: {
      heading: t.coverageHeading,
      caption: oneNative ? fill(t.captionOneNative, { native: name(lead.native) }) : t.captionPerPair,
      head,
      rows,
      note,
    },
    closing: { heading: t.closingHeading, body: fill(t.closingBody, { music: MUSIC_HREF[lang] }) },
  };
}

// The Lingua card of the Spanish home page (change: extend-site-spanish-locale, D3). The
// French and English homes' cards are literal markup and say « en anglais » / "English": true
// for their readers. Translated as it stands, the Spanish card would tell Spanish speakers
// that Lingua explains English words in Spanish before any pair does, so its words come from
// the shipped pairs, as the Lingua page's do, with the Spanish table's names, speakers and
// grammar: the languages read with a Spanish gloss once one ships; until then every language
// read, and one sentence for whom it is made.
const HOME_ES = {
  read: "Lee la web {ofStudied} con las palabras que aún no conoces resaltadas en la propia página. Un porcentaje honesto por página, un clic para la traducción y tu vocabulario, que se construye solo.",
  audience: " Pensada para {speakers}, con la interfaz y las traducciones en su idioma.",
};

/** The Lingua card of the Spanish home page: its text, ready for `set:html`, and its button's address. */
export interface HomeLinguaCard {
  body: string;
  href: string;
}

/**
 * The Spanish home page's Lingua card, from the shipped pairs: a pair glossed in Spanish
 * ships → the languages read with a Spanish gloss, no audience, the button opens
 * `/es/lingua`; none ships → every language read, the speakers of every native language as
 * its readers, the button opens `/en/lingua` (`linguaHref('es')`, as the Spanish nav).
 */
export function spanishHomeLinguaCard(pairs: LinguaPair[]): HomeLinguaCard {
  const t = LINGUA_TEXT.es;
  const g = GRAMMAR.es;
  const name = (code: string): string => {
    if (!Object.hasOwn(t.names, code)) throw new Error(`lingua-text.ts: the es table has no name for the language "${code}"`);
    return t.names[code];
  };
  const speakers = (code: string): string => {
    if (!Object.hasOwn(t.speakers, code)) {
      throw new Error(`lingua-text.ts: the es table has no speakers for the language "${code}"`);
    }
    return t.speakers[code];
  };
  const groups = pairsByNative("es", pairs);
  if (groups.length === 0) throw new Error("lingua-text.ts: no shipped pair to describe");
  const readers = groups[0].native === "es" ? groups[0] : null;
  // The languages read, once each, in `packs.json`'s order: the readers' alone once they
  // exist, every pair's until then.
  const studied: string[] = [];
  for (const p of readers ? readers.pairs : pairs) if (!studied.includes(p.studied)) studied.push(p.studied);
  let body = fill(HOME_ES.read, { ofStudied: g.or(studied.map((code) => g.of(name(code)))) });
  if (!readers) body += fill(HOME_ES.audience, { speakers: g.and(groups.map((group) => speakers(group.native))) });
  return { body, href: linguaHref("es", pairs) };
}
