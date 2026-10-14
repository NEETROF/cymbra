import type { AsyncStorageArea } from "../state/storage.ts";

// The rating request at the end of a review session (add-lingua-comeback-hints D10), decided here,
// pure: the review page counts each finished session and asks, the view renders. From the fifth
// session finished on this device, on the third distinct day of review or later; once shown it
// counts as made; « Plus tard », or no answer, asks again after ten more sessions; twice at most,
// ever; « Noter Cymbra Lingua » and « Ne plus demander » end it. Nothing here depends on how a
// session went. Kept on this device only: not synced, not in the backup, not erased with the
// reader's data (it describes no reading).

/** Where the request's state is kept, in chrome.storage.local. */
export const RATING_KEY = "cymbra-lingua-rating-request";

/** The finished sessions before the first request (Apple's sample waits for four). */
export const RATING_FIRST_SESSION = 5;
/** The distinct days of review the first request waits for: one evening's binge does not qualify. */
export const RATING_FIRST_DAY = 3;
/** The finished sessions between a request left unanswered, or put off, and the next. */
export const RATING_AGAIN_AFTER = 10;
/** The most times the request is shown on a device, ever. */
export const RATING_MAX_SHOWINGS = 2;

/** How the reader answered: rated, or asked never to be asked again. */
export type RatingAnswer = "rated" | "never";

export interface RatingState {
  /** Sessions finished on this device (an end that said « Séance terminée »). */
  sessions: number;
  /** Distinct local days on which a session was finished. */
  days: number;
  /** The last of those days, `YYYY-MM-DD` in local time; null before the first. */
  lastDay: string | null;
  /** Times the request was shown. */
  shown: number;
  /** The session count when it was last shown. */
  shownAt: number;
  /** The answer that ended it, or null while it may still be asked. */
  answer: RatingAnswer | null;
}

export const INITIAL_RATING: RatingState = { sessions: 0, days: 0, lastDay: null, shown: 0, shownAt: 0, answer: null };

/**
 * Whether a build's review asks for a rating at all (D10, D11): Chromium and Firefox. The Safari
 * extension ships inside the app App Review examines, and Apple disallows custom review prompts;
 * the Cymbra Lingua app asks through StoreKit's own prompt instead.
 */
export function ratingOffered(target: string, app: boolean): boolean {
  return !app && target !== "safari";
}

/** What the reader chose: rate, later, or never again. */
export type RatingChoice = "rate" | "later" | "never";

const count = (v: unknown): number => (typeof v === "number" && Number.isInteger(v) && v >= 0 ? v : 0);

/** A stored state, made safe: anything malformed reads as never asked. */
export function ratingStateOf(value: unknown): RatingState {
  const v = (value ?? {}) as Record<string, unknown>;
  return {
    sessions: count(v.sessions),
    days: count(v.days),
    lastDay: typeof v.lastDay === "string" ? v.lastDay : null,
    shown: count(v.shown),
    shownAt: count(v.shownAt),
    answer: v.answer === "rated" || v.answer === "never" ? v.answer : null,
  };
}

/** The reader's local day at `nowSecs`, `YYYY-MM-DD`. */
export function localDay(nowSecs: number): string {
  const d = new Date(nowSecs * 1000);
  const pad = (n: number): string => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

/** The state after a session finished on `day`. */
export function afterSession(state: RatingState, day: string): RatingState {
  const newDay = day !== state.lastDay;
  return { ...state, sessions: state.sessions + 1, days: state.days + (newDay ? 1 : 0), lastDay: day };
}

/** Whether the end of the session just counted asks for a rating. */
export function ratingDue(state: RatingState): boolean {
  if (state.answer !== null || state.shown >= RATING_MAX_SHOWINGS) return false;
  if (state.sessions < RATING_FIRST_SESSION || state.days < RATING_FIRST_DAY) return false;
  return state.shown === 0 || state.sessions - state.shownAt >= RATING_AGAIN_AFTER;
}

/** The state once the request is shown: it counts as made, answered or not. */
export function markShown(state: RatingState): RatingState {
  return { ...state, shown: state.shown + 1, shownAt: state.sessions };
}

/** The state after the reader's answer: « Plus tard » leaves it as the showing left it. */
export function answer(state: RatingState, choice: RatingChoice): RatingState {
  if (choice === "later") return state;
  return { ...state, answer: choice === "rate" ? "rated" : "never" };
}

export async function loadRating(area: AsyncStorageArea): Promise<RatingState> {
  return ratingStateOf((await area.get(RATING_KEY))[RATING_KEY]);
}

export async function saveRating(area: AsyncStorageArea, state: RatingState): Promise<void> {
  await area.set({ [RATING_KEY]: state });
}
