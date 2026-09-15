import { reactive, ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, failure, idle, reread, run } from "@/lib/async";
import { humanError } from "@/lib/errors";
import { t } from "@/i18n";
import { CancelOutcome, JobState, type QueuedJob } from "@/gen/jobs_admin_pb";
import { type ToastVariant, useToastsStore } from "@/stores/toasts";

// Jobs console (change: add-admin-jobs-console, task 6.2). The ONLY place that calls
// `api().jobs` — the view never does. The page, the statistics and the kind list are each
// an `Async` union the view matches exhaustively; a denied/failed read lands in the union
// as a localised error, never a throw. A cancellation's outcome is a toast pushed from
// here, so the view fires `cancel` and reacts to state instead of branching on a return.
//
// Privacy: these types carry queue METADATA only — the contract has no payload and no
// error text to carry (see `jobs_admin.proto`).

/** Table page size (the server accepts 1..=100). */
export const PAGE_SIZE = 25;
/** How far back the attempt history reaches; the server rejects a window starting earlier. */
export const RETENTION_DAYS = 90;

const HOUR_MS = 60 * 60 * 1000;
const DAY_MS = 24 * HOUR_MS;

/** One queued job, with epoch-ms times as plain numbers (`null` = not set). */
export interface QueuedJobRow {
  id: string;
  kind: string;
  channel: string;
  state: JobState;
  attemptsMade: number;
  attemptsLeft: number;
  enqueuedAt: number;
  nextAttemptAt: number | null;
  startedAt: number | null;
  cancellable: boolean;
}
/** One page of the table plus the total matching the filters. */
export interface JobsPage {
  jobs: QueuedJobRow[];
  total: number;
}
/** The queue as it is now. */
export interface QueueCounts {
  total: number;
  running: number;
  ready: number;
  scheduled: number;
  retryWait: number;
  blocked: number;
  exhausted: number;
}
/** Figures over the selected window. */
export interface PeriodStats {
  completed: number;
  failedAttempts: number;
  deadLettered: number;
  cancelled: number;
  /** Mean run time of succeeded attempts; `null` when none succeeded. */
  avgRunMs: number | null;
}
export interface JobStats {
  queue: QueueCounts;
  period: PeriodStats;
  /** When the attempt history starts (epoch ms); `null` while it is empty. Figures for
   *  time before it are unknown, not zero. */
  historySince: number | null;
}
/** A registered job kind (feeds the kind filter). */
export interface JobKindInfo {
  name: string;
  channel: string;
  cancellable: boolean;
}

/** The table filters. `UNSPECIFIED` = any state, `""` = any kind. */
export interface JobsParams {
  state: JobState;
  kind: string;
  offset: number;
}

export type PeriodPreset = "1h" | "24h" | "7d" | "30d" | "custom";
/** The statistics period. `from`/`to` are `datetime-local` strings, read only when the
 *  preset is `custom`. */
export interface PeriodSelection {
  preset: PeriodPreset;
  from: string;
  to: string;
}

/** A translation key naming why a custom window is refused. */
export type WindowError =
  "jobs.validation.incomplete" | "jobs.validation.order" | "jobs.validation.future" | "jobs.validation.retention";

const PRESET_MS: Record<Exclude<PeriodPreset, "custom">, number> = {
  "1h": HOUR_MS,
  "24h": DAY_MS,
  "7d": 7 * DAY_MS,
  "30d": 30 * DAY_MS,
};

/** The `[fromMs, toMs)` window a period stands for at `now`. Presets end at `now`; a custom
 *  range parses its `datetime-local` bounds as local time (an empty bound is `NaN`). */
export function windowFor(period: PeriodSelection, now: number): { fromMs: number; toMs: number } {
  if (period.preset === "custom") {
    return { fromMs: parseLocal(period.from), toMs: parseLocal(period.to) };
  }
  return { fromMs: now - PRESET_MS[period.preset], toMs: now };
}

/** The same rules the server enforces (D5), checked first so an invalid custom range never
 *  costs a round-trip and reads as a precise message instead of "Invalid request". */
export function validateWindow(fromMs: number, toMs: number, now: number): WindowError | null {
  if (!Number.isFinite(fromMs) || !Number.isFinite(toMs)) return "jobs.validation.incomplete";
  if (fromMs >= toMs) return "jobs.validation.order";
  if (toMs > now) return "jobs.validation.future";
  if (fromMs < now - RETENTION_DAYS * DAY_MS) return "jobs.validation.retention";
  return null;
}

/** A `datetime-local` value (`yyyy-MM-ddTHH:mm`, local time) for `ms`, floored to the
 *  minute — so a "now" bound never lands after now. */
export function toLocalInput(ms: number): string {
  const d = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function parseLocal(value: string): number {
  // A date-time without an offset is local time (ECMAScript); "" parses to NaN.
  return value ? new Date(value).getTime() : Number.NaN;
}

const optionalMs = (v: bigint | undefined): number | null => (v === undefined ? null : Number(v));

function toRow(j: QueuedJob): QueuedJobRow {
  return {
    id: j.id,
    kind: j.kind,
    channel: j.channel,
    state: j.state,
    attemptsMade: j.attemptsMade,
    attemptsLeft: j.attemptsLeft,
    enqueuedAt: Number(j.enqueuedAtMs),
    nextAttemptAt: optionalMs(j.nextAttemptAtMs),
    startedAt: optionalMs(j.startedAtMs),
    cancellable: j.cancellable,
  };
}

/** Cancellation outcome → toast. Keyed by the enum so a new outcome in the contract is a
 *  compile error here (ts-pattern cannot prove exhaustiveness over a numeric enum). A
 *  refusal is an outcome, not an error: it still reads as a precise, localised message. */
const OUTCOME_TOASTS = {
  [CancelOutcome.UNSPECIFIED]: ["errors.generic", "error"],
  [CancelOutcome.CANCELLED]: ["jobs.outcome.cancelled", "success"],
  [CancelOutcome.GONE]: ["jobs.outcome.gone", "info"],
  [CancelOutcome.RUNNING]: ["jobs.outcome.running", "error"],
  [CancelOutcome.PROTECTED]: ["jobs.outcome.protected", "error"],
} as const satisfies Record<CancelOutcome, readonly [string, ToastVariant]>;

export const useJobsStore = defineStore("jobs", () => {
  const page = ref<Async<JobsPage>>(idle);
  const stats = ref<Async<JobStats>>(idle);
  const kinds = ref<Async<JobKindInfo[]>>(idle);
  /** The job id whose cancellation is in flight (request + the re-read after it). */
  const cancelling = ref<string | null>(null);
  const params = reactive<JobsParams>({ state: JobState.UNSPECIFIED, kind: "", offset: 0 });
  const period = reactive<PeriodSelection>({ preset: "24h", from: "", to: "" });

  async function fetchPage(): Promise<JobsPage> {
    const r = await api().jobs.adminListJobs({
      state: params.state,
      kind: params.kind,
      limit: PAGE_SIZE,
      offset: params.offset,
    });
    return { jobs: r.jobs.map(toRow), total: Number(r.total) };
  }

  /** Load the statistics for the current period and kind. An invalid window never reaches
   *  the server: it lands in the union as the localised validation message. */
  async function loadStats(fold: typeof run = run) {
    const now = Date.now();
    const { fromMs, toMs } = windowFor(period, now);
    const invalid = validateWindow(fromMs, toMs, now);
    if (invalid) {
      stats.value = failure(t(invalid));
      return;
    }
    await fold(stats, async () => {
      const r = await api().jobs.adminGetJobStats({
        window: { fromMs: BigInt(fromMs), toMs: BigInt(toMs) },
        kind: params.kind,
      });
      const q = r.queue;
      const p = r.period;
      return {
        queue: {
          total: Number(q?.total ?? 0n),
          running: Number(q?.running ?? 0n),
          ready: Number(q?.ready ?? 0n),
          scheduled: Number(q?.scheduled ?? 0n),
          retryWait: Number(q?.retryWait ?? 0n),
          blocked: Number(q?.blocked ?? 0n),
          exhausted: Number(q?.exhausted ?? 0n),
        },
        period: {
          completed: Number(p?.completed ?? 0n),
          failedAttempts: Number(p?.failedAttempts ?? 0n),
          deadLettered: Number(p?.deadLettered ?? 0n),
          cancelled: Number(p?.cancelled ?? 0n),
          avgRunMs: optionalMs(p?.avgRunMs),
        },
        historySince: optionalMs(r.historySinceMs),
      } satisfies JobStats;
    });
  }

  /** Re-read the page under the operator. A page emptied by jobs leaving the queue (a
   *  cancellation, or the queue draining between refreshes) steps back to the last page
   *  that still has rows, instead of showing an empty table with a pager. */
  async function rereadPage() {
    const outcome = await reread(page, fetchPage);
    if (outcome.status === "success" && outcome.data.jobs.length === 0 && params.offset > 0) {
      const total = outcome.data.total;
      params.offset = total === 0 ? 0 : Math.floor((total - 1) / PAGE_SIZE) * PAGE_SIZE;
      await reread(page, fetchPage);
    }
  }

  /** Initial load: the first page and the statistics. */
  async function load() {
    await Promise.all([run(page, fetchPage), loadStats(run)]);
  }

  /** The registered kinds, for the kind filter (data-driven, no hard-coded list). */
  async function loadKinds() {
    await run(kinds, async () =>
      (await api().jobs.adminListJobKinds({})).kinds.map(
        (k) => ({ name: k.name, channel: k.channel, cancellable: k.cancellable }) satisfies JobKindInfo,
      ),
    );
  }

  /** Change the state and/or kind filter: back to the first page. The kind also scopes the
   *  statistics, so both are reloaded. */
  async function setFilters(next: { state?: JobState; kind?: string }) {
    Object.assign(params, next, { offset: 0 });
    await Promise.all([run(page, fetchPage), loadStats(run)]);
  }

  /** Move to another page (the statistics do not depend on the page). */
  async function goToPage(offset: number) {
    params.offset = Math.max(0, offset);
    await run(page, fetchPage);
  }

  /** Change the statistics period; only the statistics are reloaded. Switching to `custom`
   *  without bounds pre-fills them with the window the previous preset covered, so the
   *  inputs start from something valid. */
  async function setPeriod(next: Partial<PeriodSelection>) {
    if (next.preset === "custom" && period.preset !== "custom" && next.from === undefined && next.to === undefined) {
      const w = windowFor(period, Date.now());
      period.from = toLocalInput(w.fromMs);
      period.to = toLocalInput(w.toMs);
    }
    Object.assign(period, next);
    await loadStats(reread);
  }

  /** Re-read the page and the statistics without collapsing what is on screen. */
  async function refresh() {
    await Promise.all([rereadPage(), loadStats(reread)]);
  }

  /** Cancel one queued job. Every outcome — including a refusal — is a localised toast;
   *  the page and the statistics are then re-read, whatever the outcome (a GONE job has
   *  left the queue too, and a refused one may have changed state). */
  async function cancel(jobId: string) {
    if (cancelling.value !== null) return;
    cancelling.value = jobId;
    const toasts = useToastsStore();
    try {
      const { outcome } = await api().jobs.adminCancelJob({ jobId });
      // An outcome number this build does not know (a newer server) reads as generic.
      const [key, variant] = OUTCOME_TOASTS[outcome] ?? OUTCOME_TOASTS[CancelOutcome.UNSPECIFIED];
      toasts.push(t(key), variant);
    } catch (e) {
      toasts.error(humanError(e));
    }
    try {
      await Promise.all([rereadPage(), loadStats(reread)]);
    } finally {
      cancelling.value = null;
    }
  }

  return {
    page,
    stats,
    kinds,
    cancelling,
    params,
    period,
    load,
    loadKinds,
    setFilters,
    goToPage,
    setPeriod,
    refresh,
    cancel,
  };
});
