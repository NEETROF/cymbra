import { AccountFlow, type PendingEmailStore } from "../account/flow.ts";
import { type AccountActions, renderAccount } from "../account/view.ts";
import type { InterfaceLanguage } from "../i18n/index.ts";
import type { AppAccountSend, EmailSignIn } from "./account.ts";

// The Cymbra Lingua app's email sign-in (add-lingua-app-email-signin D2, D3): a modal `<dialog>` over
// the review holding the extension's own account flow and view — `AccountFlow` and `renderAccount` in
// their app scope: sign-in, the code step of an unverified address, « Mot de passe oublié ? » and the
// new password; no account creation, no provider, no handle step. The flow's messages go to the app's
// Session (account.ts); the password lives in the flow's memory only, for the code step, and the
// pending address in this module's memory while the panel is open — the page has no
// `storage.session`, and its own storage under a custom scheme is not guaranteed.

export interface EmailPanelDeps {
  /** The panel: `#email-panel` of app.html. */
  dialog: HTMLDialogElement;
  /** Where the account view renders, inside the dialog. */
  root: HTMLElement;
  /** « Annuler »: closes the panel without signing in. */
  cancel: HTMLButtonElement;
  /** The interface language: the flow's copy and errors. */
  language: InterfaceLanguage;
  /** The locale the requests carry (`appAccountLocale`). */
  locale: string;
}

export interface EmailPanel {
  /** Show the panel over `send`; true once signed in, false when the reader closed it. */
  signIn: EmailSignIn;
  /** Whether the panel is showing — the app keeps its page while it is (D6). */
  isOpen(): boolean;
}

/** A page-memory store: the code step's address, gone with the panel. */
function memoryPending(): PendingEmailStore {
  let email: string | null = null;
  return {
    get: async () => email,
    set: async (next) => {
      email = next;
    },
  };
}

const ignore = (): void => {};

export function emailPanel(deps: EmailPanelDeps): EmailPanel {
  const { dialog, root } = deps;
  let settle: ((signedIn: boolean) => void) | null = null;

  /** Close the panel and settle its sign-in once; the flow and its memory go with it. */
  function finish(signedIn: boolean): void {
    const done = settle;
    settle = null;
    if (typeof dialog.close === "function") dialog.close();
    dialog.removeAttribute("open");
    root.replaceChildren();
    done?.(signedIn);
  }

  deps.cancel.addEventListener("click", () => finish(false));
  // Escape, or the platform closing the dialog: no sign-in. The event comes after the fact: one left over
  // from a panel already settled (and since reopened) finds the dialog open, and is ignored.
  dialog.addEventListener("close", () => {
    if (settle && !dialog.hasAttribute("open")) finish(false);
  });

  function signIn(send: AppAccountSend): Promise<boolean> {
    // A second request while one is open answers the first with nothing, then opens anew.
    if (settle) finish(false);
    return new Promise<boolean>((resolve) => {
      settle = resolve;
      const flow = new AccountFlow(
        {
          send,
          pending: memoryPending(),
          pendingPassword: memoryPending(),
          locale: deps.locale,
          deletionLanguage: deps.language,
          language: deps.language,
          clearPersistedError: async () => {},
          scope: "app",
          onSignedIn: () => finish(true),
        },
        (state) => {
          if (settle) renderAccount(root, state, actions, deps.language);
        },
      );
      const actions: AccountActions = {
        signInEmail: (email, password) => void flow.signInEmail(email, password),
        verify: (code) => void flow.verify(code),
        resend: () => void flow.resend(),
        requestReset: (email) => void flow.requestReset(email),
        resetPassword: (code, newPassword) => void flow.resetPassword(code, newPassword),
        go: (view) => void flow.go(view),
        // Not shown in the app scope (D3, D4).
        signUp: ignore,
        signInWith: ignore,
        signOut: ignore,
        editHandle: ignore,
        commitHandle: ignore,
        abandonHandle: ignore,
        askErase: ignore,
        cancelErase: ignore,
        eraseLinguaData: ignore,
        openConnected: ignore,
        leaveConnected: ignore,
        retryIdentities: ignore,
        link: ignore,
        askRemove: ignore,
        cancelRemove: ignore,
        remove: ignore,
        showPasswordForm: ignore,
        cancelPassword: ignore,
        restartPassword: ignore,
        setPassword: ignore,
        confirmPassword: ignore,
      };
      if (typeof dialog.showModal === "function") dialog.showModal();
      else dialog.setAttribute("open", "");
      void flow.init("");
    });
  }

  return { signIn, isOpen: () => settle !== null };
}

/**
 * Whether the app's return to the foreground reloads the page (add-lingua-app-email-signin D6): not
 * while a review is under way — its queue stays — and not while the email sign-in is open, so a reader
 * back from Mail finds the code step with its address.
 */
export function reloadsOnForeground(state: { reviewing: boolean; emailPanelOpen: boolean }): boolean {
  return !state.reviewing && !state.emailPanelOpen;
}
