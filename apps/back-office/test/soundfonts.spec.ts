import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { type NewSoundFont, setUploadForTest, useSoundFontsStore } from "@/stores/soundfonts";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients } from "./fakes";

interface SfState {
  list: unknown[];
  adminListCalls: number;
  updateCalls: unknown[];
  deleteCalls: unknown[];
}

/** Patch the fake `score` client with the SoundFont admin RPCs, recording calls. */
function withSoundfonts(clients: ReturnType<typeof makeFakeClients>["clients"], initial: unknown[] = []): SfState {
  const state: SfState = { list: [...initial], adminListCalls: 0, updateCalls: [], deleteCalls: [] };
  const score = clients.score as unknown as Record<string, (req: unknown) => Promise<unknown>>;
  score.adminListSoundFonts = async () => {
    state.adminListCalls++;
    return { soundfonts: state.list };
  };
  score.updateSoundFont = async (req: unknown) => {
    state.updateCalls.push(req);
    return {};
  };
  score.deleteSoundFont = async (req: unknown) => {
    state.deleteCalls.push(req);
    return {};
  };
  return state;
}

function row(id: string, extra: Record<string, unknown> = {}) {
  return { id, label: id, objectKey: `${id}.sf2`, instrument: "piano", license: "CC0-1.0", attribution: "", sizeBytes: 0n, hasObject: true, ...extra };
}

const file = new File([new Uint8Array([1, 2, 3])], "ydp.sf2");

describe("soundfonts store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    setUploadForTest(async () => {}); // default no-op upload; overridden per test
  });

  it("lists the admin catalog into a success state", async () => {
    const { clients } = makeFakeClients();
    const sf = withSoundfonts(clients, [row("upright-piano-kw")]);
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    await store.list();

    expect(sf.adminListCalls).toBe(1);
    expect(store.catalog.status).toBe("success");
    if (store.catalog.status === "success") expect(store.catalog.data).toHaveLength(1);
  });

  it("adds a font via the upload seam then re-lists", async () => {
    const { clients } = makeFakeClients();
    const sf = withSoundfonts(clients);
    setClientsForTest(clients);
    const uploaded: NewSoundFont[] = [];
    setUploadForTest(async (font) => {
      uploaded.push(font);
      sf.list = [row(font.id, { label: font.label })]; // the re-list now returns it
    });
    const store = useSoundFontsStore();

    const outcome = await store.add({
      id: "ydp-grand",
      label: "YDP Grand",
      license: "CC-BY 3.0",
      attribution: "Roberto",
      instrument: "piano",
      file,
    });

    expect(outcome.status).toBe("success");
    expect(uploaded).toHaveLength(1);
    expect(uploaded[0].id).toBe("ydp-grand");
    // Re-listed after a successful add.
    expect(sf.adminListCalls).toBe(1);
    expect(store.catalog.status).toBe("success");
    if (store.catalog.status === "success") expect(store.catalog.data[0].id).toBe("ydp-grand");
  });

  it("edits metadata via updateSoundFont then re-lists", async () => {
    const { clients } = makeFakeClients();
    const sf = withSoundfonts(clients, [row("ydp-grand")]);
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    const outcome = await store.update({
      id: "ydp-grand",
      label: "YDP (edited)",
      license: "CC-BY 3.0",
      attribution: "Roberto",
    });

    expect(outcome.status).toBe("success");
    expect(sf.updateCalls).toEqual([
      { id: "ydp-grand", label: "YDP (edited)", license: "CC-BY 3.0", attribution: "Roberto" },
    ]);
    expect(sf.adminListCalls).toBe(1); // re-listed
  });

  it("removes a font via deleteSoundFont then re-lists", async () => {
    const { clients } = makeFakeClients();
    const sf = withSoundfonts(clients, [row("ydp-grand")]);
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    const outcome = await store.remove("ydp-grand");

    expect(outcome.status).toBe("success");
    expect(sf.deleteCalls).toEqual([{ id: "ydp-grand" }]);
    expect(sf.adminListCalls).toBe(1);
  });

  it("captures an upload failure in op (error state, no throw)", async () => {
    const { clients } = makeFakeClients();
    withSoundfonts(clients);
    setClientsForTest(clients);
    setUploadForTest(async () => {
      throw new Error("soundfont upload failed: HTTP 403");
    });
    const store = useSoundFontsStore();

    const outcome = await store.add({
      id: "x",
      label: "X",
      license: "CC0-1.0",
      attribution: "",
      instrument: "piano",
      file,
    });

    expect(outcome.status).toBe("error");
    expect(store.op.status).toBe("error");
  });

  it("captures a denied edit in op instead of throwing", async () => {
    const { clients } = makeFakeClients();
    withSoundfonts(clients, [row("ydp-grand")]);
    const score = clients.score as unknown as Record<string, () => Promise<never>>;
    score.updateSoundFont = () => Promise.reject(new Error("permission denied"));
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    const outcome = await store.update({ id: "ydp-grand", label: "x", license: "x", attribution: "" });

    expect(outcome.status).toBe("error");
  });
});
