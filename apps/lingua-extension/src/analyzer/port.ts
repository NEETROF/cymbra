import type {
  CefrLevel,
  LemmaStatus,
  LevelRow,
  NativeLanguage,
  PageAnalysis,
  PhraseGloss,
  SeedOrder,
  StudiedLanguage,
  VocabularyEstimate,
  WordGrammar,
} from "./types.ts";

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
  /** Gloss a selection: every token with its dictionary form, class and gloss, no page gate. */
  phraseGloss(text: string): Promise<PhraseGloss>;
  /**
   * A word card's grammar: its gloss, grouped by part of speech, what the word as written is as
   * `lemma`, what else it may be, and the pieces the pre-pass split it into. Pure pack data.
   */
  wordGrammar(written: string, lemma: string): Promise<WordGrammar>;
}

/** An FSRS grade. */
export type Rating = "again" | "hard" | "good" | "easy";

/** A card being reviewed (the engine's view model). */
export interface ReviewCard {
  headword: string;
  surface: string;
  sentence: string;
  /** Where the word was met, as this device kept it — a page address, or a book and its
   *  chapter. Local only: never synchronised (lingua-privacy). Absent from an older engine. */
  source?: string;
  gloss: string | null;
  revealed: boolean;
  remaining: number;
  /** The card's studied language (add-lingua-language-stats-review); absent from an older engine. */
  language?: StudiedLanguage;
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
  status: string; // "known" | "learning" | "ignored" | "cleared" (a withdrawn decision)
  provenance: string; // "manual" | "srs" | "exposure" | "import"
  updated_at: number; // epoch millis
}

/** An incoming resolved status change to apply (from a pull). */
export interface StatusChangeIn {
  language: string;
  lemma: string;
  status: string; // "known" | "learning" | "ignored" | "cleared"
  provenance?: string; // "manual" | "srs" | "exposure" | "import"; empty/absent → manual
  updated_at: number; // epoch millis
}

/** One exported declared-level decision, from `exportDeclaredLevels` (also the apply shape). */
export interface DeclaredLevelOp {
  language: string;
  level: string; // "" = débutant | "A1".."C2"
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

// The engine surface, split by what depends on the studied language
// (generalise-lingua-extension-port). `LanguagePort` holds every call whose answer depends on
// it, bound to one language; `LinguaPort` holds what concerns the whole reader and hands out
// those views. A surface cannot reach a language-bound call without naming a language: the
// type checker refuses it.

/**
 * The calls whose answer depends on the studied language, bound to one: the reading
 * AnalyzerPort plus calibration, statuses, the declared level, levels, exposures, deck capture
 * and attributions. Obtained from `LinguaPort.for(language)`.
 */
export interface LanguagePort extends AnalyzerPort {
  /** The studied language every call of this view is answered in. */
  readonly language: StudiedLanguage;
  /** The current calibration threshold. */
  calibration(): Promise<number>;
  /** Add (or replace) a deck card and mark its form learning. */
  addCard(card: NewCard): Promise<void>;
  /** Retire the card for a lemma if present (keep it, stop it coming due); `now` in epoch seconds. */
  retireCard(lemma: string, now: number): Promise<void>;
  /** The pack's bundled attribution NOTICE. */
  notice(): Promise<string>;
  /** The pack's source licences. */
  licences(): Promise<string[]>;
  /** Set a status stamped with a sync timestamp (epoch millis) for last-write-wins. */
  setStatusAt(lemma: string, status: LemmaStatus | null, atMs: number): Promise<void>;

  // CEFR levels (add-lingua-cefr-levels): the reader declares a level, reads
  // fill the ladder, and a level can seed the deck.

  /** Declare the reader's CEFR level, or `null` to clear it (back to frequency calibration). */
  setDeclaredLevel(level: CefrLevel | null): Promise<void>;
  /** Like `setDeclaredLevel`, but stamps the decision (epoch millis) for cross-device last-write-wins. */
  setDeclaredLevelAt(level: CefrLevel | null, atMs: number): Promise<void>;
  /** The declared CEFR level, or `null` if none is set. */
  declaredLevel(): Promise<CefrLevel | null>;
  /** Whether the loaded pack carries CEFR data (else the ladder/feeding fall back to frequency). */
  hasLevels(): Promise<boolean>;
  /**
   * Whether those levels are estimated from word frequency rather than taken from a CEFR list
   * (add-lingua-spanish-levels): every surface then labels the levels it shows as estimated.
   */
  levelsEstimated(): Promise<boolean>;
  /** The CEFR ladder A1→C2: confirmed / presumed / to-learn per level. Empty without CEFR data. */
  levelLadder(): Promise<LevelRow[]>;
  /** The estimated vocabulary size: each frequency band's known share, extrapolated over the pack's dictionary words. */
  vocabularyEstimate(): Promise<VocabularyEstimate>;
  /** Record one reading exposure per lemma (feeds distinct-day counters; never changes a status). */
  recordExposures(lemmas: string[], source: string, atMs: number): Promise<void>;
  /** Confirm presumed-known lemmas read on ≥ N distinct days; returns how many were promoted. */
  promoteByExposure(thresholdDays: number, atMs: number): Promise<number>;
  /** Seed up to `count` cards from a level (commonest- or rarest-first); returns how many added. */
  seedLevel(level: CefrLevel, count: number, order: SeedOrder, at: number): Promise<number>;
}

// The whole-reader surface the review change drives (add-lingua-extension-review): deck
// counts, an FSRS review session, lossless backup/restore and sync, over every language the
// state holds. The engine holds the whole lingua-core LinguaState; persistence is backup → the
// durable store → restore in every context.
export interface LinguaPort {
  /** The view of this port bound to `language`. */
  for(language: StudiedLanguage): LanguagePort;
  /** The studied languages the engine holds a pack for, the default first. */
  languages(): Promise<StudiedLanguage[]>;
  /** The native language this port was built for, read from the reader's stored profile: every
   *  pack it loads is glossed in it (generalise-lingua-native-language D7). Loads no pack. */
  nativeLanguage(): Promise<NativeLanguage>;
  /** The reader's studied languages, from their profile, the primary first
   *  (add-lingua-studied-language-profile). Not the packs held: see `languages`. */
  studiedLanguages(): Promise<StudiedLanguage[]>;
  /** Set the reader's studied languages, the primary first; throws on an empty list or a duplicate. */
  setStudiedLanguages(languages: StudiedLanguage[]): Promise<void>;
  /** A document's language among `candidates` (the reader's order), `hint` its declared language
   *  (add-lingua-language-routing). Loads no pack. */
  detectLanguage(blocks: string[], candidates: StudiedLanguage[], hint: string | null): Promise<StudiedLanguage>;
  /** How many forms are explicitly marked (any status). */
  trackedCount(): Promise<number>;
  /** Cards in the deck, in `languages`, or in every language when absent or empty
   *  (refine-lingua-review-language D4). */
  deckCount(languages?: StudiedLanguage[]): Promise<number>;
  /** Cards due at `now` (epoch seconds), in `languages`, or in every language when absent or empty
   *  (add-lingua-language-stats-review). */
  dueCount(now: number, languages?: StudiedLanguage[]): Promise<number>;
  /** Start a review session over everything due at `now`, in `languages` or in all of them, the
   *  cards of several languages in due order; returns the count. */
  startReview(now: number, languages?: StudiedLanguage[]): Promise<number>;
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

  // Sync (add-lingua-connected-clients §2): the engine timestamps mutations and
  // exports/applies ops for the KnownWordsService / DeckService. Each op names its language.

  /** Every explicit status as an op, for a push (outbox / first-sign-in upload). */
  exportStatusOps(): Promise<StatusOp[]>;
  /** Apply pulled status changes under last-write-wins; returns how many changed. */
  applyStatusChanges(changes: StatusChangeIn[]): Promise<number>;
  /** Every deck card as an op, for a push. */
  exportCardOps(): Promise<CardOp[]>;
  /** Apply pulled card ops under last-write-wins; returns how many changed. */
  applyCardOps(ops: CardOp[]): Promise<number>;
  /** Every declared-level decision as an op, for a push (rides KnownWordsService with the statuses). */
  exportDeclaredLevels(): Promise<DeclaredLevelOp[]>;
  /** Apply pulled declared-level changes under last-write-wins; returns how many changed. */
  applyDeclaredLevelChanges(changes: DeclaredLevelOp[]): Promise<number>;
}
