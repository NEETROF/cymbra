import {
  DEFAULT_INTERFACE_LANGUAGE,
  formatCount,
  formatDate,
  type InterfaceLanguage,
  NODE_SLOT,
  renderAround,
} from "../i18n/index.ts";
import type { Provider } from "../state/oidc.ts";
import { type AccountCopy, accountCopy } from "./copy.ts";
import { type AccountView, type AccountViewState, providerName } from "./flow.ts";
import { HANDLE_MAX_LENGTH, type HandleStatus } from "./handle.ts";
import type { LinkedIdentity } from "./messages.ts";

// DOM for the account page (add-lingua-account-parity). Pure rendering from the
// controller's state into a host element, wired to actions — unit-tested with jsdom like
// review/view.ts. Every reader-provided value (the email, the handle) goes through
// textContent/value, never innerHTML. Its copy is the catalogue's `account` module in the
// interface language (localise-lingua-account-onboarding D3), French when none is given.

export interface AccountActions {
  signInEmail(email: string, password: string): void;
  signUp(email: string, password: string): void;
  verify(code: string): void;
  resend(): void;
  requestReset(email: string): void;
  resetPassword(code: string, newPassword: string): void;
  signInWith(provider: Provider): void;
  signOut(): void;
  go(view: AccountView): void;
  editHandle(candidate: string): void;
  commitHandle(): void;
  abandonHandle(): void;
  askErase(): void;
  cancelErase(): void;
  eraseLinguaData(): void;
  // Comptes connectés (add-lingua-connected-accounts)
  openConnected(): void;
  leaveConnected(): void;
  retryIdentities(): void;
  link(provider: Provider): void;
  askRemove(identity: LinkedIdentity): void;
  cancelRemove(): void;
  remove(): void;
  showPasswordForm(): void;
  cancelPassword(): void;
  restartPassword(): void;
  setPassword(email: string, password: string): void;
  confirmPassword(code: string): void;
}

/** The handle field's help, by availability. */
function handleHelp(status: HandleStatus, c: AccountCopy, language: InterfaceLanguage): string {
  const max = formatCount(language, HANDLE_MAX_LENGTH);
  const help: Record<HandleStatus, string> = {
    empty: c.handleEmpty(max),
    invalid: c.handleInvalid(max),
    checking: c.handleChecking,
    available: c.handleAvailable,
    taken: c.handleTaken,
    error: c.handleUnchecked,
  };
  return help[status];
}

/** What every part of the page is rendered with: the interface language and its copy. */
interface Words {
  language: InterfaceLanguage;
  c: AccountCopy;
}

function h<K extends keyof HTMLElementTagNameMap>(tag: K, cls?: string, text?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  if (text != null) node.textContent = text;
  return node;
}

interface Field {
  wrap: HTMLLabelElement;
  input: HTMLInputElement;
}

function field(name: string, label: string, type: string, autocomplete: string, value = ""): Field {
  const wrap = h("label", "account-field");
  wrap.append(h("span", "account-label", label));
  const input = h("input", "account-input");
  input.name = name;
  input.type = type;
  input.autocomplete = autocomplete as AutoFill;
  input.value = value;
  input.required = true;
  wrap.append(input);
  return { wrap, input };
}

function form(fields: Field[], submitLabel: string, disabled: boolean, onSubmit: () => void): HTMLFormElement {
  const f = h("form", "account-form");
  f.noValidate = true;
  for (const x of fields) f.append(x.wrap);
  const submit = h("button", "account-primary", submitLabel);
  submit.type = "submit";
  submit.disabled = disabled;
  f.append(submit);
  f.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!disabled) onSubmit();
  });
  return f;
}

function links(items: [string, () => void][]): HTMLElement {
  const row = h("div", "account-links");
  for (const [label, onClick] of items) {
    const b = h("button", "account-link", label);
    b.type = "button";
    b.addEventListener("click", onClick);
    row.append(b);
  }
  return row;
}

function providerButtons(s: AccountViewState, a: AccountActions, { c }: Words): HTMLElement[] {
  const out: HTMLElement[] = [];
  const add = (provider: Provider, label: string): void => {
    const b = h("button", "account-provider", label);
    b.type = "button";
    b.dataset.provider = provider;
    b.disabled = s.busy;
    b.addEventListener("click", () => a.signInWith(provider));
    out.push(b);
  };
  if (s.providers.google) add("google", c.continueWithGoogle);
  if (s.providers.apple) add("apple", c.continueWithApple);
  if (out.length > 0) out.push(h("div", "account-sep", c.orWithEmail));
  return out;
}

function messages(s: AccountViewState): HTMLElement[] {
  const out: HTMLElement[] = [];
  if (s.notice) {
    const n = h("p", "account-notice", s.notice);
    n.setAttribute("role", "status");
    out.push(n);
  }
  if (s.error) {
    const e = h("p", "account-error", s.error);
    e.setAttribute("role", "alert");
    out.push(e);
  }
  return out;
}

function button(cls: string, label: string, disabled: boolean, onClick: () => void): HTMLButtonElement {
  const b = h("button", cls, label);
  b.type = "button";
  b.disabled = disabled;
  b.addEventListener("click", onClick);
  return b;
}

/**
 * « Tes données » (add-lingua-privacy-controls): erase Lingua only, after an explicit
 * confirmation, or leave for the site to delete the whole Cymbra account.
 */
function dataSection(s: AccountViewState, a: AccountActions, { c }: Words): HTMLElement {
  const section = h("section", "account-data");
  section.id = "data";
  section.append(h("h3", "account-subtitle", c.yourData));
  if (s.confirmingErase) {
    const warning = h("p", "account-warning", c.eraseWarning);
    warning.setAttribute("role", "alert");
    section.append(
      warning,
      button("account-danger", c.eraseYes, s.busy, () => a.eraseLinguaData()),
      button("account-secondary", c.cancel, s.busy, () => a.cancelErase()),
    );
  } else {
    section.append(
      h("p", "account-lead", c.eraseLead),
      button("account-secondary", c.erase, s.busy, () => a.askErase()),
    );
  }
  section.append(h("p", "account-footnote", c.deleteFootnote));
  const del = h("a", "account-link", c.deleteAccount);
  del.href = s.deleteAccountUrl;
  del.target = "_blank";
  del.rel = "noopener";
  section.append(del);
  return section;
}

/**
 * « Lié le 4 octobre 2026 », from Unix seconds: the day, the month and the year as the interface
 * language writes a date (`fr-FR` for the French, as before).
 */
export function linkedOn(linkedAt: number, language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE): string {
  const date = formatDate(language, new Date(linkedAt * 1000), { day: "numeric", month: "long", year: "numeric" });
  return accountCopy(language).linkedOn(date);
}

/** One linked method: its name, its address for email and password, when, and « Retirer ». */
function identityRow(s: AccountViewState, a: AccountActions, i: LinkedIdentity, only: boolean, w: Words): HTMLElement {
  const { c, language } = w;
  const row = h("li", "account-identity");
  row.dataset.provider = i.provider;
  const name = h("div", "account-identity-name", providerName(i.provider, language));
  row.append(name);
  if (i.provider === "local") row.append(h("div", "account-identity-detail", i.subject));
  row.append(h("div", "account-identity-detail", linkedOn(i.linkedAt, language)));
  const confirming = s.removing?.provider === i.provider && s.removing.subject === i.subject;
  if (confirming) {
    const ask = h("p", "account-warning", c.removeAsk(providerName(i.provider, language)));
    ask.setAttribute("role", "alert");
    row.append(
      ask,
      button("account-danger", c.remove, s.busy, () => a.remove()),
      button("account-secondary", c.cancel, s.busy, () => a.cancelRemove()),
    );
  } else if (only) {
    row.append(h("p", "account-footnote", c.onlyMethod));
  } else {
    row.append(button("account-secondary", c.remove, s.busy, () => a.askRemove(i)));
  }
  return row;
}

/** « Saisis le code envoyé à <b>email</b> »: the message rendered around the address, in bold. */
function codeSentTo(c: AccountCopy, email: string): HTMLElement {
  const to = h("p", "account-lead");
  renderAround(to, c.codeSentTo(NODE_SLOT), h("b", undefined, email));
  return to;
}

/** « Définir un mot de passe »: the offer, its form, or its code step (design D4). */
function passwordSection(s: AccountViewState, a: AccountActions, { c }: Words): HTMLElement {
  const section = h("section", "account-password");
  switch (s.passwordStep) {
    case "closed":
      section.append(button("account-secondary", c.setPassword, s.busy, () => a.showPasswordForm()));
      break;
    case "form": {
      section.append(h("h3", "account-subtitle", c.setPassword), h("p", "account-lead", c.setPasswordLead));
      const email = field("email", c.email, "email", "username", s.passwordEmail);
      const password = field("password", c.password, "password", "new-password");
      section.append(
        form([email, password], c.setPasswordSubmit, s.busy, () =>
          a.setPassword(email.input.value, password.input.value),
        ),
        links([[c.cancel, () => a.cancelPassword()]]),
      );
      break;
    }
    case "code": {
      section.append(h("h3", "account-subtitle", c.verifyEmail), codeSentTo(c, s.passwordEmail));
      const code = field("code", c.verificationCode, "text", "one-time-code");
      code.input.inputMode = "numeric";
      section.append(
        form([code], c.validate, s.busy, () => a.confirmPassword(code.input.value)),
        links([
          [c.restart, () => a.restartPassword()],
          [c.cancel, () => a.cancelPassword()],
        ]),
      );
      break;
    }
  }
  return section;
}

/** Comptes connectés (add-lingua-connected-accounts): the methods, and what can be added. */
function connectedCard(card: HTMLElement, s: AccountViewState, a: AccountActions, w: Words): void {
  const { c } = w;
  card.append(h("p", "account-lead", c.connectedLead));
  if (s.identities == null) {
    if (!s.busy) card.append(button("account-secondary", c.retry, false, () => a.retryIdentities()));
    card.append(links([[c.back, () => a.leaveConnected()]]));
    return;
  }
  const list = h("ul", "account-identities");
  for (const i of s.identities) list.append(identityRow(s, a, i, s.identities.length === 1, w));
  card.append(list);
  const has = (provider: string): boolean => s.identities?.some((i) => i.provider === provider) ?? false;
  const offers: [Provider, string][] = [];
  if (s.linkable.google && !has("google")) offers.push(["google", c.linkGoogle]);
  if (s.linkable.apple && !has("apple")) offers.push(["apple", c.linkApple]);
  for (const [provider, label] of offers) {
    const b = button("account-provider", label, s.busy, () => a.link(provider));
    b.dataset.provider = provider;
    card.append(b);
  }
  if (!has("local")) card.append(passwordSection(s, a, w));
  card.append(links([[c.back, () => a.leaveConnected()]]));
}

/** Render the page's card for `s`, in the interface language (French when none is given). */
export function renderAccount(
  root: HTMLElement,
  s: AccountViewState,
  a: AccountActions,
  language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE,
): void {
  const w: Words = { language, c: accountCopy(language) };
  const { c } = w;
  const card = h("section", "account-card");
  const title = (text: string): void => {
    card.append(h("h2", "account-title", text), ...messages(s));
  };

  switch (s.view) {
    case "signin": {
      title(c.signIn);
      card.append(...providerButtons(s, a, w));
      const email = field("email", c.email, "email", "username", s.email);
      const password = field("password", c.password, "password", "current-password");
      card.append(
        form([email, password], c.signIn, s.busy, () => a.signInEmail(email.input.value, password.input.value)),
        links([
          [c.forgotPassword, () => a.go("forgot")],
          [c.createAccount, () => a.go("signup")],
        ]),
      );
      break;
    }
    case "signup": {
      title(c.createAccountTitle);
      card.append(h("p", "account-lead", c.signUpLead));
      card.append(...providerButtons(s, a, w));
      const email = field("email", c.email, "email", "username", s.email);
      const password = field("password", c.password, "password", "new-password");
      card.append(
        form([email, password], c.createMyAccount, s.busy, () => a.signUp(email.input.value, password.input.value)),
      );
      card.append(
        s.errorKind === "alreadyExists"
          ? links([
              [c.signIn, () => a.go("signin")],
              [c.forgotPassword, () => a.go("forgot")],
            ])
          : links([[c.haveAccount, () => a.go("signin")]]),
      );
      break;
    }
    case "verify": {
      title(c.verifyEmail);
      card.append(codeSentTo(c, s.email));
      const code = field("code", c.verificationCode, "text", "one-time-code");
      code.input.inputMode = "numeric";
      card.append(
        form([code], c.verify, s.busy, () => a.verify(code.input.value)),
        links([
          [c.resendCode, () => a.resend()],
          [c.useAnotherEmail, () => a.go("signup")],
        ]),
      );
      break;
    }
    case "forgot": {
      title(c.forgotTitle);
      card.append(h("p", "account-lead", c.forgotLead));
      const email = field("email", c.email, "email", "username", s.email);
      card.append(
        form([email], c.sendCode, s.busy, () => a.requestReset(email.input.value)),
        links([[c.backToSignIn, () => a.go("signin")]]),
      );
      break;
    }
    case "reset": {
      title(c.newPasswordTitle);
      const code = field("code", c.codeByEmail, "text", "one-time-code");
      code.input.inputMode = "numeric";
      const password = field("password", c.newPassword, "password", "new-password");
      card.append(
        form([code, password], c.changePassword, s.busy, () => a.resetPassword(code.input.value, password.input.value)),
        links([
          [c.resendACode, () => a.requestReset(s.email)],
          [c.backToSignIn, () => a.go("signin")],
        ]),
      );
      break;
    }
    case "handle": {
      title(c.chooseHandle);
      card.append(h("p", "account-lead", c.handleLead));
      const handle = field("handle", c.handle, "text", "nickname", s.candidate);
      handle.input.maxLength = HANDLE_MAX_LENGTH;
      handle.input.spellcheck = false;
      handle.input.setAttribute("autocapitalize", "off");
      handle.input.addEventListener("input", () => a.editHandle(handle.input.value));
      const help = h("span", `account-help account-help-${s.handleStatus}`, handleHelp(s.handleStatus, c, language));
      help.setAttribute("aria-live", "polite");
      handle.wrap.append(help);
      const blocked =
        s.busy || s.handleStatus === "empty" || s.handleStatus === "invalid" || s.handleStatus === "taken";
      card.append(
        form([handle], c.continue, blocked, () => a.commitHandle()),
        h("p", "account-footnote", c.handleFootnote),
        links([[c.useAnotherAccount, () => a.abandonHandle()]]),
      );
      break;
    }
    case "signedin": {
      title(c.signedInTitle);
      if (s.handle) card.append(h("p", "account-handle", c.handleAt(s.handle)));
      card.append(h("p", "account-lead", c.signedInLead));
      const out = h("button", "account-secondary", c.signOut);
      out.type = "button";
      out.disabled = s.busy;
      out.addEventListener("click", () => a.signOut());
      card.append(
        button("account-secondary", c.connectedAccounts, s.busy, () => a.openConnected()),
        out,
        dataSection(s, a, w),
      );
      break;
    }
    case "connected": {
      title(c.connectedAccounts);
      connectedCard(card, s, a, w);
      break;
    }
  }

  // Re-rendering replaces the inputs: keep the reader's focus and caret where they were, so
  // live feedback (the handle's availability) never interrupts typing.
  const active = document.activeElement;
  const focused = active instanceof HTMLInputElement && root.contains(active) ? active : null;
  const name = focused?.name ?? null;
  const caret = focused?.selectionStart ?? null;
  root.replaceChildren(card);
  if (name) {
    const again = root.querySelector<HTMLInputElement>(`input[name="${name}"]`);
    again?.focus();
    if (again && caret != null) {
      try {
        again.setSelectionRange(caret, caret);
      } catch {
        // email/number inputs do not support a selection range
      }
    }
  }
}
