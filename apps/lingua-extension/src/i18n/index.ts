import type { GrammarTag, WordGrammar } from "../analyzer/types.ts";
import {
  type InterfaceLanguage,
  type InterfaceLanguageArea,
  interfaceLanguage,
  setDocumentLanguage,
} from "./language.ts";

// The catalogue's helpers (add-lingua-interface-language D1, D2, D4): how a count picks its
// plural form, how a number, a percentage, a date and a region are written in each language, how a
// page is filled from a module, and how a slot message is rendered around the parts a surface draws
// apart. No surface's copy is mapped here — a surface imports its own `./{fr,en,es}/<surface>.ts`
// and picks by the interface language — so importing this module costs an entry nothing but these
// functions.

export type { InterfaceLanguage, InterfaceLanguageArea } from "./language.ts";
export {
  DEFAULT_INTERFACE_LANGUAGE,
  INTERFACE_LANGUAGE_KEY,
  INTERFACE_LANGUAGES,
  interfaceLanguage,
  isInterfaceLanguage,
  setDocumentLanguage,
} from "./language.ts";

/** The surfaces whose copy the catalogue holds, one module each per language. */
export type Surface =
  | "popup"
  | "hud"
  | "drawer"
  | "card"
  | "selection"
  | "sidepanel"
  | "review"
  | "stats"
  | "settings"
  | "studied-languages"
  | "native-language"
  | "colours"
  | "display"
  | "translation"
  | "account-setting"
  | "account"
  | "account-errors"
  | "onboarding"
  | "reader"
  | "sync"
  | "languages"
  | "grammar";

export const SURFACES: readonly Surface[] = [
  "popup",
  "hud",
  "drawer",
  "card",
  "selection",
  "sidepanel",
  "review",
  "stats",
  "settings",
  "studied-languages",
  "native-language",
  "colours",
  "display",
  "translation",
  "account-setting",
  "account",
  "account-errors",
  "onboarding",
  "reader",
  "sync",
  "languages",
  "grammar",
];

/**
 * The forms of a counted message, by `Intl.PluralRules` category, each a function of the number
 * as the language writes it. French and Spanish have `many` (a round million, in current CLDR),
 * English has not: a missing `many` falls back to `other`. In French every form carries the same
 * string, the current one — « 3 carte(s) à revoir » — so a French module writes it twice.
 */
export interface PluralForms {
  one: (n: string) => string;
  many?: (n: string) => string;
  other: (n: string) => string;
}

/**
 * Declares a counted message in a French module with the forms' own type — not the literal's, so
 * that a translation may add the `many` the French left out, and leave out nothing else.
 */
export function pluralForms(forms: PluralForms): PluralForms {
  return forms;
}

/**
 * The studied languages whose forms a card's grammar renderer names: French since
 * add-lingua-french-word-card (D1), ahead of the extension's `StudiedLanguage`, which change 52
 * widens — until then no content a page builds says `fr`, and the renderers' French is reached by
 * tests only.
 */
export type StudiedLanguageCode = "en" | "es" | "fr";

/** A name, with the article its language gives it (« le prétérit », « la forma en -ing »). */
export interface Named {
  article: string;
  name: string;
}

/** One segment of a grammar line: plain text, or a word of the studied language (set apart). */
export type LineSegment = string | { word: string };

/** One line of a card's grammar block. */
export type GrammarLine = LineSegment[];

/**
 * The `grammar` surface (generalise-lingua-card-wording D2): not texts but a renderer — the one
 * module of the catalogue where the rule that a draft is typed after the French covers code. Each
 * language's module turns the description of a form (`reading/grammar-description.ts`) into lines
 * in its own words, articles, elisions and joining; its tables stay private to it, keyed by studied
 * language where a name depends on it (a tense, the gerund), the order of its tenses being its tense
 * table's. What is named at all, and a line's plain text (`lineText`), are the description's.
 */
export interface GrammarRenderer {
  /** The lines a card shows about the form it was opened on, or none. */
  grammarLines(
    grammar: WordGrammar,
    headword: string,
    surface: string,
    written: string,
    studied?: StudiedLanguageCode,
  ): GrammarLine[];
  /** What a reading makes of a form, in words; null when the card says nothing of it. */
  readingName(tag: GrammarTag, studied?: StudiedLanguageCode): Named | null;
  /** The heading of a group of senses: its part of speech, with the word's gender; null for none. */
  senseHeading(tag: GrammarTag | undefined): string | null;
  /** A list as the language joins it: "a, b and c". */
  join(items: readonly string[]): string;
}

const LOCALES: Record<InterfaceLanguage, string> = { fr: "fr-FR", en: "en-US", es: "es-ES" };

const rules = new Map<InterfaceLanguage, Intl.PluralRules>();

function rulesOf(language: InterfaceLanguage): Intl.PluralRules {
  let r = rules.get(language);
  if (!r) {
    r = new Intl.PluralRules(LOCALES[language]);
    rules.set(language, r);
  }
  return r;
}

/** A narrow no-break space, the RAE's separator between groups of three digits. */
const NARROW_NO_BREAK_SPACE = "\u202F";

/**
 * A number as the interface language writes it. French as the surfaces write it today, through
 * `fr-FR` (« 20 000 », a narrow no-break space between the groups); English through `en-US`
 * ("20,000"); Spanish as the RAE writes it, which `es-ES` does not: no grouping below ten
 * thousand (« 5000 »), then groups of three parted by a narrow no-break space (« 20 000 »),
 * a comma before the decimals (« 25,8 »).
 */
export function formatNumber(language: InterfaceLanguage, n: number, options?: Intl.NumberFormatOptions): string {
  if (language !== "es") return n.toLocaleString(LOCALES[language], options);
  const plain = n.toLocaleString(LOCALES.es, { ...options, useGrouping: false });
  return plain.replace(
    /^(-?)(\d{5,})/,
    (_, sign: string, digits: string) => sign + digits.replace(/\B(?=(\d{3})+$)/g, NARROW_NO_BREAK_SPACE),
  );
}

/**
 * A count as the interface language writes it: in French the raw figure the surfaces write today
 * (« 1234 », never a grouped « 1 234 »), in English and Spanish through `formatNumber` ("1,234",
 * « 1234 », « 12 345 »). A count is not a measure: French has always written it bare.
 */
export function formatCount(language: InterfaceLanguage, n: number): string {
  return language === "fr" ? String(n) : formatNumber(language, n);
}

/**
 * How a French surface writes a percentage today: the popup and the HUD « 45% », the reader's
 * text size « 45 % » with a plain space. The form only decides the French (M23 keeps each
 * surface's); Spanish always parts the sign with a narrow no-break space, English never.
 */
export type PercentForm = "tight" | "spaced";

export function formatPercent(language: InterfaceLanguage, n: number, form: PercentForm): string {
  const number = formatNumber(language, n);
  if (language === "es") return `${number}${NARROW_NO_BREAK_SPACE}%`;
  if (language === "en") return `${number}%`;
  return form === "spaced" ? `${number} %` : `${number}%`;
}

/** A date as the interface language writes it: `fr-FR` as the surfaces do today, `en-US`, `es-ES`. */
export function formatDate(language: InterfaceLanguage, date: Date, options?: Intl.DateTimeFormatOptions): string {
  return date.toLocaleDateString(LOCALES[language], options);
}

const regions = new Map<InterfaceLanguage, Intl.DisplayNames>();

/**
 * A region as the interface language names it — the place of a voice in Réglages, « États-Unis »
 * (localise-lingua-settings D4): the language's own `Intl.DisplayNames`, French through `["fr"]` as
 * Réglages named it before the catalogue; the code itself where the runtime cannot name it.
 */
export function regionName(language: InterfaceLanguage, code: string): string {
  try {
    let names = regions.get(language);
    if (!names) {
      names = new Intl.DisplayNames([language], { type: "region" });
      regions.set(language, names);
    }
    return names.of(code) ?? code;
  } catch {
    // Not a region this runtime can name (or no Intl.DisplayNames): the code says enough.
    return code;
  }
}

/**
 * The character around a part's number in a sentinel, `\u0000<i>\u0000`: one no message writes.
 * `slot(i)` writes a sentinel and `fillSlots` reads it — the one slot mechanism of the catalogue:
 * `NODE_SLOT` is `slot(0)`, so any message renders through `fillSlots` or `renderAround` alike.
 */
const SLOT_MARK = "\u0000";

/** A sentinel in a message: the mark, the part's number, the mark (built from `SLOT_MARK`). */
const SLOT_PATTERN = new RegExp(`${SLOT_MARK}(\\d+)${SLOT_MARK}`, "g");

/**
 * The sentinel a surface passes for the `index`-th part of a slot message it renders apart — a bold
 * count, a painted word, a key (README › Shape): numbered, so a translation may put the parts in
 * any order. `fillSlots` renders the message around them.
 */
export function slot(index: number): string {
  return `${SLOT_MARK}${index}${SLOT_MARK}`;
}

/** A part a slot message is rendered with: one node or text, or several in a row. */
export type SlotPart = Node | string;

const several = (part: SlotPart | readonly SlotPart[]): part is readonly SlotPart[] => Array.isArray(part);

/**
 * A slot message written with `slot(i)` for its parts, as the nodes to render: the texts between,
 * in order, and each part where the language put it — a part named twice shows twice, a node the
 * second time as a copy (a node is in one place). A sentinel with no part is left out; a part the
 * message never names is appended after it, so what a part shows is never lost (a translation that
 * dropped its slot). An empty text is left out, so a surface appends the same nodes it appended
 * before the catalogue (localise-lingua-settings D3).
 */
export function fillSlots(message: string, parts: readonly (SlotPart | readonly SlotPart[])[]): SlotPart[] {
  const nodes: SlotPart[] = [];
  const placed = new Set<number>();
  const place = (index: number): void => {
    const part = parts[index];
    if (part === undefined) return;
    const again = placed.has(index);
    placed.add(index);
    for (const p of several(part) ? part : [part]) nodes.push(again && typeof p !== "string" ? p.cloneNode(true) : p);
  };
  let at = 0;
  for (const match of message.matchAll(SLOT_PATTERN)) {
    if (match.index > at) nodes.push(message.slice(at, match.index));
    place(Number(match[1]));
    at = match.index + match[0].length;
  }
  if (at < message.length) nodes.push(message.slice(at));
  parts.forEach((_, index) => {
    if (!placed.has(index)) place(index);
  });
  return nodes;
}

/**
 * The mark a page's `<html>` carries until its static copy is filled: each page's stylesheet hides
 * `body` under it, so nothing shows empty before the catalogue (localise-lingua-reading-surfaces D2).
 */
export const COPY_PENDING_ATTR = "data-copy-pending";

/** The attribute naming the entry a text node shows: `data-copy="key"`. */
export const COPY_ATTR = "data-copy";

/**
 * The attributes a page may take from the catalogue, as `data-copy-<attribute>="key"`: the ones that
 * carry words. Nothing else — an event handler, a `href`, a `style` — is ever written from a module.
 */
const COPY_ATTRIBUTES: readonly string[] = ["aria-label", "title", "placeholder", "alt"];

/**
 * Fill a page's static copy from a surface's module (D2): an element with `data-copy="key"` has its
 * whole text replaced by the entry — whatever it held, children included, so such a node holds
 * text only — and one with `data-copy-<attribute>="key"` takes the entry as that attribute, for the
 * attributes of `COPY_ATTRIBUTES` alone (`data-copy-aria-label`, `data-copy-title`…); then
 * `data-copy-pending` leaves `<html>`, and the page shows. Only a plain text fills a node — a slot
 * message is the script's to render — and a key the module lacks leaves its node as it is: the
 * page's spec asserts every node, so a missing key fails there, never on a reader's screen.
 */
export function fillPage(document: Document, copy: Record<string, unknown>): void {
  const text = (key: string | null): string | null => {
    const entry = key == null ? undefined : copy[key];
    return typeof entry === "string" ? entry : null;
  };
  for (const el of document.querySelectorAll(`[${COPY_ATTR}]`)) {
    const entry = text(el.getAttribute(COPY_ATTR));
    if (entry != null) el.textContent = entry;
  }
  for (const attribute of COPY_ATTRIBUTES) {
    const keyed = `${COPY_ATTR}-${attribute}`;
    for (const el of document.querySelectorAll(`[${keyed}]`)) {
      const entry = text(el.getAttribute(keyed));
      if (entry != null) el.setAttribute(attribute, entry);
    }
  }
  document.documentElement.removeAttribute(COPY_PENDING_ATTR);
}

/**
 * A page's first step (D1, D2), the same for the popup, the side panel and the reader: read the
 * interface language, say it in the page's `lang`, then fill the page's static copy from the
 * surface's module in that language — which shows the page — and hand both back for what the script
 * renders next. It never rejects: a key that cannot be read is French (`interfaceLanguage`), so a
 * failed read still shows a filled page, as a page holding its French did before the catalogue.
 */
export async function fillPageInLanguage<Copy extends Record<string, unknown>>(
  document: Document,
  area: InterfaceLanguageArea,
  moduleOf: (language: InterfaceLanguage) => Copy,
): Promise<{ language: InterfaceLanguage; copy: Copy }> {
  const language = await interfaceLanguage(area);
  const copy = moduleOf(language);
  setDocumentLanguage(document, language);
  fillPage(document, copy);
  return { language, copy };
}

/**
 * What a slot message is called with where the surface renders one node of its own — the popup's
 * bold level, a studied word set apart in its language: the first part's sentinel, `slot(0)`, so the
 * message renders through `renderAround` or `fillSlots` alike (README › Shape).
 */
export const NODE_SLOT = slot(0);

/**
 * Render into `element` a message called with `NODE_SLOT` where `node` goes — « Niveau d'anglais :
 * <b>B1</b> », or a translation that puts the level first: `fillSlots` with the one part — the text
 * before the slot, the node, the text after, an empty text left out. A message without the slot (a
 * translation that dropped it) keeps its text and has the node appended, so what the node shows is
 * never lost.
 */
export function renderAround(element: Element, message: string, node: Node): void {
  element.replaceChildren(...fillSlots(message, [node]));
}

/**
 * A counted message: the form `Intl.PluralRules` picks for `n` in the language, given the number
 * as the language writes it — in French the raw count the surfaces write today (« 1234 carte(s) »,
 * never a grouped « 1 234 »), in English and Spanish the formatted one. A surface that already
 * writes the figure its own way (the ladder's « 20 000 mots ») passes it as `formatted`.
 */
export function plural(
  language: InterfaceLanguage,
  n: number,
  forms: PluralForms,
  formatted: string = formatCount(language, n),
): string {
  const category = rulesOf(language).select(n);
  const form = category === "one" ? forms.one : category === "many" ? (forms.many ?? forms.other) : forms.other;
  return form(formatted);
}
