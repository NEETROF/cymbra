import type { InterfaceLanguage } from "./language.ts";

// The catalogue's helpers (add-lingua-interface-language D1, D2, D4): how a count picks its
// plural form, how a number, a percentage and a date are written in each language. No surface's
// copy is mapped here — a surface imports its own `./{fr,en,es}/<surface>.ts` and picks by the
// interface language — so importing this module costs an entry nothing but these functions.

export type { InterfaceLanguage } from "./language.ts";
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
 * Fill a page's static copy from a surface's module (D2): an element with `data-copy="key"` takes
 * the entry as its text, one with `data-copy-<attribute>="key"` takes it as that attribute
 * (`data-copy-aria-label`, `data-copy-title`); then `data-copy-pending` leaves `<html>`, and the
 * page shows. Only a plain text fills a node — a slot message is the script's to render — and a
 * key the module lacks leaves its node as it is: the page's spec asserts every node, so a missing
 * key fails there, never on a reader's screen.
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
  for (const el of document.querySelectorAll("*")) {
    for (const { name, value } of [...el.attributes]) {
      if (!name.startsWith(`${COPY_ATTR}-`) || name === COPY_PENDING_ATTR) continue;
      const entry = text(value);
      if (entry != null) el.setAttribute(name.slice(COPY_ATTR.length + 1), entry);
    }
  }
  document.documentElement.removeAttribute(COPY_PENDING_ATTR);
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
  formatted: string = language === "fr" ? String(n) : formatNumber(language, n),
): string {
  const category = rulesOf(language).select(n);
  const form = category === "one" ? forms.one : category === "many" ? (forms.many ?? forms.other) : forms.other;
  return form(formatted);
}
