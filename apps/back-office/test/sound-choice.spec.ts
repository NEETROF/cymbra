import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { defineComponent, ref, type Ref } from "vue";
import { createPinia, setActivePinia } from "pinia";
import { useSoundFontChoice } from "@/composables/useSoundFontChoice";
import { DEFAULT_SOUNDFONT_ID, setSoundFontForTest } from "@/lib/audio/soundfont";
import type { SoundFontFamily } from "@/lib/audio/family";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients } from "./fakes";

// useSoundFontChoice drives the preview sound picker on the review/detail screens:
// it lists the public catalog filtered to the SCORE's family (change:
// add-drum-audio-channel) — keyboard rows offer keyboard fonts with the lazy
// default pre-selected, percussion rows offer kits with the first accepted one
// pre-selected and its bytes fetched eagerly (null bytes would fall back to the
// default piano in useScorePlayer, which must never answer for a drum score).

function mountChoice(family: Ref<SoundFontFamily>) {
  const Host = defineComponent({
    setup: () => useSoundFontChoice(family),
    template: "<div />",
  });
  return mount(Host);
}

function withCatalog(fonts: { id: string; label: string; instrument?: string }[]) {
  const { clients } = makeFakeClients();
  const score = clients.score as unknown as Record<string, () => Promise<unknown>>;
  score.listSoundFonts = async () => ({
    soundfonts: fonts.map((f) => ({ instrument: "piano", license: "", attribution: "", ...f })),
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
    const w = mountChoice(ref<SoundFontFamily>("keyboard"));
    await flushPromises();

    expect(w.vm.fonts).toHaveLength(2);
    expect(w.vm.selectedId).toBe(DEFAULT_SOUNDFONT_ID);
    // The default keeps the lazy loader: no bytes fetched.
    expect(w.vm.sf2Bytes).toBeNull();
  });

  it("offers a keyboard score only keyboard-family fonts — legacy `piano` included", async () => {
    withCatalog([
      // The wire still carries the legacy `piano` spelling on old rows: it reads
      // as `keyboard` (the boundary normalisation), never as a third family.
      { id: DEFAULT_SOUNDFONT_ID, label: "Upright Piano KW", instrument: "piano" },
      { id: "harpsi", label: "Harpsichord", instrument: "keyboard" },
      { id: "rock-kit", label: "Rock Kit", instrument: "percussion" },
    ]);
    const w = mountChoice(ref<SoundFontFamily>("keyboard"));
    await flushPromises();

    expect(w.vm.fonts.map((f: { id: string }) => f.id)).toEqual([DEFAULT_SOUNDFONT_ID, "harpsi"]);
    expect(w.vm.familyEmpty).toBe(false);
  });

  it("offers a percussion score kits only, defaulting to the first accepted kit with eager bytes", async () => {
    withCatalog([
      { id: DEFAULT_SOUNDFONT_ID, label: "Upright Piano KW" },
      { id: "rock-kit", label: "Rock Kit", instrument: "percussion" },
      { id: "jazz-kit", label: "Jazz Kit", instrument: "percussion" },
    ]);
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response(new Uint8Array([7, 7]), { status: 200 }))),
    );
    const w = mountChoice(ref<SoundFontFamily>("percussion"));
    await flushPromises();

    expect(w.vm.fonts.map((f: { id: string }) => f.id)).toEqual(["rock-kit", "jazz-kit"]);
    expect(w.vm.selectedId).toBe("rock-kit");
    // A kit is never the lazy default piano: its bytes are fetched up front.
    expect(w.vm.sf2Bytes).toEqual(new Uint8Array([7, 7]));
    expect(w.vm.familyEmpty).toBe(false);
  });

  it("exposes the empty-family state when the catalog holds no kit — distinct from an error", async () => {
    withCatalog([{ id: DEFAULT_SOUNDFONT_ID, label: "Upright Piano KW" }]);
    const w = mountChoice(ref<SoundFontFamily>("percussion"));
    await flushPromises();

    expect(w.vm.fonts).toEqual([]);
    expect(w.vm.familyEmpty).toBe(true);
    expect(w.vm.error).toBeNull();
    expect(w.vm.sf2Bytes).toBeNull();
  });

  it("follows a family flip and restores each family's remembered pick", async () => {
    withCatalog([
      { id: DEFAULT_SOUNDFONT_ID, label: "Upright Piano KW" },
      { id: "ydp-grand", label: "YDP Grand" },
      { id: "rock-kit", label: "Rock Kit", instrument: "percussion" },
    ]);
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response(new Uint8Array([1]), { status: 200 }))),
    );
    const family = ref<SoundFontFamily>("keyboard");
    const w = mountChoice(family);
    await flushPromises();
    w.vm.selectedId = "ydp-grand"; // a non-default keyboard pick to remember
    await flushPromises();

    family.value = "percussion";
    await flushPromises();
    expect(w.vm.selectedId).toBe("rock-kit"); // the kit default, never a piano

    family.value = "keyboard";
    await flushPromises();
    expect(w.vm.selectedId).toBe("ydp-grand"); // the remembered keyboard pick
  });

  it("fetches the bytes when a non-default font is picked", async () => {
    withCatalog([{ id: "ydp-grand", label: "YDP Grand" }]);
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response(new Uint8Array([4, 5, 6]), { status: 200 }))),
    );
    const w = mountChoice(ref<SoundFontFamily>("keyboard"));
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
    const w = mountChoice(ref<SoundFontFamily>("keyboard"));
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
    const w = mountChoice(ref<SoundFontFamily>("keyboard"));
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
    const w = mountChoice(ref<SoundFontFamily>("keyboard"));
    await flushPromises();

    expect(w.vm.fonts).toEqual([]);
    // A failed listing on a keyboard score is not the "no drum kit" state: the
    // default piano still plays through the lazy loader.
    expect(w.vm.familyEmpty).toBe(false);
  });
});
