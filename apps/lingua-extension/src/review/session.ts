import type { LinguaPort, Rating, ReviewCard } from "../analyzer/port.ts";

// The review session — one module both surfaces (the side panel and the injected
// drawer) render (design D1: two rendering hosts over one logic). It drives the
// engine's FSRS session behind the LinguaPort and exposes a small view model; the
// clock is injected so it is deterministic under test. The answer is hidden until
// `reveal()`, matching lingua-core's session.

export type ReviewPhase = "idle" | "reviewing" | "done";

export interface ReviewView {
  phase: ReviewPhase;
  card: ReviewCard | null;
}

export class ReviewController {
  private started = false;
  private card: ReviewCard | null = null;

  /**
   * `now` supplies epoch-seconds (Date.now()/1000 in production). `record` is an
   * optional daily-stats hook (add-lingua-connected-clients §3): "review" on each grade,
   * "learned" on mark-known. Both surfaces inject a storage-backed recorder; tests omit it.
   */
  constructor(
    private readonly port: LinguaPort,
    private readonly now: () => number,
    private readonly record: (event: "review" | "learned") => void = () => {},
  ) {}

  /** Start a session over everything due now. */
  async start(): Promise<ReviewView> {
    await this.port.startReview(this.now());
    this.started = true;
    this.card = await this.port.reviewCurrent();
    return this.view();
  }

  /** Reveal the current card's answer. */
  async reveal(): Promise<ReviewView> {
    await this.port.reviewReveal();
    this.card = await this.port.reviewCurrent();
    return this.view();
  }

  /** Grade the current card and move to the next. */
  async grade(rating: Rating): Promise<ReviewView> {
    await this.port.reviewGrade(rating, this.now());
    this.record("review");
    this.card = await this.port.reviewCurrent();
    return this.view();
  }

  /** Mark the current card known (retire it) and move to the next. */
  async markKnown(): Promise<ReviewView> {
    await this.port.reviewMarkKnown(this.now());
    this.record("learned");
    this.card = await this.port.reviewCurrent();
    return this.view();
  }

  view(): ReviewView {
    const phase: ReviewPhase = !this.started ? "idle" : this.card ? "reviewing" : "done";
    return { phase, card: this.card };
  }
}
