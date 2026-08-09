import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { defineComponent } from "vue";
import { createPinia, setActivePinia } from "pinia";
import { useSoundFontChoice } from "@/composables/useSoundFontChoice";
import { DEFAULT_SOUNDFONT_ID, setSoundFontForTest } from "@/lib/audio/soundfont";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients } from "./fakes";

// useSoundFontChoice drives the preview sound picker on the review/detail screens:
// it lists the public catalog (default pre-selected + lazy) and fetches the chosen
// font's bytes on demand.

const Host = defineComponent({
  setup: () => useSoundFontChoice(),
  template: "<div />",
});

function withCatalog(fonts: { id: string; label: string }[]) {
  const { clients } = makeFakeClients();
  const score = clients.score as unknown as Record<string, () => Promise<unknown>>;
  score.listSoundFonts = async () => ({
    soundfonts: fonts.map((f) => ({ ...f, license: "", attribution: "", instrument: "piano" })),
  });
  setClientsForTest(clients);
}

describe("useSoundFontChoice", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    // A picked font is cached for the tab's lifetime (Cache API + in-memory), so each
    // case starts from an empty loader — otherwise one test's bytes answer the next.
    setSoundFontForTest(null);
  });
  afterEach(() => vi.unstubAllGlobals());

  it("loads the catalog on mount with the default pre-selected and no eager bytes", async () => {
    withCatalog([
      { id: DEFAULT_SOUNDFONT_ID, label: "Upright Piano KW" },
      { id: "ydp-grand", label: "YDP Grand" },
    ]);
    const w = mount(Host);
    await flushPromises();

    expect(w.vm.fonts).toHaveLength(2);
    expect(w.vm.selectedId).toBe(DEFAULT_SOUNDFONT_ID);
    // The default keeps the lazy loader: no bytes fetched.
    expect(w.vm.sf2Bytes).toBeNull();
  });

  it("fetches the bytes when a non-default font is picked", async () => {
    withCatalog([{ id: "ydp-grand", label: "YDP Grand" }]);
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response(new Uint8Array([4, 5, 6]), { status: 200 }))),
    );
    const w = mount(Host);
    await flushPromises();

    w.vm.selectedId = "ydp-grand";
    await flushPromises();

    expect(w.vm.sf2Bytes).toEqual(new Uint8Array([4, 5, 6]));
    expect(w.vm.loading).toBe(false);
    expect(w.vm.error).toBeNull();
  });

  it("clears the bytes back to the default (lazy) selection", async () => {
    withCatalog([{ id: "ydp-grand", label: "YDP Grand" }]);
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response(new Uint8Array([1]), { status: 200 }))),
    );
    const w = mount(Host);
    await flushPromises();
    w.vm.selectedId = "ydp-grand";
    await flushPromises();
    expect(w.vm.sf2Bytes).not.toBeNull();

    w.vm.selectedId = DEFAULT_SOUNDFONT_ID;
    await flushPromises();
    expect(w.vm.sf2Bytes).toBeNull();
  });

  it("surfaces a load error without bytes", async () => {
    withCatalog([{ id: "ydp-grand", label: "YDP Grand" }]);
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response("no", { status: 404 }))),
    );
    const w = mount(Host);
    await flushPromises();

    w.vm.selectedId = "ydp-grand";
    await flushPromises();

    expect(w.vm.sf2Bytes).toBeNull();
    expect(w.vm.error).toBe("load_failed");
  });

  it("leaves the picker empty when the catalog listing fails", async () => {
    const { clients } = makeFakeClients();
    const score = clients.score as unknown as Record<string, () => Promise<never>>;
    score.listSoundFonts = () => Promise.reject(new Error("offline"));
    setClientsForTest(clients);
    const w = mount(Host);
    await flushPromises();

    expect(w.vm.fonts).toEqual([]);
  });
});
