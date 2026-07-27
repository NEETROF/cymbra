import { describe, expect, it } from "vitest";
import { Code, ConnectError } from "@connectrpc/connect";
import { humanError } from "@/lib/errors";

describe("humanError", () => {
  it("maps unauthenticated to a friendly message — never the raw code/message", () => {
    // The exact case that leaked "[unauthenticated] invalid credentials" to the UI.
    const e = new ConnectError("invalid credentials", Code.Unauthenticated);
    const msg = humanError(e);
    expect(msg).toBe("Identifiants invalides ou session expirée.");
    expect(msg).not.toContain("unauthenticated");
    expect(msg).not.toContain("invalid credentials");
    expect(msg).not.toMatch(/\[.*\]/); // no "[code]" bracket form
  });

  it("maps common codes to friendly messages", () => {
    expect(humanError(new ConnectError("x", Code.PermissionDenied))).toBe("Accès refusé.");
    expect(humanError(new ConnectError("x", Code.NotFound))).toBe("Élément introuvable.");
    expect(humanError(new ConnectError("x", Code.Unavailable))).toBe(
      "Service indisponible. Réessaie.",
    );
  });

  it("falls back to a generic message for non-Connect errors", () => {
    expect(humanError(new Error("boom"))).toBe("Une erreur est survenue. Réessaie.");
    expect(humanError("weird")).toBe("Une erreur est survenue. Réessaie.");
  });
});
