import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  createWebAuthClient,
  initWebAuth,
  setWebAuthClientForTest,
  webAuth,
  WebAuthError,
  webAuthBaseUrl,
  type WebAuthClient,
} from "@/lib/web-auth";

// A minimal stand-in for the parts of `Response` the client reads.
function fakeResponse(init: { ok: boolean; status?: number; statusText?: string; body?: unknown }): Response {
  return {
    ok: init.ok,
    status: init.status ?? (init.ok ? 200 : 400),
    statusText: init.statusText ?? "",
    json: async () => {
      if (init.body === undefined) throw new Error("no body");
      return init.body;
    },
  } as unknown as Response;
}

const fetchMock = vi.fn();

beforeEach(() => {
  fetchMock.mockReset();
  vi.stubGlobal("fetch", fetchMock);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("web-auth client (real fetch wiring)", () => {
  it("signs in locally: credentialed JSON POST with the CSRF header, returns the access token", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: true, body: { accessToken: "acc-1" } }));
    const client = createWebAuthClient();

    const tokens = await client.signInLocal("mod@x.dev", "pw", "music");

    expect(tokens).toEqual({ accessToken: "acc-1" });
    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [url, opts] = fetchMock.mock.calls[0];
    expect(url).toBe(`${webAuthBaseUrl()}/web/auth/signin`);
    expect(opts.method).toBe("POST");
    expect(opts.credentials).toBe("include");
    expect(opts.headers["Content-Type"]).toBe("application/json");
    expect(opts.headers["X-Cymbra-Web"]).toBe("1");
    expect(JSON.parse(opts.body)).toEqual({
      kind: "local",
      email: "mod@x.dev",
      password: "pw",
      audience: "music",
    });
  });

  it("signs in via OIDC with the id_token payload", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: true, body: { accessToken: "acc-2" } }));
    const client = createWebAuthClient();

    const tokens = await client.signInOidc("google-id-token", "music");

    expect(tokens).toEqual({ accessToken: "acc-2" });
    const [url, opts] = fetchMock.mock.calls[0];
    expect(url).toBe(`${webAuthBaseUrl()}/web/auth/signin`);
    expect(JSON.parse(opts.body)).toEqual({ kind: "oidc", idToken: "google-id-token", audience: "music" });
  });

  it("refreshes from the cookie with no body (the server reads the cookie)", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: true, body: { accessToken: "acc-3" } }));
    const client = createWebAuthClient();

    const tokens = await client.refresh();

    expect(tokens).toEqual({ accessToken: "acc-3" });
    const [url, opts] = fetchMock.mock.calls[0];
    expect(url).toBe(`${webAuthBaseUrl()}/web/auth/refresh`);
    expect(opts.credentials).toBe("include");
    expect(opts.body).toBeUndefined();
  });

  it("defaults accessToken to empty string when the body omits it", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: true, body: {} }));
    const client = createWebAuthClient();
    expect(await client.refresh()).toEqual({ accessToken: "" });
  });

  it("throws a WebAuthError carrying the HTTP status and the server's error message", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: false, status: 401, body: { error: "session ended" } }));
    const client = createWebAuthClient();

    await expect(client.refresh()).rejects.toMatchObject({ status: 401, message: "session ended" });
    await expect(client.refresh()).rejects.toBeInstanceOf(WebAuthError);
  });

  it("falls back to the status text when the error body is not JSON", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: false, status: 500, statusText: "Internal Server Error" }));
    const client = createWebAuthClient();

    await expect(client.signInLocal("a", "b", "music")).rejects.toMatchObject({
      status: 500,
      message: "Internal Server Error",
    });
  });

  it("single-flights concurrent refreshes into ONE request (avoids self-inflicted reuse)", async () => {
    // Two refreshes racing on the same rotating cookie would each rotate it; the second
    // replays an already-rotated token and the server revokes the session. The client
    // must dedupe concurrent calls into a single in-flight request.
    let calls = 0;
    fetchMock.mockImplementation(async () => {
      calls += 1;
      return fakeResponse({ ok: true, body: { accessToken: `acc-${calls}` } });
    });
    const client = createWebAuthClient();

    const [a, b] = await Promise.all([client.refresh(), client.refresh()]);
    expect(calls).toBe(1);
    expect(a).toEqual({ accessToken: "acc-1" });
    expect(b).toEqual({ accessToken: "acc-1" });

    // Once settled, a subsequent refresh issues a fresh request (not a stale cache).
    await client.refresh();
    expect(calls).toBe(2);
  });

  it("logout is best-effort: it resolves even if the request rejects", async () => {
    fetchMock.mockRejectedValue(new Error("network down"));
    const client = createWebAuthClient();
    await expect(client.logout()).resolves.toBeUndefined();
    expect(fetchMock).toHaveBeenCalledWith(`${webAuthBaseUrl()}/web/auth/logout`, expect.anything());
  });
});

describe("web-auth injectable seam", () => {
  it("throws when used before init", () => {
    // First test in this (isolated) module: the singleton starts unset.
    expect(() => webAuth()).toThrow(/before initWebAuth/);
  });

  it("initWebAuth wires a real client with all four methods", () => {
    initWebAuth();
    const c = webAuth();
    expect(typeof c.signInLocal).toBe("function");
    expect(typeof c.signInOidc).toBe("function");
    expect(typeof c.refresh).toBe("function");
    expect(typeof c.logout).toBe("function");
  });

  it("setWebAuthClientForTest swaps in a fake", () => {
    const fake: WebAuthClient = {
      signInLocal: async () => ({ accessToken: "fake" }),
      signInOidc: async () => ({ accessToken: "fake" }),
      refresh: async () => ({ accessToken: "fake" }),
      logout: async () => undefined,
    };
    setWebAuthClientForTest(fake);
    expect(webAuth()).toBe(fake);
  });
});
