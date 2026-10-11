import { INTERFACE_LANGUAGES, type InterfaceLanguage } from "../i18n/index.ts";
import { nativeLanguageChosen } from "../state/native-language.ts";
import { LAST_NATIVE_KEY } from "../state/store.ts";

// Which locale the account page sends Cymbra ID, and which deletion page it links to
// (localise-lingua-account-onboarding D2, decision M12 of the language matrix programme;
// send-lingua-browser-locale-on-account-emails D1–D3). The account's locale is shared — Cymbra Music
// adopts it, its reminders read it — and Cymbra ID (prefer-account-locale-for-emails) writes every
// account e-mail in the account's stored language, else the request's, else English, and records a
// request's locale only on an account that has none: only `SetLocale`, which the extension never
// calls, replaces a stored one. So no request this page sends, from any device, moves a language the
// account has, and the locale it carries decides the e-mail only for an account that has none. Hence
// one locale on the four requests (sign-up, resending the code, requesting a reset, setting a
// password): the browser's whole tag until the reader has chosen their language on this device (the
// choice is a device's, never synced), and the interface language once they have.

/**
 * The languages Cymbra speaks: Cymbra Music's locales (`apps/music/lib/l10n/app_<code>.arb`), the
 * ones Cymbra ID's e-mails are written in. A test holds this list against those files, so a language
 * added to Music is noticed here.
 */
export const CYMBRA_LANGUAGES: readonly string[] = ["fr", "en", "es", "it"];

/** What a page says when the browser gives no language: French, as it always did. */
const NO_BROWSER_LANGUAGE = "fr";

/** A language tag's primary subtag, lower-cased: `fr-FR` → `fr`, `pt_BR` → `pt`. */
function primarySubtag(tag: string): string {
  return tag.trim().split(/[-_]/)[0].toLowerCase();
}

/**
 * The account locale of a reader who has chosen their language — always a bare primary subtag, never
 * the browser's whole tag: the browser's language when Cymbra speaks it and the extension does not
 * (`it`), so a reader whose Music writes to them in Italian keeps their Italian e-mails; the interface
 * language otherwise, whatever else the browser is in (a German browser sends the interface language,
 * not `de`).
 */
export function accountLocale(interfaceLanguage: InterfaceLanguage, browserLanguage: string): string {
  const browser = primarySubtag(browserLanguage);
  const speaksIt = CYMBRA_LANGUAGES.includes(browser);
  const extensionSpeaksIt = (INTERFACE_LANGUAGES as readonly string[]).includes(browser);
  return speaksIt && !extensionSpeaksIt ? browser : interfaceLanguage;
}

/** Where the account page reads whether the reader chose: an area that answers a key. */
interface KeyArea {
  get(keys: string): Promise<Record<string, unknown>>;
}

/**
 * The language the reader chose on this device, as add-lingua-native-language-choice (change 20)
 * records it — or null while they have not. Both records are the device's own, never synced: a
 * choice made on another browser is not read here. A choice is recorded twice, and only both
 * together say the reader made it: this device marked the choice as made
 * (`cymbra-lingua-native-chosen`, in `preferences`, chrome.storage.local) and the store's owner
 * recorded the native language the reader last chose (`cymbra-lingua-last-native`, in `store`, the
 * background's). Neither alone does: an update marks every installed extension as chosen without
 * asking it (M22), and a new install's preset records the language it applies before the reader
 * answers. An answer that keeps the language the extension already had writes no record: where none
 * was written before, the page keeps to the rule of a reader who has not chosen. A read that fails
 * is no choice.
 */
export async function chosenLanguage(preferences: KeyArea, store: KeyArea): Promise<string | null> {
  try {
    if (!(await nativeLanguageChosen(preferences))) return null;
    const recorded = (await store.get(LAST_NATIVE_KEY))[LAST_NATIVE_KEY];
    return typeof recorded === "string" ? recorded : null;
  } catch (e) {
    console.warn("[Cymbra Lingua] could not read whether the language was chosen, keeping the browser's:", e);
    return null;
  }
}

/** What the account page tells Cymbra ID, and what its deletion link is chosen by (D1). */
export interface AccountLanguage {
  /** The locale the four requests carry: sign-up, the code resent, a reset requested, a password set. */
  locale: string;
  /** The tag `deleteAccountUrl` reads. */
  deletion: string;
}

/**
 * The account page's languages (D1). Until the reader has chosen the language the page is in
 * (`chosen`, `chosenLanguage`): the browser's whole tag (`en-GB`, `fr-FR`) on the four requests, as
 * the page always sent it at sign-up and when setting a password, French when the browser gives none,
 * and the deletion page by that tag, as before. Cymbra ID records it only on an account that has no
 * language, so a device where the reader never chose writes over no account's language — one chosen
 * on another device or given by Cymbra Music — and an account without one gets its e-mails in the
 * browser's language rather than English. The whole tag, not a bare code (D3): Music adopts only a
 * whole code of its own, so its screen is not moved from a device where the reader chose nothing.
 * Once they have chosen: the account locale (`accountLocale`) on all four, and the deletion page by
 * the interface language alone — a page the reader reads, where the account locale may be the
 * browser's `it`.
 */
export function accountLanguage(
  interfaceLanguage: InterfaceLanguage,
  browserLanguage: string,
  chosen: string | null,
): AccountLanguage {
  if (chosen !== interfaceLanguage) {
    const tag = browserLanguage || NO_BROWSER_LANGUAGE;
    return { locale: tag, deletion: tag };
  }
  return { locale: accountLocale(interfaceLanguage, browserLanguage), deletion: interfaceLanguage };
}

/**
 * The site page that deletes the whole Cymbra account (add-lingua-privacy-controls D5): the French
 * page for a tag that starts with `fr`, the English one otherwise until the site has a Spanish page
 * (change 29 adds it) — the rule it always had. Callers pass `AccountLanguage.deletion`: the
 * interface language once the reader has chosen it, the browser's whole tag until then.
 */
export function deleteAccountUrl(language: string): string {
  return language.toLowerCase().startsWith("fr")
    ? "https://cymbra.app/suppression-compte/"
    : "https://cymbra.app/en/delete-account/";
}
