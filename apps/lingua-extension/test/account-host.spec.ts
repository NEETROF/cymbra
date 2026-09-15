import { describe, expect, it, vi } from "vitest";
import { type AccountHostDeps, handleAccountMessage } from "@/account/host.ts";
import { isAccountMessage } from "@/account/messages.ts";
import { AccountError } from "@/state/auth-errors.ts";

function setup(overrides: Partial<AccountHostDeps["session"]> = {}) {
  let signedIn = false;
  const session = {
    state: vi.fn(() => ({ signedIn })),
    signInLocal: vi.fn(async () => {
      signedIn = true;
    }),
    signInWithProvider: vi.fn(async () => {
      signedIn = true;
      return "signedIn" as const;
    }),
    signInWithIdToken: vi.fn(async () => {
      signedIn = true;
    }),
    signOut: vi.fn(async () => {
      signedIn = false;
    }),
    signUp: vi.fn(async () => {}),
    verifyEmail: vi.fn(async () => {}),
    resendVerification: vi.fn(async () => {}),
    requestPasswordReset: vi.fn(async () => {}),
    resetPassword: vi.fn(async () => {}),
    ...overrides,
  };
  const deps = {
    session,
    account: {
      profile: vi.fn(async () => ({ handle: "alice", version: 1, displayName: null, preferences: "{}" })),
      checkHandle: vi.fn(async () => true),
      setHandle: vi.fn(async () => ({ handle: "alice", version: 2, displayName: null, preferences: "{}" })),
      deleteAccount: vi.fn(async () => {}),
    },
    providers: vi.fn(() => ({ google: true, apple: false })),
    onSignedIn: vi.fn(),
  };
  return { session, deps };
}

describe("handleAccountMessage", () => {
  it("answers the session state and the providers", async () => {
    const { deps } = setup();
    expect(await handleAccountMessage({ type: "account:state" }, deps)).toEqual({
      ok: true,
      state: { signedIn: false },
    });
    expect(await handleAccountMessage({ type: "account:providers" }, deps)).toEqual({
      ok: true,
      providers: { google: true, apple: false },
    });
  });

  it("signs in by email and schedules a sync", async () => {
    const { deps, session } = setup();
    const reply = await handleAccountMessage({ type: "account:signInLocal", email: "a@b.c", password: "pw" }, deps);
    expect(session.signInLocal).toHaveBeenCalledWith("a@b.c", "pw");
    expect(reply).toEqual({ ok: true, state: { signedIn: true } });
    expect(deps.onSignedIn).toHaveBeenCalledOnce();
  });

  it("routes Google and Apple to their provider", async () => {
    const { deps, session } = setup();
    await handleAccountMessage({ type: "account:signInApple" }, deps);
    expect(session.signInWithProvider).toHaveBeenLastCalledWith("apple");
    await handleAccountMessage({ type: "account:signInGoogle" }, deps);
    expect(session.signInWithProvider).toHaveBeenLastCalledWith("google");
    expect(deps.onSignedIn).toHaveBeenCalledTimes(2);
  });

  it("reports a cancelled provider flow without scheduling a sync", async () => {
    const { deps } = setup({ signInWithProvider: vi.fn(async () => "cancelled" as const) });
    const reply = await handleAccountMessage({ type: "account:signInApple" }, deps);
    expect(reply).toEqual({ ok: false, cancelled: true, state: { signedIn: false } });
    expect(deps.onSignedIn).not.toHaveBeenCalled();
  });

  describe("on Safari, through the host app", () => {
    function withHandOff(handed: { provider: "apple" | "google"; idToken: string } | null, session = setup()) {
      const handOff = { open: vi.fn(async () => {}), take: vi.fn(async () => handed) };
      return { ...session, deps: { ...session.deps, handOff }, handOff };
    }

    it("opens the host app instead of a provider flow", async () => {
      const { deps, session, handOff } = withHandOff(null);
      const reply = await handleAccountMessage({ type: "account:signInGoogle" }, deps);
      expect(handOff.open).toHaveBeenCalledWith("google");
      expect(session.signInWithProvider).not.toHaveBeenCalled();
      expect(reply).toEqual({ ok: false, handedOff: true, state: { signedIn: false } });
      expect(deps.onSignedIn).not.toHaveBeenCalled();
    });

    it("signs in with a handed-over id_token and schedules a sync", async () => {
      const { deps, session } = withHandOff({ provider: "apple", idToken: "tok" });
      const reply = await handleAccountMessage({ type: "account:collectHandedToken" }, deps);
      expect(session.signInWithIdToken).toHaveBeenCalledWith("apple", "tok");
      expect(reply).toEqual({ ok: true, state: { signedIn: true }, provider: "apple" });
      expect(deps.onSignedIn).toHaveBeenCalledOnce();
    });

    it("leaves the state alone when nothing was handed over", async () => {
      const { deps, session } = withHandOff(null);
      expect(await handleAccountMessage({ type: "account:collectHandedToken" }, deps)).toEqual({
        ok: true,
        state: { signedIn: false },
      });
      expect(session.signInWithIdToken).not.toHaveBeenCalled();
    });

    it("reports a rejected id_token with its provider", async () => {
      const failing = setup({
        signInWithIdToken: vi.fn(async () => {
          throw new AccountError("unauthenticated");
        }),
      });
      const { deps } = withHandOff({ provider: "google", idToken: "tok" }, failing);
      expect(await handleAccountMessage({ type: "account:collectHandedToken" }, deps)).toEqual({
        ok: false,
        error: "unauthenticated",
        provider: "google",
      });
      expect(deps.onSignedIn).not.toHaveBeenCalled();
    });

    it("collects nothing where there is no host app", async () => {
      const { deps, session } = setup();
      expect(await handleAccountMessage({ type: "account:collectHandedToken" }, deps)).toEqual({
        ok: true,
        state: { signedIn: false },
      });
      expect(session.signInWithIdToken).not.toHaveBeenCalled();
    });
  });

  it("passes sign-up, verification and reset through without signing in", async () => {
    const { deps, session } = setup();
    expect(
      await handleAccountMessage({ type: "account:signUp", email: "a@b.c", password: "pw", locale: "fr" }, deps),
    ).toEqual({ ok: true });
    expect(session.signUp).toHaveBeenCalledWith("a@b.c", "pw", "fr");
    await handleAccountMessage({ type: "account:verifyEmail", code: "123" }, deps);
    expect(session.verifyEmail).toHaveBeenCalledWith("123");
    await handleAccountMessage({ type: "account:resendVerification", email: "a@b.c", locale: "fr" }, deps);
    expect(session.resendVerification).toHaveBeenCalledWith("a@b.c", "fr");
    await handleAccountMessage({ type: "account:requestPasswordReset", email: "a@b.c", locale: "en" }, deps);
    expect(session.requestPasswordReset).toHaveBeenCalledWith("a@b.c", "en");
    await handleAccountMessage({ type: "account:resetPassword", code: "9", newPassword: "np" }, deps);
    expect(session.resetPassword).toHaveBeenCalledWith("9", "np");
    expect(deps.onSignedIn).not.toHaveBeenCalled();
  });

  it("signs out", async () => {
    const { deps, session } = setup();
    expect(await handleAccountMessage({ type: "account:signOut" }, deps)).toEqual({
      ok: true,
      state: { signedIn: false },
    });
    expect(session.signOut).toHaveBeenCalledOnce();
  });

  it("replies with the failure's category, never a message", async () => {
    const { deps } = setup({
      signUp: vi.fn(async () => {
        throw new AccountError("alreadyExists");
      }),
    });
    const reply = await handleAccountMessage(
      { type: "account:signUp", email: "a@b.c", password: "pw", locale: "fr" },
      deps,
    );
    expect(reply).toEqual({ ok: false, error: "alreadyExists" });
  });
});

describe("isAccountMessage", () => {
  it("recognizes account:* messages only", () => {
    expect(isAccountMessage({ type: "account:state" })).toBe(true);
    expect(isAccountMessage({ type: "stats:get" })).toBe(false);
    expect(isAccountMessage(null)).toBe(false);
  });
});
