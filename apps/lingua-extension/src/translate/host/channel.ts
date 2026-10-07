// The engine's worker, seen from whoever owns it: the offscreen document on Chromium, the
// event page on Firefox. Requests are correlated by id, and every wait is bounded.
//
// The start bound is not caution. The glue fails in a way that looks like slowness: when its
// init throws — as it does if it is ever loaded as a module, where `this` is undefined — the
// promise it would have resolved stays pending FOREVER. Without a bound, a broken engine is
// indistinguishable from a slow one, and a card waits on it indefinitely.
//
// The worker holds ~195 MiB once the engine has loaded, so it is put down when reading stops
// (add-lingua-translation-delivery D6): ten minutes after the last translation ASKED — or the last
// warm, which says one is coming (a selection begun, a translating page back: android D2, D3).
// Nothing else counts — a reading tab's keep-warm ping never reaches this channel — so a forgotten
// tab does not hold the engine for the afternoon, and the next translation simply pays a cold start.
//
// A trap is the one other reason to put it down before then (harden-lingua-translation-engine D2).
// The engine can trap under an input — measured through the en-es model — and a trapped instance
// poisons every model built after it, so the worker reports the trap and closes itself; here it
// is put down, a fresh one loads the route again, and the request is asked once more. Once: a
// request that traps the fresh worker too is unavailable. Everything in flight on the worker that
// trapped is asked again the same way, each once — a marked selection's two requests (relay.ts)
// both get their answer from the fresh instance. An ordinary refusal — no model, a route too long,
// not loaded — is passed on as it always was, and the worker is kept.

import type { EngineAccess, EngineReply, WorkerRequest, WorkerResponse } from "./engine.ts";
import { ENGINE_IDLE_MS } from "../port.ts";

/** Loading ~37 MB of model and instantiating the wasm. Measured at ~200 ms; the bound is generous. */
export const START_TIMEOUT_MS = 15_000;
/** One sentence. Measured at 11–136 ms in the worker; past this, the worker is stuck. */
export const TRANSLATE_TIMEOUT_MS = 10_000;
/** The answer to a request that trapped the fresh worker too (D2): asked twice, never a third time. */
export const TRAPPED_TWICE = "the engine trapped twice";

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

/** Where the channel says what it did about a trap (D5): the pair and the markup's length, never the text. */
export type ChannelLog = (message: string, detail?: unknown) => void;

const REAL_CLOCK: ChannelClock = {
  setTimeout: (fn, ms) => setTimeout(fn, ms),
  clearTimeout: (h) => clearTimeout(h as ReturnType<typeof setTimeout>),
};

const LOG: ChannelLog = (message, detail) => console.warn(`[Cymbra Lingua] ${message}`, detail ?? "");

type Waiter = (response: WorkerResponse | null) => void;

type RequestBody = { op: "load"; pair: string } | { op: "translate"; markup: string; pair: string };

type Trap = Extract<WorkerResponse, { ok: false }>;

/** How a load ended: the route is loaded, the worker refused it, or the worker trapped under it (D2). */
type Loaded = "loaded" | "refused" | "trapped";

/** A translation attempt the worker trapped under: asked again once, then given up. */
const TRAPPED = Symbol("trapped");

export class EngineChannel implements EngineAccess {
  private worker: WorkerLike | null = null;
  /** Each pair's route, loading or loaded in the worker (model-state D5, routes-by-pair D2). */
  private readonly loads = new Map<string, Promise<Loaded>>();
  private seq = 0;
  private readonly waiting = new Map<number, Waiter>();
  private readonly clock: ChannelClock;
  private readonly log: ChannelLog;
  private idleTimer: unknown = null;

  constructor(
    private readonly spawn: () => WorkerLike,
    private readonly opts: {
      clock?: ChannelClock;
      /** Called once the idle worker has been put down (the offscreen document may then close). */
      onIdle?: () => void;
      /** Where a trap and its replay are reported (D5); the console by default. */
      log?: ChannelLog;
    } = {},
  ) {
    this.clock = opts.clock ?? REAL_CLOCK;
    this.log = opts.log ?? LOG;
  }

  /** Whether the engine's worker is alive (and, once loaded, holding its memory). */
  running(): boolean {
    return this.worker !== null;
  }

  /** Put the worker down now: the setting was turned off. */
  shutDown(): void {
    this.reset();
  }

  async translate(markup: string, pair: string): Promise<EngineReply> {
    try {
      return await this.translateNow(markup, pair);
    } finally {
      this.armIdle();
    }
  }

  /**
   * Load the engine and `pair`'s route now, translate nothing; a warm engine is left as it is, its
   * countdown restarted. A load the worker traps under is asked again once, on a fresh worker (D2).
   */
  async warm(pair: string): Promise<boolean> {
    try {
      const first = await this.start(pair);
      if (first !== "trapped") return first === "loaded";
      this.log("trapped, asked again:", { pair, markup: 0 });
      const again = await this.start(pair);
      if (again === "trapped") this.log("trapped twice:", { pair, markup: 0 });
      return again === "loaded";
    } finally {
      this.armIdle();
    }
  }

  /** One attempt, and one more if the worker trapped under it; a second trap is final (D2, D5). */
  private async translateNow(markup: string, pair: string): Promise<EngineReply> {
    const first = await this.attempt(markup, pair);
    if (first !== TRAPPED) return first;
    this.log("trapped, asked again:", { pair, markup: markup.length });
    const again = await this.attempt(markup, pair);
    if (again !== TRAPPED) return again;
    this.log("trapped twice:", { pair, markup: markup.length });
    return { ok: false, reason: TRAPPED_TWICE };
  }

  /** The route loaded, the markup sent, the reply read — or TRAPPED, when the worker trapped under either. */
  private async attempt(markup: string, pair: string): Promise<EngineReply | typeof TRAPPED> {
    const started = await this.start(pair);
    if (started === "trapped") return TRAPPED;
    if (started !== "loaded") return { ok: false, reason: "the engine did not start" };
    const reply = await this.call({ op: "translate", markup, pair }, TRANSLATE_TIMEOUT_MS);
    if (!reply) {
      // A synchronous wasm call cannot be interrupted: a worker past its bound is stuck in one,
      // and everything sent after would queue behind it. Put it down; the next request starts
      // a fresh one.
      this.reset();
      return { ok: false, reason: "the translation timed out" };
    }
    if (!reply.ok) return reply.trap ? TRAPPED : { ok: false, reason: reply.error };
    return typeof reply.html === "string" ? { ok: true, html: reply.html } : { ok: false, reason: "empty answer" };
  }

  /**
   * Spawn the worker if there is none and load `pair`'s route, once. A refused load is forgotten,
   * so the next request asks again; with no other route loaded, the worker is put down with it.
   * A load the worker trapped under has already been forgotten with the worker (`trapped`).
   */
  private start(pair: string): Promise<Loaded> {
    let loading = this.loads.get(pair);
    if (!loading) {
      loading = this.load(pair).then((outcome) => {
        if (outcome === "refused" && this.loads.get(pair) === loading) {
          this.loads.delete(pair);
          if (this.loads.size === 0) this.reset();
        }
        return outcome;
      });
      this.loads.set(pair, loading);
    }
    return loading;
  }

  private async load(pair: string): Promise<Loaded> {
    if (!this.worker) {
      let worker: WorkerLike;
      try {
        worker = this.spawn();
      } catch {
        return "refused";
      }
      // Bound to the worker that sends them: whatever one already put down still says — a second
      // trap it reported before it closed — must not reach the fresh one in its place.
      worker.onmessage = (event) => this.settle(worker, event.data);
      worker.onerror = () => {
        if (this.worker === worker) this.failAll();
      };
      this.worker = worker;
    }
    const loaded = await this.call({ op: "load", pair }, START_TIMEOUT_MS);
    if (loaded?.ok) return "loaded";
    return loaded?.trap ? "trapped" : "refused";
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

  private settle(worker: WorkerLike, response: WorkerResponse): void {
    if (worker !== this.worker) return; // from a worker already put down
    if (response && !response.ok && response.trap) return this.trapped(response);
    const waiter = this.waiting.get(response?.id);
    if (!waiter) return; // late, after its own timeout
    this.waiting.delete(response.id);
    waiter(response);
  }

  /**
   * The worker trapped under `trap`'s request (D2). It has closed itself, so it is put down here
   * first; then everything in flight on it — the request that trapped and every other, a load
   * included — is answered with the trap, so that each asks again once, and finds no worker but
   * the fresh one. Whether the request was still waited for makes no difference: the instance
   * is poisoned either way.
   */
  private trapped(trap: Trap): void {
    const waiters = [...this.waiting.entries()];
    this.waiting.clear();
    this.reset();
    for (const [id, waiter] of waiters) {
      waiter(id === trap.id ? trap : { id, ok: false, error: trap.error, trap: true });
    }
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
    this.loads.clear();
  }
}
