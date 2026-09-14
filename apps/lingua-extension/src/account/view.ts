import type { Provider } from "../state/oidc.ts";
import type { AccountView, AccountViewState } from "./flow.ts";

// DOM for the account page (add-lingua-account-parity). Pure rendering from the
// controller's state into a host element, wired to actions — unit-tested with jsdom like
// review/view.ts. Every reader-provided value (the email) goes through textContent/value,
// never innerHTML.

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

function form(fields: Field[], submitLabel: string, busy: boolean, onSubmit: () => void): HTMLFormElement {
  const f = h("form", "account-form");
  f.noValidate = true;
  for (const x of fields) f.append(x.wrap);
  const submit = h("button", "account-primary", submitLabel);
  submit.type = "submit";
  submit.disabled = busy;
  f.append(submit);
  f.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!busy) onSubmit();
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

function providerButtons(s: AccountViewState, a: AccountActions): HTMLElement[] {
  const out: HTMLElement[] = [];
  const add = (provider: Provider, label: string): void => {
    const b = h("button", "account-provider", label);
    b.type = "button";
    b.dataset.provider = provider;
    b.disabled = s.busy;
    b.addEventListener("click", () => a.signInWith(provider));
    out.push(b);
  };
  if (s.providers.google) add("google", "Continuer avec Google");
  if (s.providers.apple) add("apple", "Continuer avec Apple");
  if (out.length > 0) out.push(h("div", "account-sep", "ou avec ton email"));
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

export function renderAccount(root: HTMLElement, s: AccountViewState, a: AccountActions): void {
  const card = h("section", "account-card");
  const title = (text: string): void => {
    card.append(h("h2", "account-title", text), ...messages(s));
  };

  switch (s.view) {
    case "signin": {
      title("Se connecter");
      card.append(...providerButtons(s, a));
      const email = field("email", "Email", "email", "username", s.email);
      const password = field("password", "Mot de passe", "password", "current-password");
      card.append(
        form([email, password], "Se connecter", s.busy, () => a.signInEmail(email.input.value, password.input.value)),
        links([
          ["Mot de passe oublié ?", () => a.go("forgot")],
          ["Créer un compte", () => a.go("signup")],
        ]),
      );
      break;
    }
    case "signup": {
      title("Créer un compte Cymbra");
      card.append(h("p", "account-lead", "Retrouve tes mots, ton deck et tes statistiques sur tous tes appareils."));
      card.append(...providerButtons(s, a));
      const email = field("email", "Email", "email", "username", s.email);
      const password = field("password", "Mot de passe", "password", "new-password");
      card.append(
        form([email, password], "Créer mon compte", s.busy, () => a.signUp(email.input.value, password.input.value)),
      );
      card.append(
        s.errorKind === "alreadyExists"
          ? links([
              ["Se connecter", () => a.go("signin")],
              ["Mot de passe oublié ?", () => a.go("forgot")],
            ])
          : links([["J'ai déjà un compte", () => a.go("signin")]]),
      );
      break;
    }
    case "verify": {
      title("Vérifie ton adresse email");
      const to = h("p", "account-lead", "Saisis le code envoyé à ");
      to.append(h("b", undefined, s.email));
      card.append(to);
      const code = field("code", "Code de vérification", "text", "one-time-code");
      code.input.inputMode = "numeric";
      card.append(
        form([code], "Vérifier", s.busy, () => a.verify(code.input.value)),
        links([
          ["Renvoyer le code", () => a.resend()],
          ["Utiliser un autre email", () => a.go("signup")],
        ]),
      );
      break;
    }
    case "forgot": {
      title("Mot de passe oublié");
      card.append(
        h("p", "account-lead", "Indique ton email : on t'envoie un code pour choisir un nouveau mot de passe."),
      );
      const email = field("email", "Email", "email", "username", s.email);
      card.append(
        form([email], "Envoyer un code", s.busy, () => a.requestReset(email.input.value)),
        links([["Retour à la connexion", () => a.go("signin")]]),
      );
      break;
    }
    case "reset": {
      title("Nouveau mot de passe");
      const code = field("code", "Code reçu par email", "text", "one-time-code");
      code.input.inputMode = "numeric";
      const password = field("password", "Nouveau mot de passe", "password", "new-password");
      card.append(
        form([code, password], "Modifier le mot de passe", s.busy, () =>
          a.resetPassword(code.input.value, password.input.value),
        ),
        links([
          ["Renvoyer un code", () => a.requestReset(s.email)],
          ["Retour à la connexion", () => a.go("signin")],
        ]),
      );
      break;
    }
    case "signedin": {
      title("Tu es connecté");
      card.append(
        h(
          "p",
          "account-lead",
          "Tes mots, ton deck et tes statistiques se synchronisent entre tes appareils. Tu peux fermer cet onglet.",
        ),
      );
      const out = h("button", "account-secondary", "Se déconnecter");
      out.type = "button";
      out.disabled = s.busy;
      out.addEventListener("click", () => a.signOut());
      card.append(out);
      break;
    }
  }
  root.replaceChildren(card);
}
