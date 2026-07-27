import { createI18n } from "vue-i18n";
import en from "./locales/en.json";
import fr from "./locales/fr.json";

// The console ships English + French. Messages live in `locales/*.json` (editable by
// an external translation tool). `en.json` is the canonical schema: every other
// locale is typed against it below, so a missing key is a compile error. Locale =
// saved choice → browser language → English fallback. `t` is available in templates
// ($t, globalInjection) and in plain modules via `i18n.global.t` (e.g. error mapping).
export const SUPPORTED_LOCALES = ["en", "fr"] as const;
export type Locale = (typeof SUPPORTED_LOCALES)[number];
const STORAGE_KEY = "cymbra.bo.locale";

/** Message shape, derived from the canonical English catalogue. */
export type MessageSchema = typeof en;

// Type-safety guard: each locale must satisfy the English schema — a JSON file
// missing a key (or with a wrong-typed value) fails `vue-tsc` here.
const messages: Record<Locale, MessageSchema> = { en, fr };

function detectLocale(): Locale {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "en" || saved === "fr") return saved;
  } catch {
    /* no storage (SSR/tests) */
  }
  const nav = (globalThis.navigator?.language ?? "en").slice(0, 2).toLowerCase();
  return nav === "fr" ? "fr" : "en";
}

export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: detectLocale(),
  fallbackLocale: "en",
  messages,
});

export function setLocale(locale: Locale): void {
  i18n.global.locale.value = locale;
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    /* ignore */
  }
  if (globalThis.document) document.documentElement.lang = locale;
}

export function currentLocale(): Locale {
  return i18n.global.locale.value as Locale;
}

/** Plain-module translate (outside components) — used by the error mapper. */
export const t = i18n.global.t;
