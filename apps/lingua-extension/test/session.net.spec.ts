import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Client } from "@connectrpc/connect";
import type { AuthService } from "@/gen/auth_pb";
import { Session } from "@/state/session.ts";
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

/** A fake AuthService that records calls and returns scripted token pairs. */
function fakeAuth(pairs: { local?: Pair; oidc?: Pair; refresh?: Pair | "fail" }) {
  const calls = {
    signInLocal: vi.fn(),
    signInOidc: vi.fn(),
    refresh: vi.fn(),
    logout: vi.fn(),
  };
  const client = {
    async signInLocal(req: unknown) {
      calls.signInLocal(req);
      return pairs.local ?? { accessToken: "a-local", refreshToken: "r-local" };
    },
    async signInOidc(req: unknown) {
      calls.signInOidc(req);
      return pairs.oidc ?? { accessToken: "a-oidc", refreshToken: "r-oidc" };
    },
    async refresh(req: unknown) {
      calls.refresh(req);
      if (pairs.refresh === "fail") throw new Error("refresh rejected");
      return pairs.refresh ?? { accessToken: "a-refreshed", refreshToken: "r-refreshed" };
    },
    async logout(req: unknown) {
      calls.logout(req);
      return {};
    },
  };
  return { client: client as unknown as Client<typeof AuthService>, calls };
}

function makeSession(auth: Client<typeof AuthService>, getGoogleIdToken = async () => "id-token") {
  const sessionArea = fakeArea();
  const localArea = fakeArea();
  const session = new Session({ auth: () => auth, sessionArea, localArea, getGoogleIdToken });
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
    const session = new Session({
      auth: () => client,
      sessionArea: fakeArea(),
      localArea: fakeArea(),
      getGoogleIdToken,
    });
    await session.signInWithGoogle();
    expect(getGoogleIdToken).toHaveBeenCalledOnce();
    expect(calls.signInOidc).toHaveBeenCalledWith({ idToken: "google-id-token", audience: "lingua" });
    expect(session.token()).toBe("G");
  });

  it("resumes from a cached access token without refreshing", async () => {
    const { client, calls } = fakeAuth({});
    const session = new Session({
      auth: () => client,
      sessionArea: fakeArea({ [ACCESS_KEY]: "CACHED" }),
      localArea: fakeArea({ [REFRESH_KEY]: "R" }),
      getGoogleIdToken: async () => "x",
    });
    expect(await session.resume()).toBe(true);
    expect(session.token()).toBe("CACHED");
    expect(calls.refresh).not.toHaveBeenCalled();
  });

  it("resumes by minting a new access token when only the refresh token survived", async () => {
    // Service-worker restart: session area empty, refresh token still in local.
    const { client, calls } = fakeAuth({ refresh: { accessToken: "NEW", refreshToken: "R2" } });
    const session = new Session({
      auth: () => client,
      sessionArea: fakeArea(),
      localArea: fakeArea({ [REFRESH_KEY]: "R" }),
      getGoogleIdToken: async () => "x",
    });
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
    const session = new Session({
      auth: () => client,
      sessionArea: fakeArea({ [ACCESS_KEY]: "OLD" }),
      localArea: fakeArea({ [REFRESH_KEY]: "R" }),
      getGoogleIdToken: async () => "x",
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
});
