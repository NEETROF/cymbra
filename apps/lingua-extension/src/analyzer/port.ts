import type { CefrLevel, LemmaStatus, LevelRow, PageAnalysis, SeedOrder } from "./types.ts";

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

// Sync ops (add-lingua-connected-clients §2), in the engine's own shape (snake_case,
// matching the KnownWordsService / DeckService message fields). The sync engine maps
// these to/from the Connect-ES request/response messages (adding a device_id, mapping
// `updated_at`↔`client_ts`). All timestamps are epoch millis on the wire.

/** One exported word-status op, from `exportStatusOps`. */
export interface StatusOp {
  language: string;
  lemma: string;
  status: string; // "known" | "learning" | "ignored"
  provenance: string; // "manual" | "srs" | "import"
  updated_at: number; // epoch millis
}

/** An incoming resolved status change to apply (from a pull). */
export interface StatusChangeIn {
  language: string;
  lemma: string;
  status: string; // "known" | "learning" | "ignored" | "cleared"
  updated_at: number; // epoch millis
}

/** One exported whole-card op, from `exportCardOps` (also the apply shape). */
export interface CardOp {
  client_id: string;
  language: string;
  lemma: string;
  surface_form: string;
  source_sentence: string;
  source: string;
  gloss: string;
  fsrs_state: string; // opaque JSON
  deleted: boolean;
  client_ts: number; // epoch millis
  device_id: string;
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
  /** Reset the whole state to empty defaults (a full reset: statuses, deck, exposure). */
  reset(): Promise<void>;
  /** Partial reset: clear statuses, calibration and the declared level, but KEEP the deck and exposure. */
  resetStatuses(): Promise<void>;
  /** The pack's bundled attribution NOTICE. */
  notice(): Promise<string>;
  /** The pack's source licences. */
  licences(): Promise<string[]>;

  // Sync (add-lingua-connected-clients §2): the engine timestamps mutations and
  // exports/applies ops for the KnownWordsService / DeckService.

  /** Set a status stamped with a sync timestamp (epoch millis) for last-write-wins. */
  setStatusAt(lemma: string, status: LemmaStatus | null, atMs: number): Promise<void>;
  /** Every explicit status as an op, for a push (outbox / first-sign-in upload). */
  exportStatusOps(): Promise<StatusOp[]>;
  /** Apply pulled status changes under last-write-wins; returns how many changed. */
  applyStatusChanges(changes: StatusChangeIn[]): Promise<number>;
  /** Every deck card as an op, for a push. */
  exportCardOps(): Promise<CardOp[]>;
  /** Apply pulled card ops under last-write-wins; returns how many changed. */
  applyCardOps(ops: CardOp[]): Promise<number>;

  // CEFR levels (add-lingua-cefr-levels): the reader declares a level, reads
  // fill the ladder, and a level can seed the deck.

  /** Declare the reader's CEFR level, or `null` to clear it (back to frequency calibration). */
  setDeclaredLevel(level: CefrLevel | null): Promise<void>;
  /** The declared CEFR level, or `null` if none is set. */
  declaredLevel(): Promise<CefrLevel | null>;
  /** Whether the loaded pack carries CEFR data (else the ladder/feeding fall back to frequency). */
  hasLevels(): Promise<boolean>;
  /** The CEFR ladder A1→C2: confirmed / presumed / to-learn per level. Empty without CEFR data. */
  levelLadder(): Promise<LevelRow[]>;
  /** Record one reading exposure per lemma (feeds distinct-day counters; never changes a status). */
  recordExposures(lemmas: string[], source: string, atMs: number): Promise<void>;
  /** Confirm presumed-known lemmas read on ≥ N distinct days; returns how many were promoted. */
  promoteByExposure(thresholdDays: number, atMs: number): Promise<number>;
  /** Seed up to `count` cards from a level (commonest- or rarest-first); returns how many added. */
  seedLevel(level: CefrLevel, count: number, order: SeedOrder, at: number): Promise<number>;
}
