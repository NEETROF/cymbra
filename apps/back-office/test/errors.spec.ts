import { describe, expect, it } from "vitest";
import { Code, ConnectError } from "@connectrpc/connect";
import { humanError } from "@/lib/errors";
import { WebAuthError } from "@/lib/web-auth";
import { i18n } from "@/i18n";

const t = i18n.global.t;

describe("humanError", () => {
  it("maps unauthenticated to the localized message — never the raw code/message", () => {
    // The exact case that leaked "[unauthenticated] invalid credentials" to the UI.
    const msg = humanError(new ConnectError("invalid credentials", Code.Unauthenticated));
    expect(msg).toBe(t("errors.unauthenticated"));
    expect(msg).not.toContain("unauthenticated");
    expect(msg).not.toContain("invalid credentials");
    expect(msg).not.toMatch(/\[.*\]/); // no "[code]" bracket form
  });

  it("maps common codes to their localized messages", () => {
    expect(humanError(new ConnectError("x", Code.PermissionDenied))).toBe(t("errors.permissionDenied"));
    expect(humanError(new ConnectError("x", Code.NotFound))).toBe(t("errors.notFound"));
    expect(humanError(new ConnectError("x", Code.Unavailable))).toBe(t("errors.unavailable"));
  });

  it("falls back to the generic message for non-Connect errors", () => {
    expect(humanError(new Error("boom"))).toBe(t("errors.generic"));
    expect(humanError("weird")).toBe(t("errors.generic"));
  });

  it("maps web-auth (cookie) HTTP failures by status — never the raw body", () => {
    // A 401 from the web-auth surface must localize exactly like the gRPC path, and
    // never leak the server's "invalid credentials" body into the UI.
    const msg = humanError(new WebAuthError(401, "invalid credentials"));
    expect(msg).toBe(t("errors.unauthenticated"));
    expect(msg).not.toContain("invalid credentials");

    expect(humanError(new WebAuthError(403, "x"))).toBe(t("errors.permissionDenied"));
    expect(humanError(new WebAuthError(404, "x"))).toBe(t("errors.notFound"));
    expect(humanError(new WebAuthError(412, "x"))).toBe(t("errors.notAvailable"));
    expect(humanError(new WebAuthError(400, "x"))).toBe(t("errors.invalidArgument"));
    expect(humanError(new WebAuthError(429, "x"))).toBe(t("errors.resourceExhausted"));
    expect(humanError(new WebAuthError(503, "x"))).toBe(t("errors.unavailable"));
    expect(humanError(new WebAuthError(500, "x"))).toBe(t("errors.generic"));
  });
});
