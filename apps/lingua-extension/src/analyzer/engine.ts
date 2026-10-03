import type {
  CardOp,
  DeclaredLevelOp,
  LanguagePort,
  LinguaPort,
  NewCard,
  Rating,
  ReviewCard,
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

// The Chromium LinguaPort implementation: the lingua-core WASM module instantiated
// lazily in the content script's isolated world (design D2). This is the only place
// that touches the wasm-pack output; everything else consumes the LinguaPort seam.
// The engine holds the whole lingua-core LinguaState (knowledge + deck + FSRS);
// persistence across contexts is backup → the durable store (state/store.ts) → restore.
//
// MV3 loading notes: a classic content script cannot statically import the wasm-pack
// ES module, and init()'s bare auto-fetch is unreliable under chrome-extension://, so
// we dynamic-import the glue by its extension URL and hand init() the explicit .wasm
// Response. All three files (glue, _bg.wasm, pack) are web_accessible_resources.

/** The engine's own surface (lingua-wasm). A language-bound binding takes the studied
 * language last; without one it answers in the language of the first pack loaded. */
interface WasmEngine {
  setCalibration(threshold: number, language?: string | null): void;
  calibration(language?: string | null): number;
  setStatus(lemma: string, status: string, language?: string | null): void;
  analyse(blocks: string[], language?: string | null): string;
  gloss(lemma: string, language?: string | null): string | undefined;
  phraseGloss(text: string, language?: string | null): string;
  wordGrammar(written: string, lemma: string, language?: string | null): string;
  trackedCount(): number;
  addCard(
    lemma: string,
    surface: string,
    sentence: string,
    url: string,
    gloss: string | null | undefined,
    capturedAt: number,
    language?: string | null,
  ): void;
  retireCard(lemma: string, now: number, language?: string | null): void;
  deckCount(): number;
  dueCount(now: number): number;
  startReview(now: number): number;
  reviewCurrent(): string | undefined;
  reviewReveal(): void;
  reviewGrade(rating: string, now: number): void;
  reviewMarkKnown(now: number): void;
  backup(): string;
  restore(json: string): void;
  reset(): void;
  resetStatuses(): void;
  notice(language?: string | null): string;
  licences(language?: string | null): string;
  setStatusAt(lemma: string, status: string, atMs: number, language?: string | null): void;
  exportStatusOps(): string;
  applyStatusChanges(json: string): number;
  exportCardOps(): string;
  applyCardOps(json: string): number;
  setDeclaredLevel(level: string, language?: string | null): void;
  setDeclaredLevelAt(level: string, atMs: number, language?: string | null): void;
  declaredLevel(language?: string | null): string | undefined;
  exportDeclaredLevels(): string;
  applyDeclaredLevelChanges(json: string): number;
  hasLevels(language?: string | null): boolean;
  levelLadder(language?: string | null): string;
  vocabularyEstimate(language?: string | null): string;
  recordExposures(lemmas: string[], source: string, atMs: number, language?: string | null): void;
  promoteByExposure(thresholdDays: number, atMs: number, language?: string | null): number;
  seedLevel(level: string, count: number, order: string, at: number, language?: string | null): number;
  addPack(packBytes: Uint8Array): string;
  languages(): string;
  free(): void;
}

export interface WasmModule {
  default: (init: Response | string | URL) => Promise<unknown>;
  LinguaEngine: new (packBytes: Uint8Array) => WasmEngine;
}

/** How the wasm-pack glue module is obtained. */
export type GlueLoader = () => Promise<WasmModule>;

/** Paths of the vendored wasm output + pack within the built extension. */
const GLUE_PATH = "wasm/lingua_wasm.js";
const WASM_PATH = "wasm/lingua_wasm_bg.wasm";
// The default pair's pack (packs.json, generalise-lingua-pack-build); the other listed pairs
// are package-lingua-packs-per-pair's to load.
const PACK_PATH = `assets/packs/${__LINGUA_PACKS__.split(",")[0]}.lingua`;

/** The status string that clears an explicit status in the WASM engine. */
const CLEAR = "clear";

/**
 * Default loader: dynamic-import the glue by its extension URL. This is the ONLY option
 * for a content script (a classic IIFE cannot statically import an ES module) and it
 * works in a Firefox event page — but NOT in a Chromium service worker, where the HTML
 * spec forbids dynamic `import()`. The service worker passes a static loader instead.
 */
const dynamicGlue: GlueLoader = async () =>
  (await import(/* @vite-ignore */ chrome.runtime.getURL(GLUE_PATH))) as WasmModule;

/**
 * One initialisation per glue module, shared by every engine built on it. The glue's init()
 * only short-circuits once it has finished, so two engines built at the same time — the
 * background's reading and sync engines when the event page wakes — would each instantiate
 * the module. The later instantiation replaces the module's memory, and the engine built on
 * the earlier one then reads out of bounds or someone else's state.
 */
const initialised = new WeakMap<WasmModule, Promise<unknown>>();

function initialise(mod: WasmModule): Promise<unknown> {
  let ready = initialised.get(mod);
  if (!ready) {
    const attempt = fetch(chrome.runtime.getURL(WASM_PATH)).then((wasm) => mod.default(wasm));
    // A failed initialisation is retried by the next engine rather than remembered.
    attempt.catch(() => {
      if (initialised.get(mod) === attempt) initialised.delete(mod);
    });
    initialised.set(mod, attempt);
    ready = attempt;
  }
  return ready;
}

export class WasmAnalyzerPort implements LinguaPort {
  private enginePromise: Promise<WasmEngine> | null = null;

  constructor(private readonly loadGlue: GlueLoader = dynamicGlue) {}

  /** Instantiated once per tab, on the first call. */
  private engine(): Promise<WasmEngine> {
    return (this.enginePromise ??= this.build());
  }

  private async build(): Promise<WasmEngine> {
    const mod = await this.loadGlue();
    await initialise(mod);
    const packBytes = new Uint8Array(await (await fetch(chrome.runtime.getURL(PACK_PATH))).arrayBuffer());
    // Throws if the pack is malformed or built for an incompatible analyzer_version.
    return new mod.LinguaEngine(packBytes);
  }

  /** The view of this engine bound to `language`: each call names it to the engine. */
  for(language: StudiedLanguage): LanguagePort {
    return new WasmLanguagePort(() => this.engine(), language);
  }

  async languages(): Promise<StudiedLanguage[]> {
    return JSON.parse((await this.engine()).languages()) as StudiedLanguage[];
  }

  async trackedCount(): Promise<number> {
    return (await this.engine()).trackedCount();
  }

  async deckCount(): Promise<number> {
    return (await this.engine()).deckCount();
  }

  async dueCount(now: number): Promise<number> {
    return (await this.engine()).dueCount(now);
  }

  async startReview(now: number): Promise<number> {
    return (await this.engine()).startReview(now);
  }

  async reviewCurrent(): Promise<ReviewCard | null> {
    const json = (await this.engine()).reviewCurrent();
    return json ? (JSON.parse(json) as ReviewCard) : null;
  }

  async reviewReveal(): Promise<void> {
    (await this.engine()).reviewReveal();
  }

  async reviewGrade(rating: Rating, now: number): Promise<void> {
    (await this.engine()).reviewGrade(rating, now);
  }

  async reviewMarkKnown(now: number): Promise<void> {
    (await this.engine()).reviewMarkKnown(now);
  }

  async backup(): Promise<string> {
    return (await this.engine()).backup();
  }

  async restore(json: string): Promise<void> {
    (await this.engine()).restore(json);
  }

  async reset(): Promise<void> {
    (await this.engine()).reset();
  }

  async resetStatuses(): Promise<void> {
    (await this.engine()).resetStatuses();
  }

  async exportStatusOps(): Promise<StatusOp[]> {
    return JSON.parse((await this.engine()).exportStatusOps()) as StatusOp[];
  }

  async applyStatusChanges(changes: StatusChangeIn[]): Promise<number> {
    return (await this.engine()).applyStatusChanges(JSON.stringify(changes));
  }

  async exportCardOps(): Promise<CardOp[]> {
    return JSON.parse((await this.engine()).exportCardOps()) as CardOp[];
  }

  async applyCardOps(ops: CardOp[]): Promise<number> {
    return (await this.engine()).applyCardOps(JSON.stringify(ops));
  }

  // --- CEFR levels (add-lingua-cefr-levels) ---

  async exportDeclaredLevels(): Promise<DeclaredLevelOp[]> {
    return JSON.parse((await this.engine()).exportDeclaredLevels()) as DeclaredLevelOp[];
  }

  async applyDeclaredLevelChanges(changes: DeclaredLevelOp[]): Promise<number> {
    return (await this.engine()).applyDeclaredLevelChanges(JSON.stringify(changes));
  }
}

/** The calls of `WasmAnalyzerPort` that depend on the studied language, bound to one. Holds
 *  nothing but its parent's engine accessor and the language, so it costs nothing to make. */
class WasmLanguagePort implements LanguagePort {
  constructor(
    private readonly engine: () => Promise<WasmEngine>,
    readonly language: StudiedLanguage,
  ) {}

  async analyse(blocks: string[]): Promise<PageAnalysis> {
    return JSON.parse((await this.engine()).analyse(blocks, this.language)) as PageAnalysis;
  }

  async setCalibration(threshold: number): Promise<void> {
    (await this.engine()).setCalibration(threshold, this.language);
  }

  async calibration(): Promise<number> {
    return (await this.engine()).calibration(this.language);
  }

  async setStatus(lemma: string, status: LemmaStatus | null): Promise<void> {
    (await this.engine()).setStatus(lemma, status ?? CLEAR, this.language);
  }

  async gloss(lemma: string): Promise<string | undefined> {
    return (await this.engine()).gloss(lemma, this.language);
  }

  async phraseGloss(text: string): Promise<PhraseGloss> {
    return JSON.parse((await this.engine()).phraseGloss(text, this.language)) as PhraseGloss;
  }

  async wordGrammar(written: string, lemma: string): Promise<WordGrammar> {
    return JSON.parse((await this.engine()).wordGrammar(written, lemma, this.language)) as WordGrammar;
  }

  async addCard(card: NewCard): Promise<void> {
    (await this.engine()).addCard(
      card.lemma,
      card.surface,
      card.sentence,
      card.url,
      card.gloss,
      card.capturedAt,
      this.language,
    );
  }

  async retireCard(lemma: string, now: number): Promise<void> {
    (await this.engine()).retireCard(lemma, now, this.language);
  }

  async notice(): Promise<string> {
    return (await this.engine()).notice(this.language);
  }

  async licences(): Promise<string[]> {
    return JSON.parse((await this.engine()).licences(this.language)) as string[];
  }

  async setStatusAt(lemma: string, status: LemmaStatus | null, atMs: number): Promise<void> {
    (await this.engine()).setStatusAt(lemma, status ?? CLEAR, atMs, this.language);
  }

  async setDeclaredLevel(level: CefrLevel | null): Promise<void> {
    (await this.engine()).setDeclaredLevel(level ?? "", this.language);
  }

  async setDeclaredLevelAt(level: CefrLevel | null, atMs: number): Promise<void> {
    (await this.engine()).setDeclaredLevelAt(level ?? "", atMs, this.language);
  }

  async declaredLevel(): Promise<CefrLevel | null> {
    return ((await this.engine()).declaredLevel(this.language) as CefrLevel | undefined) ?? null;
  }

  async hasLevels(): Promise<boolean> {
    return (await this.engine()).hasLevels(this.language);
  }

  async levelLadder(): Promise<LevelRow[]> {
    return JSON.parse((await this.engine()).levelLadder(this.language)) as LevelRow[];
  }

  async vocabularyEstimate(): Promise<VocabularyEstimate> {
    return JSON.parse((await this.engine()).vocabularyEstimate(this.language)) as VocabularyEstimate;
  }

  async recordExposures(lemmas: string[], source: string, atMs: number): Promise<void> {
    (await this.engine()).recordExposures(lemmas, source, atMs, this.language);
  }

  async promoteByExposure(thresholdDays: number, atMs: number): Promise<number> {
    return (await this.engine()).promoteByExposure(thresholdDays, atMs, this.language);
  }

  async seedLevel(level: CefrLevel, count: number, order: SeedOrder, at: number): Promise<number> {
    return (await this.engine()).seedLevel(level, count, order, at, this.language);
  }
}
