import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AccountMessage, AccountReply } from "@/account/messages.ts";
import type { AppAccountSend } from "@/app/account.ts";
import { emailPanel, reloadsOnForeground } from "@/app/email-panel.ts";

// The Cymbra Lingua app's email sign-in (add-lingua-app-email-signin D2, D3, D6): a `<dialog>` holding
// the extension's own account flow and view in their app scope, settled true once signed in and false
// when closed; the app keeps its page while it is open. jsdom, the account messages a test double.

let dialog: HTMLDialogElement;
let root: HTMLElement;
let cancel: HTMLButtonElement;

beforeEach(() => {
  document.body.innerHTML = `<dialog id="email-panel"><div id="email-root"></div><button id="email-cancel">Annuler</button></dialog>`;
  dialog = document.getElementById("email-panel") as HTMLDialogElement;
  root = document.getElementById("email-root")!;
  cancel = document.getElementById("email-cancel") as HTMLButtonElement;
});

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

function panel(language: "fr" | "en" | "es" = "fr") {
  return emailPanel({ dialog, root, cancel, language, locale: "fr-FR" });
}

function sender(answer: (m: AccountMessage) => AccountReply | null = () => ({ ok: true })) {
  const sent: AccountMessage[] = [];
  const send: AppAccountSend = vi.fn(async (m: AccountMessage) => {
    sent.push(m);
    return answer(m);
  });
  return { send, sent };
}

function input(name: string): HTMLInputElement {
  const el = root.querySelector<HTMLInputElement>(`input[name="${name}"]`);
  if (!el) throw new Error(`no input ${name}`);
  return el;
}

function submit(): void {
  root.querySelector("form")!.dispatchEvent(new Event("submit", { cancelable: true }));
}

const isShown = (): boolean => dialog.open || dialog.hasAttribute("open");

describe("the app's email panel", () => {
  it("opens on the sign-in view, without account creation or providers", async () => {
    const p = panel();
    const { send, sent } = sender();
    void p.signIn(send);
    await tick();
    expect(isShown()).toBe(true);
    expect(p.isOpen()).toBe(true);
    expect(root.querySelector(".account-title")?.textContent).toBe("Se connecter");
    expect(root.textContent).not.toContain("Créer un compte");
    expect(root.querySelector("[data-provider]")).toBeNull();
    expect(sent).toEqual([]);
  });

  it("closes on « Annuler », settling false, and leaves nothing behind", async () => {
    const p = panel();
    const done = p.signIn(sender().send);
    await tick();
    cancel.click();
    expect(await done).toBe(false);
    expect(isShown()).toBe(false);
    expect(p.isOpen()).toBe(false);
    expect(root.childNodes).toHaveLength(0);
  });

  it("closes when the dialog is closed by the platform (Escape)", async () => {
    const p = panel();
    const done = p.signIn(sender().send);
    await tick();
    // A stale close (the dialog still open) is ignored.
    dialog.dispatchEvent(new Event("close"));
    expect(p.isOpen()).toBe(true);
    dialog.removeAttribute("open");
    dialog.dispatchEvent(new Event("close"));
    expect(await done).toBe(false);
  });

  it("closes on sign-in, settling true", async () => {
    const p = panel();
    const { send, sent } = sender();
    const done = p.signIn(send);
    await tick();
    input("email").value = "me@example.com";
    input("password").value = "pw";
    submit();
    expect(await done).toBe(true);
    expect(sent).toEqual([{ type: "account:signInLocal", email: "me@example.com", password: "pw" }]);
    expect(isShown()).toBe(false);
    expect(p.isOpen()).toBe(false);
  });

  it("shows the extension's copy on a wrong password, and stays open", async () => {
    const p = panel();
    const done = vi.fn();
    void p.signIn(sender(() => ({ ok: false, error: "unauthenticated" })).send).then(done);
    await tick();
    input("email").value = "me@example.com";
    input("password").value = "wrong";
    submit();
    await tick();
    expect(root.querySelector(".account-error")?.textContent).toBe("Email ou mot de passe incorrect.");
    expect(p.isOpen()).toBe(true);
    expect(done).not.toHaveBeenCalled();
  });

  it("runs the code step and the forgotten password from the view", async () => {
    const p = panel();
    let signIns = 0;
    const { send, sent } = sender((m) =>
      m.type === "account:signInLocal" && signIns++ === 0 ? { ok: false, error: "failedPrecondition" } : { ok: true },
    );
    const done = p.signIn(send);
    await tick();
    const click = (label: string): void =>
      [...root.querySelectorAll("button")].find((b) => b.textContent === label)!.click();
    // Mot de passe oublié ? → the address → the code and a new password → back at sign-in.
    click("Mot de passe oublié ?");
    input("email").value = "me@example.com";
    submit();
    await tick();
    input("code").value = "654321";
    input("password").value = "new-pw";
    submit();
    await tick();
    expect(root.querySelector(".account-notice")?.textContent).toBe(
      "Mot de passe modifié. Connecte-toi avec le nouveau.",
    );
    // Sign-in → unverified → a new code → the code signs in.
    input("password").value = "new-pw";
    submit();
    await tick();
    click("Renvoyer le code");
    await tick();
    input("code").value = "123456";
    submit();
    expect(await done).toBe(true);
    expect(sent.map((m) => m.type)).toEqual([
      "account:requestPasswordReset",
      "account:resetPassword",
      "account:signInLocal",
      "account:resendVerification",
      "account:verifyEmail",
      "account:signInLocal",
    ]);
    expect(sent[0]).toEqual({ type: "account:requestPasswordReset", email: "me@example.com", locale: "fr-FR" });
  });

  it("is worded in the interface language", async () => {
    const p = panel("en");
    void p.signIn(sender().send);
    await tick();
    expect(root.querySelector(".account-title")?.textContent).toBe("Sign in");
  });

  it("a second request answers the first with nothing, then opens anew", async () => {
    const p = panel();
    const first = p.signIn(sender().send);
    await tick();
    void p.signIn(sender().send);
    expect(await first).toBe(false);
    await tick();
    expect(p.isOpen()).toBe(true);
    expect(isShown()).toBe(true);
  });

  it("forgets the code step's address once closed", async () => {
    const p = panel();
    const answer = (m: AccountMessage): AccountReply =>
      m.type === "account:signInLocal" ? { ok: false, error: "failedPrecondition" } : { ok: true };
    void p.signIn(sender(answer).send);
    await tick();
    input("email").value = "me@example.com";
    input("password").value = "pw";
    submit();
    await tick();
    expect(root.querySelector(".account-title")?.textContent).toBe("Vérifie ton adresse email");
    expect(root.textContent).toContain("me@example.com");
    cancel.click();
    void p.signIn(sender(answer).send);
    await tick();
    expect(root.querySelector(".account-title")?.textContent).toBe("Se connecter");
  });
});

describe("the foreground reload (D6)", () => {
  // (reviewing, the email panel open) → reloads
  const CASES: [boolean, boolean, boolean][] = [
    [false, false, true],
    [true, false, false],
    [false, true, false],
    [true, true, false],
  ];

  it("keeps the page while a review is under way or the email sign-in is open", () => {
    for (const [reviewing, emailPanelOpen, reloads] of CASES) {
      expect(reloadsOnForeground({ reviewing, emailPanelOpen }), `${reviewing} ${emailPanelOpen}`).toBe(reloads);
    }
  });
});
