import { fillPageInLanguage, type InterfaceLanguage, type InterfaceLanguageArea } from "../i18n/index.ts";
import { accountCopy } from "./copy.ts";
import { AccountFlow, type AccountFlowDeps, type AccountView, viewFromHash, wantsConnected } from "./flow.ts";
import { accountLanguage, chosenLanguage } from "./locale.ts";
import { type AccountActions, renderAccount } from "./view.ts";

// The account page's mount (localise-lingua-account-onboarding D1, D2), apart from the Chrome wiring
// of account.ts so that it is tested: the interface language read first, the page filled in it, the
// flow told the locale to send and the language — the browser's until the reader has chosen theirs —,
// the view rendered in it, the step a reload left resumed.

export interface AccountPageDeps extends Pick<
  AccountFlowDeps,
  "send" | "pending" | "pendingPassword" | "clearPersistedError"
> {
  /**
   * The preferences area the interface language is read from: chrome.storage.local, which also holds
   * whether the choice of native language was made on this device (change 20's marker).
   */
  area: InterfaceLanguageArea;
  /**
   * The reader's data, the background's (`messagedArea`): where its owner records the native language
   * the reader last chose (change 20) — read with the marker, never written.
   */
  store: InterfaceLanguageArea;
  /** The browser's language, `navigator.language` — whole tag or empty. */
  browserLanguage: string;
  /** The page's `#hash` now, and how the page replaces it once a view is rendered. */
  hash: () => string;
  replaceHash: (hash: string) => void;
}

export interface MountedAccountPage {
  flow: AccountFlow;
  language: InterfaceLanguage;
}

/**
 * Mount the account page into `doc` (its `#account-root`): fill the page in the interface language,
 * build the flow and render its views, follow the hash, collect a handed token (Safari) once ready and
 * each time the page is shown again. Resolves once the first step is rendered; `null` without a root.
 */
export async function mountAccountPage(doc: Document, deps: AccountPageDeps): Promise<MountedAccountPage | null> {
  // The interface language first, with this page's first read of its own (the colours' read and the
  // pending e-mails' are other things): the page's static copy is filled from the catalogue and its
  // lang said before anything shows — the body is hidden until then. A read that fails or hangs is French.
  const filled = fillPageInLanguage(doc, deps.area, accountCopy);
  // Then, alongside, whether the reader chose their language: it decides only what the requests carry
  // and where the deletion link goes (D2), so nothing waits for it before the page is filled. A read
  // that fails is no choice.
  const chosen = chosenLanguage(deps.area, deps.store);
  const { language } = await filled;
  // Until the reader has chosen the language the page is in, nothing written over the account's
  // language and the deletion page by the browser's tag; the account locale and the interface
  // language's page once they have (D2).
  const said = accountLanguage(language, deps.browserLanguage, await chosen);
  const root = doc.getElementById("account-root");
  if (!root) return null;
  // The popup's « Gérer mes données » opens #data; the hash is replaced by the view once
  // rendered, so remember the request first.
  const wantsData = deps.hash() === "#data";
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
  const syncHash = (view: AccountView): void => {
    const hash = `#${view}`;
    if (deps.hash() !== hash) deps.replaceHash(hash);
  };
  const flow = new AccountFlow(
    {
      send: deps.send,
      pending: deps.pending,
      pendingPassword: deps.pendingPassword,
      // What Cymbra ID writes to the reader in, and the deletion page (D2).
      locale: said.locale,
      keepAccountLocale: said.keepAccountLocale,
      deletionLanguage: said.deletion,
      language,
      clearPersistedError: deps.clearPersistedError,
    },
    (state) => {
      renderAccount(root, state, actions, language);
      syncHash(state.view);
    },
  );
  doc.defaultView?.addEventListener(
    "hashchange",
    () => void (wantsConnected(deps.hash()) ? flow.openConnected() : flow.go(viewFromHash(deps.hash()))),
  );
  // Safari: an Apple/Google sign-in finishes in the host app, which hands the id_token back.
  // Collect it once the page is ready, and each time the reader comes back to it.
  doc.addEventListener("visibilitychange", () => {
    if (doc.visibilityState === "visible") void flow.collectHandedToken();
  });
  await flow.init(deps.hash());
  await flow.collectHandedToken();
  if (wantsData) doc.getElementById("data")?.scrollIntoView({ block: "start" });
  return { flow, language };
}
