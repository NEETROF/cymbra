import { test as base, expect, type Page } from "@playwright/test";
import type { E2EData } from "../src/lib/e2e-seam";

// A minimal unsigned JWT — the client only decodes the payload (roles/sub) to shape
// the UI; the signature is irrelevant to the fake seam.
export function makeJwt(payload: Record<string, unknown>): string {
  const b64 = (o: unknown) => Buffer.from(JSON.stringify(o)).toString("base64url");
  return `${b64({ alg: "EdDSA" })}.${b64(payload)}.sig`;
}

// The signed-in caller's roles. `admin` is a `music`-scope admin (the common case:
// the Users page shows only `music`); `global-admin` is a break-glass admin across
// every scope (the scope selector appears); `live-admin` is an admin OUTSIDE the music
// scope, which is what gates every plan surface. The token carries `roles_by_scope` so
// the back office can derive the scopes the admin may administer.
export function tokenFor(role: "moderator" | "admin" | "global-admin" | "live-admin" | "none"): string {
  const rolesByScope: Record<string, string[]> =
    role === "global-admin"
      ? { global: ["admin"] }
      : role === "admin"
        ? { music: ["admin"] }
        : role === "live-admin"
          ? { live: ["admin"] }
          : role === "moderator"
            ? { music: ["moderator"] }
            : { global: ["user"] };
  const roles = [...new Set(Object.values(rolesByScope).flat())];
  return makeJwt({ sub: "u1", aud: "back-office", roles, roles_by_scope: rolesByScope, exp: 4102444800 });
}

export interface SeedOptions {
  /** Canned data the fake gRPC-web clients serve (see lib/e2e-seam.ts). */
  data?: E2EData;
  /** Start already authenticated by persisting a token for this role. */
  loginAs?: "moderator" | "admin" | "global-admin" | "live-admin";
}

// Seed the e2e data (and optionally a signed-in session) BEFORE the app boots, so
// the fake clients and auth store see them on first render. Call before goto().
export async function seed(page: Page, opts: SeedOptions = {}): Promise<void> {
  // Force English so assertions are deterministic regardless of the runner's browser
  // locale. `detectLocale()` reads this persisted key first; the i18n toggle test
  // still flips it at runtime by clicking FR.
  await page.addInitScript(() => {
    localStorage.setItem("cymbra.bo.locale", "en");
  });

  const data: E2EData = { ...(opts.data ?? {}) };
  if (opts.loginAs) {
    // No token is persisted anywhere (memory-only). Instead we simulate the HttpOnly
    // refresh cookie existing at boot: `session` makes the fake web-auth `refresh`
    // re-mint an access token for this role, so the app boots signed in.
    data.session = true;
    data.tokens = { accessToken: tokenFor(opts.loginAs), refreshToken: "r" };
  }

  await page.addInitScript((d) => {
    window.__CYMBRA_E2E__ = d as E2EData;
  }, data as E2EData);
}

// A sample catalog hit with every field the table + detail preview read.
export function sampleHit(over: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    id: "11111111-1111-1111-1111-111111111111",
    title: "Clair de Lune",
    composer: "Claude Debussy",
    arranger: "",
    level: "advanced",
    license: "CC0",
    source: "pdmx",
    timeSig: "9/8",
    noteCount: 640,
    tempoBpm: 66,
    ...over,
  };
}

export const test = base;
export { expect };
