import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AccountReply } from "@/account/messages.ts";
import { accountSetting as ACCOUNT_COPY } from "@/i18n/fr/account-setting.ts";
import { type AccountControls, mountAccountSetting, runtimeAccountControls } from "@/reading/account-setting.ts";
import type { PersistedSignInError } from "@/state/session.ts";

// « Compte » in Réglages › Données: the popup's sign-in, now in every host of Réglages. The
// background is faked behind AccountControls.

const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

interface Fake {
  signedIn: boolean;
  handle: string | null | undefined;
  providers: { google: boolean; apple: boolean };
  signInReply: AccountReply | null;
  persisted: PersistedSignInError | null;
}

function setup(over: Partial<Fake> = {}) {
  const fake: Fake = {
    signedIn: false,
    handle: "lea",
    providers: { google: true, apple: false },
    signInReply: { ok: true },
    persisted: null,
    ...over,
  };
  const controls: AccountControls = {
    state: vi.fn(async () => ({ signedIn: fake.signedIn })),
    providers: vi.fn(async () => fake.providers),
    handle: vi.fn(async () => fake.handle),
    signInWith: vi.fn(async () => {
      if (fake.signInReply?.ok) fake.signedIn = true;
      return fake.signInReply;
    }),
    signInLocal: vi.fn(async () => {
      if (fake.signInReply?.ok) fake.signedIn = true;
      return fake.signInReply;
    }),
    signOut: vi.fn(async () => {
      fake.signedIn = false;
      return { ok: true };
    }),
    takeSignInError: vi.fn(async () => {
      const taken = fake.persisted;
      fake.persisted = null;
      return taken;
    }),
    rememberPendingEmail: vi.fn(async () => {}),
  };
  const block = document.createElement("div");
  document.body.replaceChildren(block);
  const opened: string[] = [];
  const handedOff = vi.fn();
  const onChange = vi.fn(async () => view.refresh());
  const view = mountAccountSetting(block, controls, {
    openPage: (url) => opened.push(url),
    onChange,
    onHandedOff: handedOff,
  });
  const button = (text: string): HTMLButtonElement =>
    [...block.querySelectorAll<HTMLButtonElement>("button")].find((b) => b.textContent === text)!;
  const visible = (node: Element | null | undefined): boolean => {
    for (let n = node; n && n !== block; n = n.parentElement) if ((n as HTMLElement).hidden) return false;
    return Boolean(node);
  };
  const error = (): HTMLElement => block.querySelector<HTMLElement>(".set-warn")!;
  const input = (type: string): HTMLInputElement => block.querySelector<HTMLInputElement>(`input[type="${type}"]`)!;
  return { fake, controls, block, view, opened, handedOff, onChange, button, visible, error, input };
}

describe("Réglages › Compte", () => {
  beforeEach(() => document.body.replaceChildren());

  it("signed out, offers the providers the build has and the email form", async () => {
    const s = setup();
    await s.view.refresh();
    expect(s.visible(s.button(ACCOUNT_COPY.google))).toBe(true);
    expect(s.visible(s.button(ACCOUNT_COPY.apple))).toBe(false);
    expect(s.visible(s.button(ACCOUNT_COPY.signOut))).toBe(false);
    expect(s.block.textContent).toContain(ACCOUNT_COPY.invite);
    // A provider is offered: the email form stays folded.
    expect(s.block.querySelector("details")?.open).toBe(false);
  });

  it("unfolds the email form where no provider is offered (Safari, Firefox for Android)", async () => {
    const s = setup({ providers: { google: false, apple: false } });
    await s.view.refresh();
    expect(s.visible(s.button(ACCOUNT_COPY.google))).toBe(false);
    expect(s.block.querySelector("details")?.open).toBe(true);
  });

  it("signed in, shows the handle and sign-out, and signs out", async () => {
    const s = setup({ signedIn: true });
    await s.view.refresh();
    expect(s.block.querySelector(".set-account-row")?.textContent).toBe("@lea");
    expect(s.visible(s.button(ACCOUNT_COPY.handleOpen))).toBe(false);
    s.button(ACCOUNT_COPY.signOut).click();
    await settle();
    await settle();
    expect(s.controls.signOut).toHaveBeenCalled();
    expect(s.onChange).toHaveBeenCalled();
    expect(s.visible(s.button(ACCOUNT_COPY.signOut))).toBe(false);
  });

  it("asks for a handle when the account has none, on the account page", async () => {
    const s = setup({ signedIn: true, handle: null });
    await s.view.refresh();
    expect(s.block.querySelector(".set-account-row")?.textContent).toBe(ACCOUNT_COPY.handleMissing);
    s.button(ACCOUNT_COPY.handleOpen).click();
    expect(s.opened).toEqual(["account.html#handle"]);
  });

  it("says signed in when the handle cannot be read", async () => {
    const s = setup({ signedIn: true, handle: undefined });
    await s.view.refresh();
    expect(s.block.querySelector(".set-account-row")?.textContent).toBe(ACCOUNT_COPY.signedIn);
    expect(s.visible(s.button(ACCOUNT_COPY.handleOpen))).toBe(false);
  });

  it("signs in with a provider, then lets Réglages follow", async () => {
    const s = setup();
    await s.view.refresh();
    s.button(ACCOUNT_COPY.google).click();
    await settle();
    await settle();
    expect(s.controls.signInWith).toHaveBeenCalledWith("google");
    expect(s.onChange).toHaveBeenCalled();
    expect(s.visible(s.button(ACCOUNT_COPY.signOut))).toBe(true);
  });

  it("says a provider failure in the provider's words, and nothing for a cancel", async () => {
    const s = setup({ providers: { google: true, apple: true }, signInReply: { ok: false, cancelled: true } });
    await s.view.refresh();
    s.button(ACCOUNT_COPY.google).click();
    await settle();
    expect(s.error().hidden).toBe(true);
    s.fake.signInReply = { ok: false, error: "unavailable" };
    s.button(ACCOUNT_COPY.apple).click();
    await settle();
    expect(s.error().hidden).toBe(false);
    expect(s.error().textContent).toBe("Impossible de joindre Cymbra. Vérifie ta connexion et réessaie.");
    s.fake.signInReply = { ok: false };
    s.button(ACCOUNT_COPY.apple).click();
    await settle();
    expect(s.error().textContent).toBe("La connexion avec Apple a échoué. Réessaie.");
  });

  it("hands Safari's sign-in over to the host app", async () => {
    const s = setup({ providers: { google: true, apple: true }, signInReply: { ok: false, handedOff: true } });
    await s.view.refresh();
    s.button(ACCOUNT_COPY.apple).click();
    await settle();
    expect(s.handedOff).toHaveBeenCalled();
    expect(s.error().hidden).toBe(true);
  });

  it("shows, once, a provider failure persisted while the surface was closed", async () => {
    const s = setup({ persisted: { provider: "google", kind: "unknown" } });
    await s.view.refresh();
    expect(s.error().textContent).toBe("La connexion avec Google a échoué. Réessaie.");
    expect(s.fake.persisted).toBeNull();
  });

  it("signs in by email, keeping the password nowhere", async () => {
    const s = setup();
    await s.view.refresh();
    s.button(ACCOUNT_COPY.signIn).click(); // nothing typed: nothing sent
    await settle();
    expect(s.controls.signInLocal).not.toHaveBeenCalled();
    s.input("email").value = " lea@example.com ";
    s.input("password").value = "secret";
    s.button(ACCOUNT_COPY.signIn).click();
    await settle();
    await settle();
    expect(s.controls.signInLocal).toHaveBeenCalledWith("lea@example.com", "secret");
    expect(s.input("password").value).toBe("");
    expect(s.visible(s.button(ACCOUNT_COPY.signOut))).toBe(true);
  });

  it("sends an unverified email to the account page's code step, with the email only", async () => {
    const s = setup({ signInReply: { ok: false, error: "failedPrecondition" } });
    await s.view.refresh();
    s.input("email").value = "lea@example.com";
    s.input("password").value = "secret";
    s.button(ACCOUNT_COPY.signIn).click();
    await settle();
    await settle();
    expect(s.controls.rememberPendingEmail).toHaveBeenCalledWith("lea@example.com");
    expect(s.opened).toEqual(["account.html#verify"]);
  });

  it("says a wrong password, and an unreachable background", async () => {
    const s = setup({ signInReply: { ok: false, error: "unauthenticated" } });
    await s.view.refresh();
    s.input("email").value = "lea@example.com";
    s.input("password").value = "nope";
    s.button(ACCOUNT_COPY.signIn).click();
    await settle();
    expect(s.error().textContent).toBe("Email ou mot de passe incorrect.");
    s.fake.signInReply = null;
    s.button(ACCOUNT_COPY.signIn).click();
    await settle();
    expect(s.error().textContent).toBe("Impossible de joindre Cymbra. Vérifie ta connexion et réessaie.");
  });

  it("links signed in to the connected accounts, on the account page", async () => {
    const s = setup({ signedIn: true });
    await s.view.refresh();
    s.button(ACCOUNT_COPY.connected).click();
    expect(s.opened).toEqual(["account.html#connected"]);
  });

  it("says how to reach a Google or Apple account where one of them is missing", async () => {
    const hint = (s: ReturnType<typeof setup>) =>
      [...s.block.querySelectorAll<HTMLElement>(".set-note")].find((n) => n.textContent === ACCOUNT_COPY.providerHint)!;
    const boox = setup({ providers: { google: false, apple: false } });
    await boox.view.refresh();
    expect(boox.visible(hint(boox))).toBe(true);
    const chrome = setup({ providers: { google: true, apple: true } });
    await chrome.view.refresh();
    expect(chrome.visible(hint(chrome))).toBe(false);
  });

  it("opens the account page for creating an account and a forgotten password", async () => {
    const s = setup();
    await s.view.refresh();
    s.button(ACCOUNT_COPY.signUp).click();
    s.button(ACCOUNT_COPY.forgot).click();
    expect(s.opened).toEqual(["account.html#signup", "account.html#forgot"]);
  });
});

describe("runtimeAccountControls", () => {
  const sendMessage = vi.fn();
  const session: Record<string, unknown> = {};
  beforeEach(() => {
    sendMessage.mockReset();
    for (const k of Object.keys(session)) delete session[k];
    vi.stubGlobal("chrome", {
      runtime: { sendMessage },
      storage: {
        session: {
          get: async (key: string) => ({ [key]: session[key] }),
          set: async (items: Record<string, unknown>) => void Object.assign(session, items),
        },
      },
    });
  });
  afterEach(() => vi.unstubAllGlobals());

  it("asks the background, by account messages", async () => {
    const c = runtimeAccountControls();
    sendMessage.mockResolvedValueOnce({ ok: true, state: { signedIn: true } });
    expect(await c.state()).toEqual({ signedIn: true });
    sendMessage.mockResolvedValueOnce({ ok: true, providers: { google: true, apple: false } });
    expect(await c.providers()).toEqual({ google: true, apple: false });
    sendMessage.mockResolvedValueOnce({ ok: true, handle: null });
    expect(await c.handle()).toBeNull();
    sendMessage.mockResolvedValueOnce({ ok: false });
    expect(await c.handle()).toBeUndefined();
    sendMessage.mockResolvedValue({ ok: true });
    await c.signInWith("apple");
    await c.signInWith("google");
    await c.signInLocal("a@b.c", "pw");
    await c.signOut();
    expect(sendMessage.mock.calls.slice(4).map(([m]) => m)).toEqual([
      { type: "account:signInApple" },
      { type: "account:signInGoogle" },
      { type: "account:signInLocal", email: "a@b.c", password: "pw" },
      { type: "account:signOut" },
    ]);
  });

  it("tolerates an asleep worker", async () => {
    const c = runtimeAccountControls();
    sendMessage.mockRejectedValue(new Error("no receiver"));
    expect(await c.state()).toBeNull();
    expect(await c.providers()).toEqual({ google: false, apple: false });
    expect(await c.handle()).toBeUndefined();
    expect(await c.signOut()).toBeNull();
  });

  it("takes the persisted sign-in error once, and keeps the pending email", async () => {
    const c = runtimeAccountControls();
    expect(await c.takeSignInError()).toBeNull();
    session["cymbra-lingua-signin-error"] = { provider: "google", kind: "unknown" };
    expect(await c.takeSignInError()).toEqual({ provider: "google", kind: "unknown" });
    expect(await c.takeSignInError()).toBeNull();
    await c.rememberPendingEmail("a@b.c");
    expect(session["cymbra-lingua-pending-verify"]).toBe("a@b.c");
  });

  it("does without storage.session where it is out of reach (a drawer)", async () => {
    vi.stubGlobal("chrome", { runtime: { sendMessage }, storage: {} });
    const c = runtimeAccountControls();
    expect(await c.takeSignInError()).toBeNull();
    await expect(c.rememberPendingEmail("a@b.c")).resolves.toBeUndefined();
  });
});
