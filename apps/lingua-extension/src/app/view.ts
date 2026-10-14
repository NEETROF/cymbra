import { app as enApp } from "../i18n/en/app.ts";
import { app as esApp } from "../i18n/es/app.ts";
import { app as frApp } from "../i18n/fr/app.ts";
import { formatDate, formatNumber, formatTime, type InterfaceLanguage } from "../i18n/index.ts";
import type { AppModelView } from "./translation.ts";

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

/** What the offer's buttons do. */
export interface ModelOfferActions {
  accept(): void;
  decline(): void;
}

function button(label: string, className: string, onClick: () => void): HTMLButtonElement {
  const b = document.createElement("button");
  b.type = "button";
  b.className = className;
  b.textContent = label;
  b.addEventListener("click", onClick);
  return b;
}

/**
 * The models' line (add-lingua-review-translation-fallback D7): hidden with nothing to offer; else the
 * offer with its size, the download's progress, or its failure with « Réessayer ». Text nodes only.
 */
export function renderModelOffer(
  el: HTMLElement,
  view: AppModelView,
  language: InterfaceLanguage,
  actions: ModelOfferActions,
): void {
  const copy = appCopy(language);
  el.replaceChildren();
  el.hidden = view.phase === "none";
  const line = document.createElement("span");
  if (view.phase === "offer") {
    const size = copy.megabytes(
      formatNumber(language, view.bytes / 1_000_000, { minimumFractionDigits: 1, maximumFractionDigits: 1 }),
    );
    line.textContent = copy.modelOffer(size);
    el.append(
      line,
      button(copy.modelDownload, "app-link app-model-download", () => actions.accept()),
      button(copy.modelLater, "app-link app-model-later", () => actions.decline()),
    );
  } else if (view.phase === "downloading") {
    const percent = view.total > 0 ? Math.min(100, Math.floor((view.received / view.total) * 100)) : 0;
    line.textContent = copy.modelDownloading(formatNumber(language, percent));
    el.append(line);
  } else if (view.phase === "failed") {
    line.textContent = copy.modelFailed;
    el.append(
      line,
      button(copy.modelRetry, "app-link app-model-retry", () => actions.accept()),
      button(copy.modelLater, "app-link app-model-later", () => actions.decline()),
    );
  }
}

/** The app's session as its line shows it. */
export interface AccountLineState {
  /** « Traduction en ligne » is ticked in the extension and the account has access: the line matters. */
  relevant: boolean;
  signedIn: boolean;
  /** The reader signed out from the app: the line offers to sign in again. */
  signedOut: boolean;
}

export interface AccountLineActions {
  signIn(): void;
  signOut(): void;
}

/**
 * The app's session line (add-lingua-review-translation-fallback D8): « Connecté à Cymbra pour la
 * traduction en ligne — Se déconnecter » while signed in; otherwise, where online translation is
 * offered or after a sign-out from the app, « Se connecter pour la traduction en ligne » — an email
 * account signs in too (add-lingua-app-email-signin D8). Hidden where none applies.
 */
export function renderAccountLine(
  el: HTMLElement,
  state: AccountLineState,
  language: InterfaceLanguage,
  actions: AccountLineActions,
): void {
  const copy = appCopy(language);
  el.replaceChildren();
  const text = document.createElement("span");
  if (state.signedIn) {
    text.textContent = `${copy.signedIn} — `;
    el.append(
      text,
      button(copy.signOut, "app-link app-sign-out", () => actions.signOut()),
    );
  } else if (state.relevant) {
    el.append(button(copy.signIn, "app-link app-sign-in", () => actions.signIn()));
  }
  el.hidden = el.childNodes.length === 0;
}
