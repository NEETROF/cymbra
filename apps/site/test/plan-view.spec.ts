import { describe, expect, it } from "vitest";
import { WebAuthError } from "@cymbra/web-auth";
import {
  appleReturnUrl,
  codeFromQuery,
  identityLabel,
  humanError,
  manageAction,
  planSummary,
  productLabel,
  redeemError,
  redeemOutcome,
  transactionFromQuery,
} from "../src/lib/plan-view";
import { readConfig } from "../src/lib/config";
import { formatDate, t } from "../src/lib/i18n";
import type { PlanView } from "../src/lib/web-plans";

const free: PlanView = {
  plan: "free",
  source: null,
  ends_at: null,
  ends_without_renewal: false,
  trial_campaign_key: null,
  trial_campaign_name: null,
  trial_ends_at: null,
  betas: [],
  managed_on: null,
  can_purchase_here: true,
  purchase_channel: "web",
  products: ["premium_monthly", "premium_yearly"],
  unlocks: [],
};

describe("errors are localized, never raw", () => {
  it("maps HTTP statuses (fr / en) and unknown errors to generic copy", () => {
    expect(humanError("fr", new WebAuthError(401, "invalid credentials"))).toBe(t("fr", "errUnauthenticated"));
    expect(humanError("en", new WebAuthError(429, "too many"))).toBe(t("en", "errRate"));
    expect(humanError("en", new WebAuthError(412, "x"))).toBe(t("en", "errPrecondition"));
    expect(humanError("fr", new WebAuthError(500, "internal error"))).toBe(t("fr", "errGeneric"));
    expect(humanError("fr", new TypeError("network"))).toBe(t("fr", "errGeneric"));
    // The raw server text never leaks.
    expect(humanError("fr", new WebAuthError(401, "invalid credentials"))).not.toContain("invalid credentials");
  });

  it("redeem refusals stay neutral: unknown / used / revoked share one message", () => {
    const unknown = redeemError("fr", new WebAuthError(404, "code refused"));
    const used = redeemError("fr", new WebAuthError(404, "code refused"));
    expect(unknown).toBe(used);
    expect(unknown).toBe(t("fr", "errCodeInvalid"));
    expect(redeemError("en", new WebAuthError(412, "trial_already_active"))).toBe(t("en", "errCodeRefused"));
    expect(redeemError("en", new WebAuthError(409, "already_member"))).toBe(t("en", "errCodeRefused"));
    expect(redeemError("en", new WebAuthError(429, "rate"))).toBe(t("en", "errRate"));
    expect(redeemError("en", new WebAuthError(401, "sign in required"))).toBe(t("en", "errUnauthenticated"));
  });
});

describe("redeem outcome", () => {
  it("names the trial campaign with its end date, or the feature beta", () => {
    const trial = redeemOutcome("fr", {
      campaign_key: "beta-premium",
      campaign_name: "Bêta Premium",
      kind: "premium_trial",
      ends_at: "2026-11-14T10:00:00Z",
    });
    expect(trial.title).toBe(t("fr", "redeemedTitle"));
    expect(trial.body).toContain("Bêta Premium");
    expect(trial.body).toContain(formatDate("fr", "2026-11-14T10:00:00Z"));
    expect(trial.next).toBe(t("fr", "redeemedNext"));

    const feature = redeemOutcome("en", {
      campaign_key: "midi-drums",
      campaign_name: "MIDI drums",
      kind: "feature",
      ends_at: null,
    });
    expect(feature.body).toBe(t("en", "redeemedFeature", { name: "MIDI drums" }));
    // Never a price or a discount.
    expect(feature.body + trial.body).not.toMatch(/€|\$|%/);
  });
});

describe("query prefill", () => {
  it("reads and normalizes ?code=, and _ptxn", () => {
    expect(codeFromQuery("?code=abcd-efgh")).toBe("ABCD-EFGH");
    expect(codeFromQuery("code= x ")).toBe("X");
    expect(codeFromQuery("?other=1")).toBe("");
    expect(codeFromQuery("")).toBe("");
    expect(transactionFromQuery("?_ptxn=txn_01h")).toBe("txn_01h");
    expect(transactionFromQuery("?_ptxn=")).toBeNull();
    expect(transactionFromQuery("")).toBeNull();
  });
});

describe("account: manage action per channel", () => {
  it("web row → portal, store rows → the store page, free + open channel → purchase, else none", () => {
    expect(manageAction("fr", { ...free, plan: "premium", source: "web", managed_on: "web" })).toEqual({
      kind: "portal",
    });
    const apple = manageAction("fr", { ...free, plan: "premium", source: "apple", managed_on: "apple" });
    expect(apple.kind).toBe("store");
    if (apple.kind === "store") {
      expect(apple.url).toContain("apps.apple.com");
      expect(apple.note).toBe(t("fr", "manageStoreApple"));
    }
    const google = manageAction("en", { ...free, plan: "premium", source: "google", managed_on: "google" });
    expect(google.kind).toBe("store");
    if (google.kind === "store") expect(google.url).toContain("play.google.com");

    expect(manageAction("en", free)).toEqual({ kind: "purchase", products: ["premium_monthly", "premium_yearly"] });
    expect(manageAction("en", { ...free, can_purchase_here: false, products: [] })).toEqual({ kind: "none" });
  });

  it("summarizes the plan: free, premium renewing, cancelled with rights-end, trial", () => {
    expect(planSummary("fr", free)).toEqual({ title: t("fr", "planFree"), lines: [] });

    const renewing = planSummary("en", {
      ...free,
      plan: "premium",
      source: "web",
      managed_on: "web",
      ends_at: "2026-09-16T00:00:00Z",
      ends_without_renewal: false,
    });
    expect(renewing.title).toBe(t("en", "planPremium"));
    expect(renewing.lines).toEqual([
      t("en", "managedOn", { channel: t("en", "channelWeb") }),
      t("en", "renewsOn", { date: formatDate("en", "2026-09-16T00:00:00Z") }),
    ]);

    const cancelled = planSummary("fr", {
      ...free,
      plan: "premium",
      source: "apple",
      managed_on: "apple",
      ends_at: "2026-09-16T00:00:00Z",
      ends_without_renewal: true,
    });
    expect(cancelled.lines[1]).toBe(t("fr", "rightsEndOn", { date: formatDate("fr", "2026-09-16T00:00:00Z") }));

    const trial = planSummary("fr", {
      ...free,
      plan: "premium",
      source: "code",
      ends_at: "2026-11-14T00:00:00Z",
      ends_without_renewal: true,
      trial_campaign_key: "beta-premium",
      trial_campaign_name: "Bêta Premium",
      trial_ends_at: "2026-11-14T00:00:00Z",
    });
    expect(trial.title).toBe(t("fr", "planTrial"));
    expect(trial.lines.some((l) => l.includes("Bêta Premium"))).toBe(true);
    expect(trial.lines[0]).toBe(t("fr", "rightsEndOn", { date: formatDate("fr", "2026-11-14T00:00:00Z") }));
  });

  it("labels the known products", () => {
    expect(productLabel("fr", "premium_monthly")).toBe(t("fr", "productMonthly"));
    expect(productLabel("en", "pri_yearly_01")).toBe(t("en", "productYearly"));
    expect(productLabel("en", "pri_x")).toBe("pri_x");
  });
});

describe("config", () => {
  it("hides providers without a client id and defaults the API url + production env", () => {
    const c = readConfig({} as ImportMetaEnv);
    expect(c.googleClientId).toBeNull();
    expect(c.appleClientId).toBeNull();
    expect(c.paddleEnv).toBe("production");
    expect(c.apiUrl).toBe("http://localhost:8081");
    const d = readConfig({
      PUBLIC_API_URL: "https://api.cymbra.app",
      PUBLIC_GOOGLE_CLIENT_ID: "g",
      PUBLIC_PADDLE_ENV: "sandbox",
      PUBLIC_PADDLE_CLIENT_TOKEN: " test_tok ",
    } as ImportMetaEnv);
    expect(d.googleClientId).toBe("g");
    expect(d.paddleEnv).toBe("sandbox");
    expect(d.paddleClientToken).toBe("test_tok");
  });
});

describe("Apple return URL", () => {
  it("is the canonical registered page URL: no trailing slash, no /en prefix", () => {
    expect(appleReturnUrl("https://cymbra.app", "/redeem/")).toBe("https://cymbra.app/redeem");
    expect(appleReturnUrl("https://cymbra.app", "/en/redeem/")).toBe("https://cymbra.app/redeem");
    expect(appleReturnUrl("https://cymbra.app", "/en/account")).toBe("https://cymbra.app/account");
    expect(appleReturnUrl("https://cymbra.app", "/account")).toBe("https://cymbra.app/account");
    expect(appleReturnUrl("https://cymbra.app", "/en/")).toBe("https://cymbra.app/");
    expect(appleReturnUrl("https://cymbra.app", "/english/")).toBe("https://cymbra.app/english");
  });
});

describe("identity labels", () => {
  it("names the provider, with the e-mail for the local identity only", () => {
    expect(identityLabel("fr", { provider: "local", email: "a@x.dev", linked_at: 1 })).toBe(
      t("fr", "methodEmail", { email: "a@x.dev" }),
    );
    expect(identityLabel("en", { provider: "google", email: null, linked_at: 1 })).toBe(t("en", "methodGoogle"));
    expect(identityLabel("en", { provider: "apple", email: null, linked_at: 1 })).toBe(t("en", "methodApple"));
    expect(identityLabel("en", { provider: "github", email: null, linked_at: 1 })).toBe("github");
  });
});
