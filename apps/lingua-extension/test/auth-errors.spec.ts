import { describe, expect, it } from "vitest";
import { Code, ConnectError } from "@connectrpc/connect";
import { AccountError, authErrorFromCode, authErrorOf } from "@/state/auth-errors.ts";

describe("authErrorFromCode", () => {
  it.each([
    [Code.InvalidArgument, "invalidArgument"],
    [Code.DeadlineExceeded, "unavailable"],
    [Code.Unavailable, "unavailable"],
    [Code.NotFound, "notFound"],
    [Code.AlreadyExists, "alreadyExists"],
    [Code.ResourceExhausted, "rateLimited"],
    [Code.FailedPrecondition, "failedPrecondition"],
    [Code.Unauthenticated, "unauthenticated"],
    [Code.Internal, "unknown"],
    [Code.PermissionDenied, "unknown"],
  ])("maps code %i to %s", (code, kind) => {
    expect(authErrorFromCode(code)).toBe(kind);
  });
});

describe("authErrorOf", () => {
  it("keeps an AccountError's category", () => {
    expect(authErrorOf(new AccountError("rateLimited"))).toBe("rateLimited");
  });

  it("maps a Connect error by its code", () => {
    expect(authErrorOf(new ConnectError("locked", Code.ResourceExhausted))).toBe("rateLimited");
  });

  it("treats a failed fetch as unavailable, wrapped or not", () => {
    const wrapped = new ConnectError(
      "fetch failed",
      Code.Unknown,
      undefined,
      undefined,
      new TypeError("Failed to fetch"),
    );
    expect(authErrorOf(wrapped)).toBe("unavailable");
    expect(authErrorOf(new TypeError("Failed to fetch"))).toBe("unavailable");
  });

  it("falls back to unknown", () => {
    expect(authErrorOf(new ConnectError("boom", Code.Unknown))).toBe("unknown");
    expect(authErrorOf(new Error("provider state mismatch"))).toBe("unknown");
    expect(authErrorOf("nope")).toBe("unknown");
  });
});
