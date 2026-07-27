import { afterEach, describe, expect, it, vi } from "vitest";
import { Code, ConnectError } from "@connectrpc/connect";
import { notifyIfUnauthenticated, setUnauthenticatedHandler } from "@/lib/transport";

afterEach(() => setUnauthenticatedHandler(null));

describe("session expiry notifier", () => {
  it("fires the handler on an UNAUTHENTICATED error", () => {
    const spy = vi.fn();
    setUnauthenticatedHandler(spy);
    notifyIfUnauthenticated(new ConnectError("token rejected: expired", Code.Unauthenticated));
    expect(spy).toHaveBeenCalledTimes(1);
  });

  it("ignores other gRPC codes and non-Connect errors", () => {
    const spy = vi.fn();
    setUnauthenticatedHandler(spy);
    notifyIfUnauthenticated(new ConnectError("x", Code.PermissionDenied));
    notifyIfUnauthenticated(new ConnectError("x", Code.Internal));
    notifyIfUnauthenticated(new Error("boom"));
    expect(spy).not.toHaveBeenCalled();
  });

  it("is a no-op when no handler is registered", () => {
    setUnauthenticatedHandler(null);
    expect(() =>
      notifyIfUnauthenticated(new ConnectError("x", Code.Unauthenticated)),
    ).not.toThrow();
  });
});
