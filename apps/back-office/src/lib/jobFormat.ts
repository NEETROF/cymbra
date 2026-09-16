// Number, duration and time formatting shared by the Jobs screen and its tables. Pure:
// the locale and `t` are passed in, so the helpers are testable without a component.

type Translate = (key: string, values?: Record<string, unknown>) => string;

/** A count in the locale's digits; `—` when unknown. */
export function formatCount(v: number | undefined, locale: string): string {
  return v === undefined ? "—" : v.toLocaleString(locale);
}

/** A compact human duration: ms under a second, seconds under a minute, then minutes. */
export function formatDuration(ms: number | null | undefined, t: Translate, locale: string): string {
  if (ms === null || ms === undefined) return "—";
  const n = (v: number) => v.toLocaleString(locale, { maximumFractionDigits: 1 });
  if (ms < 1000) return t("jobs.duration.ms", { n: n(Math.round(ms)) });
  if (ms < 60_000) return t("jobs.duration.s", { n: n(ms / 1000) });
  return t("jobs.duration.min", { n: n(ms / 60_000) });
}

/** "3 minutes ago" / "in 20 seconds"; `—` when unset. The absolute time goes in a title. */
export function formatRelative(ms: number | null, now: number, locale: string): string {
  if (ms === null) return "—";
  const diff = ms - now;
  const abs = Math.abs(diff);
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  if (abs < 60_000) return rtf.format(Math.round(diff / 1000), "second");
  if (abs < 3_600_000) return rtf.format(Math.round(diff / 60_000), "minute");
  if (abs < 86_400_000) return rtf.format(Math.round(diff / 3_600_000), "hour");
  return rtf.format(Math.round(diff / 86_400_000), "day");
}

/** The full local date and time, for a title attribute. */
export function formatAbsolute(ms: number, locale: string): string {
  return new Date(ms).toLocaleString(locale);
}
