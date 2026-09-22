import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Code, ConnectError, type Interceptor } from "@connectrpc/connect";
import { AuthService } from "@/gen/auth_pb";
import {
  baseUrl,
  createClients,
  createTransport,
  notifyIfUnauthenticated,
  refreshInterceptor,
  resetRefreshState,
  setTokenRefresher,
  setUnauthenticatedHandler,
} from "@/net/transport.ts";

// The transport builder is the only thing here that would reach the network, so it is the
// only thing faked: `createGrpcWebTransport` hands back its options instead of a socket,
// which is also how the interceptor ORDER — the one property of this module a comment
// asserts and nothing else checks — becomes observable.
vi.mock("@connectrpc/connect-web", () => ({
  createGrpcWebTransport: vi.fn((options: unknown) => ({ options })),
}));

/** What the interceptors under test actually read off a request. Connect's own request
 *  type carries a dozen more fields, none of which this module touches. */
interface FakeRequest {
  method: unknown;
  header: Headers;
}
type Call = (req: FakeRequest) => Promise<{ message: string }>;

/** Apply an interceptor to a fake downstream call. */
const apply = (i: Interceptor, next: Call): Call => i(next as never) as unknown as Call;

/** A request for an ordinary RPC — anything that is not the refresh call. */
const request = (method: unknown = AuthService.method.logout): FakeRequest => ({
  method,
  header: new Headers(),
});

const expired = () => new ConnectError("token expired", Code.Unauthenticated);

beforeEach(() => {
  setUnauthenticatedHandler(null);
  setTokenRefresher(null);
  resetRefreshState();
});

afterEach(() => {
  setUnauthenticatedHandler(null);
  setTokenRefresher(null);
  resetRefreshState();
});

describe("transport session-expiry", () => {
  it("fires the handler on an UNAUTHENTICATED error", () => {
    const handler = vi.fn();
    setUnauthenticatedHandler(handler);
    notifyIfUnauthenticated(new ConnectError("expired", Code.Unauthenticated));
    expect(handler).toHaveBeenCalledOnce();
  });

  it("ignores other Connect codes and plain errors", () => {
    const handler = vi.fn();
    setUnauthenticatedHandler(handler);
    notifyIfUnauthenticated(new ConnectError("boom", Code.Internal));
    notifyIfUnauthenticated(new Error("network"));
    notifyIfUnauthenticated(null);
    expect(handler).not.toHaveBeenCalled();
  });

  it("does nothing when no handler is registered", () => {
    setUnauthenticatedHandler(null);
    expect(() => notifyIfUnauthenticated(new ConnectError("x", Code.Unauthenticated))).not.toThrow();
  });
});

describe("refreshInterceptor", () => {
  it("passes a successful call through without touching the refresher", async () => {
    const refresher = vi.fn(async () => true);
    setTokenRefresher(refresher);
    const next = vi.fn<Call>(async () => ({ message: "ok" }));
    await expect(apply(refreshInterceptor, next)(request())).resolves.toEqual({ message: "ok" });
    expect(refresher).not.toHaveBeenCalled();
    expect(next).toHaveBeenCalledOnce();
  });

  it("refreshes once and retries the call after an UNAUTHENTICATED", async () => {
    setTokenRefresher(async () => true);
    const next = vi.fn<Call>().mockRejectedValueOnce(expired()).mockResolvedValueOnce({ message: "ok" });
    await expect(apply(refreshInterceptor, next)(request())).resolves.toEqual({ message: "ok" });
    expect(next).toHaveBeenCalledTimes(2);
  });

  it("retries only once: a second expiry on the retry is the caller's answer", async () => {
    // The retry is not a loop — a token that is still refused after a successful refresh
    // means the session is gone, and the error has to reach the expiry handler.
    setTokenRefresher(async () => true);
    const next = vi.fn<Call>().mockRejectedValue(expired());
    await expect(apply(refreshInterceptor, next)(request())).rejects.toThrow("token expired");
    expect(next).toHaveBeenCalledTimes(2);
  });

  it("gives up when the refresh itself fails", async () => {
    setTokenRefresher(async () => false);
    const next = vi.fn<Call>().mockRejectedValue(expired());
    await expect(apply(refreshInterceptor, next)(request())).rejects.toThrow("token expired");
    expect(next).toHaveBeenCalledOnce();
  });

  it("does not retry when no refresher has been registered", async () => {
    setTokenRefresher(null);
    const next = vi.fn<Call>().mockRejectedValue(expired());
    await expect(apply(refreshInterceptor, next)(request())).rejects.toThrow("token expired");
    expect(next).toHaveBeenCalledOnce();
  });

  it("never refreshes the refresh call itself", async () => {
    // Refreshing a refused refresh would recurse straight into a deadlock.
    const refresher = vi.fn(async () => true);
    setTokenRefresher(refresher);
    const next = vi.fn<Call>().mockRejectedValue(expired());
    await expect(apply(refreshInterceptor, next)(request(AuthService.method.refresh))).rejects.toThrow("token expired");
    expect(refresher).not.toHaveBeenCalled();
    expect(next).toHaveBeenCalledOnce();
  });

  it("leaves an error that is not UNAUTHENTICATED alone", async () => {
    const refresher = vi.fn(async () => true);
    setTokenRefresher(refresher);
    const next = vi.fn<Call>().mockRejectedValue(new ConnectError("boom", Code.Internal));
    await expect(apply(refreshInterceptor, next)(request())).rejects.toThrow("boom");
    expect(refresher).not.toHaveBeenCalled();
    expect(next).toHaveBeenCalledOnce();
  });

  it("collapses concurrent expiries into a SINGLE refresh", async () => {
    // Every call in flight when the access token dies comes back UNAUTHENTICATED at once.
    // One refresh serves them all; N refreshes would rotate the family N times and, on a
    // rotating refresh token, revoke the session outright.
    let release!: (ok: boolean) => void;
    const refresher = vi.fn(() => new Promise<boolean>((resolve) => (release = resolve)));
    setTokenRefresher(refresher);

    const next = vi.fn<Call>(async (req) => {
      if (!req.header.has("X-Retried")) {
        req.header.set("X-Retried", "1");
        throw expired();
      }
      return { message: "ok" };
    });
    const chain = apply(refreshInterceptor, next);
    const calls = [chain(request()), chain(request()), chain(request())];

    // Let all three reach their 401 and queue on a refresh that has not answered yet:
    // that overlap IS the case under test, and it only exists before the refresh settles.
    for (let i = 0; i < 10; i++) await Promise.resolve();
    release(true);
    await expect(Promise.all(calls)).resolves.toEqual([{ message: "ok" }, { message: "ok" }, { message: "ok" }]);
    expect(refresher).toHaveBeenCalledOnce();
  });

  it("refreshes again for an expiry that arrives after the first refresh settled", async () => {
    // Single-flight must not become "once per session": the next token expires too.
    const refresher = vi.fn(async () => true);
    setTokenRefresher(refresher);
    const next = vi
      .fn<Call>()
      .mockRejectedValueOnce(expired())
      .mockResolvedValueOnce({ message: "ok" })
      .mockRejectedValueOnce(expired())
      .mockResolvedValueOnce({ message: "ok" });
    const chain = apply(refreshInterceptor, next);
    await chain(request());
    await chain(request());
    expect(refresher).toHaveBeenCalledTimes(2);
  });
});

describe("createTransport", () => {
  /** The interceptor chain `createTransport` handed to the (faked) gRPC-web transport,
   *  composed the way Connect composes it: the first is the outermost. */
  function chainOf(getToken: () => string | null, leaf: Call): Call {
    const transport = createTransport(getToken) as unknown as { options: { interceptors: Interceptor[] } };
    return transport.options.interceptors.reduceRight((next, i) => apply(i, next), leaf);
  }

  it("builds the transport against the configured backend origin", () => {
    const transport = createTransport(() => null) as unknown as { options: { baseUrl: string } };
    expect(transport.options.baseUrl).toBe(baseUrl());
    expect(baseUrl()).toBe("http://localhost:50051");
  });

  it("attaches the access token as a bearer", async () => {
    let seen: string | null = null;
    const chain = chainOf(
      () => "tok-123",
      async (req) => {
        seen = req.header.get("Authorization");
        return { message: "ok" };
      },
    );
    await chain(request());
    expect(seen).toBe("Bearer tok-123");
  });

  it("sends no Authorization header when the reader is signed out", async () => {
    let seen: string | null = "unset";
    const chain = chainOf(
      () => null,
      async (req) => {
        seen = req.header.get("Authorization");
        return { message: "ok" };
      },
    );
    await chain(request());
    expect(seen).toBeNull();
  });

  it("carries the NEW token on the retry, and keeps the expiry handler quiet", async () => {
    // This is the order the interceptors are in for: expiry outermost, refresh under it,
    // auth innermost — so the retry passes back through auth and re-reads the token the
    // refresh just wrote. Auth above refresh would retry with the dead token forever.
    const handler = vi.fn();
    setUnauthenticatedHandler(handler);
    let token = "old";
    setTokenRefresher(async () => {
      token = "new";
      return true;
    });

    const seen: Array<string | null> = [];
    const chain = chainOf(
      () => token,
      async (req) => {
        seen.push(req.header.get("Authorization"));
        if (seen.length === 1) throw expired();
        return { message: "ok" };
      },
    );

    await expect(chain(request())).resolves.toEqual({ message: "ok" });
    expect(seen).toEqual(["Bearer old", "Bearer new"]);
    expect(handler).not.toHaveBeenCalled();
  });

  it("surfaces a terminal expiry to the handler once the refresh has given up", async () => {
    const handler = vi.fn();
    setUnauthenticatedHandler(handler);
    setTokenRefresher(async () => false);
    const chain = chainOf(
      () => "dead",
      async () => {
        throw expired();
      },
    );
    await expect(chain(request())).rejects.toThrow("token expired");
    expect(handler).toHaveBeenCalledOnce();
  });
});

describe("createClients", () => {
  it("exposes one client per backend service the extension calls", () => {
    const clients = createClients(createTransport(() => null));
    expect(Object.keys(clients).sort()).toEqual(["auth", "data", "deck", "knownWords", "stats", "user"]);
    // A client is the service's RPCs, so a renamed or dropped RPC shows up here.
    expect(typeof clients.auth.refresh).toBe("function");
    expect(typeof clients.deck.pullCards).toBe("function");
  });
});
