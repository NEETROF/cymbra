// The engine's worker, seen from whoever owns it: the offscreen document on Chromium, the
// event page on Firefox. Requests are correlated by id, and every wait is bounded.
//
// The start bound is not caution. The glue fails in a way that looks like slowness: when its
// init throws — as it does if it is ever loaded as a module, where `this` is undefined — the
// promise it would have resolved stays pending FOREVER. Without a bound, a broken engine is
// indistinguishable from a slow one, and a card waits on it indefinitely.
//
// The worker holds ~195 MiB once the engine has loaded, so it is put down when reading stops
// (add-lingua-translation-delivery D6): ten minutes after the last translation ASKED. Nothing else
// counts — a reading tab's keep-warm ping never reaches this channel — so a forgotten tab does not
// hold the engine for the afternoon, and the next translation simply pays a cold start.

import type { EngineAccess, EngineReply, WorkerRequest, WorkerResponse } from "./engine.ts";

/** Loading ~37 MB of model and instantiating the wasm. Measured at ~200 ms; the bound is generous. */
export const START_TIMEOUT_MS = 15_000;
/** One sentence. Measured at 11–136 ms in the worker; past this, the worker is stuck. */
export const TRANSLATE_TIMEOUT_MS = 10_000;
/**
 * How long the engine stays loaded after the last translation asked. A starting value: long
 * enough to cover a pause in reading, short enough not to hold ~200 MiB for an afternoon.
 */
export const ENGINE_IDLE_MS = 10 * 60_000;

/** The worker, as much of it as the channel uses — a test hands in a fake. */
export interface WorkerLike {
  postMessage(message: WorkerRequest): void;
  terminate(): void;
  onmessage: ((event: { data: WorkerResponse }) => void) | null;
  onerror: ((event: unknown) => void) | null;
}

export interface ChannelClock {
  setTimeout(fn: () => void, ms: number): unknown;
  clearTimeout(handle: unknown): void;
}

const REAL_CLOCK: ChannelClock = {
  setTimeout: (fn, ms) => setTimeout(fn, ms),
  clearTimeout: (h) => clearTimeout(h as ReturnType<typeof setTimeout>),
};

type Waiter = (response: WorkerResponse | null) => void;

type RequestBody = { op: "load" } | { op: "translate"; markup: string };

export class EngineChannel implements EngineAccess {
  private worker: WorkerLike | null = null;
  private started: Promise<boolean> | null = null;
  private seq = 0;
  private readonly waiting = new Map<number, Waiter>();
  private readonly clock: ChannelClock;
  private idleTimer: unknown = null;

  constructor(
    private readonly spawn: () => WorkerLike,
    private readonly opts: {
      clock?: ChannelClock;
      /** Called once the idle worker has been put down (the offscreen document may then close). */
      onIdle?: () => void;
    } = {},
  ) {
    this.clock = opts.clock ?? REAL_CLOCK;
  }

  /** Whether the engine's worker is alive (and, once loaded, holding its memory). */
  running(): boolean {
    return this.worker !== null;
  }

  /** Put the worker down now: the setting was turned off. */
  shutDown(): void {
    this.reset();
  }

  async translate(markup: string): Promise<EngineReply> {
    try {
      return await this.translateNow(markup);
    } finally {
      this.armIdle();
    }
  }

  private async translateNow(markup: string): Promise<EngineReply> {
    if (!(await this.start())) return { ok: false, reason: "the engine did not start" };
    const reply = await this.call({ op: "translate", markup }, TRANSLATE_TIMEOUT_MS);
    if (!reply) {
      // A synchronous wasm call cannot be interrupted: a worker past its bound is stuck in one,
      // and everything sent after would queue behind it. Put it down; the next request starts
      // a fresh one.
      this.reset();
      return { ok: false, reason: "the translation timed out" };
    }
    if (!reply.ok) return { ok: false, reason: reply.error };
    return typeof reply.html === "string" ? { ok: true, html: reply.html } : { ok: false, reason: "empty answer" };
  }

  /** Spawn the worker and load the model, once; a failure leaves nothing behind to retry against. */
  private start(): Promise<boolean> {
    this.started ??= this.boot().then((ok) => {
      if (!ok) this.reset();
      return ok;
    });
    return this.started;
  }

  private async boot(): Promise<boolean> {
    let worker: WorkerLike;
    try {
      worker = this.spawn();
    } catch {
      return false;
    }
    worker.onmessage = (event) => this.settle(event.data);
    worker.onerror = () => this.failAll();
    this.worker = worker;
    const loaded = await this.call({ op: "load" }, START_TIMEOUT_MS);
    return loaded?.ok === true;
  }

  private call(body: RequestBody, ms: number): Promise<WorkerResponse | null> {
    const worker = this.worker;
    if (!worker) return Promise.resolve(null);
    const id = ++this.seq;
    return new Promise((resolve) => {
      const timer = this.clock.setTimeout(() => {
        this.waiting.delete(id);
        resolve(null);
      }, ms);
      this.waiting.set(id, (response) => {
        this.clock.clearTimeout(timer);
        resolve(response);
      });
      worker.postMessage({ id, ...body });
    });
  }

  private settle(response: WorkerResponse): void {
    const waiter = this.waiting.get(response?.id);
    if (!waiter) return; // late, after its own timeout
    this.waiting.delete(response.id);
    waiter(response);
  }

  /** The worker died: nothing in flight will be answered — say so, rather than let it time out. */
  private failAll(): void {
    const waiters = [...this.waiting.entries()];
    this.waiting.clear();
    for (const [id, waiter] of waiters) waiter({ id, ok: false, error: "the engine's worker stopped" });
    this.reset();
  }

  /** Restart the idle countdown from this translation — if there is a worker left to put down. */
  private armIdle(): void {
    this.disarmIdle();
    if (!this.worker) return;
    this.idleTimer = this.clock.setTimeout(() => {
      this.idleTimer = null;
      if (this.waiting.size > 0) return this.armIdle(); // still answering: not idle
      this.reset();
      this.opts.onIdle?.();
    }, ENGINE_IDLE_MS);
  }

  private disarmIdle(): void {
    if (this.idleTimer === null) return;
    this.clock.clearTimeout(this.idleTimer);
    this.idleTimer = null;
  }

  private reset(): void {
    this.disarmIdle();
    this.worker?.terminate();
    this.worker = null;
    this.started = null;
  }
}
