import { afterEach, describe, expect, it, vi } from "vitest";
import { Code, ConnectError } from "@connectrpc/connect";
import { notifyIfUnauthenticated, setUnauthenticatedHandler } from "@/net/transport.ts";

afterEach(() => setUnauthenticatedHandler(null));

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
