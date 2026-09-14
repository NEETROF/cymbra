import { SIGNIN_ERROR_KEY } from "../state/session.ts";
import { AccountFlow, type AccountView, viewFromHash } from "./flow.ts";
import { type AccountMessage, type AccountReply, PENDING_EMAIL_KEY } from "./messages.ts";
import { type AccountActions, renderAccount } from "./view.ts";

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

const pending = {
  async get(): Promise<string | null> {
    try {
      const value = (await chrome.storage.session.get(PENDING_EMAIL_KEY))[PENDING_EMAIL_KEY];
      return typeof value === "string" && value.length > 0 ? value : null;
    } catch {
      return null;
    }
  },
  async set(email: string | null): Promise<void> {
    try {
      await chrome.storage.session.set({ [PENDING_EMAIL_KEY]: email });
    } catch {
      // storage.session unavailable: a reload just starts at sign-in.
    }
  },
};

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
  const actions: AccountActions = {
    signInEmail: (email, password) => void flow.signInEmail(email, password),
    signUp: (email, password) => void flow.signUp(email, password),
    verify: (code) => void flow.verify(code),
    resend: () => void flow.resend(),
    requestReset: (email) => void flow.requestReset(email),
    resetPassword: (code, newPassword) => void flow.resetPassword(code, newPassword),
    signInWith: (provider) => void flow.signInWith(provider),
    signOut: () => void flow.signOut(),
    go: (view) => void flow.go(view),
  };
  const flow = new AccountFlow({ send, pending, locale: navigator.language || "fr", clearPersistedError }, (state) => {
    renderAccount(root, state, actions);
    syncHash(state.view);
  });
  window.addEventListener("hashchange", () => void flow.go(viewFromHash(location.hash)));
  void flow.init(location.hash);
}

main();
