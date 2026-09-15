import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { CancelOutcome, JobState } from "@/gen/jobs_admin_pb";
import { PAGE_SIZE, toLocalInput, useJobsStore, validateWindow, windowFor } from "@/stores/jobs";
import { useToastsStore } from "@/stores/toasts";

// Jobs console store (change: add-admin-jobs-console, task 6.2). Driven entirely through
// the injectable client seam — no network, no component.

const HOUR = 60 * 60 * 1000;
const DAY = 24 * HOUR;
const NOW = new Date("2026-09-15T12:00:00Z").getTime();

interface ListReq {
  state: JobState;
  kind: string;
  limit: number;
  offset: number;
}
interface StatsReq {
  window: { fromMs: bigint; toMs: bigint };
  kind: string;
}

const wireJob = {
  id: "0f9c2a4e-1111-4222-8333-444455556666",
  kind: "verification_email",
  channel: "identity.verification_email",
  state: JobState.RETRY_WAIT,
  attemptsMade: 1,
  attemptsLeft: 4,
  enqueuedAtMs: 1_757_000_000_000n,
  nextAttemptAtMs: 1_757_000_060_000n,
  startedAtMs: undefined,
  cancellable: true,
};

/** Fake jobs client. `pages` answers successive list calls in order (the last repeats). */
function wire(
  opts: {
    pages?: { jobs: unknown[]; total: bigint }[];
    outcome?: CancelOutcome;
    cancelError?: unknown;
    avgRunMs?: bigint;
    historySinceMs?: bigint;
  } = {},
) {
  const listCalls: ListReq[] = [];
  const statsCalls: StatsReq[] = [];
  const cancelCalls: string[] = [];
  const pages = opts.pages ?? [{ jobs: [wireJob], total: 1n }];
  const clients = {
    jobs: {
      adminListJobs: vi.fn(async (req: ListReq) => {
        listCalls.push({ ...req });
        return pages[Math.min(listCalls.length - 1, pages.length - 1)];
      }),
      adminGetJobStats: vi.fn(async (req: StatsReq) => {
        statsCalls.push(req);
        return {
          queue: { total: 15n, running: 3n, ready: 12n, scheduled: 0n, retryWait: 0n, blocked: 0n, exhausted: 0n },
          period: {
            completed: 40n,
            failedAttempts: 2n,
            deadLettered: 1n,
            cancelled: 0n,
            avgRunMs: opts.avgRunMs,
          },
          historySinceMs: opts.historySinceMs,
        };
      }),
      adminListJobKinds: vi.fn(async () => ({
        kinds: [
          { name: "verification_email", channel: "identity.verification_email", cancellable: true },
          { name: "purge_user", channel: "identity.purge_user", cancellable: false },
        ],
      })),
      adminCancelJob: vi.fn(async (req: { jobId: string }) => {
        cancelCalls.push(req.jobId);
        if (opts.cancelError) throw opts.cancelError;
        return { outcome: opts.outcome ?? CancelOutcome.CANCELLED };
      }),
    },
  } as unknown as Clients;
  setClientsForTest(clients);
  return { clients, listCalls, statsCalls, cancelCalls };
}

describe("jobs window helpers", () => {
  it("presets end now and reach back their duration", () => {
    const base = { from: "", to: "" };
    expect(windowFor({ preset: "1h", ...base }, NOW)).toEqual({ fromMs: NOW - HOUR, toMs: NOW });
    expect(windowFor({ preset: "24h", ...base }, NOW)).toEqual({ fromMs: NOW - DAY, toMs: NOW });
    expect(windowFor({ preset: "7d", ...base }, NOW)).toEqual({ fromMs: NOW - 7 * DAY, toMs: NOW });
    expect(windowFor({ preset: "30d", ...base }, NOW)).toEqual({ fromMs: NOW - 30 * DAY, toMs: NOW });
  });

  it("a custom range parses its datetime-local bounds as local time", () => {
    const w = windowFor({ preset: "custom", from: "2026-09-10T08:30", to: "2026-09-11T09:00" }, NOW);
    expect(w.fromMs).toBe(new Date(2026, 8, 10, 8, 30).getTime());
    expect(w.toMs).toBe(new Date(2026, 8, 11, 9, 0).getTime());
    expect(Number.isNaN(windowFor({ preset: "custom", from: "", to: "" }, NOW).fromMs)).toBe(true);
  });

  it("toLocalInput round-trips through windowFor, floored to the minute", () => {
    const ms = new Date(2026, 8, 14, 7, 5, 42).getTime();
    expect(toLocalInput(ms)).toBe("2026-09-14T07:05");
    const w = windowFor({ preset: "custom", from: toLocalInput(ms), to: toLocalInput(ms) }, NOW);
    expect(w.fromMs).toBe(new Date(2026, 8, 14, 7, 5).getTime());
  });

  it("validates order, future end, retention and missing bounds", () => {
    expect(validateWindow(NOW - DAY, NOW, NOW)).toBeNull();
    expect(validateWindow(NOW - 89 * DAY, NOW - DAY, NOW)).toBeNull();
    expect(validateWindow(NOW, NOW - DAY, NOW)).toBe("jobs.validation.order");
    expect(validateWindow(NOW, NOW, NOW)).toBe("jobs.validation.order");
    expect(validateWindow(NOW - DAY, NOW + 1, NOW)).toBe("jobs.validation.future");
    expect(validateWindow(NOW - 91 * DAY, NOW - DAY, NOW)).toBe("jobs.validation.retention");
    expect(validateWindow(Number.NaN, NOW, NOW)).toBe("jobs.validation.incomplete");
  });
});

describe("jobs store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(NOW);
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("loads a page, mapping bigints to numbers and unset optionals to null", async () => {
    const { listCalls } = wire();
    const store = useJobsStore();

    await store.load();

    expect(listCalls[0]).toEqual({ state: JobState.UNSPECIFIED, kind: "", limit: PAGE_SIZE, offset: 0 });
    expect(store.page.status).toBe("success");
    if (store.page.status === "success") {
      expect(store.page.data.total).toBe(1);
      expect(store.page.data.jobs[0]).toEqual({
        id: wireJob.id,
        kind: "verification_email",
        channel: "identity.verification_email",
        state: JobState.RETRY_WAIT,
        attemptsMade: 1,
        attemptsLeft: 4,
        enqueuedAt: 1_757_000_000_000,
        nextAttemptAt: 1_757_000_060_000,
        startedAt: null,
        cancellable: true,
      });
    }
  });

  it("loads the statistics for the default 24-hour window", async () => {
    const { statsCalls } = wire({ avgRunMs: 1250n, historySinceMs: 1_700_000_000_000n });
    const store = useJobsStore();

    await store.load();

    expect(statsCalls[0].window).toEqual({ fromMs: BigInt(NOW - DAY), toMs: BigInt(NOW) });
    expect(store.stats.status).toBe("success");
    if (store.stats.status === "success") {
      expect(store.stats.data.queue.total).toBe(15);
      expect(store.stats.data.queue.running).toBe(3);
      expect(store.stats.data.period.completed).toBe(40);
      expect(store.stats.data.period.avgRunMs).toBe(1250);
      expect(store.stats.data.historySince).toBe(1_700_000_000_000);
    }
  });

  it("an unset average run time and an empty history map to null", async () => {
    wire();
    const store = useJobsStore();
    await store.load();
    if (store.stats.status !== "success") throw new Error("expected success");
    expect(store.stats.data.period.avgRunMs).toBeNull();
    expect(store.stats.data.historySince).toBeNull();
  });

  it("loads the kinds for the filter", async () => {
    wire();
    const store = useJobsStore();
    await store.loadKinds();
    expect(store.kinds.status).toBe("success");
    if (store.kinds.status === "success") {
      expect(store.kinds.data.map((k) => [k.name, k.cancellable])).toEqual([
        ["verification_email", true],
        ["purge_user", false],
      ]);
    }
  });

  it("changing a filter resets the offset and scopes both reads", async () => {
    const { listCalls, statsCalls } = wire();
    const store = useJobsStore();
    await store.goToPage(50);
    expect(listCalls.at(-1)?.offset).toBe(50);

    await store.setFilters({ state: JobState.RETRY_WAIT, kind: "verification_email" });

    expect(store.params.offset).toBe(0);
    expect(listCalls.at(-1)).toEqual({
      state: JobState.RETRY_WAIT,
      kind: "verification_email",
      limit: PAGE_SIZE,
      offset: 0,
    });
    expect(statsCalls.at(-1)?.kind).toBe("verification_email");
  });

  it("changing the period reloads only the statistics", async () => {
    const { listCalls, statsCalls } = wire();
    const store = useJobsStore();
    await store.load();
    const lists = listCalls.length;

    await store.setPeriod({ preset: "7d" });

    expect(listCalls.length).toBe(lists);
    expect(statsCalls.at(-1)?.window).toEqual({ fromMs: BigInt(NOW - 7 * DAY), toMs: BigInt(NOW) });
  });

  it("switching to custom pre-fills the bounds from the previous preset", async () => {
    wire();
    const store = useJobsStore();
    await store.setPeriod({ preset: "custom" });
    expect(store.period.from).toBe(toLocalInput(NOW - DAY));
    expect(store.period.to).toBe(toLocalInput(NOW));
    expect(store.stats.status).toBe("success");
  });

  it("an invalid custom window never reaches the server and lands as a localised error", async () => {
    const { statsCalls } = wire();
    const store = useJobsStore();

    await store.setPeriod({ preset: "custom", from: "2026-09-12T10:00", to: "2026-09-11T10:00" });

    expect(statsCalls).toHaveLength(0);
    expect(store.stats).toEqual({ status: "error", error: "The start must be before the end." });
  });

  it("a cancellation toasts success and re-reads the page and the statistics", async () => {
    const { listCalls, statsCalls, cancelCalls } = wire({ outcome: CancelOutcome.CANCELLED });
    const store = useJobsStore();
    const toasts = useToastsStore();
    await store.load();
    const [lists, stats] = [listCalls.length, statsCalls.length];

    await store.cancel(wireJob.id);

    expect(cancelCalls).toEqual([wireJob.id]);
    expect(toasts.items.map((t) => [t.message, t.variant])).toEqual([["Job cancelled.", "success"]]);
    expect(listCalls.length).toBe(lists + 1);
    expect(statsCalls.length).toBe(stats + 1);
    expect(store.cancelling).toBeNull();
    // A re-read, not a reload: the table never went back to `loading`.
    expect(store.page.status).toBe("success");
  });

  it.each([
    [CancelOutcome.GONE, "This job is no longer in the queue.", "info"],
    [CancelOutcome.RUNNING, "A running job can't be cancelled.", "error"],
    [CancelOutcome.PROTECTED, "Data-erasure jobs can't be cancelled.", "error"],
  ])("outcome %s toasts its message and still re-reads", async (outcome, message, variant) => {
    const { listCalls } = wire({ outcome });
    const store = useJobsStore();
    const toasts = useToastsStore();

    await store.cancel(wireJob.id);

    expect(toasts.items.map((t) => [t.message, t.variant])).toEqual([[message, variant]]);
    expect(listCalls).toHaveLength(1);
  });

  it("a failed cancellation toasts a human error, never the raw code", async () => {
    wire({ cancelError: new ConnectError("admin required", Code.PermissionDenied) });
    const store = useJobsStore();
    const toasts = useToastsStore();
    vi.spyOn(console, "error").mockImplementation(() => {});

    await store.cancel(wireJob.id);

    expect(toasts.items.map((t) => [t.message, t.variant])).toEqual([["Access denied.", "error"]]);
    expect(store.cancelling).toBeNull();
  });

  it("steps back to the last page with rows when the current one empties", async () => {
    const full = { jobs: [wireJob], total: 26n };
    const emptied = { jobs: [], total: 25n };
    const { listCalls } = wire({ pages: [full, emptied, full] });
    const store = useJobsStore();
    await store.goToPage(PAGE_SIZE);

    await store.cancel(wireJob.id);

    expect(listCalls.map((c) => c.offset)).toEqual([PAGE_SIZE, PAGE_SIZE, 0]);
    expect(store.params.offset).toBe(0);
    expect(store.page.status).toBe("success");
  });

  it("a failed page read lands in the error union", async () => {
    const { clients } = wire();
    (clients.jobs.adminListJobs as unknown as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
      new ConnectError("nope", Code.Unavailable),
    );
    vi.spyOn(console, "error").mockImplementation(() => {});
    const store = useJobsStore();

    await store.load();

    expect(store.page).toEqual({ status: "error", error: "Service unavailable. Try again." });
  });
});
