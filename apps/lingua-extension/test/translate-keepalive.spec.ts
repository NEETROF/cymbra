import { describe, expect, it, vi } from "vitest";
import { type KeepaliveDeps, keepEngineWarm, keepWarm, PING_MS } from "@/translate/keepalive.ts";
import type { TranslationResult, TranslatorPort } from "@/translate/port.ts";

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
});
