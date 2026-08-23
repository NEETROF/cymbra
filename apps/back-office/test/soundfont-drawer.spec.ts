import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import SoundFontDrawer from "@/components/SoundFontDrawer.vue";
import { setUploadForTest } from "@/stores/soundfonts";
import { SoundFontUploadError } from "@/lib/errors";
import { setClientsForTest } from "@/lib/api";
import { i18n } from "@/i18n";
import { makeFakeClients } from "./fakes";

// The create/edit drawer for the SoundFont catalog. Driven over the fake client +
// upload seams; the audio-preview composable is mocked out (no wasm/audio in jsdom).
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

type ScoreFake = Record<string, (req?: unknown) => Promise<unknown>>;

interface Fakes {
  updateCalls: unknown[];
}

function setup({ hits = [{ id: "p1", title: "Bella Ciao" }] }: { hits?: unknown[] } = {}): Fakes {
  const { clients } = makeFakeClients({ hits: hits as never });
  const score = clients.score as unknown as ScoreFake;
  const fakes: Fakes = { updateCalls: [] };
  score.adminListSoundFonts = async () => ({ soundfonts: [] });
  score.updateSoundFont = async (req) => {
    fakes.updateCalls.push(req);
    return {};
  };
  score.getCatalogScoreBytes = async () => ({ data: new Uint8Array([1, 2, 3]) });
  setClientsForTest(clients);
  return fakes;
}

function mountDrawer(props: { mode: "create" | "edit" | null; entry?: unknown }) {
  return mount(SoundFontDrawer, {
    props: props as never,
    global: { plugins: [i18n], stubs: { teleport: true } },
  });
}

// jsdom's File lacks arrayBuffer(); a minimal stub is enough (the upload seam is
// faked, so only the drawer's preview watcher reads the bytes).
const sf2Bytes = new Uint8Array([0x52, 0x49, 0x46, 0x46, 0, 0, 0, 0, 0x73, 0x66, 0x62, 0x6b]);
const validSf2 = { name: "x.sf2", arrayBuffer: async () => sf2Bytes.buffer } as unknown as File;

describe("SoundFontDrawer", () => {
  beforeEach(() => setActivePinia(createPinia()));
  afterEach(() => vi.unstubAllGlobals());

  it("creates a font: fills the form, picks a file, and uploads on save", async () => {
    setup();
    const uploaded: { id: string; label: string; instrument: string }[] = [];
    setUploadForTest(async (font) => {
      uploaded.push(font);
    });
    const w = mountDrawer({ mode: "create" });
    await flushPromises();

    expect(w.find(".drawer").exists()).toBe(true);
    // The id is auto-minted (uuidv7) on create — there is no id input to fill.
    expect(w.find('input[aria-label="id"]').exists()).toBe(false);
    await w.find('input[aria-label="label"]').setValue("YDP Grand");
    await w.find('select[aria-label="license"]').setValue("CC-BY 3.0");

    // Pick the .sf2 file (jsdom's file input is read-only, so define + fire change).
    const fileInput = w.find('input[type="file"]');
    Object.defineProperty(fileInput.element, "files", { value: [validSf2], configurable: true });
    await fileInput.trigger("change");
    await flushPromises();

    await w.find("form").trigger("submit");
    await flushPromises();

    expect(uploaded).toHaveLength(1);
    expect(uploaded[0]).toMatchObject({ label: "YDP Grand", instrument: "keyboard" });
    // A generated uuidv7 (version nibble 7, RFC-4122 variant).
    expect(uploaded[0].id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
    expect(w.emitted("close")).toBeTruthy();
  });

  it("offers both instrument families with keyboard preselected and localized labels", async () => {
    setup();
    const w = mountDrawer({ mode: "create" });
    await flushPromises();

    const select = w.find('select[aria-label="instrument"]');
    expect(select.findAll("option").map((o) => o.attributes("value"))).toEqual(["keyboard", "percussion"]);
    expect((select.element as HTMLSelectElement).value).toBe("keyboard");
    // Localized labels, never the raw vocabulary values.
    expect(select.text()).toContain("Keyboard");
    expect(select.text()).toContain("Drum kit");
  });

  it("surfaces the family-mismatch refusal as a localized reason, never the raw string", async () => {
    setup();
    // The backend verifies the declared family against the file's preset banks
    // (change: add-drum-audio-channel) and refuses a mismatch with the typed
    // `soundfont_family_mismatch` reason — the upload route answers 422 with a
    // JSON refusal { code, message }.
    setUploadForTest(async () => {
      throw new SoundFontUploadError(
        422,
        JSON.stringify({
          code: "soundfont_family_mismatch",
          message: "soundfont_family_mismatch: declared 'percussion' but the font has no bank-128 (drum kit) presets",
        }),
      );
    });
    const w = mountDrawer({ mode: "create" });
    await flushPromises();

    await w.find('input[aria-label="label"]').setValue("Not A Kit");
    await w.find('select[aria-label="instrument"]').setValue("percussion");
    const fileInput = w.find('input[type="file"]');
    Object.defineProperty(fileInput.element, "files", { value: [validSf2], configurable: true });
    await fileInput.trigger("change");
    await flushPromises();
    await w.find("form").trigger("submit");
    await flushPromises();

    // The drawer stays open with a localized alert naming the declared family…
    const alert = w.get('[role="alert"]');
    expect(alert.text()).toContain("Drum kit");
    expect(alert.text()).toContain("declared family");
    // …and the raw server string never reaches the UI.
    expect(alert.text()).not.toContain("soundfont_family_mismatch");
    expect(w.emitted("close")).toBeFalsy();
  });

  it("shows a per-licence explanation for the selected licence", async () => {
    setup();
    const w = mountDrawer({ mode: "create" });
    await flushPromises();

    await w.find('select[aria-label="license"]').setValue("CC-BY-SA 4.0");
    expect(w.text()).toContain("ShareAlike");
  });

  it("edits a font: seeds from the entry, loads its bytes, and saves metadata", async () => {
    const fakes = setup();
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response(new Uint8Array([9]), { status: 200 }))),
    );
    const entry = {
      id: "ydp-grand",
      label: "YDP Grand",
      license: "CC-BY 3.0",
      attribution: "Roberto",
      instrument: "piano",
    };
    const w = mountDrawer({ mode: "edit", entry });
    await flushPromises();

    // Seeded from the entry (id is disabled in edit mode).
    expect((w.find('input[aria-label="label"]').element as HTMLInputElement).value).toBe("YDP Grand");
    expect(w.find('input[aria-label="id"]').attributes("disabled")).toBeDefined();

    await w.find('input[aria-label="label"]').setValue("YDP Grand (edited)");
    await w.find("form").trigger("submit");
    await flushPromises();

    expect(fakes.updateCalls).toHaveLength(1);
    expect(fakes.updateCalls[0]).toMatchObject({ id: "ydp-grand", label: "YDP Grand (edited)" });
    expect(w.emitted("close")).toBeTruthy();
  });

  it("surfaces a preview error when the stored font can't be loaded (edit)", async () => {
    setup();
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response("no", { status: 404 }))),
    );
    const w = mountDrawer({
      mode: "edit",
      entry: { id: "x", label: "X", license: "CC0-1.0", attribution: "", instrument: "piano" },
    });
    await flushPromises();

    expect(w.find(".hint.err").exists()).toBe(true);
  });

  it("renders nothing when mode is null", () => {
    setup();
    const w = mountDrawer({ mode: null });
    expect(w.find(".drawer").exists()).toBe(false);
  });
});
