import { test as base, expect, type Page } from "@playwright/test";
import type { E2EData } from "../src/lib/e2e-seam";

// A minimal unsigned JWT — the client only decodes the payload (roles/sub) to shape
// the UI; the signature is irrelevant to the fake seam.
export function makeJwt(payload: Record<string, unknown>): string {
  const b64 = (o: unknown) => Buffer.from(JSON.stringify(o)).toString("base64url");
  return `${b64({ alg: "EdDSA" })}.${b64(payload)}.sig`;
}

export function tokenFor(role: "moderator" | "admin" | "none"): string {
  const roles = role === "admin" ? ["admin"] : role === "moderator" ? ["moderator"] : [];
  return makeJwt({ sub: "u1", aud: "music", roles, exp: 4102444800 });
}

export interface SeedOptions {
  /** Canned data the fake gRPC-web clients serve (see lib/e2e-seam.ts). */
  data?: E2EData;
  /** Start already authenticated by persisting a token for this role. */
  loginAs?: "moderator" | "admin";
}

// Seed the e2e data (and optionally a persisted session) BEFORE the app boots, so
// the fake clients and auth store see them on first render. Call before goto().
export async function seed(page: Page, opts: SeedOptions = {}): Promise<void> {
  await page.addInitScript((d) => {
    window.__CYMBRA_E2E__ = d as E2EData;
  }, (opts.data ?? {}) as E2EData);

  if (opts.loginAs) {
    const token = tokenFor(opts.loginAs);
    await page.addInitScript((t) => {
      localStorage.setItem("cymbra.bo.tokens", JSON.stringify({ accessToken: t, refreshToken: "r" }));
    }, token);
  }
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
