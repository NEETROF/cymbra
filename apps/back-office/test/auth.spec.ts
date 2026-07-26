import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { decodeClaims, isAdmin, isModerator } from "@/lib/jwt";
import { useAuthStore } from "@/stores/auth";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients, makeJwt } from "./fakes";

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

describe("auth store sign-in", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("exchanges credentials and exposes the token's roles", async () => {
    const { clients } = makeFakeClients({
      tokens: { accessToken: makeJwt({ roles: ["moderator"], sub: "m1" }), refreshToken: "r" },
    });
    setClientsForTest(clients);
    const auth = useAuthStore();
    expect(auth.isAuthenticated).toBe(false);

    await auth.signInLocal("mod@x.dev", "pw");

    expect(auth.isAuthenticated).toBe(true);
    expect(auth.isModerator).toBe(true);
    expect(auth.isAdmin).toBe(false);
    expect(auth.userId).toBe("m1");
    // Tokens are persisted for reload.
    expect(localStorage.getItem("cymbra.bo.tokens")).toContain("accessToken");
  });

  it("a signed-in non-moderator is not granted console access", async () => {
    const { clients } = makeFakeClients({
      tokens: { accessToken: makeJwt({ roles: ["user"], sub: "u1" }), refreshToken: "r" },
    });
    setClientsForTest(clients);
    const auth = useAuthStore();
    await auth.signInLocal("user@x.dev", "pw");
    expect(auth.isAuthenticated).toBe(true);
    expect(auth.isModerator).toBe(false);
  });

  it("signOut clears tokens", async () => {
    const { clients } = makeFakeClients();
    setClientsForTest(clients);
    const auth = useAuthStore();
    await auth.signInLocal("m@x.dev", "pw");
    auth.signOut();
    expect(auth.isAuthenticated).toBe(false);
    expect(localStorage.getItem("cymbra.bo.tokens")).toBeNull();
  });
});
