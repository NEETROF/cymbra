import type { LinguaPort, Rating, ReviewCard, ReviewOptions, ReviewSummary } from "../analyzer/port.ts";
import type { StudiedLanguage } from "../analyzer/types.ts";
import { DEFAULT_NEW_WORDS_PER_DAY } from "../state/storage.ts";

// The review session — one module both surfaces (the side panel and the injected
// drawer) render (design D1: two rendering hosts over one logic). It drives the
// engine's FSRS session behind the LinguaPort and exposes a small view model; the
// clock is injected so it is deterministic under test. The answer is hidden until
// `reveal()`, matching lingua-core's session.
//
// A session is short (refine-lingua-review-session): at most SESSION_CARDS cards, the most
// fragile first, new words within the day's allowance; a missed card comes back in the session
// and only its first answer counts as a review; the end says what the session did.

/** How many distinct cards a session holds (refine-lingua-review-session D1). */
export const SESSION_CARDS = 10;

export type ReviewPhase = "idle" | "reviewing" | "done";

export interface ReviewView {
  phase: ReviewPhase;
  card: ReviewCard | null;
  /** What the session did, once it is done (D8). */
  summary: ReviewSummary | null;
  /** Whether another session would hold cards now: « Encore 10 » is offered only then. */
  moreDue: boolean;
}

/** The reader's local midnight before `nowSecs`, in epoch seconds: where "today" starts. */
export function localDayStart(nowSecs: number): number {
  const day = new Date(nowSecs * 1000);
  day.setHours(0, 0, 0, 0);
  return Math.floor(day.getTime() / 1000);
}

export class ReviewController {
  private started = false;
  private card: ReviewCard | null = null;
  private languages: StudiedLanguage[] | undefined;
  private summary: ReviewSummary | null = null;
  private moreDue = false;

  /**
   * `now` supplies epoch-seconds (Date.now()/1000 in production). `record` is an
   * optional daily-stats hook (add-lingua-connected-clients §3): "review" on each graded
   * answer, "learned" on mark-known. Both surfaces inject a storage-backed recorder; tests
   * omit it. `newWordsPerDay` reads the reader's daily allowance of new words (D2).
   */
  constructor(
    private readonly port: LinguaPort,
    private readonly now: () => number,
    private readonly record: (event: "review" | "learned", language?: string) => void = () => {},
    private readonly newWordsPerDay: () => Promise<number> = async () => DEFAULT_NEW_WORDS_PER_DAY,
  ) {}

  /** Start a session over what is due now, in `languages` or in every language. */
  async start(languages?: StudiedLanguage[]): Promise<ReviewView> {
    this.languages = languages;
    this.summary = null;
    this.moreDue = false;
    await this.port.startReview(this.now(), languages, await this.options());
    this.started = true;
    await this.next();
    return this.view();
  }

  /** Reveal the current card's answer. */
  async reveal(): Promise<ReviewView> {
    await this.port.reviewReveal();
    this.card = await this.port.reviewCurrent();
    return this.view();
  }

  /** Answer the current card and move to the next. Only a first answer counts as a review. */
  async grade(rating: Rating): Promise<ReviewView> {
    const language = this.card?.language ?? "en"; // counted in the card's language
    if (await this.port.reviewGrade(rating, this.now())) this.record("review", language);
    await this.next();
    return this.view();
  }

  /** Mark the current card known (retire it) and move to the next. */
  async markKnown(): Promise<ReviewView> {
    const language = this.card?.language ?? "en";
    await this.port.reviewMarkKnown(this.now());
    this.record("learned", language);
    await this.next();
    return this.view();
  }

  /** Hide the current card's word from review — « Ne plus me le montrer » — and move on. */
  async ignore(): Promise<ReviewView> {
    await this.port.reviewIgnore(this.now());
    await this.next();
    return this.view();
  }

  view(): ReviewView {
    const phase: ReviewPhase = !this.started ? "idle" : this.card ? "reviewing" : "done";
    return { phase, card: this.card, summary: this.summary, moreDue: this.moreDue };
  }

  /** The session's limits: SESSION_CARDS cards, the day's allowance from the reader's local midnight. */
  private async options(): Promise<ReviewOptions> {
    return {
      limit: SESSION_CARDS,
      newPerDay: await this.newWordsPerDay(),
      dayStart: localDayStart(this.now()),
    };
  }

  /**
   * Read the next card; at the end, the summary, then whether another session would hold cards.
   * Counting due cards would not tell: new words beyond today's allowance are due and still wait.
   * So the next session is prepared and measured — the engine holds nothing else until the reader
   * answers « Encore 10 », which starts it again.
   */
  private async next(): Promise<void> {
    this.card = await this.port.reviewCurrent();
    if (this.card) return;
    this.summary = await this.port.reviewSummary();
    this.moreDue = (await this.port.startReview(this.now(), this.languages, await this.options())) > 0;
  }
}
