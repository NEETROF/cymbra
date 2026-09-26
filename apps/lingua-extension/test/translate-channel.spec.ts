import { describe, expect, it, vi } from "vitest";
import {
  type ChannelClock,
  EngineChannel,
  START_TIMEOUT_MS,
  TRANSLATE_TIMEOUT_MS,
  type WorkerLike,
} from "@/translate/host/channel.ts";
import type { WorkerRequest, WorkerResponse } from "@/translate/host/engine.ts";
import { ENGINE_IDLE_MS } from "@/translate/port.ts";

/** A worker that records what it is sent and answers only when the test says so. */
class FakeWorker implements WorkerLike {
  readonly sent: WorkerRequest[] = [];
  terminated = false;
  onmessage: ((event: { data: WorkerResponse }) => void) | null = null;
  onerror: ((event: unknown) => void) | null = null;

  postMessage(message: WorkerRequest): void {
    this.sent.push(message);
  }
  terminate(): void {
    this.terminated = true;
  }
  /** Answer the n-th request it was sent. */
  reply(n: number, body: { ok: true; html?: string } | { ok: false; error: string }): void {
    this.onmessage?.({ data: { id: this.sent[n]!.id, ...body } as WorkerResponse });
  }
  crash(): void {
    this.onerror?.(new Event("error"));
  }
}

/** Timers the test fires by hand, so a bound elapses exactly when it says. */
function manualClock() {
  const timers = new Map<number, { fn: () => void; ms: number }>();
  let next = 0;
  const clock: ChannelClock = {
    setTimeout: (fn, ms) => {
      timers.set(++next, { fn, ms });
      return next;
    },
    clearTimeout: (h) => void timers.delete(h as number),
  };
  /** Fire every pending timer armed for `ms`. */
  const elapse = (ms: number) => {
    for (const [h, t] of [...timers]) {
      if (t.ms === ms) {
        timers.delete(h);
        t.fn();
      }
    }
  };
  return { clock, elapse, pending: () => timers.size };
}

/** Let promise continuations run. */
const flush = async () => {
  for (let i = 0; i < 10; i++) await Promise.resolve();
};

function setup(onIdle?: () => void) {
  const workers: FakeWorker[] = [];
  const { clock, elapse, pending } = manualClock();
  const channel = new EngineChannel(
    () => {
      const w = new FakeWorker();
      workers.push(w);
      return w;
    },
    { clock, onIdle },
  );
  return { channel, workers, elapse, pending };
}

describe("EngineChannel", () => {
  it("starts the worker and loads the model before the first translation", async () => {
    const { channel, workers } = setup();
    const answer = channel.translate("<b>gave up</b>");
    await flush();
    expect(workers).toHaveLength(1);
    expect(workers[0]!.sent.map((r) => r.op)).toEqual(["load"]);

    workers[0]!.reply(0, { ok: true });
    await flush();
    expect(workers[0]!.sent[1]).toMatchObject({ op: "translate", markup: "<b>gave up</b>" });

    workers[0]!.reply(1, { ok: true, html: "<b>a abandonné</b>" });
    await expect(answer).resolves.toEqual({ ok: true, html: "<b>a abandonné</b>" });
  });

  it("loads once for every request made while it starts", async () => {
    const { channel, workers } = setup();
    const a = channel.translate("one");
    const b = channel.translate("two");
    await flush();
    expect(workers).toHaveLength(1);
    workers[0]!.reply(0, { ok: true });
    await flush();
    expect(workers[0]!.sent.filter((r) => r.op === "load")).toHaveLength(1);
    workers[0]!.reply(2, { ok: true, html: "deux" }); // answered out of order
    workers[0]!.reply(1, { ok: true, html: "un" });
    await expect(Promise.all([a, b])).resolves.toEqual([
      { ok: true, html: "un" },
      { ok: true, html: "deux" },
    ]);
  });

  it("gives up on an engine that never finishes starting, instead of waiting on it", async () => {
    // The glue's failure mode: its init throws and the promise it would resolve stays pending
    // forever. Without this bound a broken engine reads as a slow one, indefinitely.
    const { channel, workers, elapse } = setup();
    const answer = channel.translate("x");
    await flush();
    elapse(START_TIMEOUT_MS);
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine did not start" });
    expect(workers[0]!.terminated).toBe(true);
  });

  it("starts afresh after a failed start, rather than staying broken for the session", async () => {
    const { channel, workers, elapse } = setup();
    const first = channel.translate("x");
    await flush();
    elapse(START_TIMEOUT_MS);
    await first;

    void channel.translate("y");
    await flush();
    expect(workers).toHaveLength(2);
    expect(workers[1]!.sent[0]!.op).toBe("load");
  });

  it("reports a model the worker could not load", async () => {
    // The ordinary development case: the build carries the engine but no model was supplied.
    const { channel, workers } = setup();
    const answer = channel.translate("x");
    await flush();
    workers[0]!.reply(0, { ok: false, error: "engine/model.bin: 404" });
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine did not start" });
  });

  it("reports a worker that could not even be constructed", async () => {
    const channel = new EngineChannel(() => {
      throw new Error("Worker is not defined");
    });
    await expect(channel.translate("x")).resolves.toEqual({ ok: false, reason: "the engine did not start" });
  });

  it("puts down a worker stuck past its bound, and the next request gets a fresh one", async () => {
    // A synchronous wasm call cannot be interrupted: everything sent after it would queue.
    const { channel, workers, elapse } = setup();
    const answer = channel.translate("x");
    await flush();
    workers[0]!.reply(0, { ok: true });
    await flush();
    elapse(TRANSLATE_TIMEOUT_MS);
    await expect(answer).resolves.toEqual({ ok: false, reason: "the translation timed out" });
    expect(workers[0]!.terminated).toBe(true);

    void channel.translate("y");
    await flush();
    expect(workers).toHaveLength(2);
  });

  it("fails everything in flight when the worker dies", async () => {
    const { channel, workers, pending } = setup();
    const answer = channel.translate("x");
    await flush();
    workers[0]!.reply(0, { ok: true });
    await flush();
    workers[0]!.crash();
    // A crash is reported as a crash: the log is what someone debugging it will read.
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine's worker stopped" });
    expect(workers[0]!.terminated).toBe(true);
    expect(pending()).toBe(0); // no timer left armed on a dead worker
  });

  it("passes on what the engine said when it could not translate", async () => {
    const { channel, workers } = setup();
    const answer = channel.translate("x");
    await flush();
    workers[0]!.reply(0, { ok: true });
    await flush();
    workers[0]!.reply(1, { ok: false, error: "the engine is not loaded" });
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine is not loaded" });
  });

  it("treats an answer with no markup as no answer", async () => {
    const { channel, workers } = setup();
    const answer = channel.translate("x");
    await flush();
    workers[0]!.reply(0, { ok: true });
    await flush();
    workers[0]!.reply(1, { ok: true });
    await expect(answer).resolves.toEqual({ ok: false, reason: "empty answer" });
  });

  it("reports a worker that dies while loading as an engine that did not start", async () => {
    const { channel, workers } = setup();
    const answer = channel.translate("x");
    await flush();
    workers[0]!.crash();
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine did not start" });
  });

  it("ignores a reply that arrives after its request timed out", async () => {
    const { channel, workers, elapse } = setup();
    const answer = channel.translate("x");
    await flush();
    const late = workers[0]!;
    elapse(START_TIMEOUT_MS);
    await answer;
    expect(() => late.reply(0, { ok: true })).not.toThrow();
  });
});

describe("EngineChannel — the engine is released when reading stops (D6)", () => {
  /** One translation, answered: the engine is loaded and idle from here on. */
  async function translated(h: ReturnType<typeof setup>) {
    const answer = h.channel.translate("x");
    await flush();
    const w = h.workers.at(-1)!;
    if (w.sent.length === 1) w.reply(0, { ok: true }); // the load
    await flush();
    w.reply(w.sent.length - 1, { ok: true, html: "y" });
    await answer;
  }

  it("puts the worker down ten minutes after the last translation asked, and says so", async () => {
    const onIdle = vi.fn();
    const h = setup(onIdle);
    await translated(h);
    expect(h.channel.running()).toBe(true);
    expect(ENGINE_IDLE_MS).toBe(10 * 60_000);

    h.elapse(ENGINE_IDLE_MS);
    expect(h.workers[0]!.terminated).toBe(true);
    expect(h.channel.running()).toBe(false);
    expect(onIdle).toHaveBeenCalledOnce();
  });

  describe("warm (add-lingua-translation-android D2, D3)", () => {
    it("loads the engine and translates nothing", async () => {
      const h = setup();
      const warmed = h.channel.warm();
      await flush();
      expect(h.workers).toHaveLength(1);
      expect(h.workers[0]!.sent.map((r) => r.op)).toEqual(["load"]);
      h.workers[0]!.reply(0, { ok: true });
      await expect(warmed).resolves.toBe(true);
      expect(h.channel.running()).toBe(true);
    });

    it("the translation that follows uses the warmed engine: no second load", async () => {
      const h = setup();
      const warmed = h.channel.warm();
      await flush();
      const answer = h.channel.translate("<b>gave up</b>"); // asked while the model loads
      await flush();
      h.workers[0]!.reply(0, { ok: true });
      await warmed;
      await flush();
      expect(h.workers).toHaveLength(1);
      expect(h.workers[0]!.sent.map((r) => r.op)).toEqual(["load", "translate"]);
      h.workers[0]!.reply(1, { ok: true, html: "a abandonné" });
      await expect(answer).resolves.toEqual({ ok: true, html: "a abandonné" });
    });

    it("counts as asking: an engine warmed for nothing is released after the idle period", async () => {
      const onIdle = vi.fn();
      const h = setup(onIdle);
      const warmed = h.channel.warm();
      await flush();
      h.workers[0]!.reply(0, { ok: true });
      await warmed;
      expect(h.pending()).toBe(1);
      h.elapse(ENGINE_IDLE_MS);
      expect(h.workers[0]!.terminated).toBe(true);
      expect(onIdle).toHaveBeenCalledOnce();
    });

    it("on a warm engine it loads nothing again and restarts the countdown", async () => {
      const h = setup();
      await translated(h);
      await expect(h.channel.warm()).resolves.toBe(true);
      expect(h.workers).toHaveLength(1);
      expect(h.workers[0]!.sent.map((r) => r.op)).toEqual(["load", "translate"]);
      expect(h.pending()).toBe(1); // one countdown, not two
    });

    it("a start that fails leaves nothing behind, and the next warm tries afresh", async () => {
      const h = setup();
      const warmed = h.channel.warm();
      await flush();
      h.workers[0]!.reply(0, { ok: false, error: "the model is not on this device" });
      await expect(warmed).resolves.toBe(false);
      expect(h.workers[0]!.terminated).toBe(true);
      expect(h.channel.running()).toBe(false);
      expect(h.pending()).toBe(0);
      void h.channel.warm();
      await flush();
      expect(h.workers).toHaveLength(2);
    });
  });

  it("counts from the LAST translation: each one restarts the countdown", async () => {
    const h = setup();
    await translated(h);
    const answer = h.channel.translate("again");
    await flush();
    h.workers[0]!.reply(2, { ok: true, html: "encore" });
    await answer;
    expect(h.pending()).toBe(1); // one countdown, not two
  });

  it("the next translation after a release loads the engine again", async () => {
    const h = setup();
    await translated(h);
    h.elapse(ENGINE_IDLE_MS);
    const answer = h.channel.translate("x");
    await flush();
    expect(h.workers).toHaveLength(2);
    expect(h.workers[1]!.sent.map((r) => r.op)).toEqual(["load"]);
    h.workers[1]!.reply(0, { ok: true });
    await flush();
    h.workers[1]!.reply(1, { ok: true, html: "y" });
    await expect(answer).resolves.toEqual({ ok: true, html: "y" });
  });

  it("does not release an engine that is still answering", async () => {
    const onIdle = vi.fn();
    const h = setup(onIdle);
    await translated(h);
    const slow = h.channel.translate("slow"); // asked, not answered yet
    await flush();
    h.elapse(ENGINE_IDLE_MS); // the countdown armed by the first one
    expect(h.workers[0]!.terminated).toBe(false);
    expect(onIdle).not.toHaveBeenCalled();
    h.workers[0]!.reply(2, { ok: true, html: "lent" });
    await expect(slow).resolves.toEqual({ ok: true, html: "lent" });
  });

  it("arms nothing when the engine never started — there is nothing to release", async () => {
    const h = setup();
    const answer = h.channel.translate("x");
    await flush();
    h.elapse(START_TIMEOUT_MS);
    await answer;
    expect(h.pending()).toBe(0);
  });

  it("shuts down at once when the setting is turned off", async () => {
    const h = setup();
    await translated(h);
    h.channel.shutDown();
    expect(h.workers[0]!.terminated).toBe(true);
    expect(h.pending()).toBe(0);
  });
});
