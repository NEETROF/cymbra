import type {
  LinguaPort,
  Rating,
  ReviewCard,
  ReviewEditResult,
  ReviewOptions,
  ReviewSummary,
} from "../analyzer/port.ts";
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
  /**
   * How many times the current card has been shown in this session, this time included: 1 on its
   * first showing, 2 or 3 when a miss brought it back (add-lingua-review-repeat-hint D1). Absent:
   * a first showing.
   */
  showing?: number;
  /** Whether the session under way is practice, whose answers change no card (add-lingua-review-practice). */
  practice?: boolean;
  /** Whether a practice session would hold cards, once nothing more is due: offered only then. */
  practiceAvailable?: boolean;
  /** Whether the reader ended the session before its last card (add-lingua-review-stop-session D3). */
  ended?: boolean;
}

/** The reader's local midnight before `nowSecs`, in epoch seconds: where "today" starts. */
export function localDayStart(nowSecs: number): number {
  const day = new Date(nowSecs * 1000);
  day.setHours(0, 0, 0, 0);
  return Math.floor(day.getTime() / 1000);
}

/** One card per studied language and lemma: the same headword in two languages is two cards. */
function showingKey(card: ReviewCard): string {
  return `${card.language ?? "en"}\u0000${card.headword}`;
}

export class ReviewController {
  private started = false;
  private card: ReviewCard | null = null;
  private languages: StudiedLanguage[] | undefined;
  private summary: ReviewSummary | null = null;
  private moreDue = false;
  /** Each card's showings this session, by language and headword (D1). */
  private showings = new Map<string, number>();
  private practice = false;
  private practiceAvailable = false;
  private ended = false;

  /**
   * `now` supplies epoch-seconds (Date.now()/1000 in production). `record` is an
   * optional daily-stats hook (add-lingua-connected-clients §3): "review" on each graded
   * answer, "learned" on mark-known, each in the card's language; the recorder counts it under the
   * engine's native language (add-lingua-native-language-sync-client D3). Both surfaces inject a
   * storage-backed recorder built on the port; tests omit it. `newWordsPerDay` reads the reader's
   * daily allowance of new words (refine-lingua-review-session D2).
   */
  constructor(
    private readonly port: LinguaPort,
    private readonly now: () => number,
    private readonly record: (event: "review" | "learned", language?: string) => void = () => {},
    private readonly newWordsPerDay: () => Promise<number> = async () => DEFAULT_NEW_WORDS_PER_DAY,
  ) {}

  /**
   * Start a session over what is due now, in `languages` or in every language — or, with
   * `practice`, over the cards already reviewed, due or not, whose answers change no card
   * (add-lingua-review-practice D4).
   */
  async start(languages?: StudiedLanguage[], practice = false): Promise<ReviewView> {
    this.languages = languages;
    this.summary = null;
    this.moreDue = false;
    this.showings.clear();
    this.practiceAvailable = false;
    this.ended = false;
    this.practice = practice;
    await this.port.startReview(this.now(), languages, await this.options(practice));
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

  /**
   * Keep `gloss` as the current card's gloss, the reader's choice (add-lingua-review-translation-fallback
   * D3): the card read again, its new gloss shown, its showing not counted again.
   */
  async keep(gloss: string): Promise<ReviewView> {
    const card = this.card;
    if (!card) return this.view();
    await this.port.for(card.language ?? "en").keepGloss(card.headword, gloss, this.now());
    this.card = await this.port.reviewCurrent();
    return this.view();
  }

  /**
   * Edit the current card — its word or expression and its translation, as the reader saved them
   * (add-lingua-card-edit D10). Not an answer: nothing is counted, the card stays current and is read
   * again, its showings carried to its new form; a card whose new form is known or ignored leaves the
   * session, and the next card arrives. The engine's answer says what it did, or why it refused.
   */
  async edit(form: string, gloss: string): Promise<{ view: ReviewView; result: ReviewEditResult }> {
    const card = this.card;
    if (!card) return { view: this.view(), result: { ok: true, changed: false } };
    const result = await this.port.reviewEdit(form, gloss, this.now());
    if (result.ok && result.retired) {
      await this.next();
    } else if (result.ok && result.changed) {
      this.card = await this.port.reviewCurrent();
      const [from, to] = [showingKey(card), this.card ? showingKey(this.card) : null];
      const shown = this.showings.get(from);
      if (to && to !== from && shown !== undefined) {
        this.showings.delete(from);
        this.showings.set(to, Math.max(shown, this.showings.get(to) ?? 0));
      }
    }
    return { view: this.view(), result };
  }

  /**
   * End the session under way before its last card (add-lingua-review-stop-session D2): the answers
   * given stay, the cards not shown are left as they are — no answer, no showing — a missed card
   * waiting to come back is not asked again and the current card stays unanswered. Then the end, as
   * after the last answer. Outside a session, nothing happens.
   */
  async end(): Promise<ReviewView> {
    if (!this.card) return this.view();
    this.ended = true;
    await this.finish();
    return this.view();
  }

  view(): ReviewView {
    const phase: ReviewPhase = !this.started ? "idle" : this.card ? "reviewing" : "done";
    const showing = this.card ? (this.showings.get(showingKey(this.card)) ?? 1) : 0;
    return {
      phase,
      card: this.card,
      summary: this.summary,
      moreDue: this.moreDue,
      showing,
      practice: this.practice,
      practiceAvailable: this.practiceAvailable,
      ended: this.ended,
    };
  }

  /**
   * The session's limits: SESSION_CARDS cards, the day's allowance from the reader's local midnight —
   * or SESSION_CARDS practice cards, to which no allowance applies.
   */
  private async options(practice = false): Promise<ReviewOptions> {
    if (practice) return { limit: SESSION_CARDS, practice: true };
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
    if (this.card) {
      // A card arriving, never a reveal of the one shown: only here is a showing counted.
      const key = showingKey(this.card);
      this.showings.set(key, (this.showings.get(key) ?? 0) + 1);
      return;
    }
    await this.finish();
  }

  /**
   * The end of a session — its last card answered, or the reader ending it — whose next session
   * replaces the engine's: what is left of its queue is forgotten.
   */
  private async finish(): Promise<void> {
    this.card = null;
    this.summary = await this.port.reviewSummary();
    // After a practice session, practice again — never « Encore 10 » over due cards it did not hold.
    this.moreDue =
      !this.practice && (await this.port.startReview(this.now(), this.languages, await this.options())) > 0;
    // Nothing more due: whether practice would hold cards, measured the same way.
    this.practiceAvailable =
      !this.moreDue && (await this.port.startReview(this.now(), this.languages, await this.options(true))) > 0;
  }
}
