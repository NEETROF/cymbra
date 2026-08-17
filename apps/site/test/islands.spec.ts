import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { WebAuthError, type WebAuthClient } from "@cymbra/web-auth";
import RedeemIsland from "../src/components/RedeemIsland.vue";
import AccountIsland from "../src/components/AccountIsland.vue";
import CheckoutIsland from "../src/components/CheckoutIsland.vue";
import { setClientsForTest } from "../src/lib/session";
import { t } from "../src/lib/i18n";
import type { PlanView, WebPlansClient } from "../src/lib/web-plans";

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
  products: ["premium_monthly"],
  unlocks: [],
};

function fakeAuth(over: Partial<WebAuthClient> = {}): WebAuthClient {
  return {
    signInLocal: vi.fn(async () => ({ accessToken: "tok-local" })),
    signInOidc: vi.fn(async () => ({ accessToken: "tok-oidc" })),
    // No cookie by default: boot lands on the sign-in form.
    refresh: vi.fn(async () => {
      throw new WebAuthError(401, "no session");
    }),
    logout: vi.fn(async () => undefined),
    ...over,
  };
}

function fakePlans(over: Partial<WebPlansClient> = {}): WebPlansClient {
  return {
    me: vi.fn(async () => free),
    account: vi.fn(async () => ({
      handle: "neetrof",
      display_name: null,
      locale: "fr",
      identities: [
        { provider: "google", email: null, linked_at: 10 },
        { provider: "local", email: "g@example.org", linked_at: 20 },
      ],
    })),
    redeem: vi.fn(async () => ({
      campaign_key: "beta-premium",
      campaign_name: "Bêta Premium",
      kind: "premium_trial",
      ends_at: "2026-11-14T10:00:00Z",
    })),
    checkout: vi.fn(async () => ({ checkout_url: "https://cymbra.app/checkout?_ptxn=txn_1" })),
    portal: vi.fn(async () => ({ portal_url: "https://portal.example/s" })),
    ...over,
  };
}

beforeEach(() => {
  vi.restoreAllMocks();
  window.history.replaceState({}, "", "/redeem");
});

describe("RedeemIsland", () => {
  it("shows the sign-in form without a cookie, keeps the ?code= prefill, redeems after sign-in", async () => {
    window.history.replaceState({}, "", "/redeem?code=abcd-1234");
    const auth = fakeAuth();
    const plans = fakePlans();
    setClientsForTest(auth, plans);
    const w = mount(RedeemIsland, { props: { lang: "fr" } });
    await flushPromises();

    // Signed out → the form; nothing stored in web storage.
    expect(w.text()).toContain(t("fr", "signInTitle"));
    expect(localStorage.length).toBe(0);

    await w.find('input[name="email"]').setValue("a@x.dev");
    await w.find('input[name="password"]').setValue("pw");
    await w.find("form").trigger("submit");
    await flushPromises();
    expect(auth.signInLocal).toHaveBeenCalledWith("a@x.dev", "pw", "web");

    // Now the redeem form, prefilled and normalized.
    const code = w.find<HTMLInputElement>('input[name="code"]');
    expect(code.element.value).toBe("ABCD-1234");
    await w.find("form").trigger("submit");
    await flushPromises();
    expect(plans.redeem).toHaveBeenCalledWith("tok-local", "ABCD-1234");
    const outcome = w.find('[data-testid="redeem-outcome"]');
    expect(outcome.exists()).toBe(true);
    expect(outcome.text()).toContain("Bêta Premium");
    expect(outcome.text()).toContain(t("fr", "redeemedNext"));
  });

  it("boots silently from the cookie and shows neutral refusals", async () => {
    const auth = fakeAuth({ refresh: vi.fn(async () => ({ accessToken: "tok-cookie" })) });
    const plans = fakePlans({
      redeem: vi.fn(async () => {
        throw new WebAuthError(404, "code refused");
      }),
    });
    setClientsForTest(auth, plans);
    const w = mount(RedeemIsland, { props: { lang: "en" } });
    await flushPromises();
    // No sign-in form: straight to the redeem form.
    expect(w.text()).not.toContain(t("en", "signInIntro"));
    await w.find('input[name="code"]').setValue("NOPE");
    await w.find("form").trigger("submit");
    await flushPromises();
    expect(w.find('[role="alert"]').text()).toBe(t("en", "errCodeInvalid"));
    expect(w.text()).not.toContain("code refused");

    // Throttled → the rate-limit copy.
    (plans.redeem as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new WebAuthError(429, "too many"));
    await w.find("form").trigger("submit");
    await flushPromises();
    expect(w.find('[role="alert"]').text()).toBe(t("en", "errRate"));
  });

  it("sign-out clears the session and returns to the form", async () => {
    const auth = fakeAuth({ refresh: vi.fn(async () => ({ accessToken: "tok-cookie" })) });
    setClientsForTest(auth, fakePlans());
    const w = mount(RedeemIsland, { props: { lang: "fr" } });
    await flushPromises();
    await w.find(".signout a").trigger("click");
    await flushPromises();
    expect(auth.logout).toHaveBeenCalled();
    expect(w.text()).toContain(t("fr", "signInTitle"));
  });
});

describe("AccountIsland", () => {
  it("shows the plan and offers the web checkout to a free account on the open web channel", async () => {
    const auth = fakeAuth({ refresh: vi.fn(async () => ({ accessToken: "tok-cookie" })) });
    const plans = fakePlans();
    setClientsForTest(auth, plans);
    const assign = vi.fn();
    vi.stubGlobal("location", { ...window.location, assign, search: "", origin: "https://cymbra.app" });
    const w = mount(AccountIsland, { props: { lang: "en" } });
    await flushPromises();
    expect(plans.me).toHaveBeenCalledWith("tok-cookie");
    expect(plans.account).toHaveBeenCalledWith("tok-cookie");
    // Who is signed in: handle + sign-in methods, never an OIDC subject.
    const who = w.find('[data-testid="identity"]');
    expect(who.text()).toContain("@neetrof");
    expect(who.text()).toContain(t("en", "methodGoogle"));
    expect(who.text()).toContain(t("en", "methodEmail", { email: "g@example.org" }));
    expect(w.find('[data-testid="plan-title"]').text()).toBe(t("en", "planFree"));
    expect(w.text()).toContain(t("en", "noBetas"));
    const buy = w.find(".products button");
    expect(buy.text()).toContain(t("en", "goPremium"));
    await buy.trigger("click");
    await flushPromises();
    expect(plans.checkout).toHaveBeenCalledWith("tok-cookie", "premium_monthly");
    expect(assign).toHaveBeenCalledWith("https://cymbra.app/checkout?_ptxn=txn_1");
    vi.unstubAllGlobals();
  });

  it("web subscriber → portal fetched at click time; store subscriber → store link, no purchase", async () => {
    const auth = fakeAuth({ refresh: vi.fn(async () => ({ accessToken: "tok-cookie" })) });
    const web: PlanView = {
      ...free,
      plan: "premium",
      source: "web",
      managed_on: "web",
      ends_at: "2026-09-16T00:00:00Z",
      can_purchase_here: false,
      products: [],
      betas: [
        { campaign_key: "midi-drums", campaign_name: "MIDI drums", kind: "feature", joined_at: "2026-08-01T00:00:00Z", ends_at: null },
      ],
    };
    const plans = fakePlans({ me: vi.fn(async () => web) });
    setClientsForTest(auth, plans);
    const assign = vi.fn();
    vi.stubGlobal("location", { ...window.location, assign, search: "", origin: "https://cymbra.app" });
    const w = mount(AccountIsland, { props: { lang: "fr" } });
    await flushPromises();
    expect(w.find('[data-testid="plan-title"]').text()).toBe(t("fr", "planPremium"));
    expect(w.text()).toContain("MIDI drums");
    const manage = w.find(".manage button");
    expect(manage.text()).toBe(t("fr", "manage"));
    await manage.trigger("click");
    await flushPromises();
    expect(plans.portal).toHaveBeenCalledWith("tok-cookie");
    expect(assign).toHaveBeenCalledWith("https://portal.example/s");
    vi.unstubAllGlobals();

    const apple: PlanView = { ...web, source: "apple", managed_on: "apple", betas: [] };
    setClientsForTest(auth, fakePlans({ me: vi.fn(async () => apple) }));
    const w2 = mount(AccountIsland, { props: { lang: "en" } });
    await flushPromises();
    expect(w2.text()).toContain(t("en", "manageStoreApple"));
    expect(w2.find(".manage a").attributes("href")).toContain("apps.apple.com");
    expect(w2.find(".products").exists()).toBe(false);
  });
});

describe("CheckoutIsland", () => {
  it("explains when there is no transaction to pay", async () => {
    window.history.replaceState({}, "", "/checkout");
    const w = mount(CheckoutIsland, { props: { lang: "fr", doneUrl: "/checkout/done" } });
    await flushPromises();
    expect(w.find('[data-testid="checkout-missing"]').text()).toBe(t("fr", "checkoutMissing"));
  });

  it("without a Paddle token, says web checkout is unavailable (no script injected)", async () => {
    window.history.replaceState({}, "", "/checkout?_ptxn=txn_1");
    const w = mount(CheckoutIsland, { props: { lang: "en", doneUrl: "/en/checkout/done" } });
    await flushPromises();
    expect(w.text()).toBe(t("en", "checkoutUnavailable"));
    expect(document.querySelector('script[src*="paddle"]')).toBeNull();
  });
});
