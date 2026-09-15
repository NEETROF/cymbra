import { describe, expect, it, vi } from "vitest";
import { hostAppSignInUrl, parseHandedIdToken, takeHandedIdToken } from "@/state/native-signin.ts";

describe("hostAppSignInUrl", () => {
  it("opens the host app on the requested provider", () => {
    expect(hostAppSignInUrl("apple")).toBe("cymbra-lingua://signin?provider=apple");
    expect(hostAppSignInUrl("google")).toBe("cymbra-lingua://signin?provider=google");
  });
});

describe("parseHandedIdToken", () => {
  it("accepts a known provider with a non-empty id_token", () => {
    expect(parseHandedIdToken({ provider: "apple", idToken: "tok" })).toEqual({ provider: "apple", idToken: "tok" });
  });

  it("reads nothing pending and malformed replies as no token", () => {
    for (const reply of [
      null,
      undefined,
      "tok",
      {},
      { provider: "apple" },
      { provider: "apple", idToken: "" },
      { provider: "github", idToken: "tok" },
      { provider: "google", idToken: 42 },
    ]) {
      expect(parseHandedIdToken(reply)).toBeNull();
    }
  });
});

describe("takeHandedIdToken", () => {
  it("asks the native handler for the pending token", async () => {
    const send = vi.fn(async () => ({ provider: "google", idToken: "tok" }));
    expect(await takeHandedIdToken(send)).toEqual({ provider: "google", idToken: "tok" });
    expect(send).toHaveBeenCalledWith({ type: "auth.takeIdToken" });
  });

  it("treats a missing or failing handler as nothing pending", async () => {
    expect(
      await takeHandedIdToken(async () => {
        throw new Error("no native handler");
      }),
    ).toBeNull();
  });
});
