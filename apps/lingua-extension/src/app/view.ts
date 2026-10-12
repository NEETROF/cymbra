import { app as enApp } from "../i18n/en/app.ts";
import { app as esApp } from "../i18n/es/app.ts";
import { app as frApp } from "../i18n/fr/app.ts";
import { formatDate, formatTime, type InterfaceLanguage } from "../i18n/index.ts";

// What the Cymbra Lingua app's home shows of its own beside the extension's Révision
// (add-lingua-apple-review): how old the deck is.

const APP_COPY: Record<InterfaceLanguage, typeof frApp> = { fr: frApp, en: enApp, es: esApp };

export function appCopy(language: InterfaceLanguage): typeof frApp {
  return APP_COPY[language];
}

/** « Cartes de Safari, 12 oct. à 9:41 », or null when the extension did not say when it saved them. */
export function savedAtLine(language: InterfaceLanguage, savedAt: number): string | null {
  if (savedAt <= 0) return null;
  const date = new Date(savedAt);
  const day = formatDate(language, date, { day: "numeric", month: "short" });
  return appCopy(language).savedAt(`${day} ${formatTime(language, date)}`);
}
