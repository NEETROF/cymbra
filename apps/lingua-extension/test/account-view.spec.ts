import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AccountViewState } from "@/account/flow.ts";
import { type AccountActions, renderAccount } from "@/account/view.ts";

let root: HTMLElement;
let actions: { [K in keyof AccountActions]: ReturnType<typeof vi.fn> };

beforeEach(() => {
  document.body.innerHTML = "<div id='r'></div>";
  root = document.getElementById("r")!;
  actions = {
    signInEmail: vi.fn(),
    signUp: vi.fn(),
    verify: vi.fn(),
    resend: vi.fn(),
    requestReset: vi.fn(),
    resetPassword: vi.fn(),
    signInWith: vi.fn(),
    signOut: vi.fn(),
    go: vi.fn(),
    editHandle: vi.fn(),
    commitHandle: vi.fn(),
    abandonHandle: vi.fn(),
    askErase: vi.fn(),
    cancelErase: vi.fn(),
    eraseLinguaData: vi.fn(),
  };
});

const state = (over: Partial<AccountViewState> = {}): AccountViewState => ({
  view: "signin",
  email: "",
  busy: false,
  error: null,
  errorKind: null,
  notice: null,
  providers: { google: true, apple: true },
  handle: null,
  candidate: "",
  handleStatus: "empty",
  confirmingErase: false,
  deleteAccountUrl: "https://cymbra.app/suppression-compte/",
  ...over,
});

function render(over: Partial<AccountViewState> = {}): void {
  renderAccount(root, state(over), actions as unknown as AccountActions);
}

function button(label: string): HTMLButtonElement {
  const b = [...root.querySelectorAll("button")].find((el) => el.textContent === label);
  if (!b) throw new Error(`no button "${label}"`);
  return b as HTMLButtonElement;
}

function input(name: string): HTMLInputElement {
  const el = root.querySelector<HTMLInputElement>(`input[name="${name}"]`);
  if (!el) throw new Error(`no input ${name}`);
  return el;
}

function submit(): void {
  root.querySelector("form")!.dispatchEvent(new Event("submit", { cancelable: true }));
}

describe("renderAccount", () => {
  it("sign-in offers Google, Apple and email, wired to the actions", () => {
    render({ email: "me@example.com" });
    button("Continuer avec Apple").click();
    expect(actions.signInWith).toHaveBeenCalledWith("apple");
    button("Continuer avec Google").click();
    expect(actions.signInWith).toHaveBeenCalledWith("google");
    expect(input("email").value).toBe("me@example.com");
    input("password").value = "pw";
    submit();
    expect(actions.signInEmail).toHaveBeenCalledWith("me@example.com", "pw");
    button("Mot de passe oublié ?").click();
    expect(actions.go).toHaveBeenCalledWith("forgot");
    button("Créer un compte").click();
    expect(actions.go).toHaveBeenCalledWith("signup");
  });

  it("hides unavailable providers entirely", () => {
    render({ providers: { google: false, apple: false } });
    expect(root.querySelector("[data-provider]")).toBeNull();
    expect(root.textContent).not.toContain("ou avec ton email");
  });

  it("sign-up submits email and password, and offers sign-in/forgot on a taken email", () => {
    render({ view: "signup" });
    input("email").value = "new@example.com";
    input("password").value = "pw";
    expect(input("password").autocomplete).toBe("new-password");
    submit();
    expect(actions.signUp).toHaveBeenCalledWith("new@example.com", "pw");
    expect(root.textContent).toContain("J'ai déjà un compte");

    render({ view: "signup", error: "Un compte utilise déjà cet email.", errorKind: "alreadyExists" });
    button("Se connecter").click();
    expect(actions.go).toHaveBeenCalledWith("signin");
    button("Mot de passe oublié ?").click();
    expect(actions.go).toHaveBeenCalledWith("forgot");
  });

  it("verify shows the email as text, submits the code and resends", () => {
    render({ view: "verify", email: "<img src=x onerror=alert(1)>" });
    expect(root.querySelector("img")).toBeNull();
    expect(root.textContent).toContain("<img src=x onerror=alert(1)>");
    input("code").value = "123456";
    submit();
    expect(actions.verify).toHaveBeenCalledWith("123456");
    button("Renvoyer le code").click();
    expect(actions.resend).toHaveBeenCalledOnce();
  });

  it("forgot and reset submit their fields", () => {
    render({ view: "forgot", email: "me@example.com" });
    submit();
    expect(actions.requestReset).toHaveBeenCalledWith("me@example.com");

    render({ view: "reset", email: "me@example.com" });
    input("code").value = "654321";
    input("password").value = "new-password";
    submit();
    expect(actions.resetPassword).toHaveBeenCalledWith("654321", "new-password");
    button("Renvoyer un code").click();
    expect(actions.requestReset).toHaveBeenLastCalledWith("me@example.com");
  });

  it("signed-in offers sign-out", () => {
    render({ view: "signedin" });
    button("Se déconnecter").click();
    expect(actions.signOut).toHaveBeenCalledOnce();
  });

  it("announces errors and notices", () => {
    render({ error: "Email ou mot de passe incorrect.", notice: "Adresse vérifiée." });
    expect(root.querySelector("[role=alert]")!.textContent).toBe("Email ou mot de passe incorrect.");
    expect(root.querySelector("[role=status]")!.textContent).toBe("Adresse vérifiée.");
  });

  it("disables submitting while busy", () => {
    render({ busy: true });
    expect(button("Se connecter").disabled).toBe(true);
    submit();
    expect(actions.signInEmail).not.toHaveBeenCalled();
  });

  it("handle step: typing reports, submitting commits, abandon leaves", () => {
    render({ view: "handle", candidate: "alice", handleStatus: "available" });
    expect(root.textContent).toContain("Disponible");
    input("handle").value = "alicia";
    input("handle").dispatchEvent(new Event("input"));
    expect(actions.editHandle).toHaveBeenCalledWith("alicia");
    submit();
    expect(actions.commitHandle).toHaveBeenCalledOnce();
    button("Utiliser un autre compte").click();
    expect(actions.abandonHandle).toHaveBeenCalledOnce();
  });

  it("handle step: an empty, invalid or taken handle cannot be submitted", () => {
    for (const handleStatus of ["empty", "invalid", "taken"] as const) {
      actions.commitHandle.mockClear();
      render({ view: "handle", candidate: "x", handleStatus });
      expect(button("Continuer").disabled).toBe(true);
      submit();
      expect(actions.commitHandle).not.toHaveBeenCalled();
    }
  });

  it("keeps focus and caret in the handle field across re-renders", () => {
    render({ view: "handle", candidate: "al", handleStatus: "checking" });
    const first = input("handle");
    first.focus();
    first.setSelectionRange(2, 2);
    render({ view: "handle", candidate: "al", handleStatus: "available" });
    const again = input("handle");
    expect(again).not.toBe(first);
    expect(document.activeElement).toBe(again);
    expect(again.selectionStart).toBe(2);
  });

  it("signed-in shows the handle as text", () => {
    render({ view: "signedin", handle: "<b>x</b>" });
    expect(root.querySelector(".account-handle")!.textContent).toBe("@<b>x</b>");
    expect(root.querySelector(".account-handle b")).toBeNull();
  });

  it("never renders the word 'lemma'/'lemme'", () => {
    for (const view of ["signin", "signup", "verify", "forgot", "reset", "handle", "signedin"] as const) {
      render({ view, email: "me@example.com" });
      expect(root.textContent ?? "").not.toMatch(/lemm/i);
    }
  });

  describe("« Tes données » (add-lingua-privacy-controls)", () => {
    it("asks before erasing and links to the whole-account deletion with its warning", () => {
      render({ view: "signedin", handle: "alice" });
      const section = root.querySelector<HTMLElement>("#data")!;
      expect(section.textContent).toContain("Music compris");
      const link = section.querySelector<HTMLAnchorElement>("a")!;
      expect(link.textContent).toBe("Supprimer mon compte Cymbra");
      expect(link.href).toBe("https://cymbra.app/suppression-compte/");
      expect(link.target).toBe("_blank");
      button("Effacer mes données Lingua…").click();
      expect(actions.askErase).toHaveBeenCalledOnce();
      expect(actions.eraseLinguaData).not.toHaveBeenCalled();
    });

    it("states what the erasure does before confirming it", () => {
      render({ view: "signedin", confirmingErase: true });
      const warning = root.querySelector("#data [role=alert]")!;
      expect(warning.textContent).toContain("tous tes appareils");
      expect(warning.textContent).toContain("Cymbra Music ne sont pas touchés");
      button("Oui, effacer mes données Lingua").click();
      expect(actions.eraseLinguaData).toHaveBeenCalledOnce();
      button("Annuler").click();
      expect(actions.cancelErase).toHaveBeenCalledOnce();
    });

    it("disables the erasure while a call is in flight", () => {
      render({ view: "signedin", confirmingErase: true, busy: true });
      expect(button("Oui, effacer mes données Lingua").disabled).toBe(true);
    });

    it("is only offered to a signed-in reader", () => {
      render({ view: "signin" });
      expect(root.querySelector("#data")).toBeNull();
    });
  });
});
