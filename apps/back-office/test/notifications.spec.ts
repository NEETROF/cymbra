import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { flushPromises, mount } from "@vue/test-utils";
import { setClientsForTest } from "@/lib/api";
import { i18n } from "@/i18n";
import { groupCategories, parseCategoryKey, useNotificationsStore } from "@/stores/notifications";
import type { FlagRow } from "@/stores/flags";
import NotificationsView from "@/views/NotificationsView.vue";
import { makeFakeClients } from "./fakes";

const boolVal = (v: boolean) => ({ kind: { case: "boolValue", value: v } });
const intVal = (v: number) => ({ kind: { case: "intValue", value: BigInt(v) } });

function def(p: Record<string, unknown> = {}) {
  return {
    key: "k",
    app: "all",
    valueType: "bool",
    defaultValue: boolVal(false),
    effectiveValue: boolVal(false),
    hasOverride: false,
    rolloutScope: "global",
    sensitive: false,
    doc: "",
    editable: true,
    updatedBy: "",
    updatedAt: "",
    ...p,
  };
}

/** The registry as it looks once a feature has declared one notification type. */
const DEFS = [
  // Not a notification key — must be filtered out.
  def({ key: "rating.enabled", app: "music" }),
  def({ key: "notifications.enabled", effectiveValue: boolVal(true), hasOverride: true }),
  def({ key: "notifications.category.practice_streak.enabled", effectiveValue: boolVal(true) }),
  def({
    key: "notifications.category.practice_streak.hour",
    valueType: "int",
    defaultValue: intVal(20),
    effectiveValue: intVal(20),
  }),
];

function row(p: Partial<FlagRow> & { key: string }): FlagRow {
  return {
    app: "all",
    valueType: "bool",
    defaultDisplay: "",
    effectiveDisplay: "",
    effectiveBool: false,
    hasOverride: false,
    rolloutScope: "global",
    sensitive: false,
    doc: "",
    editable: true,
    updatedBy: "",
    updatedAt: "",
    ...p,
  };
}

describe("notification key grouping", () => {
  it("parses a per-category key into its id and suffix", () => {
    expect(parseCategoryKey("notifications.category.practice_streak.enabled")).toEqual(["practice_streak", "enabled"]);
    expect(parseCategoryKey("notifications.category.practice_streak.hour")).toEqual(["practice_streak", "hour"]);
  });

  it("ignores keys that are not per-category", () => {
    expect(parseCategoryKey("notifications.enabled")).toBeNull();
    expect(parseCategoryKey("rating.enabled")).toBeNull();
    expect(parseCategoryKey("notifications.category.")).toBeNull();
    expect(parseCategoryKey("notifications.category.oops")).toBeNull();
  });

  it("pairs each category's two keys and orders by id", () => {
    const grouped = groupCategories([
      row({ key: "notifications.category.zeta.hour" }),
      row({ key: "notifications.category.alpha.enabled" }),
      row({ key: "notifications.category.alpha.hour" }),
      row({ key: "notifications.enabled" }),
    ]);
    expect(grouped.map((g) => g.category)).toEqual(["alpha", "zeta"]);
    expect(grouped[0].enabled?.key).toBe("notifications.category.alpha.enabled");
    expect(grouped[0].hour?.key).toBe("notifications.category.alpha.hour");
    // A category declared with no hour key is event-triggered, not broken.
    expect(grouped[1].enabled).toBeNull();
    expect(grouped[1].hour?.key).toBe("notifications.category.zeta.hour");
  });
});

describe("notifications store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("loads only the notification keys and groups the categories", async () => {
    const { clients } = makeFakeClients({ flagDefs: DEFS });
    setClientsForTest(clients);
    const store = useNotificationsStore();
    await store.load();

    expect(store.definitions.status).toBe("success");
    if (store.definitions.status === "success") {
      expect(store.definitions.data.map((r) => r.key)).not.toContain("rating.enabled");
    }
    expect(store.killSwitch?.key).toBe("notifications.enabled");
    expect(store.killSwitch?.effectiveBool).toBe(true);
    expect(store.categories).toHaveLength(1);
    expect(store.categories[0]).toMatchObject({ category: "practice_streak" });
    expect(store.categories[0].hour?.effectiveDisplay).toBe("20");
  });

  it("flipping a switch writes the opposite value and reloads", async () => {
    const { clients, state } = makeFakeClients({ flagDefs: DEFS });
    setClientsForTest(clients);
    const store = useNotificationsStore();
    await store.load();

    await store.setEnabled(store.killSwitch!, false);
    expect(state.setFlagCalls).toEqual([
      { key: "notifications.enabled", app: "all", enabled: false, rolloutScope: "global", confirm: false },
    ]);
    // The list is re-read so the panel shows server truth, not a local guess.
    expect(state.listDefinitionsCalls).toHaveLength(2);
  });

  it("saves a valid schedule hour", async () => {
    const { clients, state } = makeFakeClients({ flagDefs: DEFS });
    setClientsForTest(clients);
    const store = useNotificationsStore();
    await store.load();

    await store.setHour(store.categories[0].hour!, 8);
    expect(state.setConfigCalls).toHaveLength(1);
    expect(state.setConfigCalls[0]).toMatchObject({
      key: "notifications.category.practice_streak.hour",
      app: "all",
    });
    expect(state.setConfigCalls[0].value).toEqual({ kind: { case: "intValue", value: 8n } });
  });

  it("refuses an hour outside 0–23 without calling the API", async () => {
    const { clients, state } = makeFakeClients({ flagDefs: DEFS });
    setClientsForTest(clients);
    const store = useNotificationsStore();
    await store.load();

    for (const bad of [-1, 24, 7.5, Number.NaN]) {
      const outcome = await store.setHour(store.categories[0].hour!, bad);
      expect(outcome.status).toBe("error");
    }
    expect(state.setConfigCalls).toHaveLength(0);
  });

  it("resetting clears the override", async () => {
    const { clients, state } = makeFakeClients({ flagDefs: DEFS });
    setClientsForTest(clients);
    const store = useNotificationsStore();
    await store.load();

    await store.reset(store.categories[0].enabled!);
    expect(state.clearCalls).toEqual([
      { key: "notifications.category.practice_streak.enabled", app: "all", confirm: false },
    ]);
  });

  it("a failing load lands in the error state, not a throw", async () => {
    const { clients } = makeFakeClients({ flagDefs: DEFS, failFlags: true });
    setClientsForTest(clients);
    const store = useNotificationsStore();
    await store.load();

    expect(store.definitions.status).toBe("error");
    expect(store.killSwitch).toBeNull();
    expect(store.categories).toEqual([]);
  });
});

describe("notifications view", () => {
  beforeEach(() => setActivePinia(createPinia()));

  async function mountView(defs: unknown[]) {
    const { clients, state } = makeFakeClients({ flagDefs: defs });
    setClientsForTest(clients);
    const wrapper = mount(NotificationsView, { global: { plugins: [i18n] } });
    await flushPromises();
    return { wrapper, state };
  }

  it("renders the kill-switch and one row per declared category", async () => {
    const { wrapper } = await mountView(DEFS);
    expect(wrapper.find('[data-testid="kill-switch"]').text()).toBe("On");
    expect(wrapper.find('[data-testid="category-practice_streak"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="enable-practice_streak"]').text()).toBe("On");
    expect((wrapper.find('[data-testid="hour-practice_streak"]').element as HTMLInputElement).value).toBe("20");
  });

  it("says so when no category is declared yet", async () => {
    const { wrapper } = await mountView([def({ key: "notifications.enabled" })]);
    expect(wrapper.find('[data-testid="no-categories"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="category-practice_streak"]').exists()).toBe(false);
  });

  it("toggling the kill-switch turns it off through the store", async () => {
    const { wrapper, state } = await mountView(DEFS);
    await wrapper.find('[data-testid="kill-switch"]').trigger("click");
    await flushPromises();
    expect(state.setFlagCalls).toEqual([
      { key: "notifications.enabled", app: "all", enabled: false, rolloutScope: "global", confirm: false },
    ]);
  });

  it("editing and saving an hour writes it", async () => {
    const { wrapper, state } = await mountView(DEFS);
    await wrapper.find('[data-testid="hour-practice_streak"]').setValue("7");
    await wrapper.find('[data-testid="save-hour-practice_streak"]').trigger("click");
    await flushPromises();
    expect(state.setConfigCalls[0].value).toEqual({ kind: { case: "intValue", value: 7n } });
  });

  it("shows a load failure as a message, never a raw error", async () => {
    const { clients } = makeFakeClients({ flagDefs: DEFS, failFlags: true });
    setClientsForTest(clients);
    const wrapper = mount(NotificationsView, { global: { plugins: [i18n] } });
    await flushPromises();
    const alert = wrapper.find('[role="alert"]');
    expect(alert.exists()).toBe(true);
    expect(alert.text()).not.toContain("Error:");
  });
});
