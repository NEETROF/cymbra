import { describe, expect, it, vi } from "vitest";
import { type KeepaliveDeps, keepEngineWarm, keepWarm, PING_MS, type RestoreDeps } from "@/translate/keepalive.ts";
import { ENGINE_IDLE_MS, type TranslationResult, type TranslatorPort } from "@/translate/port.ts";

/** A controllable timer and ping. */
function deps(answers: (Promise<unknown> | Error)[] = []): KeepaliveDeps & {
  tick(): Promise<void>;
  pings: number;
  stopped: boolean;
  interval: number | null;
} {
  let fire: (() => void) | null = null;
  const self = {
    pings: 0,
    stopped: false,
    interval: null as number | null,
    ping: () => {
      const next = answers[self.pings++];
      if (next instanceof Error) return Promise.reject(next);
      return next ?? Promise.resolve(true);
    },
    every: (ms: number, cb: () => void) => {
      self.interval = ms;
      fire = cb;
      return () => {
        self.stopped = true;
      };
    },
    tick: async () => {
      fire?.();
      await Promise.resolve();
      await Promise.resolve();
    },
  };
  return self;
}

describe("keepEngineWarm", () => {
  it("pings on a timer well inside the shortest idle gap measured on the device", () => {
    const d = deps();
    keepEngineWarm(d);
    expect(d.interval).toBe(PING_MS);
    expect(PING_MS).toBeLessThan(8_000);
    expect(d.pings).toBe(0); // nothing before the first tick
  });

  it("keeps pinging for as long as the page lives", async () => {
    const d = deps();
    keepEngineWarm(d);
    await d.tick();
    await d.tick();
    await d.tick();
    expect(d.pings).toBe(3);
    expect(d.stopped).toBe(false);
  });

  it("stops when the extension context is gone — an orphaned page has nothing to hold", async () => {
    const d = deps([new Error("Extension context invalidated")]);
    keepEngineWarm(d);
    await d.tick();
    expect(d.stopped).toBe(true);
  });

  it("survives a ping that throws synchronously", async () => {
    const d = deps();
    d.ping = () => {
      throw new Error("no receiving end");
    };
    keepEngineWarm(d);
    await expect(d.tick()).resolves.toBeUndefined();
    expect(d.stopped).toBe(true);
  });

  it("stops when told to — the setting pings only while a download runs", async () => {
    const d = deps();
    const stop = keepEngineWarm(d);
    await d.tick();
    stop();
    expect(d.stopped).toBe(true);
  });

  it("skips a tick while `when` says no, without stopping", async () => {
    // The setting's view may be hidden (the drawer closed) and shown again during one download.
    const d = deps();
    let onScreen = false;
    keepEngineWarm(d, () => onScreen);
    await d.tick();
    expect(d.pings).toBe(0);
    onScreen = true;
    await d.tick();
    expect(d.pings).toBe(1);
    expect(d.stopped).toBe(false);
  });

  it("defaults to a real interval and a real runtime message", () => {
    const sendMessage = vi.fn(() => Promise.resolve(true));
    vi.stubGlobal("chrome", { runtime: { sendMessage } });
    vi.useFakeTimers();
    try {
      keepEngineWarm();
      vi.advanceTimersByTime(PING_MS * 2);
      expect(sendMessage).toHaveBeenCalledTimes(2);
      expect(sendMessage).toHaveBeenCalledWith({ type: "lingua-translate-keepalive" });
    } finally {
      vi.useRealTimers();
      vi.unstubAllGlobals();
    }
  });
});

describe("keepWarm", () => {
  const request = { sentence: "They seldom ship.", selection: { start: 5, end: 11 } };
  const answer: TranslationResult = { kind: "unavailable" };

  function inner(): TranslatorPort & { calls: number } {
    const port = {
      calls: 0,
      translate: () => {
        port.calls++;
        return Promise.resolve(answer);
      },
    };
    return port;
  }

  it("holds nothing until the first translation — an unused engine has nothing to keep", () => {
    const start = vi.fn();
    keepWarm(inner(), start);
    expect(start).not.toHaveBeenCalled();
  });

  it("starts pinging on the first translation, and only once", async () => {
    const start = vi.fn();
    const port = inner();
    const warmed = keepWarm(port, start);
    await warmed.translate(request);
    await warmed.translate(request);
    expect(start).toHaveBeenCalledOnce();
    expect(port.calls).toBe(2);
  });

  it("passes the request through and answers what the port answered", async () => {
    await expect(keepWarm(inner(), vi.fn()).translate(request)).resolves.toEqual(answer);
  });

  it("passes a warm through to the port (android D2)", () => {
    const port = { ...inner(), warm: vi.fn() };
    keepWarm(port, vi.fn()).warm?.();
    expect(port.warm).toHaveBeenCalledOnce();
  });

  describe("back to the page (add-lingua-translation-android D3)", () => {
    /** A clock the test moves, and a page it makes visible again. */
    function page() {
      let now = 1_000_000;
      const visible: Array<() => void> = [];
      const restore: RestoreDeps = { now: () => now, onVisible: (fn) => void visible.push(fn) };
      return {
        restore,
        later: (ms: number) => void (now += ms),
        show: () => visible.forEach((fn) => fn()),
        listeners: () => visible.length,
      };
    }

    it("asks for the engine again when a page that translated recently comes back", async () => {
      const p = page();
      const port = { ...inner(), warm: vi.fn() };
      const translator = keepWarm(port, vi.fn(), p.restore);
      await translator.translate(request);
      p.later(60_000); // a minute in another app
      p.show();
      expect(port.warm).toHaveBeenCalledOnce();
    });

    it("counts from the LAST translation, and not past the idle period", async () => {
      const p = page();
      const port = { ...inner(), warm: vi.fn() };
      const translator = keepWarm(port, vi.fn(), p.restore);
      await translator.translate(request);
      p.later(ENGINE_IDLE_MS - 1_000);
      await translator.translate(request); // the window starts again here
      p.later(ENGINE_IDLE_MS - 1_000);
      p.show();
      expect(port.warm).toHaveBeenCalledOnce();
      p.later(1_000); // the idle period has now passed since the last translation
      p.show();
      expect(port.warm).toHaveBeenCalledOnce();
    });

    it("listens for nothing on a page that never translated, and listens once", async () => {
      const p = page();
      const port = { ...inner(), warm: vi.fn() };
      const translator = keepWarm(port, vi.fn(), p.restore);
      expect(p.listeners()).toBe(0);
      await translator.translate(request);
      await translator.translate(request);
      expect(p.listeners()).toBe(1);
    });

    it("a port that cannot warm is left alone", async () => {
      const p = page();
      const translator = keepWarm(inner(), vi.fn(), p.restore);
      await translator.translate(request);
      expect(() => p.show()).not.toThrow();
    });

    it("follows the document's visibility by default", async () => {
      const port = { ...inner(), warm: vi.fn() };
      const translator = keepWarm(port, vi.fn());
      await translator.translate(request);
      const state = vi.spyOn(document, "visibilityState", "get");
      state.mockReturnValue("hidden");
      document.dispatchEvent(new Event("visibilitychange"));
      expect(port.warm).not.toHaveBeenCalled();
      state.mockReturnValue("visible");
      document.dispatchEvent(new Event("visibilitychange"));
      expect(port.warm).toHaveBeenCalledOnce();
      state.mockRestore();
    });
  });
});
