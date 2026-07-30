import { describe, expect, it } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { createMemoryHistory, createRouter } from "vue-router";
import ScoreEditForm from "@/components/ScoreEditForm.vue";
import ScoreDetailView from "@/views/ScoreDetailView.vue";
import { useAuthStore } from "@/stores/auth";
import { setClientsForTest } from "@/lib/api";
import { i18n } from "@/i18n";
import { makeFakeClients, makeJwt } from "./fakes";

const hit = {
  id: "s1",
  title: "Clair de Lune",
  composer: "Debussy",
  arranger: "",
  level: "advanced",
  license: "CC0",
  source: "pdmx",
  timeSig: "3/4",
  noteCount: 240,
  tempoBpm: 66,
};

describe("ScoreEditForm", () => {
  const withI18n = { plugins: [i18n] };

  it("edits only the curatorial fields and emits them on submit", async () => {
    const w = mount(ScoreEditForm, {
      global: withI18n,
      props: { hit: hit as never, submitting: false, error: null },
    });
    // The form seeds from the hit's curatorial values.
    const title = w.get('input[type="text"]');
    expect((title.element as HTMLInputElement).value).toBe("Clair de Lune");

    await title.setValue("Clair de lune (rev.)");
    await w.get("form").trigger("submit");

    const payload = w.emitted("submit")?.[0]?.[0] as Record<string, string>;
    expect(payload).toEqual({
      title: "Clair de lune (rev.)",
      composer: "Debussy",
      arranger: "",
      level: "advanced",
    });
    // Only the four curatorial fields — never a derived facet.
    expect(Object.keys(payload)).toEqual(["title", "composer", "arranger", "level"]);
  });

  it("shows the derived facets as read-only (no inputs)", () => {
    const w = mount(ScoreEditForm, {
      global: withI18n,
      props: { hit: hit as never, submitting: false, error: null },
    });
    const derived = w.get("dl.derived");
    // The derived block renders values but carries no editable control.
    expect(derived.findAll("input, select, textarea")).toHaveLength(0);
    expect(derived.text()).toContain("3/4");
    expect(derived.text()).toContain("240");
    expect(derived.text()).toContain("66");
  });

  it("surfaces a localized submit error and disables the form while submitting", () => {
    const w = mount(ScoreEditForm, {
      global: withI18n,
      props: { hit: hit as never, submitting: true, error: "Something went wrong." },
    });
    const alert = w.get('[role="alert"]');
    expect(alert.text()).toBe("Something went wrong.");
    // Saving state disables the submit button and the inputs.
    expect(w.get("button.save").attributes("disabled")).toBeDefined();
    expect(w.get("button.save").text()).toBe("Saving…");
    expect(w.get('input[type="text"]').attributes("disabled")).toBeDefined();
  });
});

// --- View-level gate: the edit form is moderator/admin-only ---------------------

function mountDetail(roles: string[]) {
  setActivePinia(createPinia());
  const { clients, state } = makeFakeClients({
    hits: [hit],
    tokens: { accessToken: makeJwt({ roles, sub: "u1" }), refreshToken: "r" },
  });
  // Keep the notation/audio composables idle: no bytes → no wasm load in jsdom.
  (clients.score as unknown as { getCatalogScoreBytes: () => Promise<never> }).getCatalogScoreBytes = () =>
    Promise.reject(new Error("no bytes"));
  setClientsForTest(clients);

  const auth = useAuthStore();
  auth.setToken(makeJwt({ roles, sub: "u1" }));

  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: "/", name: "music-queue", component: { template: "<div/>" } },
      { path: "/s/:id", name: "music-score", component: { template: "<div/>" } },
    ],
  });
  const w = mount(ScoreDetailView, {
    props: { id: "s1" },
    global: {
      plugins: [i18n, router],
      stubs: { ScorePreview: true },
    },
  });
  return { w, state };
}

describe("ScoreDetailView edit gate", () => {
  it("shows the edit form for a moderator and saves via the store, then refreshes the hit", async () => {
    const { w, state } = mountDetail(["moderator"]);
    await flushPromises();
    const form = w.findComponent(ScoreEditForm);
    expect(form.exists()).toBe(true);

    // Emitting the form's submit drives the view's saveEdit → the edit RPC…
    form.vm.$emit("submit", { title: "New", composer: "C", arranger: "", level: "beginner" });
    await flushPromises();

    expect(state.editCalls).toEqual([{ scoreId: "s1", title: "New", composer: "C", arranger: "", level: "beginner" }]);
    // …and a fresh metadata fetch so the form/preview reflect the recomputed values
    // (mount does 1 fetchHit; the successful save triggers a 2nd).
    expect(state.getCatalogScoreCalls).toBe(2);
  });

  it("hides the edit form for a signed-in non-moderator", async () => {
    const { w } = mountDetail([]);
    await flushPromises();
    expect(w.findComponent(ScoreEditForm).exists()).toBe(false);
  });
});
