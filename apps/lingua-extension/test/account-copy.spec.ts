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
  "connected",
  "linkGoogle",
  "linkApple",
  "unlink",
  "setPassword",
  "verifyPassword",
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
        // Word-bounded: « Reconnecte-toi » is French, a raw Connect error is not.
        expect(text).not.toMatch(/grpc|\bconnect\b|connecterror|status|code \d|lemm/i);
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

  it("words each linking failure for what it is (add-lingua-connected-accounts D6)", () => {
    expect(errorCopy("linkGoogle", "alreadyExists")).toBe("Ce compte Google est déjà lié à un autre compte Cymbra.");
    expect(errorCopy("linkApple", "alreadyExists")).toBe("Ce compte Apple est déjà lié à un autre compte Cymbra.");
    expect(errorCopy("linkGoogle", "unauthenticated")).toContain("Google");
    expect(errorCopy("linkApple", "unknown")).toBe("Impossible de lier Apple. Réessaie.");
    expect(errorCopy("unlink", "failedPrecondition")).toBe("Tu ne peux pas retirer ta seule méthode de connexion.");
    expect(errorCopy("unlink", "unknown")).toContain("retirer");
    expect(errorCopy("setPassword", "invalidArgument")).toContain("trop faible");
    expect(errorCopy("setPassword", "alreadyExists")).toContain("déjà utilisée");
    expect(errorCopy("verifyPassword", "alreadyExists")).toContain("vient d'être prise");
    expect(errorCopy("verifyPassword", "invalidArgument")).toContain("expiré");
    expect(errorCopy("verifyPassword", "unknown")).toBe(errorCopy("verify", "unknown"));
    expect(errorCopy("connected", "unknown")).toContain("méthodes de connexion");
    for (const context of ["connected", "unlink", "setPassword"] as FlowContext[]) {
      expect(errorCopy(context, "unauthenticated")).toBe("Ta session a expiré. Reconnecte-toi.");
    }
    for (const kind of KINDS) {
      for (const context of ["linkGoogle", "linkApple", "unlink", "setPassword"] as FlowContext[]) {
        expect(errorCopy(context, kind)).not.toBe(PASSWORD_COPY);
      }
    }
  });

  it("shares the unreachable and too-many-attempts copy across contexts", () => {
    expect(errorCopy("verify", "unavailable")).toBe(errorCopy("signInApple", "unavailable"));
    expect(errorCopy("forgot", "rateLimited")).toContain("Trop de tentatives");
  });
});
