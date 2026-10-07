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
  const number = formatted;
  const category = rulesOf(language).select(n);
  const form = category === "one" ? forms.one : category === "many" ? (forms.many ?? forms.other) : forms.other;
  return form(number);
}
