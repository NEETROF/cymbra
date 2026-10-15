// The words of the Lingua page, one table per site language, and the builders that fill
// them from the shipped pairs (change: add-site-lingua-matrix-pages, D1, D2, D4). Read by
// `src/components/LinguaPage.astro` — and, for the Lingua card of the Spanish and English home
// pages, by `src/pages/es/index.astro` and `src/pages/en/index.astro` (`spanishHomeLinguaCard`,
// change: extend-site-spanish-locale, D3; `englishHomeLinguaCard`, change:
// add-lingua-french-listings, D9).
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
//   {inStudied}     the studied languages whose card names tense and gender, after « En » / "In"
//                   (fr « espagnol et en français », en "Spanish and French", es « español y en francés »)
//   {theStudied}    the studied languages whose levels are estimated, after « Pour » / "For"
//                   (fr « l'espagnol et le français », en "Spanish and French", es « el español y el francés »)
//   {music}         the Music page's address in this language
// The English table quotes the English card's labels (`apps/lingua-extension/src/i18n/en/card.ts`,
// curly quotes where the French has « », as `apps/lingua-extension/src/i18n/README.md` writes
// English) and says "its translation", which is in the reader's language: change 34
// (enable-lingua-english-speakers, task 1.2) replaced the French buttons it quoted while every
// shipped pair was glossed in French.
// The Spanish table follows the extension's conventions (`apps/lingua-extension/src/i18n/README.md`:
// tú, neutral, no vosotros; RAE numbers, « 20 000 » and « 96 % » with a narrow no-break space) and
// quotes its Spanish labels (`apps/lingua-extension/src/i18n/es/card.ts`). `test/lingua-text.spec.ts`
// holds each table's three buttons to its catalogue.

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
  /**
   * Appended to the `click` card when a listed pair studies a language whose card names the
   * tense and the gender, and to the `level` card when one studies a language whose levels are
   * estimated (`STUDIED`, change: add-lingua-french-listings, D7).
   */
  genderCard: string;
  estimatedLevels: string;
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
  /** The captures of the extension (change: add-site-lingua-showcase, D10). */
  showcase: ShowcaseText;
  /** The line the store buttons add for a browser Lingua does not run in there (D11). */
  hints: Record<StoreHintKey, string>;
}

/** The captures the « En images » section shows after the video, in its order (D5, D10). */
export const SHOT_KEYS = ["second", "review", "stats", "reader", "reader-eink"] as const;
export type ShotKey = (typeof SHOT_KEYS)[number];

/** The store buttons' hints (`src/lib/store-choice.ts`, `StoreHint`). */
export type StoreHintKey = "ios" | "android";

/**
 * The section's words: its heading, the hero capture's alt text, the video's label and caption, and
 * each capture's alt text and caption — what each capture shows, in the page's language, quoting no
 * figure the seed could change. The captures are `yarn capture`'s (apps/lingua-extension/tool/capture/):
 * each table describes its own interface language's set — the languages its reader studies are
 * `plan.mjs`'s `STUDIED`, which `test/lingua-text.spec.ts` holds the texts to.
 */
export interface ShowcaseText {
  heading: string;
  hero: { alt: string };
  video: { label: string; caption: string };
  shots: Record<ShotKey, { alt: string; caption: string }>;
  /** The share image's alt text (`og:image:alt`), the hero capture cropped. */
  shareAlt: string;
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
    "Disponible sur Chrome, Edge et les navigateurs Chromium, sur Firefox (ordinateur et Android) et sur Safari (iPhone, iPad, Mac).",
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
  genderCard: " En {inStudied}, la carte nomme aussi le temps et le genre.",
  // « CEFR », as the extension's French interface names the scale (`levelScale`, M19) and as this
  // page's level card does: one page, one name for the scale (D7).
  estimatedLevels: " Pour {theStudied}, les niveaux sont estimés d'après la fréquence des mots, faute de liste CEFR libre de droits.",
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
  // The French reader studies English, then Spanish; the book is Pride and Prejudice.
  showcase: {
    heading: "En images",
    hero: {
      alt: "Une page en anglais lue avec Cymbra Lingua : les mots inconnus sont surlignés, la pastille donne la part de mots connus, et la carte d'un verbe est ouverte avec sa forme du dictionnaire, son temps et sa traduction.",
    },
    video: {
      label: "Démonstration de Cymbra Lingua, sans son",
      caption:
        "Une page lue, la carte d'un mot, un mot ajouté au deck, un autre marqué « Je connais », puis les statistiques.",
    },
    shots: {
      second: {
        alt: "Une page en espagnol, l'interface en français : la carte d'un nom donne son genre et sa traduction.",
        caption: "Une deuxième langue étudiée, sur sa propre page.",
      },
      review: {
        alt: "Le panneau de révision ouvert sur la page : une carte révélée, avec la phrase où le mot a été rencontré, sa traduction et les deux réponses « Pas su » et « Su ».",
        caption: "La révision, dans un panneau ouvert sur la page.",
      },
      stats: {
        alt: "La page des statistiques : le vocabulaire estimé et l'échelle des niveaux CEFR, de A1 à C2, pour l'anglais.",
        caption: "Votre niveau, sur l'échelle CEFR.",
      },
      reader: {
        alt: "Le lecteur de livres EPUB de l'extension, ouvert sur le premier chapitre d'Orgueil et Préjugés en anglais, les mots inconnus surlignés.",
        caption: "Vos livres EPUB, surlignés de la même façon.",
      },
      "reader-eink": {
        alt: "La même page du lecteur avec le préréglage « E-ink contrasté » : noir sur blanc, les mots soulignés d'un trait épais.",
        caption: "Un préréglage pour les tablettes à encre électronique.",
      },
    },
    shareAlt:
      "Cymbra Lingua sur une page en anglais : les mots inconnus surlignés et la carte d'un verbe ouverte.",
  },
  hints: {
    ios: "Sur iPhone et iPad, Lingua s'utilise dans Safari.",
    android: "Sur Android, Lingua s'utilise dans Firefox.",
  },
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
    "Available on Chrome, Edge and other Chromium browsers, on Firefox (desktop and Android) and on Safari (iPhone, iPad, Mac).",
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
      body: "Click a highlighted word: its dictionary form, its translation, its frequency in plain language, and three buttons — “I know it”, “+ Deck”, “Ignore”.",
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
  genderCard: " In {inStudied}, the card also names the tense and the gender.",
  estimatedLevels: " For {theStudied}, the levels are estimated from word frequency, as no CEFR list can be shipped freely.",
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
  // The English reader studies Spanish, then French; the book is Marianela.
  showcase: {
    heading: "See it at work",
    hero: {
      alt: "A Spanish page read with Cymbra Lingua: unknown words are highlighted, the pill shows the share of words known, and a verb's card is open with its dictionary form, its tense and its meaning.",
    },
    video: {
      label: "Cymbra Lingua demo, no sound",
      caption: "A page read, a word's card, a word added to the deck, another marked “I know it”, then the statistics.",
    },
    shots: {
      second: {
        alt: "A French page, the interface in English: a noun's card gives its gender and its meaning.",
        caption: "A second language studied, on its own page.",
      },
      review: {
        alt: "The review panel open on the page: a revealed card with the sentence the word was met in, its meaning and the two answers “Didn't know” and “Knew it”.",
        caption: "Review, in a panel opened on the page.",
      },
      stats: {
        alt: "The statistics page: the estimated vocabulary and the CEFR ladder, A1 to C2, for Spanish.",
        caption: "Your level, on the CEFR scale.",
      },
      reader: {
        alt: "The extension's EPUB reader open on the first chapter of Marianela, in Spanish, unknown words highlighted.",
        caption: "Your EPUB books, highlighted the same way.",
      },
      "reader-eink": {
        alt: "The same reader page in the “High-contrast e-ink” preset: black on white, words underlined with a thick line.",
        caption: "A preset for e-ink tablets.",
      },
    },
    shareAlt: "Cymbra Lingua on a Spanish page: unknown words highlighted and a verb's card open.",
  },
  hints: {
    ios: "On iPhone and iPad, Lingua runs in Safari.",
    android: "On Android, Lingua runs in Firefox.",
  },
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
    "Disponible en Chrome, Edge y otros navegadores Chromium, en Firefox (escritorio y Android) y en Safari (iPhone, iPad, Mac).",
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
  genderCard: " En {inStudied}, la tarjeta también indica el tiempo verbal y el género.",
  estimatedLevels:
    " Para {theStudied}, los niveles se estiman a partir de la frecuencia de las palabras, a falta de una lista MCER de uso libre.",
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
  // The Spanish reader studies English, then French; the book is Pride and Prejudice.
  showcase: {
    heading: "Míralo en acción",
    hero: {
      alt: "Una página en inglés leída con Cymbra Lingua: las palabras desconocidas aparecen resaltadas, la pastilla indica la proporción de palabras conocidas y está abierta la tarjeta de un verbo, con su forma de diccionario, su tiempo verbal y su traducción.",
    },
    video: {
      label: "Demostración de Cymbra Lingua, sin sonido",
      caption:
        "Una página leída, la tarjeta de una palabra, una palabra añadida al mazo, otra marcada con «La conozco» y, después, las estadísticas.",
    },
    shots: {
      second: {
        alt: "Una página en francés, con la interfaz en español: la tarjeta de un sustantivo indica su género y su traducción.",
        caption: "Un segundo idioma estudiado, en su propia página.",
      },
      review: {
        alt: "El panel de repaso abierto sobre la página: una tarjeta revelada, con la frase en la que apareció la palabra, su traducción y las dos respuestas «No la sabía» y «La sabía».",
        caption: "El repaso, en un panel abierto sobre la página.",
      },
      stats: {
        alt: "La página de estadísticas: el vocabulario estimado y la escala de niveles MCER, de A1 a C2, para el inglés.",
        caption: "Tu nivel, en la escala MCER.",
      },
      reader: {
        alt: "El lector de libros EPUB de la extensión, abierto en el primer capítulo de Orgullo y prejuicio en inglés, con las palabras desconocidas resaltadas.",
        caption: "Tus libros EPUB, resaltados de la misma manera.",
      },
      "reader-eink": {
        alt: "La misma página del lector con el ajuste «Tinta electrónica contrastada»: negro sobre blanco, palabras subrayadas con un trazo grueso.",
        caption: "Un ajuste para tabletas de tinta electrónica.",
      },
    },
    shareAlt: "Cymbra Lingua en una página en inglés: las palabras desconocidas resaltadas y la tarjeta de un verbo abierta.",
  },
  hints: {
    ios: "En iPhone y iPad, Lingua funciona en Safari.",
    android: "En Android, Lingua funciona en Firefox.",
  },
};

export const LINGUA_TEXT: Record<Lang, LinguaTable> = { fr, en, es };

/** What a studied language's packs are, as the page tells its readers. */
export interface StudiedLanguage {
  /** The word card names the tense and the gender (« passé simple », « nom féminin »). */
  namesTenseAndGender: boolean;
  /** The levels are estimated from word frequency, for want of a freely licensed CEFR list. */
  levelsEstimated: boolean;
}

/**
 * What each studied language's packs are (change: add-lingua-french-listings, D7), whatever the
 * language they are glossed in: English's card names no gender and its levels are CEFR-J's and
 * Octanove's; Spanish's and French's cards name the tense and the gender (the Spanish programme;
 * change 45) and their levels are estimated from word frequency (`levels_estimated`; change 46).
 * A shipped pair studying a language not described here fails the build, as one the tables
 * cannot name does.
 */
export const STUDIED: Readonly<Record<string, StudiedLanguage>> = {
  en: { namesTenseAndGender: false, levelsEstimated: false },
  es: { namesTenseAndGender: true, levelsEstimated: true },
  fr: { namesTenseAndGender: true, levelsEstimated: true },
};

/** `code`'s entry in `STUDIED`, or the build stops, naming the language. */
export function studiedLanguage(code: string): StudiedLanguage {
  // `Object.hasOwn`, as for the names: « constructor » describes nothing.
  if (!Object.hasOwn(STUDIED, code)) {
    throw new Error(`lingua-text.ts: STUDIED does not describe the studied language "${code}" (its card and its levels)`);
  }
  return STUDIED[code];
}

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
  /** A list after « En » / "In": « espagnol et en français » / "Spanish and French" / « español y en francés ». */
  inList: (items: string[]) => string;
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
    inList: (items) => GRAMMAR.fr.and(items.map((item, i) => (i === 0 ? item : `en ${item}`))),
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
    inList: (items) => GRAMMAR.en.and(items),
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
    inList: (items) => esAnd(items.map((item, i) => (i === 0 ? item : `en ${item}`))),
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
  /**
   * The coverage table. `perPair` false: one column per pair, `head` the pairs, each row a top
   * (today's markup). `perPair` true, once pairs of two native languages are listed (D8): one row
   * per pair, its first cell the pair (a row header), `head` the tops — rendered in a box that
   * scrolls on its own.
   */
  coverage: { heading: string; caption: string; perPair: boolean; head: string[]; rows: string[][]; note: string };
  closing: { heading: string; body: string };
  /** The captures' words (change: add-site-lingua-showcase, D10): attributes and captions. */
  showcase: ShowcaseText;
  /** The store buttons' hints (D11), shown by the page's script alone. */
  hints: Record<StoreHintKey, string>;
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
  // The studied languages of the listed pairs, once each, in the page's order (its readers'
  // first), and what their packs are (D7): one sentence for the cards that name the tense and
  // the gender, one for the levels that are estimated — never English.
  const studied: string[] = [];
  for (const group of groups) for (const p of group.pairs) if (!studied.includes(p.studied)) studied.push(p.studied);
  const gendered = studied.filter((code) => studiedLanguage(code).namesTenseAndGender).map(name);
  const estimated = studied.filter((code) => studiedLanguage(code).levelsEstimated).map(name);
  const appended: Partial<Record<CardKey, string>> = {};
  if (gendered.length) appended.click = fill(t.genderCard, { inStudied: g.inList(gendered) });
  if (estimated.length) appended.level = fill(t.estimatedLevels, { theStudied: g.and(estimated.map(g.the)) });
  const cards = t.cards.map((card) => ({ ...card, body: card.body + (appended[card.key] ?? "") }));
  cards.push({
    key: "languages",
    title: `🧭 ${groups.map((group) => `${capitalize(g.and(studiedOf(group)), lang)} → ${name(group.native)}`).join(" · ")}`,
    body: t.languagesCard,
  });

  // The coverage table, the readers' pairs first. While every pair is glossed in one language:
  // one column per pair, named by its studied language alone (D4). Once in two, a pair is named
  // by both and has a row of its own, one column per top — six columns of pairs would scroll a
  // phone's page sideways (change: add-lingua-french-listings, D8).
  const ordered = groups.flatMap((group) => group.pairs);
  const pairName = (p: LinguaPair): string =>
    oneNative ? capitalize(name(p.studied), lang) : `${capitalize(name(p.studied), lang)} → ${name(p.native)}`;
  const head = oneNative ? [t.commonestWords, ...ordered.map(pairName)] : [t.commonestWords, ...tops.map(g.count)];
  const rows = oneNative
    ? tops.map((top, i) => [g.count(top), ...ordered.map((p) => g.percent(p.glossed[i]))])
    : ordered.map((p) => [pairName(p), ...p.glossed.map(g.percent)]);
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
      perPair: !oneNative,
      head,
      rows,
      note,
    },
    closing: { heading: t.closingHeading, body: fill(t.closingBody, { music: MUSIC_HREF[lang] }) },
    showcase: t.showcase,
    hints: t.hints,
  };
}

// The Lingua card of the French home page (change: add-site-lingua-showcase, D15). It said « Lisez le
// web en anglais » while French speakers study English and Spanish: its languages now come from the
// shipped pairs glossed in French, in `packs.json`'s order, as the English and Spanish homes' do.
const HOME_FR =
  "Lisez le web {inStudied}, les mots que vous ne connaissez pas encore sont surlignés sur place. Un pourcentage honnête par page, un clic pour la traduction, et votre vocabulaire qui se construit tout seul.";

/**
 * The French home page's Lingua card, from the shipped pairs: the languages read with a French gloss
 * (« en anglais ou en espagnol »); the button opens `/lingua`, which always exists.
 */
export function frenchHomeLinguaCard(pairs: LinguaPair[]): HomeLinguaCard {
  const t = LINGUA_TEXT.fr;
  const studied: string[] = [];
  for (const p of pairs) if (p.native === "fr" && !studied.includes(p.studied)) studied.push(p.studied);
  if (studied.length === 0) throw new Error("lingua-text.ts: no shipped pair is glossed in French (M22)");
  const name = (code: string): string => {
    if (!Object.hasOwn(t.names, code)) throw new Error(`lingua-text.ts: the fr table has no name for the language "${code}"`);
    return t.names[code];
  };
  const body = fill(HOME_FR, { inStudied: GRAMMAR.fr.or(studied.map((code) => `en ${name(code)}`)) });
  return { body, href: linguaHref("fr", pairs) };
}

// The Lingua card of the Spanish home page (change: extend-site-spanish-locale, D3). The
// French home's card was literal markup and said « en anglais » (it is built since
// add-site-lingua-showcase, `frenchHomeLinguaCard`, above).
// Translated as it stands, the Spanish card would tell Spanish speakers that Lingua explains
// English words in Spanish before any pair does, so its words come from the shipped pairs, as
// the Lingua page's do, with the Spanish table's names, speakers and grammar: the languages
// read with a Spanish gloss once one ships; until then every language read, and one sentence
// for whom it is made. The English home's card follows the same rule from the first pair
// glossed in English (`englishHomeLinguaCard`, below).
const HOME_ES = {
  read: "Lee la web {ofStudied} con las palabras que aún no conoces resaltadas en la propia página. Un porcentaje honesto por página, un clic para la traducción y tu vocabulario, que se construye solo.",
  audience: " Pensada para {speakers}, con la interfaz y las traducciones en su idioma.",
};

/** The Lingua card of a home page: its text, ready for `set:html`, and its button's address. */
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

// The Lingua card of the English home page (change: add-lingua-french-listings, D9). Written
// for French speakers, « Read the English web… » is wrong for the English speakers who read
// `/en/` once a pair glossed in English ships (es-en, change 34), and more so once they read
// French too (fr-en, change 52). From that pair on, the card names the languages read with an
// English gloss, as the Spanish home's does with a Spanish one; until then it is today's
// paragraph, byte for byte as the build renders it.
const HOME_EN = {
  read: "Read the web in {studied} with the words you do not know yet highlighted in place. An honest per-page percentage, one click for the meaning, and a vocabulary that builds itself as you read.",
  before:
    "Read the English web with the words you do not know yet highlighted in place. An honest per-page percentage, one click for the meaning, and a vocabulary that builds itself as you read.",
};

/**
 * The English home page's Lingua card, from the shipped pairs: a pair glossed in English ships
 * → the languages read with an English gloss, in `packs.json`'s order (« in Spanish », then « in
 * Spanish or French »); none ships → today's words. The button opens `/en/lingua`, which always
 * exists (`linguaHref('en')`).
 */
export function englishHomeLinguaCard(pairs: LinguaPair[]): HomeLinguaCard {
  const t = LINGUA_TEXT.en;
  const name = (code: string): string => {
    if (!Object.hasOwn(t.names, code)) throw new Error(`lingua-text.ts: the en table has no name for the language "${code}"`);
    return t.names[code];
  };
  const studied: string[] = [];
  for (const p of pairs) if (p.native === "en" && !studied.includes(p.studied)) studied.push(p.studied);
  const body = studied.length ? fill(HOME_EN.read, { studied: GRAMMAR.en.or(studied.map(name)) }) : HOME_EN.before;
  return { body, href: linguaHref("en", pairs) };
}
