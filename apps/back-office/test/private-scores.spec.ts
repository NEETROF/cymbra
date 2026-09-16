import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { flushPromises, mount, RouterLinkStub } from "@vue/test-utils";
import { createMemoryHistory, createRouter } from "vue-router";
import { i18n } from "@/i18n";
import PrivateScoresView from "@/views/PrivateScoresView.vue";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients } from "./fakes";

// The Private scores page (change: restructure-back-office-navigation): the lookup
// criteria ride in the URL, so an account page can open it filtered on an owner and a
// lookup survives a reload; each result links back to its owner's account.

const score = {
  id: "s1",
  ownerId: "u-ada",
  title: "Reported Piece",
  composer: "Anon",
  sizeBytes: 1024n,
  createdAt: 1_760_000_000n,
  rightsBasis: "private_use",
};

async function mountAt(query: Record<string, string> = {}) {
  const { clients } = makeFakeClients();
  const searches: { ownerId?: string; title?: string }[] = [];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (clients.score as any).adminSearchUserScores = vi.fn(async (req: { ownerId?: string; title?: string }) => {
    searches.push({ ownerId: req.ownerId, title: req.title });
    return { scores: [score] };
  });
  setClientsForTest(clients);
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: "/music/private-scores", name: "music-private-scores", component: PrivateScoresView },
      { path: "/admin/users/:userId", name: "admin-user-detail", component: { template: "<div />" } },
    ],
  });
  await router.push({ name: "music-private-scores", query });
  await router.isReady();
  const w = mount(PrivateScoresView, {
    global: { plugins: [i18n, router], stubs: { RouterLink: RouterLinkStub } },
  });
  await flushPromises();
  return { w, router, searches };
}

describe("private scores page", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("runs the lookup for the owner named in the URL, with the form filled in", async () => {
    const { w, searches } = await mountAt({ owner: "u-ada" });

    expect(searches).toEqual([{ ownerId: "u-ada", title: undefined }]);
    expect((w.findAll("input")[0].element as HTMLInputElement).value).toBe("u-ada");
    expect(w.text()).toContain("Reported Piece");
  });

  it("does not search when the URL carries no criterion", async () => {
    const { w, searches } = await mountAt();

    expect(searches).toEqual([]);
    expect(w.text()).toContain("Private scores — takedown on notice");
  });

  it("submitting the form writes the criteria to the URL, which runs the lookup", async () => {
    const { w, router, searches } = await mountAt();

    await w.findAll("input")[1].setValue("Reported");
    await w.find("form").trigger("submit");
    await flushPromises();

    expect(router.currentRoute.value.query).toEqual({ title: "Reported" });
    expect(searches).toEqual([{ ownerId: undefined, title: "Reported" }]);
  });

  it("submitting the same criteria again searches again", async () => {
    const { w, searches } = await mountAt({ title: "Reported" });

    await w.find("form").trigger("submit");
    await flushPromises();

    expect(searches).toHaveLength(2);
  });

  it("links each owner to that owner's account page, by a short id that keeps the full one", async () => {
    const { w } = await mountAt({ owner: "u-ada" });

    const link = w.findAllComponents(RouterLinkStub).find((l) => l.text() === "UADA");
    expect(link?.props().to).toEqual({ name: "admin-user-detail", params: { userId: "u-ada" } });
    expect(link?.attributes("title")).toBe("u-ada");
    expect(link?.attributes("aria-label")).toBe("Account u-ada");
  });

  it("names the rights basis in words, and shows an unknown one as stored", async () => {
    const { w } = await mountAt({ owner: "u-ada" });
    expect(w.find("tbody").text()).toContain("Private use");
    expect(w.find("tbody").text()).not.toContain("private_use");

    score.rightsBasis = "licensed";
    const again = await mountAt({ owner: "u-ada" });
    expect(again.w.find("tbody").text()).toContain("licensed");
    score.rightsBasis = "private_use";
  });

  it("keeps the results table in a card that scrolls on its own", async () => {
    const { w } = await mountAt({ owner: "u-ada" });
    expect(w.find(".table-card table").exists()).toBe(true);
  });
});
