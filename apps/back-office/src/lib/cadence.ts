import { match } from "ts-pattern";

// A job kind's cadence, as the Jobs console shows it (change: add-jobs-console-history).
// The server returns each schedule's raw cron and timezone — the table an operator may
// retune at run time — and this module turns them into a label. Only the two shapes the
// seeded schedules use get words; any other expression is shown as written, so a new
// cron never needs a contract or a code change to be displayed truthfully.

/** One schedule of a kind, as the store holds it. */
export interface ScheduleInfo {
  name: string;
  cron: string;
  timezone: string;
  enabled: boolean;
}

export type Cadence =
  | { readonly type: "hourly"; readonly minute: number }
  | { readonly type: "daily"; readonly hour: number; readonly minute: number; readonly timezone: string }
  | { readonly type: "custom"; readonly cron: string; readonly timezone: string };

/** What a kind's schedules amount to. `scheduled` lists the ENABLED ones only. */
export type KindCadence =
  | { readonly type: "onDemand" }
  | { readonly type: "paused" }
  | { readonly type: "scheduled"; readonly cadences: readonly Cadence[] };

const INT = /^\d{1,2}$/;

function field(value: string, max: number): number | null {
  if (!INT.test(value)) return null;
  const n = Number(value);
  return n <= max ? n : null;
}

/** Read a cron expression. The scheduler also accepts a leading seconds field; a `0`
 *  there changes nothing about the cadence, so it is dropped before matching. */
export function parseCron(cron: string, timezone: string): Cadence {
  const custom: Cadence = { type: "custom", cron, timezone };
  let parts = cron.trim().split(/\s+/);
  if (parts.length === 6 && parts[0] === "0") parts = parts.slice(1);
  if (parts.length !== 5) return custom;
  const [m, h, dom, mon, dow] = parts;
  if (dom !== "*" || mon !== "*" || dow !== "*") return custom;
  const minute = field(m, 59);
  if (minute === null) return custom;
  if (h === "*") return { type: "hourly", minute };
  const hour = field(h, 23);
  return hour === null ? custom : { type: "daily", hour, minute, timezone };
}

/** A kind with no schedule runs on demand; one whose schedules are all disabled is paused. */
export function kindCadence(schedules: readonly ScheduleInfo[]): KindCadence {
  if (schedules.length === 0) return { type: "onDemand" };
  const enabled = schedules.filter((s) => s.enabled);
  if (enabled.length === 0) return { type: "paused" };
  return { type: "scheduled", cadences: enabled.map((s) => parseCron(s.cron, s.timezone)) };
}

type Translate = (key: string, values?: Record<string, unknown>) => string;

const two = (n: number) => String(n).padStart(2, "0");

/** One cadence in words. An hourly cron carries no timezone: minutes past the hour are the
 *  same everywhere that matters here. */
export function cadenceText(cadence: Cadence, t: Translate): string {
  return match(cadence)
    .with({ type: "hourly" }, ({ minute }) => t("jobs.cadence.hourly", { mm: two(minute) }))
    .with({ type: "daily" }, ({ hour, minute, timezone }) =>
      t("jobs.cadence.daily", { hh: two(hour), mm: two(minute), tz: timezone }),
    )
    .with({ type: "custom" }, ({ cron, timezone }) => t("jobs.cadence.custom", { cron, tz: timezone }))
    .exhaustive();
}

/** The label a kind carries wherever it is shown. `t` is vue-i18n's, injected so this
 *  stays a pure function. */
export function kindCadenceLabel(cadence: KindCadence, t: Translate): string {
  return match(cadence)
    .with({ type: "onDemand" }, () => t("jobs.cadence.onDemand"))
    .with({ type: "paused" }, () => t("jobs.cadence.paused"))
    .with({ type: "scheduled" }, ({ cadences }) =>
      t("jobs.cadence.scheduled", { cadence: cadences.map((c) => cadenceText(c, t)).join(", ") }),
    )
    .exhaustive();
}
