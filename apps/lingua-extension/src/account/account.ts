import { SIGNIN_ERROR_KEY } from "../state/session.ts";
import { AccountFlow, type AccountView, type PendingEmailStore, viewFromHash, wantsConnected } from "./flow.ts";
import { type AccountMessage, type AccountReply, PENDING_EMAIL_KEY, PENDING_PASSWORD_EMAIL_KEY } from "./messages.ts";
import { type AccountActions, renderAccount } from "./view.ts";
import { followSurfaceLook } from "../reading/surface-look.ts";
import { interfaceLanguage } from "../i18n/language.ts";
import { reloadOnNativeLanguageChange } from "../state/native-language.ts";

// This page is a surface: it follows the reader's colours and text size (add-lingua-colour-settings D8, D9).
followSurfaceLook(document.documentElement);
// Another native language chosen anywhere: the page reloads in it (add-lingua-native-language-choice D3).
reloadOnNativeLanguageChange(interfaceLanguage({ get: (key) => chrome.storage.local.get(key) }));

// Account page bootstrap (add-lingua-account-parity, design D1): a tab — unlike the popup
// it survives the reader switching to their mailbox for the code. Wires the controller to
// the background (runtime messages), chrome.storage.session (pending email only) and the
// URL hash (so a reload resumes the same step). Excluded from coverage (DOM/Chrome wiring;
// flow.ts and view.ts are unit-tested).

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

function syncHash(view: AccountView): void {
  const hash = `#${view}`;
  if (location.hash !== hash) history.replaceState(null, "", hash);
}

function main(): void {
  const root = document.getElementById("account-root");
  if (!root) return;
  // The popup's « Gérer mes données » opens #data; the hash is replaced by the view once
  // rendered, so remember the request first.
  const wantsData = location.hash === "#data";
  // The handle's availability is asked once typing pauses; the controller drops stale answers.
  let handleCheck: ReturnType<typeof setTimeout> | null = null;
  const actions: AccountActions = {
    editHandle: (candidate) => {
      const state = flow.editHandle(candidate);
      if (handleCheck !== null) clearTimeout(handleCheck);
      handleCheck = state.handleStatus === "checking" ? setTimeout(() => void flow.checkHandle(), 400) : null;
    },
    commitHandle: () => void flow.commitHandle(),
    askErase: () => void flow.askErase(),
    cancelErase: () => void flow.cancelErase(),
    eraseLinguaData: () => void flow.eraseLinguaData(),
    abandonHandle: () => void flow.abandonHandle(),
    signInEmail: (email, password) => void flow.signInEmail(email, password),
    signUp: (email, password) => void flow.signUp(email, password),
    verify: (code) => void flow.verify(code),
    resend: () => void flow.resend(),
    requestReset: (email) => void flow.requestReset(email),
    resetPassword: (code, newPassword) => void flow.resetPassword(code, newPassword),
    signInWith: (provider) => void flow.signInWith(provider),
    signOut: () => void flow.signOut(),
    go: (view) => void flow.go(view),
    openConnected: () => void flow.openConnected(),
    leaveConnected: () => void flow.leaveConnected(),
    retryIdentities: () => void flow.loadIdentities(),
    link: (provider) => void flow.link(provider),
    askRemove: (identity) => void flow.askRemove(identity),
    cancelRemove: () => void flow.cancelRemove(),
    remove: () => void flow.remove(),
    showPasswordForm: () => void flow.showPasswordForm(),
    cancelPassword: () => void flow.cancelPassword(),
    restartPassword: () => void flow.restartPassword(),
    setPassword: (email, password) => void flow.setPassword(email, password),
    confirmPassword: (code) => void flow.confirmPassword(code),
  };
  const flow = new AccountFlow(
    { send, pending, pendingPassword, locale: navigator.language || "fr", clearPersistedError },
    (state) => {
      renderAccount(root, state, actions);
      syncHash(state.view);
    },
  );
  window.addEventListener(
    "hashchange",
    () => void (wantsConnected(location.hash) ? flow.openConnected() : flow.go(viewFromHash(location.hash))),
  );
  // Safari: an Apple/Google sign-in finishes in the host app, which hands the id_token back.
  // Collect it once the page is ready, and each time the reader comes back to it.
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible") void flow.collectHandedToken();
  });
  void flow
    .init(location.hash)
    .then(() => flow.collectHandedToken())
    .then(() => {
      if (wantsData) document.getElementById("data")?.scrollIntoView({ block: "start" });
    });
}

main();
