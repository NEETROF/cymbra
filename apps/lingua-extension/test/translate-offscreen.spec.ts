import { describe, expect, it, vi } from "vitest";
import {
  isOffscreenEvent,
  isOffscreenRequest,
  OFFSCREEN_EVENT,
  OFFSCREEN_TYPE,
  type OffscreenApi,
  OffscreenEngine,
  offscreenIdle,
  type OffscreenParts,
  serveOffscreen,
} from "@/translate/host/offscreen-engine.ts";

function api(over: Partial<OffscreenApi> = {}) {
  const created: Array<{ url: string; reasons: string[]; justification: string }> = [];
  let exists = false;
  let closed = 0;
  const offscreen: OffscreenApi = {
    hasDocument: vi.fn(async () => exists),
    createDocument: vi.fn(async (p) => {
      created.push(p);
      exists = true;
    }),
    closeDocument: vi.fn(async () => {
      closed++;
      exists = false;
    }),
    ...over,
  };
  return { offscreen, created, setExists: (v: boolean) => (exists = v), closed: () => closed };
}

const answering = () => vi.fn(async () => ({ ok: true, html: "<b>a abandonné</b>" }));

describe("OffscreenEngine", () => {
  it("creates the document on first use, for the WORKERS reason, and relays to it", async () => {
    const { offscreen, created } = api();
    const send = answering();
    const engine = new OffscreenEngine(offscreen, send);

    await expect(engine.translate("<b>gave up</b>")).resolves.toEqual({ ok: true, html: "<b>a abandonné</b>" });
    expect(created).toEqual([
      {
        url: "offscreen.html",
        reasons: ["WORKERS"],
        justification: expect.stringContaining("off every thread that paints"),
      },
    ]);
    expect(send).toHaveBeenCalledWith({ type: OFFSCREEN_TYPE, op: "translate", markup: "<b>gave up</b>" });
  });

  it("creates one document however many requests arrive while it is being made", async () => {
    const { offscreen, created } = api();
    const engine = new OffscreenEngine(offscreen, answering());
    await Promise.all([engine.translate("a"), engine.translate("b"), engine.startDownload()]);
    expect(created).toHaveLength(1);
  });

  it("reuses a document that outlived a restarted service worker", async () => {
    // A second createDocument is refused while the first exists; a fresh worker must ask.
    const { offscreen, created, setExists } = api();
    setExists(true);
    const engine = new OffscreenEngine(offscreen, answering());
    await engine.translate("a");
    expect(created).toEqual([]);
  });

  it("answers unavailable when the document cannot be made, and tries again next time", async () => {
    let fail = true;
    const { offscreen } = api({
      createDocument: vi.fn(async () => {
        if (fail) throw new Error("offscreen refused");
      }),
    });
    const engine = new OffscreenEngine(offscreen, answering());

    await expect(engine.translate("a")).resolves.toMatchObject({ ok: false });
    fail = false;
    await expect(engine.translate("a")).resolves.toEqual({ ok: true, html: "<b>a abandonné</b>" });
  });

  it("forgets a document that went away, so the next request makes another", async () => {
    const { offscreen, created, setExists } = api();
    const send = vi.fn().mockResolvedValueOnce({ ok: true, html: "un" });
    const engine = new OffscreenEngine(offscreen, send);
    await engine.translate("a");

    // Chrome closed it: no listener answers any more.
    setExists(false);
    send.mockRejectedValueOnce(new Error("Receiving end does not exist."));
    await expect(engine.translate("b")).resolves.toMatchObject({ ok: false });

    send.mockResolvedValueOnce({ ok: true, html: "trois" });
    await expect(engine.translate("c")).resolves.toEqual({ ok: true, html: "trois" });
    expect(created).toHaveLength(2);
  });

  it("treats a reply that is not an engine reply as no answer", async () => {
    const { offscreen } = api();
    const engine = new OffscreenEngine(
      offscreen,
      vi.fn(async () => undefined),
    );
    await expect(engine.translate("a")).resolves.toMatchObject({ ok: false });
  });

  describe("warm (add-lingua-translation-android D2, D3)", () => {
    it("creates the document when needed and asks it to load the engine, translating nothing", async () => {
      const { offscreen, created } = api();
      const send = vi.fn(async () => true);
      const engine = new OffscreenEngine(offscreen, send);
      await expect(engine.warm()).resolves.toBe(true);
      expect(created).toHaveLength(1);
      expect(send).toHaveBeenCalledWith({ type: OFFSCREEN_TYPE, op: "warm" });
      expect(send).toHaveBeenCalledOnce();
    });

    it("says false when the document cannot be made, or went away, and makes another next time", async () => {
      const { offscreen, created } = api({ createDocument: vi.fn(async () => Promise.reject(new Error("no"))) });
      await expect(
        new OffscreenEngine(
          offscreen,
          vi.fn(async () => true),
        ).warm(),
      ).resolves.toBe(false);
      expect(created).toHaveLength(0);

      const second = api();
      const send = vi.fn<() => Promise<unknown>>(async () => Promise.reject(new Error("Receiving end does not exist")));
      const engine = new OffscreenEngine(second.offscreen, send);
      await expect(engine.warm()).resolves.toBe(false);
      second.setExists(false);
      send.mockResolvedValueOnce(true);
      await expect(engine.warm()).resolves.toBe(true);
      expect(second.created).toHaveLength(2);
    });

    it("treats anything but true as not loaded", async () => {
      const { offscreen } = api();
      await expect(
        new OffscreenEngine(
          offscreen,
          vi.fn(async () => ({ ok: true })),
        ).warm(),
      ).resolves.toBe(false);
    });
  });

  it("starts a download in the document, creating it when needed", async () => {
    const { offscreen, created } = api();
    const send = vi.fn(async () => true);
    await new OffscreenEngine(offscreen, send).startDownload();
    expect(created).toHaveLength(1);
    expect(send).toHaveBeenCalledWith({ type: OFFSCREEN_TYPE, op: "download" });
  });

  it("asks the document whether a download runs — and without one, none does", async () => {
    const { offscreen, setExists } = api();
    const send = vi.fn(async () => true);
    const engine = new OffscreenEngine(offscreen, send);
    await expect(engine.downloading()).resolves.toBe(false);
    expect(send).not.toHaveBeenCalled(); // no document is created just to ask

    setExists(true);
    await expect(engine.downloading()).resolves.toBe(true);
    expect(send).toHaveBeenCalledWith({ type: OFFSCREEN_TYPE, op: "downloading" });

    send.mockRejectedValueOnce(new Error("gone"));
    await expect(engine.downloading()).resolves.toBe(false);
  });

  it("cancels a download only in a document that exists, and never throws doing it", async () => {
    const { offscreen, setExists } = api();
    const send = vi.fn(async () => true);
    const engine = new OffscreenEngine(offscreen, send);
    await engine.cancelDownload();
    expect(send).not.toHaveBeenCalled();

    setExists(true);
    send.mockRejectedValueOnce(new Error("gone"));
    await expect(engine.cancelDownload()).resolves.toBeUndefined();
    expect(send).toHaveBeenCalledWith({ type: OFFSCREEN_TYPE, op: "cancel" });
  });

  it("closes the document to give the engine's memory back, and makes a new one next time", async () => {
    const { offscreen, created, closed } = api();
    const engine = new OffscreenEngine(offscreen, answering());
    await engine.translate("a");
    await engine.close();
    expect(closed()).toBe(1);
    await engine.close(); // nothing left to close
    expect(closed()).toBe(1);
    await engine.translate("b");
    expect(created).toHaveLength(2);
  });
});

describe("serveOffscreen — the document's side", () => {
  function parts(over: Partial<{ engine: boolean; download: boolean }> = {}) {
    const state = { engine: false, download: false, ...over };
    const p = {
      channel: {
        translate: vi.fn(async (markup: string) => ({ ok: true as const, html: `<b>${markup}</b>` })),
        warm: vi.fn(async () => {
          state.engine = true;
          return true;
        }),
        running: () => state.engine,
      },
      downloads: {
        start: vi.fn(() => void (state.download = true)),
        cancel: vi.fn(() => void (state.download = false)),
        running: () => state.download,
      },
      persist: vi.fn(),
    } satisfies OffscreenParts;
    return { p, state };
  }

  it("relays a translation to the engine's channel and answers later", async () => {
    const { p } = parts();
    const sendResponse = vi.fn();
    expect(serveOffscreen({ type: OFFSCREEN_TYPE, op: "translate", markup: "x" }, p, sendResponse)).toBe(true);
    await Promise.resolve();
    await Promise.resolve();
    expect(sendResponse).toHaveBeenCalledWith({ ok: true, html: "<b>x</b>" });
  });

  it("warms the engine's channel and answers once it has loaded", async () => {
    const { p, state } = parts();
    const sendResponse = vi.fn();
    expect(serveOffscreen({ type: OFFSCREEN_TYPE, op: "warm" }, p, sendResponse)).toBe(true);
    await Promise.resolve();
    await Promise.resolve();
    expect(p.channel.warm).toHaveBeenCalledOnce();
    expect(p.channel.translate).not.toHaveBeenCalled();
    expect(sendResponse).toHaveBeenCalledWith(true);
    expect(state.engine).toBe(true);
  });

  it("starts a download, asking the browser to keep what it stores, and answers at once", () => {
    const { p } = parts();
    const sendResponse = vi.fn();
    expect(serveOffscreen({ type: OFFSCREEN_TYPE, op: "download" }, p, sendResponse)).toBe(false);
    expect(p.persist).toHaveBeenCalled();
    expect(p.downloads.start).toHaveBeenCalled();
    expect(sendResponse).toHaveBeenCalledWith(true);
  });

  it("cancels, and says whether a download runs", () => {
    const { p } = parts({ download: true });
    const said = vi.fn();
    serveOffscreen({ type: OFFSCREEN_TYPE, op: "downloading" }, p, said);
    serveOffscreen({ type: OFFSCREEN_TYPE, op: "cancel" }, p, vi.fn());
    serveOffscreen({ type: OFFSCREEN_TYPE, op: "downloading" }, p, said);
    expect(said.mock.calls).toEqual([[true], [false]]);
  });

  it("is idle only when it holds neither the engine nor a download", () => {
    expect(offscreenIdle(parts().p)).toBe(true);
    expect(offscreenIdle(parts({ engine: true }).p)).toBe(false);
    expect(offscreenIdle(parts({ download: true }).p)).toBe(false);
  });
});

describe("offscreen messages", () => {
  it("recognises the document's own requests only", () => {
    expect(isOffscreenRequest({ type: OFFSCREEN_TYPE, op: "translate", markup: "x" })).toBe(true);
    expect(isOffscreenRequest({ type: OFFSCREEN_TYPE, op: "download" })).toBe(true);
    expect(isOffscreenRequest({ type: OFFSCREEN_TYPE, op: "warm" })).toBe(true);
    expect(isOffscreenRequest({ type: OFFSCREEN_TYPE, op: "translate" })).toBe(false);
    expect(isOffscreenRequest({ type: OFFSCREEN_TYPE, op: "explode" })).toBe(false);
    expect(isOffscreenRequest({ type: "lingua-translate", request: {} })).toBe(false);
    expect(isOffscreenRequest(null)).toBe(false);
  });

  it("recognises what the document reports", () => {
    expect(isOffscreenEvent({ type: OFFSCREEN_EVENT, idle: true })).toBe(true);
    expect(isOffscreenEvent({ type: OFFSCREEN_EVENT, event: { kind: "done" } })).toBe(true);
    expect(isOffscreenEvent({ type: OFFSCREEN_EVENT })).toBe(false);
    expect(isOffscreenEvent({ type: OFFSCREEN_TYPE, op: "download" })).toBe(false);
    expect(isOffscreenEvent(undefined)).toBe(false);
  });
});
