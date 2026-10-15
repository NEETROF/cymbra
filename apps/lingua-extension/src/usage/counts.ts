import type { InterfaceLanguage } from "../i18n/index.ts";

// The anonymous usage counts' state and rules (change: add-lingua-usage-counts, spec
// `lingua-usage-counts`, design D1, D4, D5, D6, D7). Pure: what an installation has recorded, when
// a step is recorded, when the waiting steps may be tried, and what each browser's consent allows.
// The background (usage/host.ts) reads and writes it; nothing here touches a browser API.
//
// What leaves the device is the installation's day (day 0), a browser family, a version and the
// steps — never an identifier, never anything read. A step leaves at the first wake after it
// happens (owner, 2026-10-16: the figures as fast as possible); day 0 stays here 60 days at most.

/** The counts' state on this device: `chrome.storage.local`, never in the backup, never synced. */
export const USAGE_COUNTS_KEY = "cymbra-lingua-usage-counts";

/**
 * The switch on Chromium and Safari (D6): `true` off, `false` on — on Chromium, the reader said
 * yes — and absent unanswered: off on Chromium, on in Safari. Firefox's switch is its own consent.
 * A full reset leaves it as it is.
 */
export const USAGE_COUNTS_OFF_KEY = "cymbra-lingua-usage-counts-off";

/** The six steps, in their wire order (`usage_counts.proto`). */
export const USAGE_STEPS = [
  "installed",
  "set_up",
  "first_analysis",
  "returned_day_1",
  "returned_days_7_13",
  "returned_days_30_59",
] as const;
export type UsageStep = (typeof USAGE_STEPS)[number];

/** The last day a step is recorded and sent on: at day 60 the installation is forgotten. */
export const LAST_DAY = 59;

/** After a failed report, the next attempt waits this long (D5). */
export const RETRY_AFTER_MS = 60 * 60 * 1000;

/** The browser family, from the build. */
export type BrowserFamily = "chromium" | "firefox" | "safari";

/** What a variant's first-run sentence does (D7): `ask` on Chromium, `on` elsewhere. */
export type CountsMode = "on" | "ask";

export interface UsageState {
  /**
   * The local day number the installation was made on, or null: nothing is ever recorded. It is
   * what a report is filed under on the server.
   */
  day0: number | null;
  /** Steps reported (or refused as malformed): never recorded again. */
  sent: UsageStep[];
  /** Steps recorded and not yet reported: they leave at the next wake the consent allows. */
  waiting: UsageStep[];
  /** Whether the first-run sentence was shown (Safari's gate; on Chromium, with the answer). */
  noticeShown: boolean;
  /** When the last report failed (epoch ms), or null. */
  lastFailure: number | null;
  /**
   * Whether this installation was made by a release that counts (`onInstalled` with `install`).
   * One that predates the counts arrived by an update and never has it: it is never asked,
   * counted or sent to the leaving page (owner decision 2026-10-15). Kept when everything else
   * is forgotten: it is a fact about the installation, not a count.
   */
  countable: boolean;
}

export const EMPTY_STATE: UsageState = {
  day0: null,
  sent: [],
  waiting: [],
  noticeShown: false,
  lastFailure: null,
  countable: false,
};

const isStep = (s: unknown): s is UsageStep => (USAGE_STEPS as readonly unknown[]).includes(s);
const isInt = (n: unknown): n is number => typeof n === "number" && Number.isInteger(n);

/** The stored state, or the empty one: a malformed value never throws. */
export function parseState(raw: unknown): UsageState {
  if (!raw || typeof raw !== "object") return { ...EMPTY_STATE };
  const r = raw as Record<string, unknown>;
  const steps = (v: unknown): UsageStep[] => (Array.isArray(v) ? [...new Set(v.filter(isStep))] : []);
  return {
    day0: isInt(r.day0) ? r.day0 : null,
    sent: steps(r.sent),
    waiting: steps(r.waiting),
    noticeShown: r.noticeShown === true,
    lastFailure: isInt(r.lastFailure) ? r.lastFailure : null,
    countable: r.countable === true,
  };
}

/**
 * The local calendar day of `date` as a day number (days since 1970-01-01): "the next day" is the
 * reader's next day, not a UTC rollover in the evening (D1). A day with a daylight-saving change
 * is still one day.
 */
export function localDay(date: Date): number {
  return Math.floor(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()) / 86_400_000);
}

/** The first day this installation keeps nothing: day 60 (D4). */
export function forgetsOn(day0: number): number {
  return day0 + LAST_DAY + 1;
}

/** A new installation on `today`: day 0, and « installed » waiting. */
export function install(state: UsageState, today: number): UsageState {
  return {
    ...state,
    day0: today,
    sent: [],
    waiting: ["installed"],
    lastFailure: null,
    countable: true,
  };
}

/**
 * Everything this installation recorded, forgotten: it never reports again. The notice stays
 * shown, and an installation made by a counting release stays one.
 */
export function forget(state: UsageState): UsageState {
  return { ...EMPTY_STATE, noticeShown: state.noticeShown, countable: state.countable };
}

/**
 * `state` as it is on `today` (D4): from day 60 the device forgets day 0 and every step it still
 * holds, sent or not. Every path that changes the state goes through this first.
 */
export function expire(state: UsageState, today: number): UsageState {
  if (state.day0 === null || today < forgetsOn(state.day0)) return state;
  return forget(state);
}

/** Record `step` once, if this installation still records on `today` (days 0 to 59). */
export function record(state: UsageState, step: UsageStep, today: number): UsageState {
  const s = expire(state, today);
  if (s.day0 === null || today < s.day0) return s; // a clock set back before day 0 records nothing
  if (today - s.day0 > LAST_DAY) return s;
  if (s.sent.includes(step) || s.waiting.includes(step)) return s;
  return { ...s, waiting: [...s.waiting, step] };
}

/** The return a use on `day` of the installation counts as, if any (D1). */
export function returnOn(day: number): UsageStep | null {
  if (day === 1) return "returned_day_1";
  if (day >= 7 && day <= 13) return "returned_days_7_13";
  if (day >= 30 && day <= LAST_DAY) return "returned_days_30_59";
  return null;
}

/** A use — a page analysed in a studied language, or a Lingua surface opened — on `today`. */
export function recordUse(state: UsageState, today: number): UsageState {
  const s = expire(state, today);
  if (s.day0 === null) return s;
  const step = returnOn(today - s.day0);
  return step ? record(s, step, today) : s;
}

/** A page analysed in a studied language: the first analysis, and a use. */
export function recordAnalysis(state: UsageState, today: number): UsageState {
  return recordUse(record(state, "first_analysis", today), today);
}

/**
 * Whether a report may be tried at `now` (epoch ms): a step waits, and no failure happened in the
 * last hour. A failure the clock now puts in the future — a clock set back — does not hold the
 * steps until the clock catches up.
 */
export function due(state: UsageState, now: number): boolean {
  if (state.waiting.length === 0) return false;
  const last = state.lastFailure;
  return last === null || now - last >= RETRY_AFTER_MS || now < last;
}

/** How a report ended (usage/client.ts). */
export type ReportOutcome = "accepted" | "invalid" | "retry";

/**
 * The state after reporting `steps`: accepted or refused as malformed, they are done — never sent
 * again, so a bug cannot loop; otherwise they wait, and the next try waits an hour (D5).
 *
 * Delivery is at least once: if the background is stopped after the server counted a report and
 * before this state is saved, the steps are still waiting and are sent again at the next wake.
 */
export function afterReport(
  state: UsageState,
  steps: readonly UsageStep[],
  outcome: ReportOutcome,
  now: number,
): UsageState {
  if (outcome === "retry") return { ...state, lastFailure: now };
  return {
    ...state,
    sent: [...new Set([...state.sent, ...steps])],
    waiting: state.waiting.filter((s) => !steps.includes(s)),
    lastFailure: null,
  };
}

/**
 * What the reader's choice is, per browser (D6): Firefox's built-in data-collection consent — which
 * Firefox before 140 (Android 142) does not have — or the switch's key elsewhere.
 */
export type Consent =
  { kind: "grant"; supported: boolean; granted: boolean } | { kind: "switch"; off: boolean | undefined };

/** Whether steps may be recorded at all (held, on Chromium, until the reader answers). */
export function recording(consent: Consent): boolean {
  return consent.kind === "grant" ? consent.supported && consent.granted : consent.off !== true;
}

/** Whether the waiting steps may leave the device (D6, D7). */
export function sending(mode: CountsMode, consent: Consent, state: UsageState): boolean {
  if (consent.kind === "grant") return consent.supported && consent.granted;
  if (mode === "ask") return consent.off === false;
  return consent.off !== true && state.noticeShown;
}

/** Whether counting is on — what the switch shows, and what the leaving page follows. */
export function countingOn(mode: CountsMode, consent: Consent): boolean {
  if (consent.kind === "grant") return consent.supported && consent.granted;
  return mode === "ask" ? consent.off === false : consent.off !== true;
}

/**
 * Whether the leaving page is registered (D12): counting is on, for an installation made by a
 * release that counts — never for one that predates the counts, whose steps were never counted
 * (owner decision 2026-10-15).
 */
export function leavingPageOn(mode: CountsMode, consent: Consent, state: UsageState): boolean {
  return state.countable && countingOn(mode, consent);
}

/** What the first-run sentence must do now: nothing, inform, or ask (two buttons). */
export type NoticeKind = "none" | "inform" | "ask";

/**
 * Nothing for an installation that has nothing to count — one that predates the counts (owner
 * decision 2026-10-15: never asked, on any surface), or whose days have run out; otherwise, on a
 * build that asks, the question until it is answered, and elsewhere the sentence once.
 */
export function noticeKind(mode: CountsMode, consent: Consent, state: UsageState): NoticeKind {
  if (!state.countable || state.day0 === null) return "none";
  if (consent.kind === "switch" && mode === "ask") return consent.off === undefined ? "ask" : "none";
  return countingOn(mode, consent) && !state.noticeShown ? "inform" : "none";
}

/** What Settings → Data shows: whether the switch is there (not on an old Firefox), and its position. */
export interface SwitchView {
  shown: boolean;
  on: boolean;
}

export function switchView(mode: CountsMode, consent: Consent): SwitchView {
  return { shown: consent.kind === "switch" || consent.supported, on: countingOn(mode, consent) };
}

/** `major.minor` of the extension's version (`1.9.0` → `1.9`). */
export function shortVersion(version: string): string {
  const [major = "0", minor = "0"] = version.split(".");
  return `${major}.${minor}`;
}

/** The site's leaving page in each interface language (D12): pinned routes of the site. */
export const LEAVING_PAGES: Record<InterfaceLanguage, string> = {
  fr: "https://cymbra.app/lingua/au-revoir/",
  en: "https://cymbra.app/en/lingua/goodbye/",
  es: "https://cymbra.app/es/lingua/adios/",
};

/** The uninstall address while counting is on: the leaving page, with the browser and version only. */
export function leavingUrl(language: InterfaceLanguage, browser: BrowserFamily, version: string): string {
  const query = new URLSearchParams({ b: browser, v: shortVersion(version) });
  return `${LEAVING_PAGES[language]}?${query.toString()}`;
}
