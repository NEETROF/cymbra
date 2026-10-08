import { errorCopy } from "../account/copy.ts";
import { type AccountReply, type AccountState, PENDING_EMAIL_KEY } from "../account/messages.ts";
import { accountSetting as frAccountSetting } from "../i18n/fr/account-setting.ts";
import type { OpenPage } from "../state/open-page.ts";
import type { Provider, Providers } from "../state/oidc.ts";
import { isPersistedSignInError, type PersistedSignInError, SIGNIN_ERROR_KEY } from "../state/session.ts";
import type { AccountSettingCopy } from "./settings-copy.ts";

// « Compte » in Réglages › Données: sign in, see the account, sign out — in every host that
// renders Réglages (the popup, the side panel, the in-page drawer), which is why it moved here
// from the popup's main view, the only place it used to be. Built as plain DOM into the block
// the settings view gives it, like « Traduction étendue ».
//
// The background owns the session (add-lingua-account-parity D2): this only messages it. The
// steps that need a page of their own — creating an account, a forgotten password, the email
// code, the handle — open the account page, a tab that survives the reader leaving for their
// mailbox. The password never leaves this block; for the code step only the email goes across.
//
// Its copy is the catalogue's `account-setting` module, handed by the settings view in the interface
// language (localise-lingua-settings); the sign-in errors it shows are the account page's
// (`errorCopy`), which localise-lingua-account-onboarding moves.

/** What the block needs from the background. A test hands in a fake. */
export interface AccountControls {
  state(): Promise<AccountState | null>;
  providers(): Promise<Providers>;
  /** The account's handle: a string, `null` for none yet, `undefined` when it cannot be read. */
  handle(): Promise<string | null | undefined>;
  signInWith(provider: Provider): Promise<AccountReply | null>;
  signInLocal(email: string, password: string): Promise<AccountReply | null>;
  signOut(): Promise<AccountReply | null>;
  /**
   * A provider sign-in that failed after the surface asking for it was torn down (the popup,
   * when the provider's window takes the focus), persisted by the background. Read once: it is
   * cleared as it is taken.
   */
  takeSignInError(): Promise<PersistedSignInError | null>;
  /** Remember the email whose code the account page will ask for (never the password). */
  rememberPendingEmail(email: string): Promise<void>;
}

async function sendRuntime(message: unknown): Promise<AccountReply | null> {
  try {
    return ((await chrome.runtime.sendMessage(message)) as AccountReply | undefined) ?? null;
  } catch {
    return null; // an asleep or restarting worker
  }
}

/**
 * The background, by runtime messages. `storage.session` may be out of reach (a content
 * script's drawer, by default): the persisted error is then not read — the drawer outlives a
 * provider's window and hears the reply itself — and the account page opens on sign-in.
 */
export function runtimeAccountControls(): AccountControls {
  return {
    state: async () => (await sendRuntime({ type: "account:state" }))?.state ?? null,
    providers: async () =>
      (await sendRuntime({ type: "account:providers" }))?.providers ?? { google: false, apple: false },
    handle: async () => {
      const res = await sendRuntime({ type: "account:profile" });
      return res?.ok ? (res.handle ?? null) : undefined;
    },
    signInWith: (provider) =>
      sendRuntime({ type: provider === "apple" ? "account:signInApple" : "account:signInGoogle" }),
    signInLocal: (email, password) => sendRuntime({ type: "account:signInLocal", email, password }),
    signOut: () => sendRuntime({ type: "account:signOut" }),
    takeSignInError: async () => {
      try {
        const err = (await chrome.storage.session.get(SIGNIN_ERROR_KEY))[SIGNIN_ERROR_KEY];
        if (!isPersistedSignInError(err)) return null;
        await chrome.storage.session.set({ [SIGNIN_ERROR_KEY]: null });
        return err;
      } catch {
        return null;
      }
    },
    rememberPendingEmail: async (email) => {
      try {
        await chrome.storage.session.set({ [PENDING_EMAIL_KEY]: email });
      } catch {
        // storage.session out of reach: the account page opens on sign-in instead.
      }
    },
  };
}

export interface AccountSettingOptions {
  openPage: OpenPage;
  /**
   * Signed in or out here: redraw Réglages, this block included (`refresh`), so the rest follows
   * (Synchronisation, Réinitialisation).
   */
  onChange: () => Promise<void> | void;
  /** Safari: the sign-in went on in the host app (the popup closes, the next open collects it). */
  onHandedOff?: () => void;
  /** The block's copy in the interface language; the French module when not given. */
  copy?: AccountSettingCopy;
}

export interface AccountSettingView {
  /** Show the session as it stands (it may have changed in another surface). */
  refresh(): Promise<void>;
}

/** The block's French copy, the catalogue's module: what it shows when mounted without a language. */
export const ACCOUNT_COPY: AccountSettingCopy = frAccountSetting;

function el<K extends keyof HTMLElementTagNameMap>(
  doc: Document,
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = doc.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
}

/** Mount the account controls into `block` (titled by the settings view). */
export function mountAccountSetting(
  block: HTMLElement,
  controls: AccountControls,
  opts: AccountSettingOptions,
): AccountSettingView {
  const doc = block.ownerDocument;
  const copy = opts.copy ?? ACCOUNT_COPY;
  const button = (className: string, text: string): HTMLButtonElement => {
    const b = el(doc, "button", className, text);
    b.type = "button";
    return b;
  };

  // — Signed out —
  const out = el(doc, "div", "set-account");
  out.hidden = true;
  const google = button("set-primary", copy.google);
  const apple = button("set-primary", copy.apple);
  google.hidden = true;
  apple.hidden = true;
  const local = el(doc, "details", "set-account-local");
  const email = el(doc, "input");
  email.type = "email";
  email.placeholder = copy.email;
  email.autocomplete = "username";
  email.setAttribute("aria-label", copy.email);
  const password = el(doc, "input");
  password.type = "password";
  password.placeholder = copy.password;
  password.autocomplete = "current-password";
  password.setAttribute("aria-label", copy.password);
  const signIn = button("set-reset", copy.signIn);
  const forgot = button("linklike", copy.forgot);
  local.append(el(doc, "summary", undefined, copy.local), email, password, signIn, forgot);
  const signUp = button("linklike", copy.signUp);
  const error = el(doc, "div", "set-warn");
  error.hidden = true;
  // add-lingua-connected-accounts D7: shown where Google or Apple is not offered.
  const providerHint = el(doc, "div", "set-note", copy.providerHint);
  providerHint.hidden = true;
  out.append(el(doc, "div", "set-note", copy.invite), google, apple, local, providerHint, signUp, error);

  // — Signed in —
  const inside = el(doc, "div", "set-account");
  inside.hidden = true;
  const who = el(doc, "b");
  const row = el(doc, "div", "set-account-row");
  row.append(who);
  const handleCta = el(doc, "div");
  handleCta.hidden = true;
  const handleOpen = button("set-primary", copy.handleOpen);
  handleCta.append(el(doc, "div", "set-note", copy.handleCta), handleOpen);
  const signOut = button("set-reset", copy.signOut);
  // The methods linked to the account: on the account page, which survives the mailbox (D1).
  const connected = button("linklike", copy.connected);
  inside.append(row, handleCta, connected, signOut);

  block.append(out, inside);

  const showError = (message: string): void => {
    error.textContent = message;
    error.hidden = false;
  };
  const context = (provider: Provider) => (provider === "apple" ? "signInApple" : "signInGoogle");

  const signedInOrOut = async (): Promise<void> => {
    await opts.onChange();
  };

  async function withProvider(provider: Provider): Promise<void> {
    error.hidden = true;
    const res = await controls.signInWith(provider);
    if (res?.ok) await signedInOrOut();
    else if (res?.handedOff) opts.onHandedOff?.();
    // A closed provider window is a cancel; no reply (a popup torn down) shows next open.
    else if (res && !res.cancelled) showError(errorCopy(context(provider), res.error ?? "unknown"));
  }

  google.addEventListener("click", () => void withProvider("google"));
  apple.addEventListener("click", () => void withProvider("apple"));
  signIn.addEventListener("click", async () => {
    const address = email.value.trim();
    if (!address || !password.value) return;
    error.hidden = true;
    signIn.disabled = true;
    const res = await controls.signInLocal(address, password.value);
    signIn.disabled = false;
    if (res?.ok) {
      password.value = "";
      await signedInOrOut();
    } else if (res?.error === "failedPrecondition") {
      // Unverified email: the account page's code step takes over, with the email only.
      await controls.rememberPendingEmail(address);
      opts.openPage("account.html#verify");
    } else showError(errorCopy("signInEmail", res?.error ?? "unavailable"));
  });
  forgot.addEventListener("click", () => opts.openPage("account.html#forgot"));
  signUp.addEventListener("click", () => opts.openPage("account.html#signup"));
  handleOpen.addEventListener("click", () => opts.openPage("account.html#handle"));
  connected.addEventListener("click", () => opts.openPage("account.html#connected"));
  signOut.addEventListener("click", async () => {
    await controls.signOut();
    await signedInOrOut();
  });

  async function refresh(): Promise<void> {
    const signedIn = (await controls.state())?.signedIn ?? false;
    out.hidden = signedIn;
    inside.hidden = !signedIn;
    if (signedIn) {
      error.hidden = true;
      const handle = await controls.handle();
      who.textContent = handle ? copy.handle(handle) : handle === null ? copy.handleMissing : copy.signedIn;
      handleCta.hidden = handle !== null;
      return;
    }
    const providers = await controls.providers();
    google.hidden = !providers.google;
    apple.hidden = !providers.apple;
    // No provider (Safari, Firefox for Android, or none configured): email is the only way in.
    local.open = !providers.google && !providers.apple;
    providerHint.hidden = providers.google && providers.apple;
    const failed = await controls.takeSignInError();
    if (failed) showError(errorCopy(context(failed.provider), failed.kind));
  }

  return { refresh };
}
