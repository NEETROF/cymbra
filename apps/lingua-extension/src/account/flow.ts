import { DEFAULT_INTERFACE_LANGUAGE, type InterfaceLanguage } from "../i18n/index.ts";
import type { AuthErrorKind } from "../state/auth-errors.ts";
import type { Provider, Providers } from "../state/oidc.ts";
import { type AccountCopy, accountCopy, errorCopy, type FlowContext } from "./copy.ts";
import { type HandleStatus, isValidHandle, localHandleStatus } from "./handle.ts";
import { deleteAccountUrl } from "./locale.ts";
import type { AccountMessage, AccountReply, LinkedIdentity } from "./messages.ts";

export { deleteAccountUrl } from "./locale.ts";

// The account page's controller (add-lingua-account-parity): sign-in, sign-up →
// verification code → automatic sign-in, password reset, and the handle step every
// handle-less account goes through after signing in (design D9). It holds the view state
// and, for the verification step only, the password in MEMORY (design D6) — the one thing
// persisted is the pending email (chrome.storage.session), so a reloaded page resumes on
// the code step. All calls go to the background; the DOM is rendered by view.ts. Its notices and
// errors are worded in the interface language it is given (localise-lingua-account-onboarding D3).

export type AccountView = "signin" | "signup" | "verify" | "forgot" | "reset" | "handle" | "signedin" | "connected";

/** « Définir un mot de passe » on the connected accounts: closed, its form, or its code step. */
export type PasswordStep = "closed" | "form" | "code";

export interface AccountViewState {
  view: AccountView;
  email: string;
  busy: boolean;
  /** Reader-facing copy of the last failure, or null. */
  error: string | null;
  /** The failure's category (the view adds "sign in" / "forgot" links on alreadyExists). */
  errorKind: AuthErrorKind | null;
  notice: string | null;
  providers: Providers;
  /** The signed-in account's handle, once known. */
  handle: string | null;
  /** The handle being typed on the handle step, and its live status. */
  candidate: string;
  handleStatus: HandleStatus;
  /** The erasure is waiting for the reader's explicit confirmation. */
  confirmingErase: boolean;
  /** Where the whole Cymbra account is deleted (the site), in the reader's language. */
  deleteAccountUrl: string;
  /** Comptes connectés: the linked methods, null until read (or when they could not be). */
  identities: LinkedIdentity[] | null;
  /** The providers this browser can link (D3). */
  linkable: Providers;
  /** The method whose removal waits for the reader's confirmation. */
  removing: LinkedIdentity | null;
  passwordStep: PasswordStep;
  /** The address « Définir un mot de passe » sent its code to. */
  passwordEmail: string;
}

export interface PendingEmailStore {
  get(): Promise<string | null>;
  set(email: string | null): Promise<void>;
}

export interface AccountFlowDeps {
  send: (message: AccountMessage) => Promise<AccountReply | null>;
  pending: PendingEmailStore;
  /** The address of a set-password waiting for its code (D4) — its own key, never the password. */
  pendingPassword: PendingEmailStore;
  /**
   * The locale sent on the requests that carry one (`AccountLanguage.locale`, D2): the browser's
   * whole tag until the reader has chosen their language, the account locale once they have, so the
   * account's e-mails come in the language they chose.
   */
  locale: string;
  /**
   * Whether resending the code and requesting a reset carry no locale (D2): Cymbra ID records the
   * locale those two requests carry over the account's own, and keeps the account's — writing the
   * e-mail in it — when they carry none. True until the reader has chosen their language
   * (`AccountLanguage.keepAccountLocale`), so that this device never writes over a language the
   * account was given elsewhere; false — `locale` sent, as it always was — when not given.
   */
  keepAccountLocale?: boolean;
  /**
   * What the deletion page is chosen by (`AccountLanguage.deletion`, D2): the browser's whole tag
   * until the reader has chosen their language — `locale`, as it always was, when not given — and the
   * interface language once they have, never the account locale, which may be the browser's `it`.
   */
  deletionLanguage?: string;
  /** The interface language: the notices and the errors; French when not given. */
  language?: InterfaceLanguage;
  /** Drop the background's persisted provider failure once it was shown live. */
  clearPersistedError: () => Promise<void>;
}

const NAVIGABLE: readonly AccountView[] = ["signin", "signup", "verify", "forgot", "reset"];

/** Whether a `#hash` asks for the connected accounts (signed in only). */
export function wantsConnected(hash: string): boolean {
  return hash.replace(/^#/, "").split("?")[0] === "connected";
}

/** The view a `#hash` asks for (`#signup`, `#forgot`, `#verify`, …); sign-in otherwise. */
export function viewFromHash(hash: string): AccountView {
  const name = hash.replace(/^#/, "").split("?")[0] as AccountView;
  return NAVIGABLE.includes(name) ? name : "signin";
}

/** How the page names a sign-in method, in the interface language. */
export function providerName(provider: string, language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE): string {
  const c = accountCopy(language);
  if (provider === "google") return c.providerGoogle;
  if (provider === "apple") return c.providerApple;
  if (provider === "local") return c.providerLocal;
  return provider;
}

export class AccountFlow {
  /** The password typed at sign-up / sign-in, held only until the code is verified. */
  private password: string | null = null;
  /** Bumped on every handle edit, so a slower availability answer never overwrites a newer one. */
  private checkSeq = 0;
  /** The interface language, and the copy the notices and the errors are worded in. */
  private readonly language: InterfaceLanguage;
  private readonly copy: AccountCopy;
  private s: AccountViewState = {
    view: "signin",
    email: "",
    busy: false,
    error: null,
    errorKind: null,
    notice: null,
    providers: { google: false, apple: false },
    handle: null,
    candidate: "",
    handleStatus: "empty",
    confirmingErase: false,
    deleteAccountUrl: "",
    identities: null,
    linkable: { google: false, apple: false },
    removing: null,
    passwordStep: "closed",
    passwordEmail: "",
  };

  constructor(
    private readonly deps: AccountFlowDeps,
    private readonly onChange: (state: AccountViewState) => void = () => {},
  ) {
    this.language = deps.language ?? DEFAULT_INTERFACE_LANGUAGE;
    this.copy = accountCopy(this.language);
    // A page the reader reads: by the language they chose, or by the browser's until then (D2).
    this.s.deleteAccountUrl = deleteAccountUrl(deps.deletionLanguage ?? deps.locale);
  }

  view(): AccountViewState {
    return {
      ...this.s,
      providers: { ...this.s.providers },
      linkable: { ...this.s.linkable },
      identities: this.s.identities?.map((i) => ({ ...i })) ?? null,
    };
  }

  async init(hash: string): Promise<AccountViewState> {
    const [state, providers, pending] = await Promise.all([
      this.deps.send({ type: "account:state" }),
      this.deps.send({ type: "account:providers" }),
      this.deps.pending.get(),
    ]);
    this.s.providers = providers?.providers ?? { google: false, apple: false };
    if (state?.state?.signedIn) {
      const landed = await this.resolveProfile(null);
      return landed.view === "signedin" && wantsConnected(hash) ? this.openConnected() : landed;
    }
    if (pending) this.s.email = pending;
    const wanted = viewFromHash(hash);
    // The code step needs an email to verify; without a pending one, start at sign-in.
    return this.set({ view: wanted === "verify" && !pending ? "signin" : wanted });
  }

  /** Follow a link. Clears the last error/notice. A signed-in page cannot be navigated away. */
  go(view: AccountView): AccountViewState {
    if (this.s.view === "signedin" || this.s.view === "handle") return this.view();
    if (view === "signedin" || view === "handle" || view === this.s.view) return this.view();
    const target = view === "verify" && !this.s.email ? "signin" : view;
    return this.set({ view: target, error: null, errorKind: null, notice: null });
  }

  async signInEmail(email: string, password: string): Promise<AccountViewState> {
    email = email.trim();
    if (!email || !password) return this.view();
    this.s.email = email;
    const reply = await this.run("signInEmail", { type: "account:signInLocal", email, password });
    if (reply?.ok) {
      this.password = null;
      return this.resolveProfile(null);
    }
    if (reply?.error === "failedPrecondition") {
      // Unverified email: not a credential error — go verify it, keeping the password in
      // memory so a valid code signs straight in.
      this.password = password;
      await this.deps.pending.set(email);
      return this.set({
        view: "verify",
        error: null,
        errorKind: null,
        notice: this.copy.verifyFirst,
      });
    }
    return this.view();
  }

  async signUp(email: string, password: string): Promise<AccountViewState> {
    email = email.trim();
    if (!email || !password) return this.view();
    this.s.email = email;
    const reply = await this.run("signUp", { type: "account:signUp", email, password, locale: this.deps.locale });
    if (!reply?.ok) return this.view();
    this.password = password;
    await this.deps.pending.set(email);
    return this.set({ view: "verify", notice: this.copy.codeSent(email) });
  }

  async verify(code: string): Promise<AccountViewState> {
    code = code.trim();
    if (!code) return this.view();
    const reply = await this.run("verify", { type: "account:verifyEmail", code });
    if (!reply?.ok) return this.view();
    await this.deps.pending.set(null);
    const password = this.password;
    this.password = null;
    if (password == null) {
      // Reloaded since sign-up: the password is gone, so the reader signs in once more.
      return this.set({ view: "signin", notice: this.copy.verifiedSignIn });
    }
    const signIn = await this.run("signInEmail", { type: "account:signInLocal", email: this.s.email, password });
    if (signIn?.ok) return this.resolveProfile(this.copy.verifiedSignedIn);
    return this.set({ view: "signin", notice: this.copy.verified });
  }

  /**
   * The locale of a request Cymbra ID records over the account's own — resending the code,
   * requesting a reset: none while the account's is to be kept (D2). Sign-up records it on an account
   * that has none yet, and setting a password records nothing: both carry `locale`.
   */
  private overwritingLocale(): string {
    return this.deps.keepAccountLocale ? "" : this.deps.locale;
  }

  async resend(): Promise<AccountViewState> {
    if (!this.s.email) return this.view();
    const reply = await this.run("resend", {
      type: "account:resendVerification",
      email: this.s.email,
      locale: this.overwritingLocale(),
    });
    return reply?.ok ? this.set({ notice: this.copy.newCodeSent }) : this.view();
  }

  async requestReset(email: string): Promise<AccountViewState> {
    email = email.trim();
    if (!email) return this.view();
    this.s.email = email;
    const reply = await this.run("forgot", {
      type: "account:requestPasswordReset",
      email,
      locale: this.overwritingLocale(),
    });
    if (!reply?.ok) return this.view();
    // Identical whether or not the account exists (no enumeration).
    return this.set({ view: "reset", notice: this.copy.resetCodeSent(email) });
  }

  async resetPassword(code: string, newPassword: string): Promise<AccountViewState> {
    code = code.trim();
    if (!code || !newPassword) return this.view();
    const reply = await this.run("reset", { type: "account:resetPassword", code, newPassword });
    if (!reply?.ok) return this.view();
    return this.set({ view: "signin", notice: this.copy.passwordChanged });
  }

  async signInWith(provider: Provider): Promise<AccountViewState> {
    const context: FlowContext = provider === "apple" ? "signInApple" : "signInGoogle";
    const type = provider === "apple" ? "account:signInApple" : "account:signInGoogle";
    const reply = await this.run(context, { type });
    if (reply?.ok) return this.resolveProfile(null);
    // Safari: the host app shows the provider's sheet; the page collects the token on return.
    if (reply?.handedOff) return this.set({ notice: this.copy.finishInApp });
    // Shown live here, so the background's persisted copy must not re-show in the popup.
    if (reply && !reply.cancelled) await this.deps.clearPersistedError();
    return this.view();
  }

  /**
   * Collect an id_token the host app handed back (Safari), when the page opens and each time
   * the reader comes back to it. With nothing pending the page stays as it is.
   */
  async collectHandedToken(): Promise<AccountViewState> {
    const reply = await this.deps.send({ type: "account:collectHandedToken" });
    if (!reply?.provider) return this.view();
    if (reply.ok) return this.resolveProfile(null);
    // Shown live here, so the background's persisted copy must not re-show in the popup.
    await this.deps.clearPersistedError();
    this.fail(reply.provider === "apple" ? "signInApple" : "signInGoogle", reply.error ?? "unknown");
    return this.view();
  }

  /**
   * Comptes connectés (add-lingua-connected-accounts): the linked methods, read fresh. A
   * set-password waiting for its code (a reload, the mailbox) reopens on the code step.
   */
  async openConnected(): Promise<AccountViewState> {
    if (this.s.view !== "signedin" && this.s.view !== "connected") return this.view();
    const waiting = await this.deps.pendingPassword.get();
    this.set({
      view: "connected",
      removing: null,
      notice: null,
      error: null,
      errorKind: null,
      ...(waiting ? { passwordStep: "code" as const, passwordEmail: waiting } : {}),
    });
    return this.loadIdentities();
  }

  /** Back to the signed-in view; a set-password waiting for its code stays remembered. */
  leaveConnected(): AccountViewState {
    if (this.s.view !== "connected") return this.view();
    return this.set({ view: "signedin", removing: null, notice: null, error: null, errorKind: null });
  }

  async loadIdentities(): Promise<AccountViewState> {
    const reply = await this.run("connected", { type: "account:identities" });
    if (!reply?.ok) return this.set({ identities: null });
    return this.set({
      identities: reply.identities ?? [],
      linkable: reply.linkable ?? { google: false, apple: false },
    });
  }

  async link(provider: Provider): Promise<AccountViewState> {
    const reply = await this.run(provider === "apple" ? "linkApple" : "linkGoogle", {
      type: "account:linkProvider",
      provider,
    });
    if (!reply?.ok) return this.view(); // a cancel says nothing; a failure is already shown
    await this.loadIdentities();
    return this.set({ notice: this.copy.linked(providerName(provider, this.language)) });
  }

  askRemove(identity: LinkedIdentity): AccountViewState {
    return this.set({ removing: { ...identity }, notice: null, error: null, errorKind: null });
  }

  cancelRemove(): AccountViewState {
    return this.set({ removing: null });
  }

  async remove(): Promise<AccountViewState> {
    const target = this.s.removing;
    if (!target) return this.view();
    const reply = await this.run("unlink", {
      type: "account:unlink",
      provider: target.provider,
      subject: target.subject,
    });
    this.s.removing = null;
    if (!reply?.ok) return this.set({});
    await this.loadIdentities();
    return this.set({ notice: this.copy.methodRemoved(providerName(target.provider, this.language)) });
  }

  showPasswordForm(): AccountViewState {
    return this.set({ passwordStep: "form", notice: null, error: null, errorKind: null });
  }

  /** Close the form or the code step; a code already sent simply expires. */
  async cancelPassword(): Promise<AccountViewState> {
    await this.deps.pendingPassword.set(null);
    return this.set({ passwordStep: "closed", error: null, errorKind: null });
  }

  /** « Recommencer »: a parked set-password cannot be resent (D4), so it is submitted again. */
  async restartPassword(): Promise<AccountViewState> {
    await this.deps.pendingPassword.set(null);
    return this.set({ passwordStep: "form", notice: null, error: null, errorKind: null });
  }

  /** Submit the address and the password: the server emails a code and binds nothing yet. */
  async setPassword(email: string, password: string): Promise<AccountViewState> {
    email = email.trim();
    if (!email || !password) return this.view();
    this.s.passwordEmail = email;
    const reply = await this.run("setPassword", {
      type: "account:setPassword",
      email,
      password,
      locale: this.deps.locale,
    });
    if (!reply?.ok) return this.view();
    await this.deps.pendingPassword.set(email);
    return this.set({ passwordStep: "code", notice: this.copy.codeSent(email) });
  }

  /** The emailed code binds the password; the reader stays signed in. */
  async confirmPassword(code: string): Promise<AccountViewState> {
    code = code.trim();
    if (!code) return this.view();
    const reply = await this.run("verifyPassword", { type: "account:verifyEmail", code });
    if (!reply?.ok) return this.view();
    await this.deps.pendingPassword.set(null);
    const email = this.s.passwordEmail;
    this.set({ passwordStep: "closed" });
    await this.loadIdentities();
    return this.set({ notice: this.copy.passwordSet(email) });
  }

  /** « Effacer mes données Lingua » asks for an explicit confirmation first. */
  askErase(): AccountViewState {
    return this.set({ confirmingErase: true, error: null, errorKind: null, notice: null });
  }

  cancelErase(): AccountViewState {
    return this.set({ confirmingErase: false });
  }

  /** Erase the reader's Lingua data (server and this device); the account stays signed in. */
  async eraseLinguaData(): Promise<AccountViewState> {
    if (!this.s.confirmingErase) return this.view();
    const reply = await this.run("eraseData", { type: "account:eraseLinguaData" });
    if (reply?.ok) return this.set({ confirmingErase: false, notice: this.copy.erased });
    return this.set({ confirmingErase: false });
  }

  /** The reader typed on the handle step: local status now, the server check comes later. */
  editHandle(candidate: string): AccountViewState {
    this.checkSeq++; // any availability answer still in flight is now stale
    return this.set({ candidate, handleStatus: localHandleStatus(candidate), error: null, errorKind: null });
  }

  /** Ask the server whether the current candidate is free (the page debounces the call). */
  async checkHandle(): Promise<AccountViewState> {
    if (this.s.handleStatus !== "checking") return this.view();
    const seq = this.checkSeq;
    const reply = await this.deps.send({ type: "account:checkHandle", handle: this.s.candidate.trim() });
    if (seq !== this.checkSeq) return this.view(); // superseded by newer typing
    if (reply?.ok) return this.set({ handleStatus: reply.available ? "available" : "taken" });
    return this.set({ handleStatus: "error" });
  }

  /** Save the candidate as the account's handle; the server is the authority on uniqueness. */
  async commitHandle(): Promise<AccountViewState> {
    const handle = this.s.candidate.trim();
    if (!isValidHandle(handle)) return this.set({ handleStatus: handle ? "invalid" : "empty" });
    this.checkSeq++;
    const reply = await this.run("handle", { type: "account:setHandle", handle });
    if (reply?.ok) {
      const saved = reply.handle ?? handle;
      return this.set({
        view: "signedin",
        handle: saved,
        candidate: "",
        handleStatus: "empty",
        notice: this.copy.handleSaved(saved),
      });
    }
    if (reply?.error === "alreadyExists" || reply?.error === "conflict") return this.set({ handleStatus: "taken" });
    return this.view();
  }

  /** Leave the handle step: a handle-less account is deleted (Music's rule), then signed out. */
  async abandonHandle(): Promise<AccountViewState> {
    await this.run("handle", { type: "account:abandon" });
    return this.set({
      view: "signin",
      handle: null,
      candidate: "",
      handleStatus: "empty",
      error: null,
      errorKind: null,
      notice: this.copy.signedOut,
    });
  }

  async signOut(): Promise<AccountViewState> {
    await this.run("signInEmail", { type: "account:signOut" });
    await this.deps.pendingPassword.set(null);
    return this.set({
      view: "signin",
      handle: null,
      error: null,
      errorKind: null,
      notice: null,
      confirmingErase: false,
      identities: null,
      removing: null,
      passwordStep: "closed",
      passwordEmail: "",
    });
  }

  /**
   * After a sign-in, or when the page opens signed in: an account without a handle must pick
   * one — the backend's orphan reaper deletes handle-less accounts — so it lands on the
   * handle step. A profile that cannot be read keeps the session; the popup asks again later.
   */
  private async resolveProfile(notice: string | null): Promise<AccountViewState> {
    const reply = await this.run("handle", { type: "account:profile" });
    if (reply?.ok && reply.handle == null) {
      this.checkSeq++;
      return this.set({ view: "handle", handle: null, candidate: "", handleStatus: "empty", notice });
    }
    return this.set({ view: "signedin", handle: reply?.ok ? (reply.handle ?? null) : null, notice });
  }

  /** Send a message with busy/error bookkeeping. A null reply is an unreachable worker. */
  private async run(context: FlowContext, message: AccountMessage): Promise<AccountReply | null> {
    this.set({ busy: true, error: null, errorKind: null });
    const reply = await this.deps.send(message);
    this.s.busy = false;
    if (reply == null) this.fail(context, "unavailable");
    else if (!reply.ok && !reply.cancelled && !reply.handedOff) this.fail(context, reply.error ?? "unknown");
    else this.set({});
    return reply;
  }

  private fail(context: FlowContext, kind: AuthErrorKind): void {
    this.set({ error: errorCopy(context, kind, this.language), errorKind: kind, notice: null });
  }

  private set(patch: Partial<AccountViewState>): AccountViewState {
    this.s = { ...this.s, ...patch };
    const view = this.view();
    this.onChange(view);
    return view;
  }
}
