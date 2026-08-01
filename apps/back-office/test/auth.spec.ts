import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { decodeClaims, isAdmin, isModerator } from "@/lib/jwt";
import { useAuthStore } from "@/stores/auth";
import { setWebAuthClientForTest, WebAuthError, type WebAuthClient } from "@/lib/web-auth";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients, makeJwt } from "./fakes";

// A fake web-auth client (the injectable seam the store depends on). Override just the
// method a test cares about; the rest return a default moderator token.
function fakeWebAuth(over: Partial<WebAuthClient> = {}): WebAuthClient {
  const token = () => ({ accessToken: makeJwt({ roles: ["moderator"], sub: "m1" }) });
  return {
    signInLocal: async () => token(),
    signInOidc: async () => token(),
    refresh: async () => token(),
    logout: async () => undefined,
    ...over,
  };
}

describe("jwt role decoding", () => {
  it("reads roles from the payload", () => {
    const claims = decodeClaims(makeJwt({ roles: ["user", "moderator"], sub: "u9" }));
    expect(claims.roles).toEqual(["user", "moderator"]);
    expect(claims.sub).toBe("u9");
  });

  it("gates moderator vs admin vs normal", () => {
    expect(isModerator(["user", "moderator"])).toBe(true);
    expect(isModerator(["user", "admin"])).toBe(true);
    expect(isModerator(["user"])).toBe(false);
    expect(isAdmin(["user", "moderator"])).toBe(false);
    expect(isAdmin(["admin"])).toBe(true);
  });

  it("tolerates a malformed token", () => {
    expect(decodeClaims("garbage").roles).toEqual([]);
  });
});

describe("auth store — memory-only session", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
    sessionStorage.clear();
    // A successful sign-in/bootstrap now reconciles the account language over gRPC
    // (change: sync-account-language-preference), so the API seam must be wired.
    setClientsForTest(makeFakeClients().clients);
  });

  it("keeps the access token in memory and writes nothing to web storage on sign-in", async () => {
    setWebAuthClientForTest(
      fakeWebAuth({ signInLocal: async () => ({ accessToken: makeJwt({ roles: ["moderator"], sub: "m1" }) }) }),
    );
    const auth = useAuthStore();
    expect(auth.isAuthenticated).toBe(false);

    await auth.signInLocal("mod@x.dev", "pw");

    expect(auth.isAuthenticated).toBe(true);
    expect(auth.isModerator).toBe(true);
    expect(auth.isAdmin).toBe(false);
    expect(auth.userId).toBe("m1");
    // The whole point: no token in any JS-readable storage.
    expect(localStorage.length).toBe(0);
    expect(sessionStorage.length).toBe(0);
  });

  it("a signed-in non-moderator is not granted console access", async () => {
    setWebAuthClientForTest(
      fakeWebAuth({ signInLocal: async () => ({ accessToken: makeJwt({ roles: ["user"], sub: "u1" }) }) }),
    );
    const auth = useAuthStore();
    await auth.signInLocal("user@x.dev", "pw");
    expect(auth.isAuthenticated).toBe(true);
    expect(auth.isModerator).toBe(false);
  });

  it("signOut revokes the server session and clears the in-memory token", async () => {
    const logout = vi.fn(async () => undefined);
    setWebAuthClientForTest(fakeWebAuth({ logout }));
    const auth = useAuthStore();
    await auth.signInLocal("m@x.dev", "pw");
    expect(auth.isAuthenticated).toBe(true);

    await auth.signOut();

    expect(auth.isAuthenticated).toBe(false);
    expect(logout).toHaveBeenCalledTimes(1);
    expect(localStorage.length).toBe(0);
  });

  it("bootstrap re-mints an access token from the refresh cookie", async () => {
    setWebAuthClientForTest(
      fakeWebAuth({ refresh: async () => ({ accessToken: makeJwt({ roles: ["admin"], sub: "a1" }) }) }),
    );
    const auth = useAuthStore();
    await auth.bootstrap();
    expect(auth.isAuthenticated).toBe(true);
    expect(auth.isAdmin).toBe(true);
    expect(auth.bootstrapped).toBe(true);
  });

  it("bootstrap with no session stays signed out and does not throw", async () => {
    setWebAuthClientForTest(
      fakeWebAuth({
        refresh: async () => {
          throw new WebAuthError(401, "no session");
        },
      }),
    );
    const auth = useAuthStore();
    await auth.bootstrap();
    expect(auth.isAuthenticated).toBe(false);
    expect(auth.bootstrapped).toBe(true);
  });
});
