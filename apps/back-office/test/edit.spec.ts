import { afterEach, describe, expect, it } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { createMemoryHistory, createRouter } from "vue-router";
import ScoreEditDrawer from "@/components/ScoreEditDrawer.vue";
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

describe("ScoreEditDrawer", () => {
  const withI18n = { plugins: [i18n] };
  const props = (over: Record<string, unknown> = {}) => ({
    open: true,
    hit: hit as never,
    submitting: false,
    error: null,
    ...over,
  });
  // The drawer teleports to <body>, so assertions read the document, not the wrapper.
  const field = (label: string) =>
    [...document.querySelectorAll<HTMLElement>("dialog.drawer label")].find((l) => l.textContent?.startsWith(label))!;

  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("edits only the curatorial fields and emits them on submit", async () => {
    const w = mount(ScoreEditDrawer, { global: withI18n, props: props(), attachTo: document.body });
    // The form seeds from the hit's curatorial values.
    const title = field("Title").querySelector("input") as HTMLInputElement;
    expect(title.value).toBe("Clair de Lune");

    title.value = "Clair de lune (rev.)";
    title.dispatchEvent(new Event("input"));
    await flushPromises();
    document.querySelector("form.edit")!.dispatchEvent(new Event("submit"));

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

  it("renders nothing while closed", () => {
    mount(ScoreEditDrawer, { global: withI18n, props: props({ open: false }), attachTo: document.body });
    expect(document.querySelector("dialog.drawer")).toBeNull();
  });

  it("surfaces a localized submit error and disables the form while submitting", () => {
    const w = mount(ScoreEditDrawer, {
      global: withI18n,
      props: props({ submitting: true, error: "Something went wrong." }),
      attachTo: document.body,
    });
    const alert = document.querySelector('[role="alert"]')!;
    expect(alert.textContent).toBe("Something went wrong.");
    // Saving state disables the submit button and the inputs.
    const save = document.querySelector("button.save") as HTMLButtonElement;
    expect(save.disabled).toBe(true);
    expect(save.textContent?.trim()).toBe("Saving…");
    expect((field("Title").querySelector("input") as HTMLInputElement).disabled).toBe(true);
    // Closing is always available, even mid-save it emits (the parent decides).
    expect(w.emitted("close")).toBeUndefined();
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
  it("opens the drawer for a moderator and saves via the store, then refreshes the hit", async () => {
    const { w, state } = mountDetail(["moderator"]);
    await flushPromises();
    const drawer = w.findComponent(ScoreEditDrawer);
    // The drawer is mounted but closed until the moderator asks for it.
    expect(drawer.props("open")).toBe(false);
    await w.get("button.edit").trigger("click");
    expect(drawer.props("open")).toBe(true);

    // Emitting the drawer's submit drives the view's saveEdit → the edit RPC…
    drawer.vm.$emit("submit", { title: "New", composer: "C", arranger: "", level: "beginner" });
    await flushPromises();

    expect(state.editCalls).toEqual([{ scoreId: "s1", title: "New", composer: "C", arranger: "", level: "beginner" }]);
    // …and a fresh metadata fetch so the summary reflects the recomputed values
    // (mount does 1 fetchHit; the successful save triggers a 2nd).
    expect(state.getCatalogScoreCalls).toBe(2);
    // A successful save closes the drawer; the summary underneath is the surface.
    expect(drawer.props("open")).toBe(false);
  });

  it("offers no edit entry point to a signed-in non-moderator", async () => {
    const { w } = mountDetail([]);
    await flushPromises();
    expect(w.find("button.edit").exists()).toBe(false);
    expect(w.findComponent(ScoreEditDrawer).props("open")).toBe(false);
  });
});
