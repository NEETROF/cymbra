import {
  type InterfaceLanguage,
  type InterfaceLanguageArea,
  interfaceLanguage,
  setDocumentLanguage,
} from "./language.ts";

// The catalogue's helpers (add-lingua-interface-language D1, D2, D4): how a count picks its
// plural form, how a number, a percentage and a date are written in each language, and how a page
// is filled from a module. No surface's copy is mapped here — a surface imports its own
// `./{fr,en,es}/<surface>.ts` and picks by the interface language — so importing this module costs
// an entry nothing but these functions.

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
  | "colours"
  | "display"
  | "translation"
  | "account-setting"
  | "account"
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
  "colours",
  "display",
  "translation",
  "account-setting",
  "account",
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
 * What a slot message is called with where the surface renders a node of its own — the popup's bold
 * level, a studied word set apart in its language: a character no message writes, so the message
 * splits around it (README › Shape).
 */
export const NODE_SLOT = "\u0000";

/**
 * Render into `element` a message called with `NODE_SLOT` where `node` goes — « Niveau d'anglais :
 * <b>B1</b> », or a translation that puts the level first: the text before the slot, the node, the
 * text after; an empty text is left out. A message without the slot (a translation that dropped it)
 * keeps its text and has the node appended, so what the node shows is never lost.
 */
export function renderAround(element: Element, message: string, node: Node): void {
  const [before = "", ...rest] = message.split(NODE_SLOT);
  const parts: (Node | string)[] = rest.length === 0 ? [before, node] : [before, node, rest.join("")];
  element.replaceChildren(...parts.filter((part) => part !== ""));
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
