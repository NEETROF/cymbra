import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { PAGE_INTERVAL_MS, SURFACE_INTERVAL_MS, SyncScheduler } from "@/sync/scheduler.ts";

function makeScheduler(options: { signedIn?: boolean; fail?: () => unknown } = {}) {
  let signedIn = options.signedIn ?? true;
  let clock = 1_000_000;
  let lastSynced = 0;
  const runs: number[] = [];
  const errors: unknown[] = [];
  let release: (() => void) | null = null;
  const scheduler = new SyncScheduler({
    sync: async () => {
      runs.push(clock);
      if (release) await new Promise<void>((resolve) => (release = resolve));
      const failure = options.fail?.();
      if (failure) throw failure;
    },
    signedIn: () => signedIn,
    now: () => clock,
    lastSynced: async () => lastSynced,
    onSynced: async (at) => void (lastSynced = at),
    onError: (e) => void errors.push(e),
  });
  return {
    scheduler,
    runs,
    errors,
    lastSynced: () => lastSynced,
    signOut: () => (signedIn = false),
    advance: (ms: number) => (clock += ms),
    /** Make the next run block until `finish()` is called. */
    blockNextRun: () => (release = () => {}),
    finish: () => {
      const resolve = release;
      release = null;
      resolve?.();
    },
  };
}

describe("SyncScheduler", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("coalesces a burst of triggers into one run", async () => {
    const h = makeScheduler();

    h.scheduler.schedule(2000);
    h.scheduler.schedule(2000);
    h.scheduler.schedule(2000);
    await vi.advanceTimersByTimeAsync(2000);

    expect(h.runs).toHaveLength(1);
  });

  it("runs nothing while signed out", async () => {
    const h = makeScheduler({ signedIn: false });

    h.scheduler.schedule(0);
    await h.scheduler.onOpen();
    await vi.advanceTimersByTimeAsync(5000);

    expect(h.runs).toHaveLength(0);
    expect(await h.scheduler.syncNow()).toBe("unauthenticated");
  });

  it("drains a trigger that arrived during a run", async () => {
    const h = makeScheduler();
    h.blockNextRun();
    h.scheduler.schedule(0);
    await vi.advanceTimersByTimeAsync(0);
    expect(h.runs).toHaveLength(1);

    h.scheduler.schedule(0); // a mutation mid-run
    h.finish();
    await vi.advanceTimersByTimeAsync(0);

    expect(h.runs).toHaveLength(2);
  });

  it("grants an open once per interval, counting the last success", async () => {
    const h = makeScheduler();

    await h.scheduler.onOpen();
    await h.scheduler.onOpen(); // straight away: ignored
    expect(h.runs).toHaveLength(1);

    h.advance(SURFACE_INTERVAL_MS);
    await h.scheduler.onOpen();
    expect(h.runs).toHaveLength(2);

    // A page load waits longer between runs than a surface the reader just opened.
    h.advance(SURFACE_INTERVAL_MS);
    await h.scheduler.onOpen(PAGE_INTERVAL_MS);
    expect(h.runs).toHaveLength(2);
    h.advance(PAGE_INTERVAL_MS);
    await h.scheduler.onOpen(PAGE_INTERVAL_MS);
    expect(h.runs).toHaveLength(3);
  });

  it("resolves only once its run is over, so the caller can keep the page alive", async () => {
    const h = makeScheduler();
    h.blockNextRun();

    let done = false;
    const open = h.scheduler.onOpen().then(() => (done = true));
    await vi.advanceTimersByTimeAsync(0);
    expect(h.runs).toHaveLength(1);
    expect(done).toBe(false); // the exchange is still going

    h.finish();
    await open;
    expect(done).toBe(true);
  });

  it("records the last success and reports a failure's category", async () => {
    const failing = makeScheduler({ fail: () => new TypeError("fetch failed") });

    expect(await failing.scheduler.syncNow()).toBe("unavailable");
    expect(failing.errors).toHaveLength(1);
    expect(failing.lastSynced()).toBe(0);

    const ok = makeScheduler();
    expect(await ok.scheduler.syncNow()).toBeNull();
    expect(ok.lastSynced()).toBeGreaterThan(0);
  });

  it("waits for a run in flight before forcing one", async () => {
    const h = makeScheduler();
    h.blockNextRun();
    h.scheduler.schedule(0);
    await vi.advanceTimersByTimeAsync(0);

    const forced = h.scheduler.syncNow();
    h.finish();
    await vi.advanceTimersByTimeAsync(0);

    expect(await forced).toBeNull();
    expect(h.runs).toHaveLength(2);
  });

  it("holds every run while an exclusive task runs, then drains", async () => {
    const h = makeScheduler();
    let erased = false;

    await h.scheduler.exclusive(async () => {
      h.scheduler.schedule(0); // a trigger during the erasure
      expect(await h.scheduler.syncNow()).toBe("conflict");
      await vi.advanceTimersByTimeAsync(2000);
      expect(h.runs).toHaveLength(0);
      erased = true;
    });
    await vi.advanceTimersByTimeAsync(0);

    expect(erased).toBe(true);
    expect(h.runs).toHaveLength(1); // the held trigger, once the erasure is done
  });
});
