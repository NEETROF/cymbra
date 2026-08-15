import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { flushPromises, mount } from "@vue/test-utils";
import SoundFontsView from "@/views/SoundFontsView.vue";
import { useAuthStore } from "@/stores/auth";
import { setClientsForTest } from "@/lib/api";
import { i18n } from "@/i18n";
import { makeFakeClients, makeJwt } from "./fakes";

// Reward pricing on the Sound fonts screen (change: add-soundfont-reward-pricing).
// Driven over the fake client seam; the drawer's audio preview is mocked out (no
// wasm/audio in jsdom).
vi.mock("@/composables/useScorePlayer", () => ({
  useScorePlayer: () => ({
    audio: { value: { status: "idle" } },
    playing: { value: false },
    canPlay: { value: false },
    schedule: { value: { status: "idle" } },
    elapsedMs: { value: 0 },
    toggle: vi.fn(),
    stop: vi.fn(),
    playFrom: vi.fn(),
  }),
}));

const t = i18n.global.t;

function row(extra: Record<string, unknown> = {}) {
  return {
    id: "ydp-grand",
    label: "YDP Grand",
    objectKey: "ydp-grand.sf2",
    instrument: "piano",
    license: "CC0-1.0",
    attribution: "",
    sizeBytes: 0n,
    hasObject: true,
    hasPreview: true,
    moderationStatus: "accepted",
    pointCost: 0,
    redeemable: true,
    ...extra,
  };
}

function setup(font: Record<string, unknown>, roles: string[]) {
  const { clients } = makeFakeClients();
  const score = clients.score as unknown as Record<string, (req?: unknown) => Promise<unknown>>;
  const pricingCalls: unknown[] = [];
  score.adminListSoundFonts = async () => ({
    soundfonts: [font],
    total: 1,
    nextOffset: 1,
    totalCount: 1,
    pendingCount: 0,
    acceptedCount: 1,
    rejectedCount: 0,
  });
  score.setSoundFontPricing = async (req) => {
    pricingCalls.push(req);
    return {};
  };
  setClientsForTest(clients);
  useAuthStore().setToken(makeJwt({ roles, sub: "u1" }));
  return { pricingCalls };
}

const mountView = () => mount(SoundFontsView, { global: { plugins: [i18n], stubs: { teleport: true } } });

/** The row's "Set price" button, or undefined when the control is not offered. */
const setPriceButton = (w: ReturnType<typeof mountView>) =>
  w.findAll("button").find((b) => b.text() === t("soundfonts.setPrice"));

describe("SoundFonts pricing control", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("shows a free font's price and lets an admin price it", async () => {
    const { pricingCalls } = setup(row(), ["user", "admin"]);
    const w = mountView();
    await flushPromises();

    expect(w.text()).toContain(t("soundfonts.priceFree"));

    await setPriceButton(w)!.trigger("click");
    const cost = w.get("#soundfont-price-ydp-grand");
    await cost.setValue("250");
    await w
      .findAll("button")
      .find((b) => b.text() === t("soundfonts.save"))!
      .trigger("click");
    await flushPromises();

    expect(pricingCalls).toEqual([{ id: "ydp-grand", pointCost: 250, redeemable: true }]);
  });

  it("clamps a negative cost to free rather than sending it", async () => {
    const { pricingCalls } = setup(row(), ["user", "admin"]);
    const w = mountView();
    await flushPromises();

    await setPriceButton(w)!.trigger("click");
    await w.get("#soundfont-price-ydp-grand").setValue("-5");
    await w
      .findAll("button")
      .find((b) => b.text() === t("soundfonts.save"))!
      .trigger("click");
    await flushPromises();

    expect(pricingCalls).toEqual([{ id: "ydp-grand", pointCost: 0, redeemable: true }]);
  });

  it("renders a costed non-redeemable font as coming later", async () => {
    setup(row({ pointCost: 250, redeemable: false }), ["user", "admin"]);
    const w = mountView();
    await flushPromises();

    expect(w.text()).toContain(t("soundfonts.pricePoints", { n: "250" }));
    expect(w.text()).toContain(t("soundfonts.priceComingLater"));
  });

  it("hints that a costed font with no sample is not auditionable", async () => {
    setup(row({ hasPreview: false }), ["user", "admin"]);
    const w = mountView();
    await flushPromises();

    await setPriceButton(w)!.trigger("click");
    // Still free: no hint yet.
    expect(w.text()).not.toContain(t("soundfonts.priceNoSampleHint"));

    await w.get("#soundfont-price-ydp-grand").setValue("250");
    expect(w.text()).toContain(t("soundfonts.priceNoSampleHint"));
  });

  it("offers no pricing control to a moderator, but still shows the price", async () => {
    setup(row({ pointCost: 250, redeemable: true }), ["user", "moderator"]);
    const w = mountView();
    await flushPromises();

    // A moderator may edit metadata and moderate, but pricing is admin-only.
    expect(setPriceButton(w)).toBeUndefined();
    expect(w.text()).toContain(t("soundfonts.pricePoints", { n: "250" }));
  });
});
