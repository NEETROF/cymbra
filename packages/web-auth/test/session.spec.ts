import { describe, expect, it, vi } from "vitest";
import { createWebSession } from "../src/session";
import { WebAuthError, type WebAuthClient } from "../src/client";

function fakeClient(over: Partial<WebAuthClient> = {}): WebAuthClient {
  return {
    signInLocal: vi.fn(async () => ({ accessToken: "local-tok" })),
    signInOidc: vi.fn(async () => ({ accessToken: "oidc-tok" })),
    refresh: vi.fn(async () => ({ accessToken: "fresh-tok" })),
    logout: vi.fn(async () => undefined),
    ...over,
  };
}

describe("web session", () => {
  it("starts signed out; boot re-mints silently from the cookie", async () => {
    const s = createWebSession(fakeClient(), "web");
    expect(s.state.value.kind).toBe("signedOut");
    expect(s.accessToken.value).toBeNull();
    await s.boot();
    expect(s.state.value).toEqual({ kind: "signedIn", accessToken: "fresh-tok" });
    expect(s.accessToken.value).toBe("fresh-tok");
    // Boot is not an "attempt": the form's error slot stays idle.
    expect(s.attempt.value.status).toBe("idle");
  });

  it("boot without a cookie stays signed out without an error", async () => {
    const client = fakeClient({
      refresh: vi.fn(async () => {
        throw new WebAuthError(401, "no session");
      }),
    });
    const s = createWebSession(client, "web");
    await s.boot();
    expect(s.state.value.kind).toBe("signedOut");
    expect(s.attempt.value.status).toBe("idle");
  });

  it("signs in with email and with an OIDC token for the audience", async () => {
    const client = fakeClient();
    const s = createWebSession(client, "web");
    const out = await s.signInLocal("a@x.dev", "pw");
    expect(out.status).toBe("success");
    expect(client.signInLocal).toHaveBeenCalledWith("a@x.dev", "pw", "web");
    expect(s.accessToken.value).toBe("local-tok");

    await s.signInOidc("id-token");
    expect(client.signInOidc).toHaveBeenCalledWith("id-token", "web");
    expect(s.accessToken.value).toBe("oidc-tok");
  });

  it("a failed sign-in lands in the attempt union (mapped), session unchanged", async () => {
    const client = fakeClient({
      signInLocal: vi.fn(async () => {
        throw new WebAuthError(401, "invalid credentials");
      }),
    });
    const s = createWebSession(client, "web", (e) => `mapped:${(e as Error).message}`);
    const out = await s.signInLocal("a@x.dev", "wrong");
    expect(out).toEqual({ status: "error", error: "mapped:invalid credentials" });
    expect(s.state.value.kind).toBe("signedOut");
  });

  it("refresh signs out when the cookie is gone; sign-out revokes and clears", async () => {
    const client = fakeClient();
    const s = createWebSession(client, "web");
    await s.signInLocal("a@x.dev", "pw");
    (client.refresh as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new WebAuthError(401, "session ended"));
    expect(await s.refresh()).toBeNull();
    expect(s.state.value.kind).toBe("signedOut");

    await s.signInLocal("a@x.dev", "pw");
    await s.signOut();
    expect(client.logout).toHaveBeenCalled();
    expect(s.state.value.kind).toBe("signedOut");
    expect(s.attempt.value.status).toBe("idle");
  });
});
