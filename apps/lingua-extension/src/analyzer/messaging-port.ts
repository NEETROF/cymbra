import type {
  CardOp,
  DeclaredLevelOp,
  LanguagePort,
  LinguaPort,
  NewCard,
  Rating,
  ReviewCard,
  ReviewOptions,
  ReviewSummary,
  StatusChangeIn,
  StatusOp,
} from "./port.ts";
import type {
  CefrLevel,
  LemmaStatus,
  LevelRow,
  PageAnalysis,
  PhraseGloss,
  SeedOrder,
  StudiedLanguage,
  VocabularyEstimate,
  WordGrammar,
} from "./types.ts";
import { sendRpc } from "./rpc.ts";

// The Firefox AnalyzerPort implementation: a thin LinguaPort that forwards every call
// to the WASM engine hosted in the event page (background.ts), over runtime messaging.
// The reading code and side panel consume the same LinguaPort seam, unaware of where
// the engine runs. `send` is injected so the RPC transport can be faked in tests.
//
// A call that depends on the studied language goes through the view `for(language)`
// returns, and its request carries the language; the host answers it on its own view of
// that language (generalise-lingua-extension-port). Whole-reader calls carry none.

export type RpcSend = (method: string, args: unknown[], language?: StudiedLanguage) => Promise<unknown>;

export class MessagingLinguaPort implements LinguaPort {
  constructor(private readonly send: RpcSend = sendRpc) {}

  private rpc<T>(method: string, args: unknown[] = []): Promise<T> {
    return this.send(method, args) as Promise<T>;
  }

  for(language: StudiedLanguage): LanguagePort {
    return new MessagingLanguagePort(this.send, language);
  }
  languages(): Promise<StudiedLanguage[]> {
    return this.rpc("languages");
  }
  studiedLanguages(): Promise<StudiedLanguage[]> {
    return this.rpc("studiedLanguages");
  }
  setStudiedLanguages(languages: StudiedLanguage[]): Promise<void> {
    return this.rpc("setStudiedLanguages", [languages]);
  }
  detectLanguage(blocks: string[], candidates: StudiedLanguage[], hint: string | null): Promise<StudiedLanguage> {
    return this.rpc("detectLanguage", [blocks, candidates, hint]);
  }
  trackedCount(): Promise<number> {
    return this.rpc("trackedCount");
  }
  deckCount(languages?: StudiedLanguage[]): Promise<number> {
    return this.rpc("deckCount", languages ? [languages] : []);
  }
  dueCount(now: number, languages?: StudiedLanguage[]): Promise<number> {
    return this.rpc("dueCount", languages ? [now, languages] : [now]);
  }
  startReview(now: number, languages?: StudiedLanguage[], options?: ReviewOptions): Promise<number> {
    // Undefined becomes null on the wire; the engine reads a null as "every language".
    if (options) return this.rpc("startReview", [now, languages ?? null, options]);
    return this.rpc("startReview", languages ? [now, languages] : [now]);
  }
  reviewCurrent(): Promise<ReviewCard | null> {
    return this.rpc("reviewCurrent");
  }
  reviewReveal(): Promise<void> {
    return this.rpc("reviewReveal");
  }
  reviewGrade(rating: Rating, now: number): Promise<boolean> {
    return this.rpc("reviewGrade", [rating, now]);
  }
  reviewMarkKnown(now: number): Promise<void> {
    return this.rpc("reviewMarkKnown", [now]);
  }
  reviewIgnore(now: number): Promise<void> {
    return this.rpc("reviewIgnore", [now]);
  }
  reviewSummary(): Promise<ReviewSummary | null> {
    return this.rpc("reviewSummary");
  }
  backup(): Promise<string> {
    return this.rpc("backup");
  }
  restore(json: string): Promise<void> {
    return this.rpc("restore", [json]);
  }
  reset(): Promise<void> {
    return this.rpc("reset");
  }
  resetStatuses(): Promise<void> {
    return this.rpc("resetStatuses");
  }
  exportStatusOps(): Promise<StatusOp[]> {
    return this.rpc("exportStatusOps");
  }
  applyStatusChanges(changes: StatusChangeIn[]): Promise<number> {
    return this.rpc("applyStatusChanges", [changes]);
  }
  exportCardOps(): Promise<CardOp[]> {
    return this.rpc("exportCardOps");
  }
  applyCardOps(ops: CardOp[]): Promise<number> {
    return this.rpc("applyCardOps", [ops]);
  }
  exportDeclaredLevels(): Promise<DeclaredLevelOp[]> {
    return this.rpc("exportDeclaredLevels");
  }
  applyDeclaredLevelChanges(changes: DeclaredLevelOp[]): Promise<number> {
    return this.rpc("applyDeclaredLevelChanges", [changes]);
  }
}

/** The calls of `MessagingLinguaPort` that depend on the studied language: every request
 *  carries `language`. Holds nothing but the transport and the language. */
class MessagingLanguagePort implements LanguagePort {
  constructor(
    private readonly send: RpcSend,
    readonly language: StudiedLanguage,
  ) {}

  private rpc<T>(method: string, args: unknown[] = []): Promise<T> {
    return this.send(method, args, this.language) as Promise<T>;
  }

  analyse(blocks: string[]): Promise<PageAnalysis> {
    return this.rpc("analyse", [blocks]);
  }
  setCalibration(threshold: number): Promise<void> {
    return this.rpc("setCalibration", [threshold]);
  }
  calibration(): Promise<number> {
    return this.rpc("calibration");
  }
  setStatus(lemma: string, status: LemmaStatus | null): Promise<void> {
    return this.rpc("setStatus", [lemma, status]);
  }
  gloss(lemma: string): Promise<string | undefined> {
    return this.rpc("gloss", [lemma]);
  }
  phraseGloss(text: string): Promise<PhraseGloss> {
    return this.rpc("phraseGloss", [text]);
  }
  wordGrammar(written: string, lemma: string): Promise<WordGrammar> {
    return this.rpc("wordGrammar", [written, lemma]);
  }
  addCard(card: NewCard): Promise<void> {
    return this.rpc("addCard", [card]);
  }
  retireCard(lemma: string, now: number): Promise<void> {
    return this.rpc("retireCard", [lemma, now]);
  }
  notice(): Promise<string> {
    return this.rpc("notice");
  }
  licences(): Promise<string[]> {
    return this.rpc("licences");
  }
  setStatusAt(lemma: string, status: LemmaStatus | null, atMs: number): Promise<void> {
    return this.rpc("setStatusAt", [lemma, status, atMs]);
  }
  setDeclaredLevel(level: CefrLevel | null): Promise<void> {
    return this.rpc("setDeclaredLevel", [level]);
  }
  setDeclaredLevelAt(level: CefrLevel | null, atMs: number): Promise<void> {
    return this.rpc("setDeclaredLevelAt", [level, atMs]);
  }
  declaredLevel(): Promise<CefrLevel | null> {
    return this.rpc("declaredLevel");
  }
  hasLevels(): Promise<boolean> {
    return this.rpc("hasLevels");
  }
  levelsEstimated(): Promise<boolean> {
    return this.rpc("levelsEstimated");
  }
  levelLadder(): Promise<LevelRow[]> {
    return this.rpc("levelLadder");
  }
  vocabularyEstimate(): Promise<VocabularyEstimate> {
    return this.rpc("vocabularyEstimate");
  }
  recordExposures(lemmas: string[], source: string, atMs: number): Promise<void> {
    return this.rpc("recordExposures", [lemmas, source, atMs]);
  }
  promoteByExposure(thresholdDays: number, atMs: number): Promise<number> {
    return this.rpc("promoteByExposure", [thresholdDays, atMs]);
  }
  seedLevel(level: CefrLevel, count: number, order: SeedOrder, at: number): Promise<number> {
    return this.rpc("seedLevel", [level, count, order, at]);
  }
}
