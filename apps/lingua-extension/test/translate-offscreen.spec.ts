import { describe, expect, it, vi } from "vitest";
import {
  isOffscreenMessage,
  OFFSCREEN_TYPE,
  type OffscreenApi,
  OffscreenEngine,
} from "@/translate/host/offscreen-engine.ts";

function api(over: Partial<OffscreenApi> = {}) {
  const created: Array<{ url: string; reasons: string[]; justification: string }> = [];
  let exists = false;
  const offscreen: OffscreenApi = {
    hasDocument: vi.fn(async () => exists),
    createDocument: vi.fn(async (p) => {
      created.push(p);
      exists = true;
    }),
    ...over,
  };
  return { offscreen, created, setExists: (v: boolean) => (exists = v) };
}

const answering = () => vi.fn(async () => ({ ok: true, html: "<b>a abandonné</b>" }));

describe("OffscreenEngine", () => {
  it("creates the document on first use, for the WORKERS reason, and relays to it", async () => {
    const { offscreen, created } = api();
    const send = answering();
    const engine = new OffscreenEngine(offscreen, send);

    await expect(engine.translate("<b>gave up</b>")).resolves.toEqual({ ok: true, html: "<b>a abandonné</b>" });
    expect(created).toEqual([
      { url: "offscreen.html", reasons: ["WORKERS"], justification: expect.stringContaining("off every thread that paints") },
    ]);
    expect(send).toHaveBeenCalledWith({ type: OFFSCREEN_TYPE, markup: "<b>gave up</b>" });
  });

  it("creates one document however many requests arrive while it is being made", async () => {
    const { offscreen, created } = api();
    const engine = new OffscreenEngine(offscreen, answering());
    await Promise.all([engine.translate("a"), engine.translate("b"), engine.translate("c")]);
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
    const engine = new OffscreenEngine(offscreen, vi.fn(async () => undefined));
    await expect(engine.translate("a")).resolves.toMatchObject({ ok: false });
  });
});

describe("isOffscreenMessage", () => {
  it("recognises the offscreen document's own messages only", () => {
    expect(isOffscreenMessage({ type: OFFSCREEN_TYPE, markup: "x" })).toBe(true);
    expect(isOffscreenMessage({ type: OFFSCREEN_TYPE })).toBe(false);
    expect(isOffscreenMessage({ type: "lingua-translate", request: {} })).toBe(false);
    expect(isOffscreenMessage(null)).toBe(false);
  });
});
