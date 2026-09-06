import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { flushPromises, mount, RouterLinkStub, type VueWrapper } from "@vue/test-utils";
import { createMemoryHistory, createRouter, type Router } from "vue-router";
import { i18n } from "@/i18n";
import UserDetailView from "@/views/UserDetailView.vue";
import { setClientsForTest } from "@/lib/api";
import { useAuthStore } from "@/stores/auth";
import { makeFakeClients, makeJwt } from "./fakes";

// The account detail page (change: restructure-back-office-users-console): one account,
// one address. What is asserted here is what the split has to keep true — the page
// stands on its own by id, it never paints one account's data under another's name, and
// the subscription block is absent (not empty) for an admin outside the `music` scope.

const ada = {
  userId: "u-ada",
  handle: "ada",
  displayName: "Ada Lovelace",
  rolesByScope: [{ scope: "music", roles: [] as string[] }],
};
const bob = {
  userId: "u-bob",
  handle: "bob",
  displayName: "Bob Ross",
  rolesByScope: [{ scope: "music", roles: [] as string[] }],
};

/** Sign in as an admin of the given scopes (the plan surfaces are `music`-only). */
function signIn(scopes: Record<string, string[]>) {
  useAuthStore().setToken(makeJwt({ sub: "admin-1", roles: ["admin"], roles_by_scope: scopes, exp: 4102444800 }));
}

/** The page reads its tab from the URL, so the mount needs a real router. A memory
 *  history keeps it in-process; `tab` selects the section under test. */
async function mountDetail(userId: string, data: Record<string, unknown> = {}, tab?: "subscription" | "roles") {
  const { clients, state } = makeFakeClients(data);
  setClientsForTest(clients);
  const router: Router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: "/users", name: "users", component: { template: "<div />" } },
      { path: "/users/:userId", name: "user-detail", component: UserDetailView, props: true },
    ],
  });
  await router.push({ name: "user-detail", params: { userId }, query: tab ? { tab } : {} });
  await router.isReady();
  const w = mount(UserDetailView, {
    props: { userId },
    global: { plugins: [i18n, router], stubs: { RouterLink: RouterLinkStub } },
  });
  await flushPromises();
  return { w, state, router };
}

const buttonNamed = (w: VueWrapper, label: string) => w.findAll("button").find((b) => b.text() === label);

describe("account detail page", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    signIn({ music: ["admin"] });
  });

  it("loads the account by id — a deep link needs no directory page first", async () => {
    const { w, state } = await mountDetail("u-bob", { accounts: [ada, bob], plans: {} });

    expect(state.listAccountsCalls).toEqual([{ query: "", limit: 1, offset: 0, ids: ["u-bob"] }]);
    expect(w.text()).toContain("bob");
    expect(w.text()).not.toContain("Ada Lovelace");
    // The plan block is loaded for the same account, by id — no handle to retype.
    expect(state.lookupCalls).toEqual([{ userId: "u-bob", handle: "" }]);
  });

  it("an unknown id shows a localized not-found state, not a raw error", async () => {
    const { w } = await mountDetail("nobody", { accounts: [ada] });

    expect(w.text()).toContain("Account not found");
    expect(w.text()).not.toContain("[");
    expect(w.find('[data-testid="effective-plan"]').exists()).toBe(false);
  });

  it("shows the subscription of the account", async () => {
    const lookup = {
      userId: "u-ada",
      snapshot: { plan: "premium", source: "admin", endsAt: "2027-01-01T00:00:00Z", betas: [] },
      rows: [{ id: "e1", source: "admin", startsAt: "2026-01-01T00:00:00Z", status: "active", providerRef: "" }],
      memberships: [],
    };
    const grants = [
      {
        targetUserId: "u-ada",
        scope: "music",
        role: "moderator",
        action: "grant",
        actingAdmin: "019f60be-6cd9-74e2-a600-9893bd2aaa3a",
        actingAdminHandle: "bossadmin",
        at: 1_700_000_000n,
      },
    ];
    const { w } = await mountDetail("u-ada", { accounts: [ada], lookup, grants });

    expect(w.find('[data-testid="effective-plan"]').text()).toContain("Premium");
    expect(w.find('[data-testid="entitlements"]').text()).toContain("admin");
    // The roles half is a tab away, not on this one.
    expect(w.find('[data-testid="role-history"]').exists()).toBe(false);

    // On the roles tab, the audit history reads by handle, never by raw id.
    const { w: roles } = await mountDetail("u-ada", { accounts: [ada], lookup, grants }, "roles");
    const history = roles.find('[data-testid="role-history"]');
    expect(history.text()).toContain("bossadmin");
    expect(history.text()).not.toContain("019f60be-6cd9");
    expect(roles.find('[data-testid="effective-plan"]').exists()).toBe(false);
  });

  it("the two halves are tabs, and the chosen one rides in the URL", async () => {
    const { w, router } = await mountDetail("u-ada", { accounts: [ada] });

    const tabs = w.findAll('[role="tab"]');
    expect(tabs.map((t) => t.text())).toEqual(["Subscription", "Roles"]);
    expect(tabs[0].attributes("aria-selected")).toBe("true");

    await tabs[1].trigger("click");
    await flushPromises();

    // Addressable: a reload or a shared link lands on the same section.
    expect(router.currentRoute.value.query.tab).toBe("roles");
    expect(w.find('[data-testid="role-history"]').exists()).toBe(true);
  });

  it("an unknown tab in the URL falls back to the first rather than showing nothing", async () => {
    const { w } = await mountDetail("u-ada", { accounts: [ada] }, "nonsense" as "roles");

    expect(w.find('[data-testid="effective-plan"]').exists()).toBe(true);
  });

  it("a single available section shows no tab strip", async () => {
    signIn({ live: ["admin"] });
    const { w } = await mountDetail("u-ada", { accounts: [ada] });

    // Nothing to switch between: an inert single tab would be noise.
    expect(w.findAll('[role="tab"]')).toHaveLength(0);
    expect(w.find('[data-testid="role-history"]').exists()).toBe(true);
  });

  it("an admin outside the music scope gets NO subscription block and no plan RPC", async () => {
    signIn({ live: ["admin"] });
    const { w, state } = await mountDetail("u-ada", { accounts: [ada] });

    // Absent, not empty: an empty "Subscription" heading would read as "no plan".
    expect(w.text()).not.toContain("Subscription");
    expect(w.find('[data-testid="effective-plan"]').exists()).toBe(false);
    expect(state.lookupCalls).toEqual([]);
    expect(state.plansForAccountsCalls).toEqual([]);
    // The rest of the page is still theirs to work with.
    expect(w.find('[data-testid="role-history"]').exists()).toBe(true);
  });

  it("grants a role in a named scope and re-reads THIS account", async () => {
    signIn({ global: ["admin"] });
    const { w, state } = await mountDetail("u-ada", { accounts: [ada] }, "roles");

    // Every authorized scope is on the page at once — no selector to remember.
    const grant = w.find('[aria-label="Grant moderator in live"]');
    expect(grant.exists()).toBe(true);
    await grant.trigger("click");
    await flushPromises();

    expect(state.grantCalls).toEqual([{ userId: "u-ada", scope: "live", role: "moderator" }]);
    // Two single-account reads (initial + refresh); the directory is not on screen.
    expect(state.listAccountsCalls.every((c) => c.ids?.[0] === "u-ada")).toBe(true);
    expect(state.listAccountsCalls).toHaveLength(2);
  });

  it("a single-scope admin manages only their scope", async () => {
    const { w } = await mountDetail("u-ada", { accounts: [ada] }, "roles");

    expect(w.find('[aria-label="Grant moderator in music"]').exists()).toBe(true);
    expect(w.find('[aria-label="Grant moderator in live"]').exists()).toBe(false);
    expect(w.find('[aria-label="Grant admin in global"]').exists()).toBe(false);
  });

  it("revoking every session is confirmed in-app before it fires", async () => {
    const { w, state } = await mountDetail("u-ada", { accounts: [ada] });

    await buttonNamed(w, "Revoke sessions")!.trigger("click");
    await flushPromises();
    // Nothing has been cut off yet — the question is asked in a modal, never
    // `window.confirm`, which blocks the renderer and is unreachable from e2e.
    const dialog = w.find("dialog");
    expect(dialog.exists()).toBe(true);
    expect(state.revokeAccountSessionsCalls).toEqual([]);

    await dialog
      .findAll("button")
      .find((b) => b.text() === "Confirm")!
      .trigger("click");
    await flushPromises();

    expect(state.revokeAccountSessionsCalls).toEqual(["u-ada"]);
  });

  it("revoking an entitlement asks in-app and will not send without a reason", async () => {
    const lookup = {
      userId: "u-ada",
      snapshot: { plan: "premium", source: "admin", betas: [] },
      rows: [{ id: "e1", source: "admin", startsAt: "2026-01-01T00:00:00Z", status: "active", providerRef: "" }],
      memberships: [],
    };
    const { w, state } = await mountDetail("u-ada", { accounts: [ada], lookup });

    await buttonNamed(w, "Revoke")!.trigger("click");
    await flushPromises();

    const dialog = w.find("dialog");
    const confirm = () => dialog.findAll("button").find((b) => b.text() === "Confirm")!;
    // The reason is audited, so an empty one must not get through.
    expect(confirm().attributes("disabled")).toBeDefined();
    expect(state.revokeEntitlementCalls).toEqual([]);

    await dialog.get("input").setValue("granted by mistake");
    await confirm().trigger("click");
    await flushPromises();

    expect(state.revokeEntitlementCalls).toEqual([{ entitlementId: "e1", reason: "granted by mistake" }]);
  });

  it("a campaign the account is already a live member of is offered disabled, with the reason", async () => {
    // Enrolling into it is refused by the server (`already_member`); the console can see
    // that coming from the memberships it is already showing.
    const lookup = {
      userId: "u-ada",
      snapshot: { plan: "free", betas: [] },
      rows: [],
      memberships: [{ campaignKey: "midi-drums", campaignName: "MIDI drums", kind: "feature", userId: "u-ada" }],
    };
    const campaigns = [
      { id: "c1", key: "midi-drums", name: "MIDI drums", kind: "feature", acceptsEnrolment: true },
      { id: "c2", key: "spring-trial", name: "Spring trial", kind: "premium_trial", acceptsEnrolment: true },
    ];
    const { w } = await mountDetail("u-ada", { accounts: [ada], lookup, campaigns });

    await buttonNamed(w, "Enrol in campaign")!.trigger("click");
    await flushPromises();

    const options = w.findAll("dialog option");
    const drums = options.find((o) => o.text().includes("MIDI drums"))!;
    expect(drums.attributes("disabled")).toBeDefined();
    expect(drums.text()).toContain("already a member");
    // The one it CAN join is offered, and preselected.
    const trial = options.find((o) => o.text().includes("Spring trial"))!;
    expect(trial.attributes("disabled")).toBeUndefined();
    expect((w.find("dialog select").element as HTMLSelectElement).value).toBe("spring-trial");
  });

  it("a REVOKED membership does not disable its campaign — re-enrolling is the point", async () => {
    const lookup = {
      userId: "u-ada",
      snapshot: { plan: "free", betas: [] },
      rows: [],
      memberships: [
        {
          campaignKey: "midi-drums",
          campaignName: "MIDI drums",
          kind: "feature",
          userId: "u-ada",
          revokedAt: "2026-08-10T00:00:00Z",
        },
      ],
    };
    const campaigns = [{ id: "c1", key: "midi-drums", name: "MIDI drums", kind: "feature", acceptsEnrolment: true }];
    const { w } = await mountDetail("u-ada", { accounts: [ada], lookup, campaigns });

    await buttonNamed(w, "Enrol in campaign")!.trigger("click");
    await flushPromises();

    const drums = w.findAll("dialog option").find((o) => o.text().includes("MIDI drums"))!;
    expect(drums.attributes("disabled")).toBeUndefined();
    expect(drums.text()).not.toContain("already a member");
  });

  it("shows the audited reason behind each plan change", async () => {
    // The console demands a free-text reason on every grant, enrolment and revocation.
    // It used to be written and never read back — a justification the operator typed for
    // an audit trail no surface could show.
    const planAudit = [
      {
        action: "revoke_membership",
        actingAdmin: "019f60be-6cd9-74e2-a600-9893bd2aaa3a",
        actingAdminHandle: "bossadmin",
        targetRef: "midi-drums",
        reason: "left the beta programme",
        at: 1_756_000_000n,
      },
      { action: "grant", actingAdmin: "u-other", targetRef: undefined, reason: "beta thanks", at: 1_755_000_000n },
    ];
    const { w, state } = await mountDetail("u-ada", { accounts: [ada], planAudit });

    expect(state.planAuditCalls).toEqual([{ userId: "u-ada", limit: 50 }]);
    const table = w.find('[data-testid="plan-audit"]');
    expect(table.text()).toContain("left the beta programme");
    expect(table.text()).toContain("beta thanks");
    expect(table.text()).toContain("midi-drums");
    // Actions read as words, and the admin by handle rather than a raw id.
    expect(table.text()).toContain("membership revoked");
    expect(table.text()).toContain("bossadmin");
    expect(table.text()).not.toContain("019f60be-6cd9");
  });

  it("an account with nothing audited says so, and asks for no audit outside the music scope", async () => {
    const { w } = await mountDetail("u-ada", { accounts: [ada] });
    expect(w.find('[data-testid="plan-audit"]').text()).toContain("No plan change recorded");

    signIn({ live: ["admin"] });
    const { w: w2, state } = await mountDetail("u-ada", { accounts: [ada] });
    expect(w2.find('[data-testid="plan-audit"]').exists()).toBe(false);
    expect(state.planAuditCalls).toEqual([]);
  });

  it("switching accounts never paints the previous account's rights under the new name", async () => {
    const lookup = {
      userId: "u-ada",
      snapshot: { plan: "premium", source: "admin", betas: [] },
      rows: [{ id: "e1", source: "admin", startsAt: "2026-01-01T00:00:00Z", status: "active", providerRef: "" }],
      memberships: [],
    };
    const { w } = await mountDetail("u-ada", { accounts: [ada, bob], lookup });
    expect(w.find('[data-testid="effective-plan"]').text()).toContain("Premium");

    // Navigating to another account re-runs the load. Between the two, the shared store
    // slots are dropped — the page must never show account A's plan next to B's handle.
    await w.setProps({ userId: "u-bob" });
    expect(w.find('[data-testid="effective-plan"]').exists()).toBe(false);

    await flushPromises();
    expect(w.text()).toContain("bob");
  });
});
