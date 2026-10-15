import type { InterfaceLanguage } from "../i18n/index.ts";

// The anonymous usage counts' state and rules (change: add-lingua-usage-counts, spec
// `lingua-usage-counts`, design D1, D4, D6, D7). Pure: what an installation has recorded, when a
// step is recorded, when the waiting steps may leave, and what each browser's consent allows. The
// background (usage/host.ts) reads and writes it; nothing here touches a browser API.
//
// What leaves the device is a week, a browser family, a version and the steps — never day 0, which
// stays here for 60 days at most, never an identifier, never anything read.

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

/** The last day a step is recorded on: after it, the installation records nothing more. */
export const LAST_DAY = 59;

/** After a failed report, the next attempt waits this long (D5). */
export const RETRY_AFTER_MS = 60 * 60 * 1000;

/** The browser family, from the build. */
export type BrowserFamily = "chromium" | "firefox" | "safari";

/** What a variant's first-run sentence does (D7): `ask` on Chromium, `on` elsewhere. */
export type CountsMode = "on" | "ask";

/**
 * A step recorded and not yet sent, with the day it may leave on: the Monday after the week it
 * happened in (owner decision 2026-10-15). Sending it the following week means the moment a
 * report arrives says nothing about the day the step happened. Only that Monday is kept, never
 * the step's day.
 */
export interface HeldStep {
  step: UsageStep;
  /** The day number of the Monday from which the step may leave. */
  from: number;
}

export interface UsageState {
  /** The local day number the installation was made on, or null: nothing is ever recorded. */
  day0: number | null;
  /** The day number of the Monday that starts day 0's local week — what leaves the device. */
  week: number | null;
  /** Steps reported (or refused as malformed): never recorded again. */
  sent: UsageStep[];
  /** Steps recorded and not yet reported, each with the Monday it may leave from. */
  waiting: HeldStep[];
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
  week: null,
  sent: [],
  waiting: [],
  noticeShown: false,
  lastFailure: null,
  countable: false,
};

const isStep = (s: unknown): s is UsageStep => (USAGE_STEPS as readonly unknown[]).includes(s);
const isInt = (n: unknown): n is number => typeof n === "number" && Number.isInteger(n);

function heldSteps(v: unknown): HeldStep[] {
  if (!Array.isArray(v)) return [];
  const out: HeldStep[] = [];
  for (const item of v) {
    const h = item as Partial<HeldStep> | null;
    if (h && isStep(h.step) && isInt(h.from) && !out.some((o) => o.step === h.step)) {
      out.push({ step: h.step, from: h.from });
    }
  }
  return out;
}

/** The stored state, or the empty one: a malformed value never throws. */
export function parseState(raw: unknown): UsageState {
  if (!raw || typeof raw !== "object") return { ...EMPTY_STATE };
  const r = raw as Record<string, unknown>;
  const steps = (v: unknown): UsageStep[] => (Array.isArray(v) ? [...new Set(v.filter(isStep))] : []);
  const day0 = isInt(r.day0) ? r.day0 : null;
  return {
    day0,
    week: day0 === null ? null : mondayOf(day0),
    sent: steps(r.sent),
    waiting: heldSteps(r.waiting),
    noticeShown: r.noticeShown === true,
    lastFailure: isInt(r.lastFailure) ? r.lastFailure : null,
    countable: r.countable === true,
  };
}

/**
 * The local calendar day of `date` as a day number (days since 1970-01-01): "the next day" is the
 * reader's next day, not a UTC rollover in the evening (D1).
 */
export function localDay(date: Date): number {
  return Math.floor(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()) / 86_400_000);
}

/** The Monday that starts the week holding `day` (1970-01-01 was a Thursday). */
export function mondayOf(day: number): number {
  return day - ((((day - 4) % 7) + 7) % 7);
}

/** The Monday from which a step that happened on `day` may leave: the one after its week. */
export function leavesFrom(day: number): number {
  return mondayOf(day) + 7;
}

/**
 * The first day this installation keeps nothing: the Monday two weeks after the one that starts
 * day 59's week. The last steps are recorded in day 59's week and leave the week after it, which
 * gives each of them a whole week for a wake to send it; then everything goes (73 days at most).
 */
export function forgetsOn(day0: number): number {
  return mondayOf(day0 + LAST_DAY) + 14;
}

/** A new installation on `today`: day 0, and « installed » waiting for the next Monday. */
export function install(state: UsageState, today: number): UsageState {
  return {
    ...state,
    day0: today,
    week: mondayOf(today),
    sent: [],
    waiting: [{ step: "installed", from: leavesFrom(today) }],
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
 * `state` as it is on `today` (D4): from {@link forgetsOn} the device forgets day 0 and every
 * step it still holds. Every path that changes the state goes through this first.
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
  if (s.sent.includes(step) || s.waiting.some((w) => w.step === step)) return s;
  return { ...s, waiting: [...s.waiting, { step, from: leavesFrom(today) }] };
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
 * The steps that may leave on `today`: those whose Monday has come — never one in the week it
 * happened in, so a report's arrival cannot tell the day.
 */
export function ready(state: UsageState, today: number): UsageStep[] {
  return state.waiting.filter((w) => w.from <= today).map((w) => w.step);
}

/**
 * Whether a report may be tried on `today` at `now` (epoch ms): a step is ready, and an hour has
 * passed since a failure.
 */
export function due(state: UsageState, today: number, now: number): boolean {
  return ready(state, today).length > 0 && (state.lastFailure === null || now - state.lastFailure >= RETRY_AFTER_MS);
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
    waiting: state.waiting.filter((w) => !steps.includes(w.step)),
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
