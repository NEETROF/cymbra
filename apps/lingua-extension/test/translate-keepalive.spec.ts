import { describe, expect, it, vi } from "vitest";
import { type KeepaliveDeps, type KeepalivePort, keepEngineWarm, RECONNECT_MS } from "@/translate/keepalive.ts";

/** A port whose disconnect the test fires by hand. */
function fakePort(): KeepalivePort & { disconnect(): void } {
  const listeners: (() => void)[] = [];
  return {
    onDisconnect: { addListener: (l) => listeners.push(l) },
    disconnect: () => listeners.forEach((l) => l()),
  };
}

/** `connect` answers from the queue, throwing an entry that is an Error. */
function deps(answers: (KeepalivePort | Error)[]): KeepaliveDeps & { retries: number[]; opened: number } {
  const retries: number[] = [];
  const pending: (() => void)[] = [];
  const self = {
    opened: 0,
    retries,
    connect: () => {
      const next = answers[self.opened++] ?? fakePort();
      if (next instanceof Error) throw next;
      return next;
    },
    schedule: (retry: () => void, ms: number) => {
      retries.push(ms);
      pending.push(retry);
    },
    run: () => pending.splice(0).forEach((r) => r()),
  };
  return self;
}

describe("keepEngineWarm", () => {
  it("opens a port at once, and holds it", () => {
    const d = deps([]);
    keepEngineWarm(d);
    expect(d.opened).toBe(1);
    expect(d.retries).toEqual([]);
  });

  it("reopens after the background goes away anyway", () => {
    const first = fakePort();
    const d = deps([first]);
    keepEngineWarm(d);

    first.disconnect();
    expect(d.retries).toEqual([RECONNECT_MS]);
    expect(d.opened).toBe(1); // scheduled, not yet reopened
    (d as unknown as { run(): void }).run();
    expect(d.opened).toBe(2);
  });

  it("keeps reopening for as long as the page lives", () => {
    const ports = [fakePort(), fakePort(), fakePort()];
    const d = deps(ports);
    keepEngineWarm(d);
    for (const port of ports) {
      port.disconnect();
      (d as unknown as { run(): void }).run();
    }
    expect(d.opened).toBe(4);
  });

  it("gives up when the extension context is gone — an orphaned page has nothing to hold", () => {
    const d = deps([new Error("Extension context invalidated")]);
    expect(() => keepEngineWarm(d)).not.toThrow();
    expect(d.opened).toBe(1);
    expect(d.retries).toEqual([]);
  });

  it("stops retrying once reconnecting throws", () => {
    const first = fakePort();
    const d = deps([first, new Error("Extension context invalidated")]);
    keepEngineWarm(d);
    first.disconnect();
    (d as unknown as { run(): void }).run();
    expect(d.opened).toBe(2);
    expect(d.retries).toEqual([RECONNECT_MS]); // no second retry scheduled
  });

  it("defaults to a real runtime port, named so the background can tell it apart, and retries on a real timer", () => {
    const ports = [fakePort(), fakePort()];
    const connect = vi.fn(() => ports[connect.mock.calls.length - 1] ?? fakePort());
    vi.stubGlobal("chrome", { runtime: { connect } });
    vi.useFakeTimers();
    try {
      keepEngineWarm();
      expect(connect).toHaveBeenCalledWith({ name: "lingua-translate-keepalive" });

      ports[0].disconnect();
      expect(connect).toHaveBeenCalledOnce(); // scheduled, not immediate
      vi.advanceTimersByTime(RECONNECT_MS);
      expect(connect).toHaveBeenCalledTimes(2);
    } finally {
      vi.useRealTimers();
      vi.unstubAllGlobals();
    }
  });
});
