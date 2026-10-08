import { INTERFACE_LANGUAGES, type InterfaceLanguage } from "../i18n/index.ts";

// Which language Cymbra ID writes to the reader in, and which deletion page the account page links
// to (localise-lingua-account-onboarding D2, decision M12 of the language matrix programme). The
// account's locale is shared: Cymbra ID stores it as sent and Cymbra Music adopts it, so what the
// extension sends moves the e-mails of every Cymbra app.

/**
 * The languages Cymbra speaks: Cymbra Music's locales (`apps/music/lib/l10n/app_<code>.arb`), the
 * ones Cymbra ID's e-mails are written in. A test holds this list against those files, so a language
 * added to Music is noticed here.
 */
export const CYMBRA_LANGUAGES: readonly string[] = ["fr", "en", "es", "it"];

/** A language tag's primary subtag, lower-cased: `fr-FR` → `fr`, `pt_BR` → `pt`. */
function primarySubtag(tag: string): string {
  return tag.trim().split(/[-_]/)[0].toLowerCase();
}

/**
 * The locale sent on the four requests that carry one — sign-up, resending the code, requesting a
 * password reset, setting a password — always a bare primary subtag, never the browser's whole tag:
 * the browser's language when Cymbra speaks it and the extension does not (`it`), so a reader whose
 * Music writes to them in Italian keeps their Italian e-mails; the interface language otherwise,
 * whatever else the browser is in (a German browser sends the interface language, not `de`).
 */
export function accountLocale(interfaceLanguage: InterfaceLanguage, browserLanguage: string): string {
  const browser = primarySubtag(browserLanguage);
  const speaksIt = CYMBRA_LANGUAGES.includes(browser);
  const extensionSpeaksIt = (INTERFACE_LANGUAGES as readonly string[]).includes(browser);
  return speaksIt && !extensionSpeaksIt ? browser : interfaceLanguage;
}

/**
 * The site page that deletes the whole Cymbra account (add-lingua-privacy-controls D5), chosen by the
 * interface language alone — a page the reader reads, not an e-mail: the French page for `fr`, the
 * English one otherwise until the site has a Spanish page (change 29 adds it). A whole tag is read by
 * its primary subtag, as the page's first callers passed the browser's.
 */
export function deleteAccountUrl(interfaceLanguage: string): string {
  return primarySubtag(interfaceLanguage) === "fr"
    ? "https://cymbra.app/suppression-compte/"
    : "https://cymbra.app/en/delete-account/";
}
