import { afterEach, describe, expect, it, vi } from "vitest";
import { Code, ConnectError, type Interceptor } from "@connectrpc/connect";
import { refreshInterceptor, setTokenRefresher } from "@/lib/transport";
import { AuthService } from "@/gen/auth_pb";
import { ScoreService } from "@/gen/score_pb";

// The interceptor's `next` and request are large generated Connect types; the
// interceptor only touches `req.method` and awaits `next(req)`, so minimal stand-ins
// (cast to the real param types) exercise it without a live transport.
type Next = Parameters<Interceptor>[0];
type Req = Parameters<ReturnType<Interceptor>>[0];

const unauth = () => new ConnectError("token expired", Code.Unauthenticated);
const req = (method: unknown) => ({ method }) as Req;
const asNext = (fn: () => Promise<unknown>) => fn as unknown as Next;

afterEach(() => setTokenRefresher(null));

describe("silent token refresh interceptor", () => {
  it("refreshes once and retries a call that returned UNAUTHENTICATED", async () => {
    setTokenRefresher(async () => true);
    let n = 0;
    const next = vi.fn(async () => {
      n += 1;
      if (n === 1) throw unauth();
      return { ok: true };
    });
    const res = await refreshInterceptor(asNext(next))(req(ScoreService.method.searchCatalog));
    expect(next).toHaveBeenCalledTimes(2);
    expect(res).toEqual({ ok: true });
  });

  it("propagates the error (→ sign out) when the refresh fails", async () => {
    setTokenRefresher(async () => false);
    const next = vi.fn(async () => {
      throw unauth();
    });
    await expect(refreshInterceptor(asNext(next))(req(ScoreService.method.searchCatalog))).rejects.toBeInstanceOf(
      ConnectError,
    );
    expect(next).toHaveBeenCalledTimes(1); // no retry
  });

  it("never refreshes the refresh RPC itself (no recursion)", async () => {
    const refresher = vi.fn(async () => true);
    setTokenRefresher(refresher);
    const next = vi.fn(async () => {
      throw unauth();
    });
    await expect(refreshInterceptor(asNext(next))(req(AuthService.method.refresh))).rejects.toBeInstanceOf(
      ConnectError,
    );
    expect(refresher).not.toHaveBeenCalled();
  });

  it("single-flights concurrent 401s into one refresh", async () => {
    const refresher = vi.fn(async () => true);
    setTokenRefresher(refresher);
    const oneShot = () => {
      let n = 0;
      return asNext(async () => {
        n += 1;
        if (n === 1) throw unauth();
        return { ok: true };
      });
    };
    await Promise.all([
      refreshInterceptor(oneShot())(req(ScoreService.method.searchCatalog)),
      refreshInterceptor(oneShot())(req(ScoreService.method.searchCatalog)),
    ]);
    expect(refresher).toHaveBeenCalledTimes(1);
  });
});
