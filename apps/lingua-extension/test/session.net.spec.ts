import { beforeEach, describe, expect, it, type Mock, vi } from "vitest";
import { Code, ConnectError, type Client } from "@connectrpc/connect";
import type { AuthService } from "@/gen/auth_pb";
import { AccountError } from "@/state/auth-errors.ts";
import { isPersistedSignInError, Session, SIGNIN_ERROR_KEY } from "@/state/session.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";

// A store-backed storage area (mirrors the one in storage.spec).
function fakeArea(seed: Record<string, unknown> = {}): AsyncStorageArea & { store: Record<string, unknown> } {
  const store: Record<string, unknown> = { ...seed };
  return {
    store,
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const k of list) if (k in store) out[k] = store[k];
      return out;
    },
    async set(items) {
      Object.assign(store, items);
    },
  };
}

interface Pair {
  accessToken: string;
  refreshToken: string;
}

type Method =
  | "signInLocal"
  | "signInOidc"
  | "refresh"
  | "logout"
  | "signUpLocal"
  | "verifyEmail"
  | "resendVerification"
  | "requestPasswordReset"
  | "resetPassword";

/** A fake AuthService that records calls, returns scripted token pairs, and can fail. */
function fakeAuth(
  pairs: { local?: Pair; oidc?: Pair; refresh?: Pair | "fail" },
  fail: Partial<Record<Method, Code>> = {},
) {
  const calls = Object.fromEntries(
    (
      [
        "signInLocal",
        "signInOidc",
        "refresh",
        "logout",
        "signUpLocal",
        "verifyEmail",
        "resendVerification",
        "requestPasswordReset",
        "resetPassword",
      ] as Method[]
    ).map((m) => [m, vi.fn<(req: unknown) => void>()]),
  ) as Record<Method, Mock<(req: unknown) => void>>;
  const call = async <T>(method: Method, req: unknown, result: T): Promise<T> => {
    calls[method](req);
    const code = fail[method];
    if (code != null) throw new ConnectError(`${method} failed`, code);
    return result;
  };
  const client = {
    signInLocal: (req: unknown) =>
      call("signInLocal", req, pairs.local ?? { accessToken: "a-local", refreshToken: "r-local" }),
    signInOidc: (req: unknown) =>
      call("signInOidc", req, pairs.oidc ?? { accessToken: "a-oidc", refreshToken: "r-oidc" }),
    async refresh(req: unknown) {
      calls.refresh(req);
      if (pairs.refresh === "fail") throw new Error("refresh rejected");
      return pairs.refresh ?? { accessToken: "a-refreshed", refreshToken: "r-refreshed" };
    },
    logout: (req: unknown) => call("logout", req, {}),
    signUpLocal: (req: unknown) => call("signUpLocal", req, {}),
    verifyEmail: (req: unknown) => call("verifyEmail", req, {}),
    resendVerification: (req: unknown) => call("resendVerification", req, {}),
    requestPasswordReset: (req: unknown) => call("requestPasswordReset", req, {}),
    resetPassword: (req: unknown) => call("resetPassword", req, {}),
  };
  return { client: client as unknown as Client<typeof AuthService>, calls };
}

function makeSession(
  auth: Client<typeof AuthService>,
  opts: {
    getGoogleIdToken?: () => Promise<string | null>;
    getAppleIdToken?: () => Promise<string | null>;
    sessionSeed?: Record<string, unknown>;
    localSeed?: Record<string, unknown>;
  } = {},
) {
  const sessionArea = fakeArea(opts.sessionSeed);
  const localArea = fakeArea(opts.localSeed);
  const session = new Session({
    auth: () => auth,
    sessionArea,
    localArea,
    getGoogleIdToken: opts.getGoogleIdToken ?? (async () => "google-id-token"),
    getAppleIdToken: opts.getAppleIdToken ?? (async () => "apple-id-token"),
  });
  return { session, sessionArea, localArea };
}

const ACCESS_KEY = "cymbra-lingua-access";
const REFRESH_KEY = "cymbra-lingua-refresh";

beforeEach(() => {
  vi.clearAllMocks();
});

describe("Session", () => {
  it("stores the access token in the session area and the refresh token in local", async () => {
    const { client } = fakeAuth({ local: { accessToken: "AAA", refreshToken: "RRR" } });
    const { session, sessionArea, localArea } = makeSession(client);
    await session.signInLocal("me@example.com", "pw");
    expect(session.token()).toBe("AAA");
    expect(session.state().signedIn).toBe(true);
    expect(sessionArea.store[ACCESS_KEY]).toBe("AAA");
    expect(localArea.store[REFRESH_KEY]).toBe("RRR");
  });

  it("signs in with Google via the id_token exchange", async () => {
    const { client, calls } = fakeAuth({ oidc: { accessToken: "G", refreshToken: "GR" } });
    const getGoogleIdToken = vi.fn(async () => "google-id-token");
    const { session } = makeSession(client, { getGoogleIdToken });
    expect(await session.signInWithProvider("google")).toBe("signedIn");
    expect(getGoogleIdToken).toHaveBeenCalledOnce();
    expect(calls.signInOidc).toHaveBeenCalledWith({ idToken: "google-id-token", audience: "lingua" });
    expect(session.token()).toBe("G");
  });

  it("signs in with Apple through the same exchange, lingua audience", async () => {
    const { client, calls } = fakeAuth({ oidc: { accessToken: "A", refreshToken: "AR" } });
    const getAppleIdToken = vi.fn(async () => "apple-id-token");
    const getGoogleIdToken = vi.fn(async () => "never");
    const { session } = makeSession(client, { getAppleIdToken, getGoogleIdToken });
    expect(await session.signInWithProvider("apple")).toBe("signedIn");
    expect(getGoogleIdToken).not.toHaveBeenCalled();
    expect(calls.signInOidc).toHaveBeenCalledWith({ idToken: "apple-id-token", audience: "lingua" });
    expect(session.state().signedIn).toBe(true);
  });

  it("treats a closed provider window as a cancel: no RPC, nothing persisted", async () => {
    const { client, calls } = fakeAuth({});
    const { session, sessionArea } = makeSession(client, { getAppleIdToken: async () => null });
    expect(await session.signInWithProvider("apple")).toBe("cancelled");
    expect(calls.signInOidc).not.toHaveBeenCalled();
    expect(sessionArea.store[SIGNIN_ERROR_KEY]).toBeUndefined();
    expect(session.state().signedIn).toBe(false);
  });

  it("persists a provider failure as a category and clears it on the next success", async () => {
    const failing = fakeAuth({}, { signInOidc: Code.Unauthenticated });
    const { session, sessionArea } = makeSession(failing.client);
    const err = await session.signInWithProvider("apple").catch((e: unknown) => e);
    expect(err).toBeInstanceOf(AccountError);
    expect((err as AccountError).kind).toBe("unauthenticated");
    expect(sessionArea.store[SIGNIN_ERROR_KEY]).toEqual({ provider: "apple", kind: "unauthenticated" });
    expect(isPersistedSignInError(sessionArea.store[SIGNIN_ERROR_KEY])).toBe(true);

    const ok = fakeAuth({});
    const again = new Session({
      auth: () => ok.client,
      sessionArea,
      localArea: fakeArea(),
      getGoogleIdToken: async () => "g",
      getAppleIdToken: async () => "a",
    });
    await again.signInWithProvider("google");
    expect(sessionArea.store[SIGNIN_ERROR_KEY]).toBeNull();
  });

  it("categorizes a failed provider flow that never reached the backend", async () => {
    const { client } = fakeAuth({});
    const { session, sessionArea } = makeSession(client, {
      getGoogleIdToken: async () => {
        throw new Error("provider state mismatch");
      },
    });
    await expect(session.signInWithProvider("google")).rejects.toMatchObject({ kind: "unknown" });
    expect(sessionArea.store[SIGNIN_ERROR_KEY]).toEqual({ provider: "google", kind: "unknown" });
  });

  it("maps a local sign-in failure to a category without persisting it", async () => {
    const { client } = fakeAuth({}, { signInLocal: Code.FailedPrecondition });
    const { session, sessionArea } = makeSession(client);
    await expect(session.signInLocal("me@example.com", "pw")).rejects.toMatchObject({ kind: "failedPrecondition" });
    expect(sessionArea.store[SIGNIN_ERROR_KEY]).toBeUndefined();
    expect(session.state().signedIn).toBe(false);
  });

  it("signs up with the locale, verifies with the code as the token", async () => {
    const { client, calls } = fakeAuth({});
    const { session } = makeSession(client);
    await session.signUp("new@example.com", "long-password", "fr-FR");
    expect(calls.signUpLocal).toHaveBeenCalledWith({
      email: "new@example.com",
      password: "long-password",
      locale: "fr-FR",
    });
    await session.verifyEmail("123456");
    expect(calls.verifyEmail).toHaveBeenCalledWith({ token: "123456" });
    await session.resendVerification("new@example.com", "fr-FR");
    expect(calls.resendVerification).toHaveBeenCalledWith({ email: "new@example.com", locale: "fr-FR" });
    // Sign-up alone never signs in.
    expect(session.state().signedIn).toBe(false);
  });

  it("requests and completes a password reset", async () => {
    const { client, calls } = fakeAuth({});
    const { session } = makeSession(client);
    await session.requestPasswordReset("me@example.com", "en");
    expect(calls.requestPasswordReset).toHaveBeenCalledWith({ email: "me@example.com", locale: "en" });
    await session.resetPassword("654321", "new-password");
    expect(calls.resetPassword).toHaveBeenCalledWith({ token: "654321", newPassword: "new-password" });
  });

  it("rethrows sign-up failures as categories", async () => {
    const { client } = fakeAuth({}, { signUpLocal: Code.AlreadyExists, resetPassword: Code.InvalidArgument });
    const { session } = makeSession(client);
    await expect(session.signUp("taken@example.com", "pw", "fr")).rejects.toMatchObject({ kind: "alreadyExists" });
    await expect(session.resetPassword("bad", "pw")).rejects.toMatchObject({ kind: "invalidArgument" });
  });

  it("resumes from a cached access token without refreshing", async () => {
    const { client, calls } = fakeAuth({});
    const { session } = makeSession(client, {
      sessionSeed: { [ACCESS_KEY]: "CACHED" },
      localSeed: { [REFRESH_KEY]: "R" },
    });
    expect(await session.resume()).toBe(true);
    expect(session.token()).toBe("CACHED");
    expect(calls.refresh).not.toHaveBeenCalled();
  });

  it("resumes by minting a new access token when only the refresh token survived", async () => {
    // Service-worker restart: session area empty, refresh token still in local.
    const { client, calls } = fakeAuth({ refresh: { accessToken: "NEW", refreshToken: "R2" } });
    const { session } = makeSession(client, { localSeed: { [REFRESH_KEY]: "R" } });
    expect(await session.resume()).toBe(true);
    expect(calls.refresh).toHaveBeenCalledWith({ refreshToken: "R" });
    expect(session.token()).toBe("NEW");
  });

  it("resumes to signed-out when there is nothing stored", async () => {
    const { client } = fakeAuth({});
    const { session } = makeSession(client);
    expect(await session.resume()).toBe(false);
    expect(session.state().signedIn).toBe(false);
  });

  it("purges the session when a refresh fails", async () => {
    const { client } = fakeAuth({ refresh: "fail" });
    const { session } = makeSession(client, {
      sessionSeed: { [ACCESS_KEY]: "OLD" },
      localSeed: { [REFRESH_KEY]: "R" },
    });
    expect(await session.refresh()).toBe(false);
    expect(session.token()).toBeNull();
    expect(session.state().signedIn).toBe(false);
  });

  it("revokes the refresh token and purges on sign-out", async () => {
    const { client, calls } = fakeAuth({ local: { accessToken: "A", refreshToken: "R" } });
    const { session, localArea } = makeSession(client);
    await session.signInLocal("me@example.com", "pw");
    await session.signOut();
    expect(calls.logout).toHaveBeenCalledWith({ refreshToken: "R" });
    expect(session.state().signedIn).toBe(false);
    expect(session.token()).toBeNull();
    expect(localArea.store[REFRESH_KEY]).toBeNull();
  });

  it("purges locally even when the server logout fails", async () => {
    const { client } = fakeAuth({}, { logout: Code.Unavailable });
    const { session } = makeSession(client);
    await session.signInLocal("me@example.com", "pw");
    await session.signOut();
    expect(session.state().signedIn).toBe(false);
  });
});

describe("isPersistedSignInError", () => {
  it("accepts only a provider + category", () => {
    expect(isPersistedSignInError({ provider: "google", kind: "unknown" })).toBe(true);
    expect(isPersistedSignInError("Google did not return an id_token.")).toBe(false);
    expect(isPersistedSignInError({ provider: "github", kind: "unknown" })).toBe(false);
    expect(isPersistedSignInError(null)).toBe(false);
  });
});
