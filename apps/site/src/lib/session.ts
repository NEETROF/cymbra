// One in-memory web session per page (module singleton, shared by every island of
// the page — Astro islands are separate Vue roots but load the same module). Audience
// `web`: the public site's plain-user audience (spec `web-auth-session`).

import { createWebAuthClient, createWebSession, type WebAuthClient, type WebSession } from "@cymbra/web-auth";
import { config } from "./config";
import { humanError } from "./plan-view";
import type { Lang } from "./i18n";
import { createWebPlansClient, type WebPlansClient } from "./web-plans";

export const SITE_AUDIENCE = "web";

let authClient: WebAuthClient | null = null;
let plansClient: WebPlansClient | null = null;
let session: WebSession | null = null;

/** The page's session (created on first use; `lang` localizes sign-in errors). */
export function useSession(lang: Lang): WebSession {
  authClient ??= createWebAuthClient(config.apiUrl);
  session ??= createWebSession(authClient, SITE_AUDIENCE, (e) => humanError(lang, e));
  return session;
}

export function usePlans(): WebPlansClient {
  plansClient ??= createWebPlansClient(config.apiUrl);
  return plansClient;
}

/** Test seam: inject fakes (and reset the singleton session). */
export function setClientsForTest(auth: WebAuthClient, plans: WebPlansClient): void {
  authClient = auth;
  plansClient = plans;
  session = null;
}
