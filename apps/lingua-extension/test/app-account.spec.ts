import { Code, ConnectError } from "@connectrpc/connect";
import { describe, expect, it, vi } from "vitest";
import {
  APP_SIGNED_OUT_KEY,
  type AppAccountClients,
  type AppAccountSend,
  appAccount,
  appAccountLocale,
  type EmailSignIn,
  keychainArea,
} from "@/app/account.ts";
import type { AppMessage } from "@/app/bridge.ts";
import { ACCESS_KEY, REFRESH_KEY } from "@/state/session.ts";

// The Cymbra Lingua app's own session, for « Traduction en ligne » only (add-lingua-review-translation-fallback
// D8): opened on the first press through the app's sheet (`auth.signIn`), exchanged through
// `SignInOidc`, the refresh token in the Keychain through the bridge, the access token in memory, the
// extension's session never involved. The bridge and the clients are test doubles.

/** A JWT-shaped token whose `exp` is an hour away, so the Session keeps it. */
function token(sub: string): string {
  const payload = btoa(JSON.stringify({ sub, exp: Math.floor(Date.now() / 1000) + 3600 }));
  return `h.${payload}.s`;
}

function bridge(over: { sheet?: unknown; keychain?: string | null } = {}) {
  const sent: AppMessage[] = [];
  let keychain = over.keychain ?? null;
  const send = vi.fn(async (message: AppMessage): Promise<unknown> => {
    sent.push(message);
    switch (message.type) {
      case "auth.signIn":
        return over.sheet ?? {};
      case "auth.readRefresh":
        return keychain ? { refresh: keychain } : {};
      case "auth.writeRefresh":
        keychain = message.refresh;
        return {};
      case "auth.clear":
        keychain = null;
        return {};
      default:
        return {};
    }
  });
  return { send, sent, keychain: () => keychain };
}

function clients(access = true) {
  const auth = {
    signInOidc: vi.fn(async () => ({ accessToken: token("u1"), refreshToken: "refresh-1" })),
    signInLocal: vi.fn(async () => ({ accessToken: token("u1"), refreshToken: "refresh-email" })),
    verifyEmail: vi.fn(async () => ({})),
    resendVerification: vi.fn(async () => ({})),
    refresh: vi.fn(async () => ({ accessToken: token("u1"), refreshToken: "refresh-2" })),
    logout: vi.fn(async () => ({})),
  };
  const flags = {
    getEffectiveFlags: vi.fn(async () => ({
      version: "1",
      unchanged: false,
      flags: [{ key: "lingua.online_translation.enabled", value: { kind: { case: "boolValue", value: access } } }],
    })),
  };
  const onlineTranslation = {
    translate: vi.fn(async () => ({
      outcome: { case: "translated", value: { translation: "La corde se relâche.", marks: [{ start: 9, end: 19 }] } },
    })),
  };
  const user = {
    getAccount: vi.fn(async (): Promise<{ handle?: string }> => ({ handle: "alice" })),
    listIdentities: vi.fn(async (): Promise<{ identities: { provider: string; subject: string }[] }> => ({
      identities: [],
    })),
  };
  return {
    auth,
    flags,
    onlineTranslation,
    user,
    all: { auth, flags, onlineTranslation, user } as unknown as AppAccountClients,
  };
}

function storage() {
  const items: Record<string, string> = {};
  return {
    items,
    getItem: (k: string) => items[k] ?? null,
    setItem: (k: string, v: string) => void (items[k] = v),
    removeItem: (k: string) => void delete items[k],
  };
}

const REQUEST = { sentence: "The rope began to slacken.", selection: { start: 18, end: 25 }, source: "en" };

function account(
  over: {
    sheet?: unknown;
    keychain?: string | null;
    chosen?: boolean;
    relayedAccess?: boolean;
    access?: boolean;
    emailSignIn?: EmailSignIn;
  } = {},
) {
  const b = bridge(over);
  const c = clients(over.access ?? true);
  const local = storage();
  const onChange = vi.fn();
  const a = appAccount({
    send: b.send,
    clients: () => c.all,
    native: async () => "fr",
    chosen: over.chosen ?? true,
    relayedAccess: over.relayedAccess ?? true,
    local,
    onChange,
    emailSignIn: over.emailSignIn,
  });
  return { a, b, c, local, onChange };
}

describe("the app's session (add-lingua-review-translation-fallback D8)", () => {
  // (ticked, relayed access) → offered before the app has a session
  const OFFERED: [boolean, boolean, boolean][] = [
    [true, true, true],
    [true, false, false],
    [false, true, false],
  ];

  it("offers the button before signing in from the extension's setting and access snapshot", () => {
    for (const [chosen, relayedAccess, offered] of OFFERED) {
      expect(account({ chosen, relayedAccess }).a.online.offered(), `${chosen} ${relayedAccess}`).toBe(offered);
    }
  });

  it("The app signs in for online translation: the sheet on the first press, then the translation", async () => {
    const s = account({ sheet: { provider: "apple", idToken: "apple-id-token" } });
    const result = await s.a.online.translate(REQUEST);

    expect(s.b.sent.map((m) => m.type)).toEqual(["auth.signIn", "auth.writeRefresh"]);
    expect(s.c.auth.signInOidc).toHaveBeenCalledWith({ idToken: "apple-id-token", audience: "lingua" });
    expect(s.b.keychain()).toBe("refresh-1");
    expect(s.a.signedIn()).toBe(true);
    expect(s.a.token()).toBe(token("u1"));
    expect(s.c.onlineTranslation.translate).toHaveBeenCalledWith(
      expect.objectContaining({
        sentence: REQUEST.sentence,
        selectionStart: 18,
        selectionEnd: 25,
        targetLanguage: "fr",
      }),
    );
    expect(result).toEqual({
      kind: "translated",
      translation: { sentence: "La corde se relâche.", marks: [{ start: 9, end: 19 }] },
    });
    // A second press asks no sheet again.
    await s.a.online.translate(REQUEST);
    expect(s.b.sent.filter((m) => m.type === "auth.signIn")).toHaveLength(1);
  });

  it("the account's own access decides once signed in", async () => {
    const s = account({ sheet: { provider: "google", idToken: "g" }, access: false });
    await s.a.signIn();
    expect(s.a.online.offered()).toBe(false);
    expect(s.onChange).toHaveBeenCalled();
  });

  // What the sheet answers → no translation, nothing sent, no session.
  const CLOSED: unknown[] = [{}, null, { provider: "github", idToken: "x" }, { provider: "apple", idToken: "" }];

  it("a closed or failed sheet sends nothing and opens no session", async () => {
    for (const sheet of CLOSED) {
      const s = account({ sheet });
      expect(await s.a.online.translate(REQUEST), JSON.stringify(sheet)).toEqual({ kind: "unavailable" });
      expect(s.c.auth.signInOidc).not.toHaveBeenCalled();
      expect(s.c.onlineTranslation.translate).not.toHaveBeenCalled();
      expect(s.a.signedIn()).toBe(false);
    }
  });

  it("a refused exchange opens no session", async () => {
    const s = account({ sheet: { provider: "apple", idToken: "t" } });
    s.c.auth.signInOidc.mockRejectedValueOnce(new Error("refused"));
    expect(await s.a.signIn()).toBe(false);
    expect(s.a.signedIn()).toBe(false);
  });

  it("sends nothing for a request that must not leave the page", async () => {
    const s = account({ sheet: { provider: "apple", idToken: "t" } });
    expect(await s.a.online.translate({ ...REQUEST, sentence: "  ", selection: { start: 0, end: 1 } })).toEqual({
      kind: "unavailable",
    });
    expect(s.b.sent).toEqual([]);
  });

  it("reopens the session the Keychain holds, with a fresh access token in memory", async () => {
    const s = account({ keychain: "refresh-1" });
    expect(await s.a.resume()).toBe(true);
    expect(s.c.auth.refresh).toHaveBeenCalledWith({ refreshToken: "refresh-1" });
    expect(s.b.keychain()).toBe("refresh-2");
    expect(await s.a.refresh()).toBe(true);
    expect(await account().a.resume()).toBe(false);
  });

  it("Signing out of the app: revoked, the Keychain cleared, no button until signing in again", async () => {
    const s = account({ sheet: { provider: "apple", idToken: "t" } });
    await s.a.signIn();
    await s.a.signOut();
    expect(s.c.auth.logout).toHaveBeenCalledWith({ refreshToken: "refresh-1" });
    expect(s.b.keychain()).toBeNull();
    expect(s.a.signedIn()).toBe(false);
    expect(s.a.signedOut()).toBe(true);
    expect(s.local.items[APP_SIGNED_OUT_KEY]).toBe("1");
    expect(s.a.online.offered()).toBe(false);
    await s.a.signIn();
    expect(s.a.signedOut()).toBe(false);
    expect(s.a.online.offered()).toBe(true);
  });

  it("keeps on when the flags cannot be read", async () => {
    const s = account({ sheet: { provider: "apple", idToken: "t" } });
    s.c.flags.getEffectiveFlags.mockRejectedValueOnce(Object.assign(new Error("x"), { code: 14 }));
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    expect(await s.a.signIn()).toBe(true);
    expect(s.a.online.offered()).toBe(true); // the snapshot still decides
    warn.mockRestore();
  });
});

describe("the Session's storage in the app", () => {
  it("keeps the refresh token in the Keychain and everything else in memory only", async () => {
    const b = bridge();
    const area = keychainArea(b.send);
    await area.set({ [REFRESH_KEY]: "r", [ACCESS_KEY]: { token: "a" }, other: 1 });
    expect(b.keychain()).toBe("r");
    expect(b.sent.map((m) => m.type)).toEqual(["auth.writeRefresh"]);
    expect(await area.get([REFRESH_KEY, ACCESS_KEY, "absent"])).toEqual({
      [REFRESH_KEY]: "r",
      [ACCESS_KEY]: { token: "a" },
    });
    expect(await area.get(null)).toEqual({ [ACCESS_KEY]: { token: "a" }, other: 1 });
    expect(await area.get(ACCESS_KEY)).toEqual({ [ACCESS_KEY]: { token: "a" } });
    await area.set({ [REFRESH_KEY]: null });
    expect(b.keychain()).toBeNull();
    expect(await area.get(REFRESH_KEY)).toEqual({});
  });
});

describe("the app's email sign-in (add-lingua-app-email-signin D2, D3)", () => {
  const EMAIL = "me@example.com";
  const PASSWORD = "correct horse battery";

  /** The panel's double: signs in with the email and password through `send`, like the flow would. */
  function typing(password = PASSWORD): EmailSignIn & { sends: AppAccountSend[] } {
    const sends: AppAccountSend[] = [];
    const panel = async (send: AppAccountSend): Promise<boolean> => {
      sends.push(send);
      const reply = await send({ type: "account:signInLocal", email: EMAIL, password });
      return reply?.ok === true;
    };
    return Object.assign(panel, { sends });
  }

  it("An email account signs in: the panel opens, SignInLocal for lingua, only the refresh token kept", async () => {
    const panel = typing();
    const s = account({ sheet: { method: "email" }, emailSignIn: panel });
    const result = await s.a.online.translate(REQUEST);

    expect(panel.sends).toHaveLength(1);
    expect(s.c.auth.signInLocal).toHaveBeenCalledWith({ email: EMAIL, password: PASSWORD, audience: "lingua" });
    expect(s.c.auth.signInOidc).not.toHaveBeenCalled();
    expect(s.b.sent.map((m) => m.type)).toEqual(["auth.signIn", "auth.writeRefresh"]);
    expect(s.b.keychain()).toBe("refresh-email");
    // The password never crosses the bridge, nor lands in the page's storage.
    expect(JSON.stringify(s.b.sent)).not.toContain(PASSWORD);
    expect(JSON.stringify(s.local.items)).not.toContain(PASSWORD);
    expect(s.a.signedIn()).toBe(true);
    // The online translation that asked for the sign-in is requested.
    expect(s.c.onlineTranslation.translate).toHaveBeenCalledOnce();
    expect(result.kind).toBe("translated");
    expect(s.onChange).toHaveBeenCalled();
  });

  it("a closed panel: no session, no translation", async () => {
    const closed: EmailSignIn = async () => false;
    const s = account({ sheet: { method: "email" }, emailSignIn: closed });
    expect(await s.a.online.translate(REQUEST)).toEqual({ kind: "unavailable" });
    expect(s.c.onlineTranslation.translate).not.toHaveBeenCalled();
    expect(s.a.signedIn()).toBe(false);
    expect(s.b.sent.map((m) => m.type)).toEqual(["auth.signIn"]);
  });

  it("a panel that fails, or no panel at all, opens no session", async () => {
    const failing: EmailSignIn = async () => {
      throw new Error("panel");
    };
    for (const emailSignIn of [failing, undefined]) {
      const s = account({ sheet: { method: "email" }, emailSignIn });
      expect(await s.a.signIn()).toBe(false);
      expect(s.a.signedIn()).toBe(false);
    }
  });

  it("a panel that says done without a session is not believed", async () => {
    const liar: EmailSignIn = async () => true;
    const s = account({ sheet: { method: "email" }, emailSignIn: liar });
    expect(await s.a.signIn()).toBe(false);
  });

  it("Wrong password: the dispatch answers its category, nothing stored", async () => {
    const panel = typing("wrong");
    const s = account({ sheet: { method: "email" }, emailSignIn: panel });
    s.c.auth.signInLocal.mockRejectedValue(new ConnectError("bad", Code.Unauthenticated));
    expect(await s.a.signIn()).toBe(false);
    expect(await panel.sends[0]({ type: "account:signInLocal", email: EMAIL, password: "wrong" })).toEqual({
      ok: false,
      error: "unauthenticated",
    });
    expect(s.b.sent.map((m) => m.type)).toEqual(["auth.signIn"]);
    expect(s.b.keychain()).toBeNull();
    expect(s.a.signedIn()).toBe(false);
  });

  it("answers the flow's messages over the app's Session: categories, no provider, no profile", async () => {
    const panel = typing();
    const s = account({ sheet: { method: "email" }, emailSignIn: panel });
    s.c.auth.signInLocal.mockRejectedValueOnce(new ConnectError("unverified", Code.FailedPrecondition));
    expect(await s.a.signIn()).toBe(false);
    const send = panel.sends[0];
    expect(await send({ type: "account:providers" })).toEqual({ ok: true, providers: { google: false, apple: false } });
    expect(await send({ type: "account:verifyEmail", code: "123456" })).toEqual({ ok: true });
    expect(s.c.auth.verifyEmail).toHaveBeenCalledWith({ token: "123456" });
    expect(await send({ type: "account:resendVerification", email: EMAIL, locale: "fr" })).toEqual({ ok: true });
    // The profile port is never reached in the app scope: a call is refused, loud in a test.
    expect((await send({ type: "account:checkHandle", handle: "x" }))?.ok).toBe(false);
    expect((await send({ type: "account:eraseLinguaData" }))?.ok).toBe(false);
  });

  it("the requests' locale: the language chosen in the extension, else the device's tag, else French", () => {
    expect(appAccountLocale("es", "fr-FR")).toBe("es");
    expect(appAccountLocale(null, "en-GB")).toBe("en-GB");
    expect(appAccountLocale(null, "")).toBe("fr");
  });
});

describe("the account the app is signed in to (add-lingua-app-email-signin D9)", () => {
  it("The account signed in is named: its handle, read once after an Apple or Google sign-in", async () => {
    const s = account({ sheet: { provider: "google", idToken: "g" } });
    expect(s.a.name()).toBeNull();
    await s.a.signIn();
    expect(s.a.name()).toEqual({ handle: "alice", email: null });
    expect(s.c.user.getAccount).toHaveBeenCalledOnce();
    expect(s.c.user.listIdentities).not.toHaveBeenCalled();
    await s.a.online.translate(REQUEST);
    expect(s.c.user.getAccount).toHaveBeenCalledOnce();
  });

  it("read on opening when the Keychain holds a session", async () => {
    const s = account({ keychain: "refresh-1" });
    await s.a.resume();
    expect(s.a.name()).toEqual({ handle: "alice", email: null });
  });

  it("an email sign-in without a handle is named by the address it typed — never the password", async () => {
    const panel: EmailSignIn = async (send) =>
      (await send({ type: "account:signInLocal", email: "me@example.com", password: "pw-secret" }))?.ok === true;
    const s = account({ sheet: { method: "email" }, emailSignIn: panel });
    s.c.user.getAccount.mockResolvedValueOnce({});
    await s.a.signIn();
    expect(s.a.name()).toEqual({ handle: null, email: "me@example.com" });
    expect(s.c.user.listIdentities).not.toHaveBeenCalled();
    expect(JSON.stringify(s.a.name())).not.toContain("pw-secret");
  });

  it("no handle after a provider sign-in: the email-and-password identity's address, else nothing", async () => {
    const s = account({ sheet: { provider: "apple", idToken: "t" } });
    s.c.user.getAccount.mockResolvedValue({});
    s.c.user.listIdentities.mockResolvedValueOnce({
      identities: [
        { provider: "apple", subject: "000123.abc" },
        { provider: "local", subject: "me@example.com" },
      ],
    });
    await s.a.signIn();
    expect(s.a.name()).toEqual({ handle: null, email: "me@example.com" });
    const none = account({ sheet: { provider: "apple", idToken: "t" } });
    none.c.user.getAccount.mockResolvedValue({});
    await none.a.signIn();
    expect(none.a.name()).toBeNull();
  });

  it("a profile read that fails leaves no name (the plain line), and signing out forgets it", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const s = account({ sheet: { provider: "apple", idToken: "t" } });
    s.c.user.getAccount.mockRejectedValueOnce(Object.assign(new Error("x"), { code: 14 }));
    expect(await s.a.signIn()).toBe(true);
    expect(s.a.name()).toBeNull();
    warn.mockRestore();
    const t = account({ sheet: { provider: "apple", idToken: "t" } });
    await t.a.signIn();
    await t.a.signOut();
    expect(t.a.name()).toBeNull();
  });

  it("a failed read after an email sign-in still names the address typed", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const panel: EmailSignIn = async (send) =>
      (await send({ type: "account:signInLocal", email: "me@example.com", password: "pw" }))?.ok === true;
    const s = account({ sheet: { method: "email" }, emailSignIn: panel });
    s.c.user.getAccount.mockRejectedValueOnce(new Error("down"));
    await s.a.signIn();
    expect(s.a.name()).toEqual({ handle: null, email: "me@example.com" });
    warn.mockRestore();
  });
});

describe("the account's id for the synchronisation (add-lingua-app-deck-sync D7)", () => {
  it("reuses the GetAccount the name read, and reads it again only when that failed or after a sign-out", async () => {
    const s = account({ sheet: { provider: "google", idToken: "g" } });
    s.c.user.getAccount.mockResolvedValue({ handle: "alice", userId: "user-1" } as { handle?: string });
    await s.a.signIn();
    expect(await s.a.userId()).toBe("user-1");
    expect(s.c.user.getAccount).toHaveBeenCalledOnce();
    await s.a.signOut();
    expect(await s.a.userId()).toBe("user-1");
    expect(s.c.user.getAccount).toHaveBeenCalledTimes(2);
  });

  it("a name read that failed: the id is asked on its own", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const s = account({ sheet: { provider: "google", idToken: "g" } });
    s.c.user.getAccount.mockRejectedValueOnce(new Error("down"));
    await s.a.signIn();
    s.c.user.getAccount.mockResolvedValue({ handle: "alice", userId: "user-2" } as { handle?: string });
    expect(await s.a.userId()).toBe("user-2");
    warn.mockRestore();
  });
});
