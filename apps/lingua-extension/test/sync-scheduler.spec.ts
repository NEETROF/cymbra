import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { OPEN_INTERVAL_MS, SyncScheduler } from "@/sync/scheduler.ts";

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

  it("grants an open at most once a minute, counting the last success", async () => {
    const h = makeScheduler();

    await h.scheduler.onOpen();
    await vi.advanceTimersByTimeAsync(0);
    await h.scheduler.onOpen(); // same minute: ignored
    await vi.advanceTimersByTimeAsync(0);
    expect(h.runs).toHaveLength(1);

    h.advance(OPEN_INTERVAL_MS);
    await h.scheduler.onOpen();
    await vi.advanceTimersByTimeAsync(0);
    expect(h.runs).toHaveLength(2);
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
