import { describe, expect, it, vi } from "vitest";
import {
  type ChannelClock,
  EngineChannel,
  START_TIMEOUT_MS,
  TRANSLATE_TIMEOUT_MS,
  TRAPPED_TWICE,
  type WorkerLike,
} from "@/translate/host/channel.ts";
import {
  isTrap,
  LONG_ROUTE,
  NO_MODEL,
  NOT_LOADED,
  type WorkerRequest,
  type WorkerResponse,
} from "@/translate/host/engine.ts";
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
  reply(n: number, body: { ok: true; html?: string } | { ok: false; error: string; trap?: true; reload?: true }): void {
    this.onmessage?.({ data: { id: this.sent[n]!.id, ...body } as WorkerResponse });
  }
  /** The engine trapped under the n-th request, as engine-worker.ts reports it before closing itself (D1). */
  trap(n: number): void {
    this.reply(n, { ok: false, error: "Aborted(memory access out of bounds)", trap: true });
  }
  /** The worker no longer holds the n-th request's route — deleted for another pair's (D3) — as engine-worker.ts refuses it. */
  notLoaded(n: number): void {
    this.reply(n, { ok: false, error: NOT_LOADED, reload: true });
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

/** Let promise continuations run — a replay after a trap chains a dozen of them. */
const flush = async () => {
  for (let i = 0; i < 20; i++) await Promise.resolve();
};

function setup(onIdle?: () => void) {
  const workers: FakeWorker[] = [];
  const { clock, elapse, pending } = manualClock();
  const log = vi.fn<(message: string, detail?: unknown) => void>();
  const channel = new EngineChannel(
    () => {
      const w = new FakeWorker();
      workers.push(w);
      return w;
    },
    { clock, onIdle, log },
  );
  return { channel, workers, elapse, pending, log };
}

describe("EngineChannel", () => {
  it("starts the worker and loads the model before the first translation", async () => {
    const { channel, workers } = setup();
    const answer = channel.translate("<b>gave up</b>", "en-fr");
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
    const a = channel.translate("one", "en-fr");
    const b = channel.translate("two", "en-fr");
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
    const answer = channel.translate("x", "en-fr");
    await flush();
    elapse(START_TIMEOUT_MS);
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine did not start" });
    expect(workers[0]!.terminated).toBe(true);
  });

  it("starts afresh after a failed start, rather than staying broken for the session", async () => {
    const { channel, workers, elapse } = setup();
    const first = channel.translate("x", "en-fr");
    await flush();
    elapse(START_TIMEOUT_MS);
    await first;

    void channel.translate("y", "en-fr");
    await flush();
    expect(workers).toHaveLength(2);
    expect(workers[1]!.sent[0]!.op).toBe("load");
  });

  it("reports a model the worker could not load", async () => {
    // The ordinary development case: the build carries the engine but no model was supplied.
    const { channel, workers } = setup();
    const answer = channel.translate("x", "en-fr");
    await flush();
    workers[0]!.reply(0, { ok: false, error: "engine/model.bin: 404" });
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine did not start" });
  });

  it("reports a worker that could not even be constructed", async () => {
    const channel = new EngineChannel(() => {
      throw new Error("Worker is not defined");
    });
    await expect(channel.translate("x", "en-fr")).resolves.toEqual({ ok: false, reason: "the engine did not start" });
  });

  it("puts down a worker stuck past its bound, and the next request gets a fresh one", async () => {
    // A synchronous wasm call cannot be interrupted: everything sent after it would queue.
    const { channel, workers, elapse } = setup();
    const answer = channel.translate("x", "en-fr");
    await flush();
    workers[0]!.reply(0, { ok: true });
    await flush();
    elapse(TRANSLATE_TIMEOUT_MS);
    await expect(answer).resolves.toEqual({ ok: false, reason: "the translation timed out" });
    expect(workers[0]!.terminated).toBe(true);

    void channel.translate("y", "en-fr");
    await flush();
    expect(workers).toHaveLength(2);
  });

  it("fails everything in flight when the worker dies", async () => {
    const { channel, workers, pending } = setup();
    const answer = channel.translate("x", "en-fr");
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
    const answer = channel.translate("x", "en-fr");
    await flush();
    workers[0]!.reply(0, { ok: true });
    await flush();
    workers[0]!.reply(1, { ok: false, error: "the engine is not loaded" });
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine is not loaded" });
  });

  it("treats an answer with no markup as no answer", async () => {
    const { channel, workers } = setup();
    const answer = channel.translate("x", "en-fr");
    await flush();
    workers[0]!.reply(0, { ok: true });
    await flush();
    workers[0]!.reply(1, { ok: true });
    await expect(answer).resolves.toEqual({ ok: false, reason: "empty answer" });
  });

  it("reports a worker that dies while loading as an engine that did not start", async () => {
    const { channel, workers } = setup();
    const answer = channel.translate("x", "en-fr");
    await flush();
    workers[0]!.crash();
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine did not start" });
  });

  it("loads and translates through the pair asked (generalise-lingua-translation-model-state D5, routes-by-pair D2)", async () => {
    const { channel, workers } = setup();
    const answer = channel.translate("<b>faro</b>", "es-fr");
    await flush();
    expect(workers[0]!.sent[0]).toMatchObject({ op: "load", pair: "es-fr" });
    workers[0]!.reply(0, { ok: true });
    await flush();
    expect(workers[0]!.sent[1]).toMatchObject({ op: "translate", markup: "<b>faro</b>", pair: "es-fr" });
    workers[0]!.reply(1, { ok: true, html: "<b>phare</b>" });
    await expect(answer).resolves.toEqual({ ok: true, html: "<b>phare</b>" });
  });

  it("loads each pair's route once, in the same worker", async () => {
    const { channel, workers } = setup();
    const english = channel.translate("one", "en-fr");
    await flush();
    workers[0]!.reply(0, { ok: true });
    await flush();
    workers[0]!.reply(1, { ok: true, html: "un" });
    await english;

    const spanish = channel.translate("dos", "es-fr");
    await flush();
    expect(workers).toHaveLength(1);
    expect(workers[0]!.sent[2]).toMatchObject({ op: "load", pair: "es-fr" });
    workers[0]!.reply(2, { ok: true });
    await flush();
    workers[0]!.reply(3, { ok: true, html: "deux" });
    await expect(spanish).resolves.toEqual({ ok: true, html: "deux" });

    void channel.translate("three", "en-fr");
    await flush();
    expect(workers[0]!.sent.filter((r) => r.op === "load")).toHaveLength(2); // en-fr is still loaded
  });

  it("The same page for another native language: en-es's route is loaded, and nothing of en-fr", async () => {
    const { channel, workers } = setup();
    const answer = channel.translate("<b>gave up</b>", "en-es");
    await flush();
    expect(workers[0]!.sent).toEqual([{ id: 1, op: "load", pair: "en-es" }]);
    workers[0]!.reply(0, { ok: true });
    await flush();
    workers[0]!.reply(1, { ok: true, html: "<b>se rindió</b>" });
    await expect(answer).resolves.toEqual({ ok: true, html: "<b>se rindió</b>" });
    expect(workers[0]!.sent.map((r) => r.op === "load" && r.pair)).toEqual(["en-es", false]);
  });

  it("A pair without a route: the worker has no models for it, and the engine is put down unstarted", async () => {
    const { channel, workers } = setup();
    const answer = channel.translate("x", "en-es");
    await flush();
    workers[0]!.reply(0, { ok: false, error: NO_MODEL });
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine did not start" });
    expect(workers[0]!.terminated).toBe(true);
    expect(channel.running()).toBe(false);
  });

  it("keeps the routes it loaded when another pair's cannot load", async () => {
    const { channel, workers } = setup();
    const english = channel.translate("one", "en-fr");
    await flush();
    workers[0]!.reply(0, { ok: true });
    await flush();
    workers[0]!.reply(1, { ok: true, html: "un" });
    await english;

    const spanish = channel.translate("dos", "es-fr");
    await flush();
    workers[0]!.reply(2, { ok: false, error: "a route through another language is not supported yet" });
    await expect(spanish).resolves.toEqual({ ok: false, reason: "the engine did not start" });
    expect(workers[0]!.terminated).toBe(false);

    const again = channel.translate("two", "en-fr");
    await flush();
    expect(workers[0]!.sent.at(-1)).toMatchObject({ op: "translate", pair: "en-fr" });
    workers[0]!.reply(workers[0]!.sent.length - 1, { ok: true, html: "deux" });
    await expect(again).resolves.toEqual({ ok: true, html: "deux" });
  });

  it("ignores a reply that arrives after its request timed out", async () => {
    const { channel, workers, elapse } = setup();
    const answer = channel.translate("x", "en-fr");
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
    const answer = h.channel.translate("x", "en-fr");
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
      const warmed = h.channel.warm("en-fr");
      await flush();
      expect(h.workers).toHaveLength(1);
      expect(h.workers[0]!.sent.map((r) => r.op)).toEqual(["load"]);
      h.workers[0]!.reply(0, { ok: true });
      await expect(warmed).resolves.toBe(true);
      expect(h.channel.running()).toBe(true);
    });

    it("the translation that follows uses the warmed engine: no second load", async () => {
      const h = setup();
      const warmed = h.channel.warm("en-fr");
      await flush();
      const answer = h.channel.translate("<b>gave up</b>", "en-fr"); // asked while the model loads
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
      const warmed = h.channel.warm("en-fr");
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
      await expect(h.channel.warm("en-fr")).resolves.toBe(true);
      expect(h.workers).toHaveLength(1);
      expect(h.workers[0]!.sent.map((r) => r.op)).toEqual(["load", "translate"]);
      expect(h.pending()).toBe(1); // one countdown, not two
    });

    it("a start that fails leaves nothing behind, and the next warm tries afresh", async () => {
      const h = setup();
      const warmed = h.channel.warm("en-fr");
      await flush();
      h.workers[0]!.reply(0, { ok: false, error: "the model is not on this device" });
      await expect(warmed).resolves.toBe(false);
      expect(h.workers[0]!.terminated).toBe(true);
      expect(h.channel.running()).toBe(false);
      expect(h.pending()).toBe(0);
      void h.channel.warm("en-fr");
      await flush();
      expect(h.workers).toHaveLength(2);
    });
  });

  it("counts from the LAST translation: each one restarts the countdown", async () => {
    const h = setup();
    await translated(h);
    const answer = h.channel.translate("again", "en-fr");
    await flush();
    h.workers[0]!.reply(2, { ok: true, html: "encore" });
    await answer;
    expect(h.pending()).toBe(1); // one countdown, not two
  });

  it("the next translation after a release loads the engine again", async () => {
    const h = setup();
    await translated(h);
    h.elapse(ENGINE_IDLE_MS);
    const answer = h.channel.translate("x", "en-fr");
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
    const slow = h.channel.translate("slow", "en-fr"); // asked, not answered yet
    await flush();
    h.elapse(ENGINE_IDLE_MS); // the countdown armed by the first one
    expect(h.workers[0]!.terminated).toBe(false);
    expect(onIdle).not.toHaveBeenCalled();
    h.workers[0]!.reply(2, { ok: true, html: "lent" });
    await expect(slow).resolves.toEqual({ ok: true, html: "lent" });
  });

  it("arms nothing when the engine never started — there is nothing to release", async () => {
    const h = setup();
    const answer = h.channel.translate("x", "en-fr");
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

describe("isTrap — a trap told from a refusal (harden-lingua-translation-engine D1)", () => {
  it("is the runtime's own error, as a memory access out of bounds raises it", () => {
    expect(isTrap(new WebAssembly.RuntimeError("memory access out of bounds"))).toBe(true);
  });

  it("is the glue's abort, which throws a message starting with Aborted(", () => {
    expect(isTrap(new Error("Aborted(native code called abort()). Build with -s ASSERTIONS=1 for more info."))).toBe(
      true,
    );
    expect(isTrap(new WebAssembly.RuntimeError("Aborted(OOM)"))).toBe(true);
  });

  it("is a C++ exception the glue rethrows as the bare pointer it is: nothing unwound the wasm stack", () => {
    expect(isTrap(5_247_104)).toBe(true);
    expect(isTrap(0)).toBe(true);
  });

  it("is the wasm stack blown: V8's and JavaScriptCore's RangeError, SpiderMonkey's InternalError", () => {
    expect(isTrap(new RangeError("Maximum call stack size exceeded"))).toBe(true);
    expect(isTrap(new RangeError("Maximum call stack size exceeded."))).toBe(true);
    const firefox = new Error("too much recursion");
    firefox.name = "InternalError";
    expect(isTrap(firefox)).toBe(true);
  });

  it("is none of the engine's refusals, nor a RangeError over a length", () => {
    for (const refusal of [
      NO_MODEL,
      LONG_ROUTE,
      NOT_LOADED,
      "the engine is not loaded",
      "engine/model.bin: 404",
      "aborted",
    ]) {
      expect(isTrap(new Error(refusal))).toBe(false);
    }
    expect(isTrap(new RangeError("Invalid array length"))).toBe(false);
    expect(isTrap(new RangeError("Invalid typed array length: 234291200"))).toBe(false);
    expect(isTrap("Aborted(a string, not an error)")).toBe(false);
    expect(isTrap(undefined)).toBe(false);
    expect(isTrap(null)).toBe(false);
  });
});

describe("EngineChannel — the engine survives a trap (harden-lingua-translation-engine D2)", () => {
  /** The route loaded and `markups` sent, each as a translation: the worker's load answered, then every request in flight. */
  async function inFlight(h: ReturnType<typeof setup>, pair: string, ...markups: string[]) {
    const answers = markups.map((markup) => h.channel.translate(markup, pair));
    await flush();
    h.workers.at(-1)!.reply(0, { ok: true });
    await flush();
    return answers;
  }

  it("A trap in a translation: the worker is put down, a fresh one loads the route, and it translates the sentence", async () => {
    const h = setup();
    const [answer] = await inFlight(h, "en-fr", "<b>gave up</b>");
    expect(h.workers[0]!.sent.map((r) => r.op)).toEqual(["load", "translate"]);

    h.workers[0]!.trap(1);
    await flush();
    expect(h.workers[0]!.terminated).toBe(true);
    expect(h.workers).toHaveLength(2);
    expect(h.workers[1]!.sent).toEqual([{ id: 3, op: "load", pair: "en-fr" }]);

    h.workers[1]!.reply(0, { ok: true });
    await flush();
    expect(h.workers[1]!.sent[1]).toMatchObject({ op: "translate", markup: "<b>gave up</b>", pair: "en-fr" });
    h.workers[1]!.reply(1, { ok: true, html: "<b>a abandonné</b>" });
    await expect(answer).resolves.toEqual({ ok: true, html: "<b>a abandonné</b>" });
    expect(h.log).toHaveBeenCalledExactlyOnceWith("trapped, asked again:", { pair: "en-fr", markup: 14 });
    expect(h.channel.running()).toBe(true);
  });

  it("A trap while a model is built: the fresh worker builds the route and answers the request", async () => {
    const h = setup();
    const answer = h.channel.translate("x", "es-fr");
    await flush();
    h.workers[0]!.trap(0); // the load
    await flush();
    expect(h.workers[0]!.terminated).toBe(true);
    expect(h.workers[1]!.sent).toEqual([{ id: 2, op: "load", pair: "es-fr" }]);
    h.workers[1]!.reply(0, { ok: true });
    await flush();
    h.workers[1]!.reply(1, { ok: true, html: "y" });
    await expect(answer).resolves.toEqual({ ok: true, html: "y" });
    expect(h.workers[1]!.sent.filter((r) => r.op === "load")).toHaveLength(1);
  });

  it("A second trap: the answer is unavailable, and the log names the pair and the markup's length, never its text", async () => {
    const h = setup();
    const [answer] = await inFlight(h, "en-fr", "<b>gave up</b> on it");
    h.workers[0]!.trap(1);
    await flush();
    h.workers[1]!.reply(0, { ok: true });
    await flush();
    h.workers[1]!.trap(1);
    await expect(answer).resolves.toEqual({ ok: false, reason: TRAPPED_TWICE });
    expect(h.workers[1]!.terminated).toBe(true);
    expect(h.workers).toHaveLength(2); // asked twice, never a third time
    expect(h.channel.running()).toBe(false);
    expect(h.pending()).toBe(0); // nothing armed on a worker put down — no bound, no countdown

    expect(h.log.mock.calls).toEqual([
      ["trapped, asked again:", { pair: "en-fr", markup: 20 }],
      ["trapped twice:", { pair: "en-fr", markup: 20 }],
    ]);
    expect(JSON.stringify(h.log.mock.calls)).not.toContain("gave up");
  });

  it("The selection's two requests: the sentence traps while its fragment is in flight, and both are asked again on the fresh worker", async () => {
    const h = setup();
    const [sentence, fragment] = await inFlight(h, "en-fr", "She <b>gave up</b>.", "gave up");
    expect(h.workers[0]!.sent.map((r) => r.op)).toEqual(["load", "translate", "translate"]);

    h.workers[0]!.trap(1); // the sentence
    await flush();
    expect(h.workers).toHaveLength(2);
    h.workers[1]!.reply(0, { ok: true });
    await flush();
    // One load, then both requests again — the fragment was not failed as a crash would fail it.
    expect(h.workers[1]!.sent.map((r) => (r.op === "translate" ? r.markup : r.op))).toEqual([
      "load",
      "She <b>gave up</b>.",
      "gave up",
    ]);
    h.workers[1]!.reply(1, { ok: true, html: "Elle <b>a abandonné</b>." });
    h.workers[1]!.reply(2, { ok: true, html: "a abandonné" });
    await expect(sentence).resolves.toEqual({ ok: true, html: "Elle <b>a abandonné</b>." });
    await expect(fragment).resolves.toEqual({ ok: true, html: "a abandonné" });
    expect(h.log).toHaveBeenCalledTimes(2); // each request, asked again once
  });

  it("each of the two requests is asked again once, and no more: a trap under the replay is final for it alone", async () => {
    const h = setup();
    const [sentence, fragment] = await inFlight(h, "en-fr", "She <b>gave up</b>.", "gave up");
    h.workers[0]!.trap(2); // the fragment, with the sentence in flight
    await flush();
    h.workers[1]!.reply(0, { ok: true });
    await flush();
    h.workers[1]!.trap(1); // the sentence's replay: final for both — both were asked again once
    await expect(sentence).resolves.toEqual({ ok: false, reason: TRAPPED_TWICE });
    await expect(fragment).resolves.toEqual({ ok: false, reason: TRAPPED_TWICE });
    expect(h.workers).toHaveLength(2);
    expect(h.channel.running()).toBe(false);
  });

  it("a load in flight for another pair when the worker traps is asked again too", async () => {
    const h = setup();
    const [english] = await inFlight(h, "en-fr", "one");
    const spanish = h.channel.translate("dos", "es-fr");
    await flush();
    expect(h.workers[0]!.sent.at(-1)).toMatchObject({ op: "load", pair: "es-fr" });

    h.workers[0]!.trap(1); // en-fr's translation, es-fr's load in flight
    await flush();
    expect(h.workers).toHaveLength(2);
    const loads = h.workers[1]!.sent.filter((r) => r.op === "load").map((r) => r.pair);
    expect(loads.sort()).toEqual(["en-fr", "es-fr"]);
    for (const [i, r] of h.workers[1]!.sent.entries()) if (r.op === "load") h.workers[1]!.reply(i, { ok: true });
    await flush();
    for (const [i, r] of h.workers[1]!.sent.entries()) {
      if (r.op === "translate") h.workers[1]!.reply(i, { ok: true, html: r.markup === "one" ? "un" : "deux" });
    }
    await expect(english).resolves.toEqual({ ok: true, html: "un" });
    await expect(spanish).resolves.toEqual({ ok: true, html: "deux" });
  });

  it("A refusal is no trap: the answer is passed on, the worker is kept, and nothing is asked again", async () => {
    const h = setup();
    const [english] = await inFlight(h, "en-fr", "one");
    h.workers[0]!.reply(1, { ok: true, html: "un" });
    await english;

    const long = h.channel.translate("dos", "es-fr");
    await flush();
    h.workers[0]!.reply(2, { ok: false, error: LONG_ROUTE }); // the load refused
    await expect(long).resolves.toEqual({ ok: false, reason: "the engine did not start" });

    const unloaded = h.channel.translate("two", "en-fr");
    await flush();
    h.workers[0]!.reply(3, { ok: false, error: "the engine is not loaded" });
    await expect(unloaded).resolves.toEqual({ ok: false, reason: "the engine is not loaded" });

    expect(h.workers).toHaveLength(1);
    expect(h.workers[0]!.terminated).toBe(false);
    expect(h.workers[0]!.sent).toHaveLength(4); // nothing sent twice
    expect(h.log).not.toHaveBeenCalled();
  });

  it("a warm that traps while loading is asked again on a fresh worker, once", async () => {
    const h = setup();
    const warmed = h.channel.warm("en-fr");
    await flush();
    h.workers[0]!.trap(0);
    await flush();
    expect(h.workers[1]!.sent).toEqual([{ id: 2, op: "load", pair: "en-fr" }]);
    h.workers[1]!.reply(0, { ok: true });
    await expect(warmed).resolves.toBe(true);
    expect(h.log).toHaveBeenCalledExactlyOnceWith("trapped, asked again:", { pair: "en-fr", markup: 0 });

    const twice = h.channel.warm("es-fr");
    await flush();
    h.workers[1]!.trap(1);
    await flush();
    h.workers[2]!.trap(0);
    await expect(twice).resolves.toBe(false);
    expect(h.workers).toHaveLength(3);
    expect(h.log).toHaveBeenLastCalledWith("trapped twice:", { pair: "es-fr", markup: 0 });
  });

  it("what a worker already put down still says does not reach the fresh one", async () => {
    const h = setup();
    const [sentence, fragment] = await inFlight(h, "en-fr", "She <b>gave up</b>.", "gave up");
    const old = h.workers[0]!;
    old.trap(1);
    await flush();
    old.trap(2); // reported before it closed, delivered after it was put down
    await flush();
    expect(h.workers).toHaveLength(2);
    expect(h.workers[1]!.terminated).toBe(false);
    h.workers[1]!.reply(0, { ok: true });
    await flush();
    h.workers[1]!.reply(1, { ok: true, html: "Elle <b>a abandonné</b>." });
    h.workers[1]!.reply(2, { ok: true, html: "a abandonné" });
    await expect(sentence).resolves.toMatchObject({ ok: true });
    await expect(fragment).resolves.toMatchObject({ ok: true });
  });

  it("a trap with the idle countdown armed: the countdown goes with the worker put down, and nothing stale fires on the fresh one", async () => {
    const onIdle = vi.fn();
    const h = setup(onIdle);
    const [first] = await inFlight(h, "en-fr", "x");
    h.workers[0]!.reply(1, { ok: true, html: "y" });
    await first;
    expect(h.pending()).toBe(1); // the countdown from that answer

    const again = h.channel.translate("again", "en-fr");
    await flush();
    h.workers[0]!.trap(2);
    await flush();
    expect(h.workers).toHaveLength(2);
    expect(h.pending()).toBe(1); // the fresh worker's load bound — the old countdown went with the old worker
    h.elapse(ENGINE_IDLE_MS); // were it still armed, it would put the fresh worker down mid-replay
    expect(h.workers[1]!.terminated).toBe(false);
    expect(onIdle).not.toHaveBeenCalled();

    h.workers[1]!.reply(0, { ok: true });
    await flush();
    h.workers[1]!.reply(1, { ok: true, html: "encore" });
    await expect(again).resolves.toEqual({ ok: true, html: "encore" });
    expect(h.pending()).toBe(1); // one countdown, on the fresh worker
  });

  it("a trap restarts the idle countdown from the answer the fresh worker gave", async () => {
    const h = setup();
    const [answer] = await inFlight(h, "en-fr", "x");
    h.workers[0]!.trap(1);
    await flush();
    h.workers[1]!.reply(0, { ok: true });
    await flush();
    h.workers[1]!.reply(1, { ok: true, html: "y" });
    await answer;
    expect(h.pending()).toBe(1); // one countdown, on the fresh worker
    h.elapse(ENGINE_IDLE_MS);
    expect(h.workers[1]!.terminated).toBe(true);
  });
});

describe("EngineChannel — a route the worker deleted is loaded again (harden-lingua-translation-engine D3)", () => {
  /** en-fr loaded and one translation answered: the channel believes en-fr is in the worker. */
  async function loadedEnglish(h: ReturnType<typeof setup>) {
    const answer = h.channel.translate("one", "en-fr");
    await flush();
    h.workers[0]!.reply(0, { ok: true });
    await flush();
    h.workers[0]!.reply(1, { ok: true, html: "un" });
    await answer;
  }

  it("a translate refused with reload: the pair is forgotten, loaded again, and the markup sent once more — the worker kept", async () => {
    const h = setup();
    await loadedEnglish(h);
    const answer = h.channel.translate("<b>gave up</b>", "en-fr");
    await flush();
    expect(h.workers[0]!.sent.at(-1)).toMatchObject({ op: "translate", markup: "<b>gave up</b>" }); // no load: en-fr is believed held
    h.workers[0]!.notLoaded(2); // deleted since, for another pair's route
    await flush();
    expect(h.workers).toHaveLength(1);
    expect(h.workers[0]!.terminated).toBe(false);
    expect(h.workers[0]!.sent.at(-1)).toEqual({ id: 4, op: "load", pair: "en-fr" });
    h.workers[0]!.reply(3, { ok: true });
    await flush();
    expect(h.workers[0]!.sent.at(-1)).toMatchObject({ op: "translate", markup: "<b>gave up</b>", pair: "en-fr" });
    h.workers[0]!.reply(4, { ok: true, html: "<b>a abandonné</b>" });
    await expect(answer).resolves.toEqual({ ok: true, html: "<b>a abandonné</b>" });
    expect(h.log).not.toHaveBeenCalled(); // not a trap's replay
  });

  it("the load again runs under the start bound, not the translate bound: a pivot's rebuild on a tablet can pass ten seconds", async () => {
    const h = setup();
    await loadedEnglish(h);
    const answer = h.channel.translate("x", "en-fr");
    await flush();
    h.workers[0]!.notLoaded(2);
    await flush();
    expect(h.workers[0]!.sent.at(-1)).toMatchObject({ op: "load" });
    h.elapse(TRANSLATE_TIMEOUT_MS); // ten seconds into the rebuild: nothing gives up
    expect(h.workers[0]!.terminated).toBe(false);
    h.workers[0]!.reply(3, { ok: true });
    await flush();
    h.workers[0]!.reply(4, { ok: true, html: "y" });
    await expect(answer).resolves.toEqual({ ok: true, html: "y" });
  });

  it("a load again that never answers ends at the start bound, as a first load does", async () => {
    const h = setup();
    await loadedEnglish(h);
    const answer = h.channel.translate("x", "en-fr");
    await flush();
    h.workers[0]!.notLoaded(2);
    await flush();
    h.elapse(START_TIMEOUT_MS);
    await expect(answer).resolves.toEqual({ ok: false, reason: "the engine did not start" });
    expect(h.workers[0]!.terminated).toBe(true); // no route left loaded
    expect(h.pending()).toBe(0);
  });

  it("refused with reload a second time, the refusal is passed on: asked once more, never a third time", async () => {
    const h = setup();
    await loadedEnglish(h);
    const answer = h.channel.translate("x", "en-fr");
    await flush();
    h.workers[0]!.notLoaded(2);
    await flush();
    h.workers[0]!.reply(3, { ok: true });
    await flush();
    h.workers[0]!.notLoaded(4);
    await expect(answer).resolves.toEqual({ ok: false, reason: NOT_LOADED });
    expect(h.workers[0]!.sent.map((r) => r.op)).toEqual(["load", "translate", "translate", "load", "translate"]);
    expect(h.workers[0]!.terminated).toBe(false);
    expect(h.log).not.toHaveBeenCalled();
  });

  it("two translations through the pair refused with reload share the one load again", async () => {
    const h = setup();
    await loadedEnglish(h);
    const a = h.channel.translate("a", "en-fr");
    const b = h.channel.translate("b", "en-fr");
    await flush();
    h.workers[0]!.notLoaded(2);
    h.workers[0]!.notLoaded(3);
    await flush();
    expect(h.workers[0]!.sent.filter((r) => r.op === "load")).toHaveLength(2); // the first, and one again
    h.workers[0]!.reply(4, { ok: true });
    await flush();
    expect(h.workers[0]!.sent.slice(5).map((r) => r.op)).toEqual(["translate", "translate"]);
    h.workers[0]!.reply(5, { ok: true, html: "A" });
    h.workers[0]!.reply(6, { ok: true, html: "B" });
    await expect(a).resolves.toEqual({ ok: true, html: "A" });
    await expect(b).resolves.toEqual({ ok: true, html: "B" });
  });

  it("a trap under the load again is the request's one replay, on a fresh worker", async () => {
    const h = setup();
    await loadedEnglish(h);
    const answer = h.channel.translate("x", "en-fr");
    await flush();
    h.workers[0]!.notLoaded(2);
    await flush();
    h.workers[0]!.trap(3); // the load again
    await flush();
    expect(h.workers).toHaveLength(2);
    expect(h.workers[1]!.sent).toEqual([{ id: 5, op: "load", pair: "en-fr" }]);
    h.workers[1]!.reply(0, { ok: true });
    await flush();
    h.workers[1]!.reply(1, { ok: true, html: "y" });
    await expect(answer).resolves.toEqual({ ok: true, html: "y" });
    expect(h.log).toHaveBeenCalledExactlyOnceWith("trapped, asked again:", { pair: "en-fr", markup: 1 });
  });

  it("a worker built before the flag refuses without it, and the refusal is passed on as it always was", async () => {
    const h = setup();
    await loadedEnglish(h);
    const answer = h.channel.translate("x", "en-fr");
    await flush();
    h.workers[0]!.reply(2, { ok: false, error: NOT_LOADED }); // no `reload`
    await expect(answer).resolves.toEqual({ ok: false, reason: NOT_LOADED });
    expect(h.workers[0]!.sent).toHaveLength(3); // nothing loaded again
  });
});
