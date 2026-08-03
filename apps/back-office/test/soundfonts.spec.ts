import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import {
  type NewSoundFont,
  setUploadForTest,
  SOUNDFONTS_PAGE_SIZE,
  useSoundFontsStore,
} from "@/stores/soundfonts";
import { SoundFontUploadError } from "@/lib/errors";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients } from "./fakes";

interface SfState {
  list: unknown[];
  adminListCalls: number;
  updateCalls: unknown[];
  deleteCalls: unknown[];
  moderationCalls: unknown[];
}

/** Patch the fake `score` client with the SoundFont admin RPCs, recording calls. */
function withSoundfonts(clients: ReturnType<typeof makeFakeClients>["clients"], initial: unknown[] = []): SfState {
  const state: SfState = {
    list: [...initial],
    adminListCalls: 0,
    updateCalls: [],
    deleteCalls: [],
    moderationCalls: [],
  };
  const score = clients.score as unknown as Record<string, (req: unknown) => Promise<unknown>>;
  score.adminListSoundFonts = async (req: unknown) => {
    state.adminListCalls++;
    const r = (req ?? {}) as { limit?: number; offset?: number; moderationStatus?: string };
    const status = r.moderationStatus ?? "";
    const filtered = status
      ? state.list.filter(
          (f) => (((f as Record<string, unknown>).moderationStatus as string) || "pending") === status,
        )
      : state.list;
    const offset = r.offset ?? 0;
    const limit = r.limit && r.limit > 0 ? r.limit : filtered.length;
    const page = filtered.slice(offset, offset + limit);
    const by = (s: string) =>
      state.list.filter((f) => (((f as Record<string, unknown>).moderationStatus as string) || "pending") === s).length;
    return {
      soundfonts: page,
      total: filtered.length,
      nextOffset: offset + page.length,
      totalCount: state.list.length,
      pendingCount: by("pending"),
      acceptedCount: by("accepted"),
      rejectedCount: by("rejected"),
    };
  };
  score.setSoundFontModerationStatus = async (req: unknown) => {
    state.moderationCalls.push(req);
    return {};
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
  return {
    id,
    label: id,
    objectKey: `${id}.sf2`,
    instrument: "piano",
    license: "CC0-1.0",
    attribution: "",
    sizeBytes: 0n,
    hasObject: true,
    ...extra,
  };
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

  it("paginates the admin listing and tracks the total", async () => {
    const { clients } = makeFakeClients();
    withSoundfonts(
      clients,
      Array.from({ length: 30 }, (_, i) => row(`f${String(i).padStart(2, "0")}`)),
    );
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    await store.list({ status: "", offset: 0 });
    expect(store.total).toBe(30);
    if (store.catalog.status === "success") {
      expect(store.catalog.data).toHaveLength(SOUNDFONTS_PAGE_SIZE);
    }

    // Next page: only the remainder.
    await store.list({ offset: SOUNDFONTS_PAGE_SIZE });
    expect(store.offset).toBe(SOUNDFONTS_PAGE_SIZE);
    if (store.catalog.status === "success") {
      expect(store.catalog.data).toHaveLength(30 - SOUNDFONTS_PAGE_SIZE);
    }
  });

  it("passes the status filter to the server and resets the page", async () => {
    const { clients } = makeFakeClients();
    withSoundfonts(clients, [
      row("a", { moderationStatus: "accepted" }),
      row("b", { moderationStatus: "pending" }),
    ]);
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    await store.list({ status: "pending", offset: 0 });
    expect(store.total).toBe(1);
    if (store.catalog.status === "success") {
      expect(store.catalog.data.map((f) => f.id)).toEqual(["b"]);
    }
    // The KPI counts stay catalog-wide, not scoped to the pending filter.
    expect(store.counts).toEqual({ total: 2, pending: 1, accepted: 1, rejected: 0 });
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

  it("maps a 409 upload conflict to an 'already exists' message", async () => {
    const { clients } = makeFakeClients();
    withSoundfonts(clients);
    setClientsForTest(clients);
    setUploadForTest(async () => {
      throw new SoundFontUploadError(409); // duplicate id or identical content
    });
    const store = useSoundFontsStore();

    const outcome = await store.add({
      id: "dup",
      label: "Dup",
      license: "CC0-1.0",
      attribution: "",
      instrument: "piano",
      file,
    });

    expect(outcome.status).toBe("error");
    // A clear, dedicated message — not the generic fallback.
    expect(store.op.status === "error" && /exist/i.test(store.op.error)).toBe(true);
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

  it("accepts a font via setSoundFontModerationStatus then re-lists", async () => {
    const { clients } = makeFakeClients();
    const sf = withSoundfonts(clients, [row("ydp-grand", { moderationStatus: "pending" })]);
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    const outcome = await store.setModerationStatus("ydp-grand", "accepted");

    expect(outcome.status).toBe("success");
    expect(sf.moderationCalls).toEqual([{ id: "ydp-grand", status: "accepted" }]);
    expect(sf.adminListCalls).toBe(1); // re-listed after the decision
  });

  it("captures a denied moderation decision in op instead of throwing", async () => {
    const { clients } = makeFakeClients();
    withSoundfonts(clients, [row("ydp-grand")]);
    const score = clients.score as unknown as Record<string, () => Promise<never>>;
    score.setSoundFontModerationStatus = () => Promise.reject(new Error("permission denied"));
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    const outcome = await store.setModerationStatus("ydp-grand", "rejected");

    expect(outcome.status).toBe("error");
    expect(store.op.status).toBe("error");
  });

  // --- Preview data (used by the create/edit drawer's audition feature) ---

  it("publicList returns the server's downloadable catalog", async () => {
    const { clients } = makeFakeClients();
    const score = clients.score as unknown as Record<string, () => Promise<unknown>>;
    score.listSoundFonts = async () => ({
      soundfonts: [
        { id: "ydp-grand", label: "YDP Grand", license: "CC-BY 3.0", attribution: "R", instrument: "piano" },
      ],
    });
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    const list = await store.publicList();

    expect(list).toHaveLength(1);
    expect(list[0].id).toBe("ydp-grand");
  });

  it("previewPieces queries accepted catalog scores", async () => {
    const { clients } = makeFakeClients({ hits: [{ id: "p1", title: "Bella Ciao" } as never] });
    withSoundfonts(clients);
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    const pieces = await store.previewPieces("bella");

    expect(pieces).toHaveLength(1);
    expect(pieces[0].id).toBe("p1");
  });

  it("pieceBytes returns the MusicXML bytes for a catalog piece", async () => {
    const { clients } = makeFakeClients();
    withSoundfonts(clients);
    const score = clients.score as unknown as Record<string, () => Promise<unknown>>;
    score.getCatalogScoreBytes = async () => ({ data: new Uint8Array([1, 2, 3]) });
    setClientsForTest(clients);
    const store = useSoundFontsStore();

    const bytes = await store.pieceBytes("p1");

    expect(bytes).toEqual(new Uint8Array([1, 2, 3]));
  });

  it("fontBytes fetches the stored font from the delivery route", async () => {
    const { clients } = makeFakeClients();
    withSoundfonts(clients);
    setClientsForTest(clients);
    const fetchMock = vi.fn(() => Promise.resolve(new Response(new Uint8Array([9, 8, 7]), { status: 200 })));
    vi.stubGlobal("fetch", fetchMock);
    const store = useSoundFontsStore();

    const bytes = await store.fontBytes("ydp-grand");

    expect(bytes).toEqual(new Uint8Array([9, 8, 7]));
    expect(String((fetchMock.mock.calls[0] as unknown[])[0])).toContain("/soundfonts/ydp-grand");
  });

  it("fontBytes throws on a non-200 so the caller can surface it", async () => {
    const { clients } = makeFakeClients();
    withSoundfonts(clients);
    setClientsForTest(clients);
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response("no", { status: 404 }))),
    );
    const store = useSoundFontsStore();

    await expect(store.fontBytes("missing")).rejects.toThrow();
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
});
