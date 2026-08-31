// The site's client for the backend's browser plan routes (`/web/plans/*`, spec
// `music-plan-web-api`): bearer JSON calls through the shared `fetchJson` helper.
// Field names are the proto's (snake_case), exactly as `GetMyPlan` answers.

import { fetchJson } from "@cymbra/web-auth";

export interface BetaView {
  campaign_key: string;
  campaign_name: string;
  kind: "premium_trial" | "feature" | string;
  joined_at: string;
  ends_at: string | null;
}

export interface PlanView {
  plan: "free" | "premium" | string;
  source: string | null;
  ends_at: string | null;
  ends_without_renewal: boolean;
  trial_campaign_key: string | null;
  trial_campaign_name: string | null;
  trial_ends_at: string | null;
  betas: BetaView[];
  managed_on: "apple" | "google" | "web" | null;
  can_purchase_here: boolean;
  purchase_channel: string | null;
  products: string[];
  unlocks: string[];
}

export interface RedeemView {
  campaign_key: string;
  campaign_name: string;
  kind: "premium_trial" | "feature" | string;
  ends_at: string | null;
}

/** One linked sign-in method; `email` only for the `local` provider. */
export interface IdentityView {
  provider: "local" | "google" | "apple" | string;
  email: string | null;
  linked_at: number;
}

/** The caller's account summary (`GET /web/account/me`, Cymbra ID). */
export interface AccountView {
  handle: string | null;
  display_name: string | null;
  locale: string | null;
  identities: IdentityView[];
}

export interface WebPlansClient {
  me(accessToken: string): Promise<PlanView>;
  /** Handle + linked sign-in methods (spec `web-auth-session`). */
  account(accessToken: string): Promise<AccountView>;
  /** Erase the caller's own account — irreversible (`POST /web/account/delete`). */
  deleteAccount(accessToken: string): Promise<{ deleted: boolean }>;
  redeem(accessToken: string, code: string): Promise<RedeemView>;
  checkout(accessToken: string, productId: string): Promise<{ checkout_url: string }>;
  portal(accessToken: string): Promise<{ portal_url: string }>;
}

export function createWebPlansClient(apiUrl: string): WebPlansClient {
  const url = (path: string) => `${apiUrl}${path}`;
  return {
    me: (accessToken) => fetchJson<PlanView>(url("/web/plans/me"), { accessToken }),
    account: (accessToken) => fetchJson<AccountView>(url("/web/account/me"), { accessToken }),
    deleteAccount: (accessToken) =>
      fetchJson<{ deleted: boolean }>(url("/web/account/delete"), { accessToken, body: {} }),
    redeem: (accessToken, code) =>
      fetchJson<RedeemView>(url("/web/plans/redeem"), { accessToken, body: { code: code.trim() } }),
    checkout: (accessToken, productId) =>
      fetchJson<{ checkout_url: string }>(url("/web/plans/checkout"), {
        accessToken,
        body: { product_id: productId },
      }),
    portal: (accessToken) => fetchJson<{ portal_url: string }>(url("/web/plans/portal"), { accessToken }),
  };
}
