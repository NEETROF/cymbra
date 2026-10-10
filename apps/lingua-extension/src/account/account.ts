import { SIGNIN_ERROR_KEY } from "../state/session.ts";
import type { PendingEmailStore } from "./flow.ts";
import { type AccountMessage, type AccountReply, PENDING_EMAIL_KEY, PENDING_PASSWORD_EMAIL_KEY } from "./messages.ts";
import { mountAccountPage } from "./page.ts";
import { followSurfaceLook } from "../reading/surface-look.ts";
import { DEFAULT_INTERFACE_LANGUAGE } from "../i18n/language.ts";
import { reloadOnNativeLanguageChange } from "../state/native-language.ts";
import { messagedArea } from "../state/store.ts";

// This page is a surface: it follows the reader's colours and text size (add-lingua-colour-settings D8, D9).
followSurfaceLook(document.documentElement);

// Account page bootstrap (add-lingua-account-parity, design D1): a tab — unlike the popup
// it survives the reader switching to their mailbox for the code. Wires the controller to
// the background (runtime messages), chrome.storage.session (pending email only) and the
// URL hash (so a reload resumes the same step). `mountAccountPage` (page.ts, tested) reads the
// interface language first (localise-lingua-account-onboarding D1) and tells the server the locale
// to write in — the browser's until the reader has chosen their language, which it reads from
// chrome.storage.local and the background's store (D2). Excluded from coverage (Chrome wiring only).

async function send(message: AccountMessage): Promise<AccountReply | null> {
  try {
    return ((await chrome.runtime.sendMessage(message)) as AccountReply | undefined) ?? null;
  } catch {
    return null;
  }
}

/** An email waiting for its code, in chrome.storage.session under `key` (never a password). */
function pendingEmail(key: string): PendingEmailStore {
  return {
    async get(): Promise<string | null> {
      try {
        const value = (await chrome.storage.session.get(key))[key];
        return typeof value === "string" && value.length > 0 ? value : null;
      } catch {
        return null;
      }
    },
    async set(email: string | null): Promise<void> {
      try {
        await chrome.storage.session.set({ [key]: email });
      } catch {
        // storage.session unavailable: a reload just starts over.
      }
    },
  };
}

const pending = pendingEmail(PENDING_EMAIL_KEY);
/** « Définir un mot de passe »'s own code step (add-lingua-connected-accounts D4). */
const pendingPassword = pendingEmail(PENDING_PASSWORD_EMAIL_KEY);

async function clearPersistedError(): Promise<void> {
  try {
    await chrome.storage.session.set({ [SIGNIN_ERROR_KEY]: null });
  } catch {
    // nothing to clear
  }
}

const mounted = mountAccountPage(document, {
  area: { get: (key) => chrome.storage.local.get(key) },
  store: messagedArea(),
  browserLanguage: navigator.language || "",
  send,
  pending,
  pendingPassword,
  clearPersistedError,
  hash: () => location.hash,
  replaceHash: (hash) => history.replaceState(null, "", hash),
});

// Another native language chosen anywhere: the page reloads in it (add-lingua-native-language-choice
// D3), compared with the language it was filled in — French when it could not be mounted.
reloadOnNativeLanguageChange(
  mounted.then(
    (page) => page?.language ?? DEFAULT_INTERFACE_LANGUAGE,
    () => DEFAULT_INTERFACE_LANGUAGE,
  ),
);
