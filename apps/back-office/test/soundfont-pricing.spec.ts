import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { flushPromises, mount } from "@vue/test-utils";
import SoundFontDrawer from "@/components/SoundFontDrawer.vue";
import { useAuthStore } from "@/stores/auth";
import { setClientsForTest } from "@/lib/api";
import { i18n } from "@/i18n";
import { makeFakeClients, makeJwt } from "./fakes";

// Reward pricing (change: add-soundfont-reward-pricing). The price is DISPLAYED on the
// Sound fonts listing but EDITED here, in the font's edit drawer, and only by an admin.
// Driven over the fake client seam; the audio preview is mocked out (no wasm in jsdom).
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

function entry(extra: Record<string, unknown> = {}) {
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

interface Calls {
  pricing: unknown[];
  update: unknown[];
}

function setup(roles: string[]): Calls {
  const { clients } = makeFakeClients();
  const score = clients.score as unknown as Record<string, (req?: unknown) => Promise<unknown>>;
  const calls: Calls = { pricing: [], update: [] };
  score.adminListSoundFonts = async () => ({ soundfonts: [] });
  score.updateSoundFont = async (req) => {
    calls.update.push(req);
    return {};
  };
  score.setSoundFontPricing = async (req) => {
    calls.pricing.push(req);
    return {};
  };
  setClientsForTest(clients);
  useAuthStore().setToken(makeJwt({ roles, sub: "u1" }));
  return calls;
}

const mountDrawer = (props: { mode: "create" | "edit" | null; entry?: unknown }) =>
  mount(SoundFontDrawer, { props: props as never, global: { plugins: [i18n], stubs: { teleport: true } } });

type Drawer = ReturnType<typeof mountDrawer>;

/** The pricing cost field, or undefined when the section is not offered. */
const costField = (w: Drawer) => w.find(`input[aria-label="${t("soundfonts.priceCostLabel")}"]`);
const submit = (w: Drawer) => w.get("form").trigger("submit");

describe("SoundFont pricing in the edit drawer", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("prices a free font and sends only the pricing call it changed", async () => {
    const calls = setup(["user", "admin"]);
    const w = mountDrawer({ mode: "edit", entry: entry() });
    await flushPromises();

    await costField(w).setValue("250");
    await submit(w);
    await flushPromises();

    expect(calls.pricing).toEqual([{ id: "ydp-grand", pointCost: 250, redeemable: true }]);
    // The metadata save still happens (one drawer, one Save).
    expect(calls.update).toHaveLength(1);
    expect(w.emitted("close")).toBeTruthy();
  });

  it("does not call pricing when the price was not touched", async () => {
    const calls = setup(["user", "admin"]);
    const w = mountDrawer({ mode: "edit", entry: entry({ pointCost: 250 }) });
    await flushPromises();

    // Edit the label only.
    await w.get('input[aria-label="label"]').setValue("Renamed");
    await submit(w);
    await flushPromises();

    expect(calls.update).toHaveLength(1);
    expect(calls.pricing).toEqual([]); // an unchanged price costs no admin-only call
  });

  it("reverts a costed font to free, and marks one coming later", async () => {
    const calls = setup(["user", "admin"]);
    const w = mountDrawer({ mode: "edit", entry: entry({ pointCost: 250 }) });
    await flushPromises();

    await costField(w).setValue("0");
    await submit(w);
    await flushPromises();
    expect(calls.pricing).toEqual([{ id: "ydp-grand", pointCost: 0, redeemable: true }]);

    // Unchecking "redeemable" alone is a pricing change too ("coming later").
    const w2 = mountDrawer({ mode: "edit", entry: entry({ pointCost: 250 }) });
    await flushPromises();
    await w2.get(`input[type="checkbox"]`).setValue(false);
    await submit(w2);
    await flushPromises();
    expect(calls.pricing[1]).toEqual({ id: "ydp-grand", pointCost: 250, redeemable: false });
  });

  it("clamps a negative cost to free rather than sending it", async () => {
    const calls = setup(["user", "admin"]);
    const w = mountDrawer({ mode: "edit", entry: entry({ pointCost: 250 }) });
    await flushPromises();

    await costField(w).setValue("-5");
    await submit(w);
    await flushPromises();

    expect(calls.pricing).toEqual([{ id: "ydp-grand", pointCost: 0, redeemable: true }]);
  });

  it("hints that a costed font with no sample is not auditionable", async () => {
    setup(["user", "admin"]);
    const w = mountDrawer({ mode: "edit", entry: entry({ hasPreview: false }) });
    await flushPromises();

    expect(w.text()).not.toContain(t("soundfonts.priceNoSampleHint")); // still free
    await costField(w).setValue("250");
    expect(w.text()).toContain(t("soundfonts.priceNoSampleHint"));
  });

  it("offers no pricing section to a moderator", async () => {
    const calls = setup(["user", "moderator"]);
    const w = mountDrawer({ mode: "edit", entry: entry({ pointCost: 250 }) });
    await flushPromises();

    // A moderator may edit metadata but not price — and saving never attempts it.
    expect(costField(w).exists()).toBe(false);
    await submit(w);
    await flushPromises();
    expect(calls.update).toHaveLength(1);
    expect(calls.pricing).toEqual([]);
  });

  it("offers no pricing section when creating a font (no row to price yet)", async () => {
    setup(["user", "admin"]);
    const w = mountDrawer({ mode: "create" });
    await flushPromises();

    expect(costField(w).exists()).toBe(false);
  });
});
