// Pure view logic of the islands (unit-tested): error → localized message, the
// redeem outcome, the account page's manage action, the `?code=` prefill.

import { WebAuthError } from "@cymbra/web-auth";
import { formatDate, t, type Lang, type MessageKey } from "./i18n";
import type { IdentityView, PlanView, RedeemView } from "./web-plans";

/** Localized, never raw: HTTP status → copy; unknown → generic. */
export function humanError(lang: Lang, e: unknown): string {
  if (e instanceof WebAuthError) {
    switch (e.status) {
      case 401:
        return t(lang, "errUnauthenticated");
      case 403:
        return t(lang, "errForbidden");
      case 404:
        return t(lang, "errNotFound");
      case 412:
        return t(lang, "errPrecondition");
      case 400:
        return t(lang, "errInvalid");
      case 429:
        return t(lang, "errRate");
      case 503:
        return t(lang, "errUnavailable");
      default:
        return t(lang, "errGeneric");
    }
  }
  return t(lang, "errGeneric");
}

/**
 * Redemption refusals keep the server's neutral shape: an unknown / used / revoked
 * code (404, one message for all three) reads "invalid or already used"; a rule
 * refusal (412 another trial running / beta closed, 409 already enrolled) reads as
 * such; the throttle (429) as such — never a hint about whether a code exists.
 */
export function redeemError(lang: Lang, e: unknown): string {
  if (e instanceof WebAuthError) {
    if (e.status === 404 || e.status === 400) return t(lang, "errCodeInvalid");
    if (e.status === 412 || e.status === 409) return t(lang, "errCodeRefused");
  }
  return humanError(lang, e);
}

/** The success copy of a redemption. */
export function redeemOutcome(lang: Lang, out: RedeemView): { title: string; body: string; next: string } {
  const body =
    out.kind === "premium_trial" && out.ends_at
      ? t(lang, "redeemedTrial", { name: out.campaign_name, date: formatDate(lang, out.ends_at) })
      : t(lang, "redeemedFeature", { name: out.campaign_name });
  return { title: t(lang, "redeemedTitle"), body, next: t(lang, "redeemedNext") };
}

/** Prefill from `?code=…` (trimmed, upper-cased); empty when absent. */
export function codeFromQuery(search: string): string {
  const params = new URLSearchParams(search.startsWith("?") ? search.slice(1) : search);
  return (params.get("code") ?? "").trim().toUpperCase();
}

/** The `_ptxn` transaction id from the checkout page's query, or `null`. */
export function transactionFromQuery(search: string): string | null {
  const params = new URLSearchParams(search.startsWith("?") ? search.slice(1) : search);
  const v = (params.get("_ptxn") ?? "").trim();
  return v.length ? v : null;
}

/** Store management pages (D6): the store owns store subscriptions. */
export const STORE_MANAGE_URL: Record<"apple" | "google", string> = {
  apple: "https://apps.apple.com/account/subscriptions",
  google: "https://play.google.com/store/account/subscriptions",
};

export type ManageAction =
  | { kind: "portal" } // active web row → fetch the provider portal URL at click time
  | { kind: "store"; channel: "apple" | "google"; url: string; note: string }
  | { kind: "purchase"; products: string[] } // free (or code/admin) + web channel open
  | { kind: "none" };

/** Which management action the account page offers for this plan. */
export function manageAction(lang: Lang, plan: PlanView): ManageAction {
  if (plan.managed_on === "web") return { kind: "portal" };
  if (plan.managed_on === "apple" || plan.managed_on === "google") {
    const channel = plan.managed_on;
    return {
      kind: "store",
      channel,
      url: STORE_MANAGE_URL[channel],
      note: t(lang, channel === "apple" ? "manageStoreApple" : "manageStoreGoogle"),
    };
  }
  if (plan.can_purchase_here && plan.products.length) return { kind: "purchase", products: plan.products };
  return { kind: "none" };
}

/** Plan headline + the dated line under it. */
export function planSummary(lang: Lang, plan: PlanView): { title: string; lines: string[] } {
  const isTrial = plan.plan === "premium" && plan.source === "code" && plan.trial_campaign_name;
  const title = plan.plan === "premium" ? t(lang, isTrial ? "planTrial" : "planPremium") : t(lang, "planFree");
  const lines: string[] = [];
  if (plan.managed_on) {
    const channelKey: MessageKey =
      plan.managed_on === "apple" ? "channelApple" : plan.managed_on === "google" ? "channelGoogle" : "channelWeb";
    lines.push(t(lang, "managedOn", { channel: t(lang, channelKey) }));
  }
  if (plan.plan === "premium" && plan.ends_at) {
    const date = formatDate(lang, plan.ends_at);
    lines.push(plan.ends_without_renewal ? t(lang, "rightsEndOn", { date }) : t(lang, "renewsOn", { date }));
  }
  if (isTrial && plan.trial_ends_at) {
    lines.push(
      t(lang, "trialEndsOn", { name: plan.trial_campaign_name ?? "", date: formatDate(lang, plan.trial_ends_at) }),
    );
  }
  return { title, lines };
}

/** Product id → label (the two known ids; anything else shown as-is). */
export function productLabel(lang: Lang, productId: string): string {
  if (productId.includes("month")) return t(lang, "productMonthly");
  if (productId.includes("year")) return t(lang, "productYearly");
  return productId;
}

/**
 * The Sign in with Apple return URL for the current page: the canonical, registered
 * form — no trailing slash, no `/en` locale prefix (`https://cymbra.app/redeem`,
 * `https://cymbra.app/account`). Apple matches Return URLs exactly.
 */
export function appleReturnUrl(origin: string, pathname: string): string {
  const path = pathname.replace(/^\/en(?=\/|$)/, "").replace(/\/+$/, "");
  return `${origin}${path || "/"}`;
}

/** "E-mail (address)" / "Google" / "Apple" — never an OIDC subject. */
export function identityLabel(lang: Lang, id: IdentityView): string {
  switch (id.provider) {
    case "local":
      return t(lang, "methodEmail", { email: id.email ?? "" });
    case "google":
      return t(lang, "methodGoogle");
    case "apple":
      return t(lang, "methodApple");
    default:
      return id.provider;
  }
}
