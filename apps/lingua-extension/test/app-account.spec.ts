import { describe, expect, it, vi } from "vitest";
import { APP_SIGNED_OUT_KEY, type AppAccountClients, appAccount, keychainArea } from "@/app/account.ts";
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
  return { auth, flags, onlineTranslation, all: { auth, flags, onlineTranslation } as unknown as AppAccountClients };
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
  over: { sheet?: unknown; keychain?: string | null; chosen?: boolean; relayedAccess?: boolean; access?: boolean } = {},
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
