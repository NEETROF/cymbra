import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createWebAuthClient, fetchJson, WebAuthError } from "../src/client";

const BASE = "https://api.example";

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
  globalThis.localStorage?.clear();
  globalThis.sessionStorage?.clear();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("web-auth client", () => {
  it("signs in locally: credentialed JSON POST with the CSRF header, token in memory only", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: true, body: { accessToken: "acc-1" } }));
    const client = createWebAuthClient(BASE);

    const tokens = await client.signInLocal("a@x.dev", "pw", "web");

    expect(tokens).toEqual({ accessToken: "acc-1" });
    const [url, opts] = fetchMock.mock.calls[0];
    expect(url).toBe(`${BASE}/web/auth/signin`);
    expect(opts.method).toBe("POST");
    expect(opts.credentials).toBe("include");
    expect(opts.headers["Content-Type"]).toBe("application/json");
    expect(opts.headers["X-Cymbra-Web"]).toBe("1");
    expect(JSON.parse(opts.body)).toEqual({ kind: "local", email: "a@x.dev", password: "pw", audience: "web" });
    // Nothing persisted in web storage.
    expect(localStorage.length).toBe(0);
    expect(sessionStorage.length).toBe(0);
  });

  it("signs in via OIDC and refreshes from the cookie", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: true, body: { accessToken: "acc-2" } }));
    const client = createWebAuthClient(BASE);
    expect(await client.signInOidc("id-token", "web")).toEqual({ accessToken: "acc-2" });
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual({ kind: "oidc", idToken: "id-token", audience: "web" });

    expect(await client.refresh()).toEqual({ accessToken: "acc-2" });
    const [url, opts] = fetchMock.mock.calls[1];
    expect(url).toBe(`${BASE}/web/auth/refresh`);
    expect(opts.credentials).toBe("include");
    expect(opts.body).toBeUndefined();
  });

  it("throws a WebAuthError with the status and the server message; falls back to status text", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: false, status: 401, body: { error: "session ended" } }));
    const client = createWebAuthClient(BASE);
    await expect(client.refresh()).rejects.toMatchObject({ status: 401, message: "session ended" });
    await expect(client.refresh()).rejects.toBeInstanceOf(WebAuthError);

    fetchMock.mockResolvedValue(fakeResponse({ ok: false, status: 500, statusText: "Internal Server Error" }));
    await expect(client.signInLocal("a", "b", "web")).rejects.toMatchObject({
      status: 500,
      message: "Internal Server Error",
    });
  });

  it("single-flights concurrent refreshes", async () => {
    let calls = 0;
    fetchMock.mockImplementation(async () => {
      calls += 1;
      return fakeResponse({ ok: true, body: { accessToken: `acc-${calls}` } });
    });
    const client = createWebAuthClient(BASE);
    const [a, b] = await Promise.all([client.refresh(), client.refresh()]);
    expect(calls).toBe(1);
    expect(a).toEqual(b);
    await client.refresh();
    expect(calls).toBe(2);
  });

  it("logout is best-effort", async () => {
    fetchMock.mockRejectedValue(new Error("network down"));
    const client = createWebAuthClient(BASE);
    await expect(client.logout()).resolves.toBeUndefined();
    expect(fetchMock).toHaveBeenCalledWith(`${BASE}/web/auth/logout`, expect.anything());
  });
});

describe("fetchJson (bearer helper)", () => {
  it("GETs with the bearer and no cookies, returns the JSON", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: true, body: { plan: "premium" } }));
    const out = await fetchJson<{ plan: string }>(`${BASE}/web/plans/me`, { accessToken: "tok" });
    expect(out.plan).toBe("premium");
    const [url, opts] = fetchMock.mock.calls[0];
    expect(url).toBe(`${BASE}/web/plans/me`);
    expect(opts.method).toBe("GET");
    expect(opts.headers.Authorization).toBe("Bearer tok");
    expect(opts.credentials).toBeUndefined();
    expect(opts.body).toBeUndefined();
  });

  it("POSTs a JSON body when given one and maps a non-OK answer to WebAuthError", async () => {
    fetchMock.mockResolvedValue(fakeResponse({ ok: true, body: { campaign_key: "beta" } }));
    await fetchJson(`${BASE}/web/plans/redeem`, { accessToken: "tok", body: { code: "X" } });
    const [, opts] = fetchMock.mock.calls[0];
    expect(opts.method).toBe("POST");
    expect(opts.headers["Content-Type"]).toBe("application/json");
    expect(JSON.parse(opts.body)).toEqual({ code: "X" });

    fetchMock.mockResolvedValue(fakeResponse({ ok: false, status: 429, body: { error: "too many attempts" } }));
    await expect(
      fetchJson(`${BASE}/web/plans/redeem`, { accessToken: "tok", body: { code: "X" } }),
    ).rejects.toMatchObject({ status: 429, message: "too many attempts" });
  });
});
