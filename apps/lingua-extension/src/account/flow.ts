import type { AuthErrorKind } from "../state/auth-errors.ts";
import type { Provider, Providers } from "../state/oidc.ts";
import { errorCopy, type FlowContext } from "./copy.ts";
import { type HandleStatus, isValidHandle, localHandleStatus } from "./handle.ts";
import type { AccountMessage, AccountReply } from "./messages.ts";

// The account page's controller (add-lingua-account-parity): sign-in, sign-up →
// verification code → automatic sign-in, password reset, and the handle step every
// handle-less account goes through after signing in (design D9). It holds the view state
// and, for the verification step only, the password in MEMORY (design D6) — the one thing
// persisted is the pending email (chrome.storage.session), so a reloaded page resumes on
// the code step. All calls go to the background; the DOM is rendered by view.ts.

export type AccountView = "signin" | "signup" | "verify" | "forgot" | "reset" | "handle" | "signedin";

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
}

export interface PendingEmailStore {
  get(): Promise<string | null>;
  set(email: string | null): Promise<void>;
}

export interface AccountFlowDeps {
  send: (message: AccountMessage) => Promise<AccountReply | null>;
  pending: PendingEmailStore;
  /** The browser UI language, so verification/reset emails match the reader. */
  locale: string;
  /** Drop the background's persisted provider failure once it was shown live. */
  clearPersistedError: () => Promise<void>;
}

const NAVIGABLE: readonly AccountView[] = ["signin", "signup", "verify", "forgot", "reset"];

/** The view a `#hash` asks for (`#signup`, `#forgot`, `#verify`, …); sign-in otherwise. */
export function viewFromHash(hash: string): AccountView {
  const name = hash.replace(/^#/, "").split("?")[0] as AccountView;
  return NAVIGABLE.includes(name) ? name : "signin";
}

export class AccountFlow {
  /** The password typed at sign-up / sign-in, held only until the code is verified. */
  private password: string | null = null;
  /** Bumped on every handle edit, so a slower availability answer never overwrites a newer one. */
  private checkSeq = 0;
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
  };

  constructor(
    private readonly deps: AccountFlowDeps,
    private readonly onChange: (state: AccountViewState) => void = () => {},
  ) {}

  view(): AccountViewState {
    return { ...this.s, providers: { ...this.s.providers } };
  }

  async init(hash: string): Promise<AccountViewState> {
    const [state, providers, pending] = await Promise.all([
      this.deps.send({ type: "account:state" }),
      this.deps.send({ type: "account:providers" }),
      this.deps.pending.get(),
    ]);
    this.s.providers = providers?.providers ?? { google: false, apple: false };
    if (state?.state?.signedIn) return this.resolveProfile(null);
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
        notice: "Ton adresse email n'est pas encore vérifiée. Saisis le code reçu par email.",
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
    return this.set({ view: "verify", notice: `Un code de vérification a été envoyé à ${email}.` });
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
      return this.set({ view: "signin", notice: "Adresse vérifiée. Connecte-toi." });
    }
    const signIn = await this.run("signInEmail", { type: "account:signInLocal", email: this.s.email, password });
    if (signIn?.ok) return this.resolveProfile("Adresse vérifiée, tu es connecté.");
    return this.set({ view: "signin", notice: "Adresse vérifiée." });
  }

  async resend(): Promise<AccountViewState> {
    if (!this.s.email) return this.view();
    const reply = await this.run("resend", {
      type: "account:resendVerification",
      email: this.s.email,
      locale: this.deps.locale,
    });
    return reply?.ok ? this.set({ notice: "Un nouveau code a été envoyé." }) : this.view();
  }

  async requestReset(email: string): Promise<AccountViewState> {
    email = email.trim();
    if (!email) return this.view();
    this.s.email = email;
    const reply = await this.run("forgot", {
      type: "account:requestPasswordReset",
      email,
      locale: this.deps.locale,
    });
    if (!reply?.ok) return this.view();
    // Identical whether or not the account exists (no enumeration).
    return this.set({
      view: "reset",
      notice: `Si un compte existe pour ${email}, un code vient d'y être envoyé.`,
    });
  }

  async resetPassword(code: string, newPassword: string): Promise<AccountViewState> {
    code = code.trim();
    if (!code || !newPassword) return this.view();
    const reply = await this.run("reset", { type: "account:resetPassword", code, newPassword });
    if (!reply?.ok) return this.view();
    return this.set({ view: "signin", notice: "Mot de passe modifié. Connecte-toi avec le nouveau." });
  }

  async signInWith(provider: Provider): Promise<AccountViewState> {
    const context: FlowContext = provider === "apple" ? "signInApple" : "signInGoogle";
    const type = provider === "apple" ? "account:signInApple" : "account:signInGoogle";
    const reply = await this.run(context, { type });
    if (reply?.ok) return this.resolveProfile(null);
    // Shown live here, so the background's persisted copy must not re-show in the popup.
    if (reply && !reply.cancelled) await this.deps.clearPersistedError();
    return this.view();
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
        notice: `C'est noté : ton pseudo est @${saved}.`,
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
      notice: "Tu es déconnecté. Connecte-toi avec un autre compte ou crée-en un.",
    });
  }

  async signOut(): Promise<AccountViewState> {
    await this.run("signInEmail", { type: "account:signOut" });
    return this.set({ view: "signin", handle: null, error: null, errorKind: null, notice: null });
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
    else if (!reply.ok && !reply.cancelled) this.fail(context, reply.error ?? "unknown");
    else this.set({});
    return reply;
  }

  private fail(context: FlowContext, kind: AuthErrorKind): void {
    this.set({ error: errorCopy(context, kind), errorKind: kind, notice: null });
  }

  private set(patch: Partial<AccountViewState>): AccountViewState {
    this.s = { ...this.s, ...patch };
    const view = this.view();
    this.onChange(view);
    return view;
  }
}
