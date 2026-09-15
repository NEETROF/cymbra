import { describe, expect, it } from "vitest";
import { errorCopy, type FlowContext } from "@/account/copy.ts";
import type { AuthErrorKind } from "@/state/auth-errors.ts";

const CONTEXTS: FlowContext[] = [
  "signInEmail",
  "signInGoogle",
  "signInApple",
  "signUp",
  "verify",
  "resend",
  "forgot",
  "reset",
];
const KINDS: AuthErrorKind[] = [
  "unauthenticated",
  "alreadyExists",
  "rateLimited",
  "failedPrecondition",
  "invalidArgument",
  "notFound",
  "unavailable",
  "unknown",
];
const PASSWORD_COPY = "Email ou mot de passe incorrect.";

describe("errorCopy", () => {
  it("returns plain words for every context and category", () => {
    for (const context of CONTEXTS) {
      for (const kind of KINDS) {
        const text = errorCopy(context, kind);
        expect(text.length, `${context}/${kind}`).toBeGreaterThan(10);
        expect(text).not.toMatch(/grpc|connect|status|code \d|lemm/i);
      }
    }
  });

  it("never words a provider failure as a password error", () => {
    for (const kind of KINDS) {
      expect(errorCopy("signInGoogle", kind)).not.toBe(PASSWORD_COPY);
      expect(errorCopy("signInApple", kind)).not.toBe(PASSWORD_COPY);
    }
    expect(errorCopy("signInApple", "unauthenticated")).toContain("Apple");
    expect(errorCopy("signInGoogle", "unauthenticated")).toContain("Google");
  });

  it("uses the context-specific copy", () => {
    expect(errorCopy("signInEmail", "unauthenticated")).toBe(PASSWORD_COPY);
    expect(errorCopy("signUp", "alreadyExists")).toContain("déjà");
    expect(errorCopy("signUp", "invalidArgument")).toContain("trop faible");
    expect(errorCopy("verify", "invalidArgument")).toContain("expiré");
    expect(errorCopy("reset", "notFound")).toContain("expiré");
  });

  it("shares the unreachable and too-many-attempts copy across contexts", () => {
    expect(errorCopy("verify", "unavailable")).toBe(errorCopy("signInApple", "unavailable"));
    expect(errorCopy("forgot", "rateLimited")).toContain("Trop de tentatives");
  });
});
