import type { LemmaStatus, PageAnalysis } from "./types.ts";

// The AnalyzerPort seam (design D2). The content script consumes analysis exclusively
// through this interface, never touching the WASM module directly. In this Chromium
// change the implementation is the WASM engine instantiated in the content script (see
// engine.ts); the later Firefox (WASM in the event page) and Safari (nativeMessaging)
// variants provide a different implementation of the SAME interface, with no change to
// the reading code. Tests inject a fake.

export interface AnalyzerPort {
  /** Analyse a batch of blocks (studied-language text), returning classified tokens. */
  analyse(blocks: string[]): Promise<PageAnalysis>;
  /** Set the calibration threshold ("I know the N most common words"). */
  setCalibration(threshold: number): Promise<void>;
  /** Set (or, with null, clear) an explicit status for a dictionary form. */
  setStatus(lemma: string, status: LemmaStatus | null): Promise<void>;
  /** The native-language gloss for a form, if the pack carries one. */
  gloss(lemma: string): Promise<string | undefined>;
}

/** An FSRS grade. */
export type Rating = "again" | "hard" | "good" | "easy";

/** A card being reviewed (the engine's view model). */
export interface ReviewCard {
  headword: string;
  surface: string;
  sentence: string;
  gloss: string | null;
  revealed: boolean;
  remaining: number;
}

/** The context captured when a form is added to the deck. */
export interface NewCard {
  lemma: string;
  surface: string;
  sentence: string;
  url: string;
  gloss: string | null;
  capturedAt: number;
}

// The full engine surface the review change drives (add-lingua-extension-review): the
// reading AnalyzerPort plus deck building, an FSRS review session, lossless
// backup/restore and pack attributions. The engine holds the whole lingua-core
// LinguaState; persistence is backup → chrome.storage.local → restore in every context.
export interface LinguaPort extends AnalyzerPort {
  /** The current calibration threshold. */
  calibration(): Promise<number>;
  /** How many forms are explicitly marked (any status). */
  trackedCount(): Promise<number>;
  /** Add (or replace) a deck card and mark its form learning. */
  addCard(card: NewCard): Promise<void>;
  /** Total cards in the deck. */
  deckCount(): Promise<number>;
  /** Cards due at `now` (epoch seconds). */
  dueCount(now: number): Promise<number>;
  /** Start a review session over everything due at `now`; returns the count. */
  startReview(now: number): Promise<number>;
  /** The current card, or null when the session is finished / not started. */
  reviewCurrent(): Promise<ReviewCard | null>;
  /** Reveal the current card's answer. */
  reviewReveal(): Promise<void>;
  /** Grade the current card and advance. */
  reviewGrade(rating: Rating, now: number): Promise<void>;
  /** Mark the current card known (retire it) and advance. */
  reviewMarkKnown(now: number): Promise<void>;
  /** The whole state as a lossless, versioned backup string. */
  backup(): Promise<string>;
  /** Replace the whole state from a backup string (throws on an unknown version). */
  restore(json: string): Promise<void>;
  /** Reset the whole state to empty defaults (a full reset). */
  reset(): Promise<void>;
  /** The pack's bundled attribution NOTICE. */
  notice(): Promise<string>;
  /** The pack's source licences. */
  licences(): Promise<string[]>;
}
