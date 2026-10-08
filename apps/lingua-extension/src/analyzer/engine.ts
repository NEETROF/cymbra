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
import { DEFAULT_NATIVE, defaultPair, packPath, pairFor, pairsOf, SHIPPED_PAIRS } from "./pairs.ts";
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

// The Chromium LinguaPort implementation: the lingua-core WASM module instantiated
// lazily in the content script's isolated world (design D2). This is the only place
// that touches the wasm-pack output; everything else consumes the LinguaPort seam.
// The engine holds the whole lingua-core LinguaState (knowledge + deck + FSRS);
// persistence across contexts is backup → the durable store (state/store.ts) → restore.
//
// MV3 loading notes: a classic content script cannot statically import the wasm-pack
// ES module, and init()'s bare auto-fetch is unreliable under chrome-extension://, so
// we dynamic-import the glue by its extension URL and hand init() the explicit .wasm
// Response. The glue, the _bg.wasm and every listed pack are web_accessible_resources.

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
  frequencyRank(lemma: string, language?: string | null): number | undefined;
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
  deckCount(languages?: string[] | null): number;
  dueCount(now: number, languages?: string[] | null): number;
  startReview(now: number, languages?: string[] | null): number;
  reviewCurrent(): string | undefined;
  reviewCurrentLanguage(): string | undefined;
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
  levelsEstimated(language?: string | null): boolean;
  levelLadder(language?: string | null): string;
  vocabularyEstimate(language?: string | null): string;
  recordExposures(lemmas: string[], source: string, atMs: number, language?: string | null): void;
  promoteByExposure(thresholdDays: number, atMs: number, language?: string | null): number;
  seedLevel(level: string, count: number, order: string, at: number, language?: string | null): number;
  addPack(packBytes: Uint8Array): string;
  languages(): string;
  nativeLanguage(): string;
  profileNativeLanguage(): string;
  studiedLanguages(): string;
  setStudiedLanguages(tags: string[]): void;
  setProfile(native: string, languages: string[]): void;
  detectLanguage(blocks: string[], candidates: string[], hint?: string | null): string;
  free(): void;
}

export interface WasmModule {
  default: (init: Response | string | URL) => Promise<unknown>;
  LinguaEngine: new (packBytes: Uint8Array) => WasmEngine;
  /** A backup with another profile: a pure function of the glue, no engine and no pack (D2). */
  reprofileBackup(backup: string, native: string, studied: string[]): string;
}

/** How the wasm-pack glue module is obtained. */
export type GlueLoader = () => Promise<WasmModule>;

/** The reader's native language, as their stored profile names it (`storedNativeLanguage`). */
export type NativeResolver = () => Promise<NativeLanguage>;

/** Paths of the vendored wasm output within the built extension (the packs': pairs.ts). */
const GLUE_PATH = "wasm/lingua_wasm.js";
const WASM_PATH = "wasm/lingua_wasm_bg.wasm";

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

/**
 * `backup` with the reader's profile replaced by `native`, studying `studied` (the primary first)
 * — lingua-wasm's `reprofileBackup`, validating the choice as the engine's profile does and writing
 * the version the state needs (add-lingua-native-language-choice D2). Pure: it needs the module
 * initialised, never an engine nor a pack, so the store's owner rewrites a backup of any native
 * language. Rejects on a refused choice, the backup left as it was.
 */
export async function reprofileBackup(
  backup: string,
  native: NativeLanguage,
  studied: readonly StudiedLanguage[],
  loadGlue: GlueLoader = dynamicGlue,
): Promise<string> {
  const mod = await loadGlue();
  await initialise(mod);
  return mod.reprofileBackup(backup, native, [...studied]);
}

/** A pair's pack, read from the package. */
async function fetchPack(pair: string): Promise<Uint8Array> {
  return new Uint8Array(await (await fetch(chrome.runtime.getURL(packPath(pair)))).arrayBuffer());
}

export class WasmAnalyzerPort implements LinguaPort {
  private enginePromise: Promise<WasmEngine> | null = null;
  private nativePromise: Promise<NativeLanguage> | null = null;
  /** The packs added after the default's, one load per language (package-lingua-packs-per-pair). */
  private readonly added = new Map<string, Promise<void>>();
  /**
   * The engine built for a backup of another native language than this one's (`restore`, D3): the
   * restores of one change share it rather than each building their own.
   */
  private rebuilt: { native: NativeLanguage; engine: Promise<WasmEngine> } | null = null;

  /**
   * `pairs`: the pairs listed, the default language's first. `resolveNative`: the reader's native
   * language, asked once, before the first pack is fetched; the engine loads only the pairs glossed
   * in it, starting with the first of them (generalise-lingua-native-language D7).
   */
  constructor(
    private readonly loadGlue: GlueLoader = dynamicGlue,
    private readonly pairs: readonly string[] = SHIPPED_PAIRS,
    private readonly resolveNative: NativeResolver = async () => DEFAULT_NATIVE,
  ) {}

  /**
   * The native language this engine is built for, asked of the resolver once. One no listed pair is
   * glossed in is refused here, before anything is fetched. A resolver that failed — the store did
   * not answer — is asked again by the next call, whichever call it is, rather than remembered: the
   * build waits for the native language before it is cached (`engine`).
   */
  private native(): Promise<NativeLanguage> {
    if (!this.nativePromise) {
      const resolved = this.resolveNative();
      const attempt = resolved.then((native) => {
        if (defaultPair(native, this.pairs) === null) {
          throw new Error(`no shipped pair is glossed in "${native}" (shipped pairs: ${this.pairs.join(", ")})`);
        }
        return native;
      });
      resolved.catch(() => {
        if (this.nativePromise === attempt) this.nativePromise = null;
      });
      this.nativePromise = attempt;
    }
    return this.nativePromise;
  }

  /**
   * Instantiated once per tab, on the first call, with the native language's default pair's pack.
   * The native language is known before the build is cached, so a store that failed to answer never
   * leaves a failed build behind for every later call (generalise-lingua-native-language D7).
   */
  private async engine(): Promise<WasmEngine> {
    const native = await this.native();
    return (this.enginePromise ??= this.build(native));
  }

  private async build(native: NativeLanguage): Promise<WasmEngine> {
    const pair = defaultPair(native, this.pairs) as string;
    const mod = await this.loadGlue();
    await initialise(mod);
    const packBytes = await fetchPack(pair);
    // Throws if the pack is malformed or built for an incompatible analyzer_version.
    return new mod.LinguaEngine(packBytes);
  }

  /**
   * The engine once it holds `language`'s pack. The native language's default pair's built it;
   * another of its pairs' is added the first time its language is asked for. A language no pair of
   * the native language studies is refused here, before the engine sees anything.
   */
  private async engineFor(language: string): Promise<WasmEngine> {
    const native = await this.native();
    const pair = pairFor(language, native, this.pairs);
    if (pair === null) {
      throw new Error(
        `no shipped pack studies "${language}" (shipped pairs: ${pairsOf(native, this.pairs).join(", ")})`,
      );
    }
    const engine = await this.engine();
    if (pair !== defaultPair(native, this.pairs)) await this.addPack(engine, language, pair);
    return engine;
  }

  /** Adds `pair`'s pack to the engine once: a pending load is shared, a failed one forgotten. */
  private addPack(engine: WasmEngine, language: string, pair: string): Promise<void> {
    let load = this.added.get(language);
    if (!load) {
      const attempt = fetchPack(pair).then((bytes) => {
        engine.addPack(bytes);
      });
      // A failed load is retried by the next call rather than remembered.
      attempt.catch(() => {
        if (this.added.get(language) === attempt) this.added.delete(language);
      });
      this.added.set(language, attempt);
      load = attempt;
    }
    return load;
  }

  /**
   * The engine once it holds the pack of every listed language `records` name. The engine skips
   * a record whose language it holds no pack for, and a sync then moves its cursor past it: a
   * record of a language the package ships must never be skipped for want of a load. A language
   * no listed pair studies loads nothing, and its records stay skipped.
   */
  private async engineHolding(records: ReadonlyArray<{ language: string }>): Promise<WasmEngine> {
    const native = await this.native();
    const languages = new Set(records.map((record) => record.language));
    const listed = [...languages].filter((language) => pairFor(language, native, this.pairs) !== null);
    await Promise.all(listed.map((language) => this.engineFor(language)));
    return this.engine();
  }

  /** The view of this engine bound to `language`: each call names it to the engine, once the
   *  engine holds its pack. */
  for(language: StudiedLanguage): LanguagePort {
    return new WasmLanguagePort(() => this.engineFor(language), language);
  }

  async languages(): Promise<StudiedLanguage[]> {
    return JSON.parse((await this.engine()).languages()) as StudiedLanguage[];
  }

  nativeLanguage(): Promise<NativeLanguage> {
    return this.native();
  }

  async studiedLanguages(): Promise<StudiedLanguage[]> {
    return JSON.parse((await this.engine()).studiedLanguages()) as StudiedLanguage[];
  }

  async setStudiedLanguages(languages: StudiedLanguage[]): Promise<void> {
    (await this.engine()).setStudiedLanguages(languages);
  }

  async detectLanguage(blocks: string[], candidates: StudiedLanguage[], hint: string | null): Promise<StudiedLanguage> {
    return (await this.engine()).detectLanguage(blocks, candidates, hint) as StudiedLanguage;
  }

  async trackedCount(): Promise<number> {
    return (await this.engine()).trackedCount();
  }

  async deckCount(languages?: StudiedLanguage[]): Promise<number> {
    return (await this.engine()).deckCount(languages ?? null);
  }

  async dueCount(now: number, languages?: StudiedLanguage[]): Promise<number> {
    return (await this.engine()).dueCount(now, languages ?? null);
  }

  async startReview(now: number, languages?: StudiedLanguage[]): Promise<number> {
    return (await this.engine()).startReview(now, languages ?? null);
  }

  async reviewCurrent(): Promise<ReviewCard | null> {
    const engine = await this.engine();
    const json = engine.reviewCurrent();
    if (!json) return null;
    // The card's view stays as the English baseline pins it; its language is a call of its own.
    const language = engine.reviewCurrentLanguage() as StudiedLanguage | undefined;
    return { ...(JSON.parse(json) as ReviewCard), ...(language ? { language } : {}) };
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

  /**
   * Restore the reader's state. A backup whose profile names another native language than this
   * engine's — the reader chose it in another context, or a file or a sync brought it — has this
   * port drop its engine and its added packs and build again for that native language, then
   * restore into the new engine (add-lingua-native-language-choice D3): every port follows the
   * backup's native language. One no listed pair is glossed in is restored whole into this engine,
   * as before, and the reader is served in its language (M22). The engine itself says which native
   * language the backup names, after its own parse — nothing is parsed twice on this path, which
   * every store change takes.
   */
  async restore(json: string): Promise<void> {
    const engine = await this.engine();
    engine.restore(json);
    const wanted = engine.profileNativeLanguage() as NativeLanguage;
    if (wanted === engine.nativeLanguage() || defaultPair(wanted, this.pairs) === null) return;
    (await this.rebuildFor(wanted)).restore(json);
  }

  /**
   * This port's engine for `native` from now on: built again, its packs added again as their
   * languages are asked. A build for the same native language already under way is shared, so two
   * restores of one change land in one engine, in the order they asked.
   */
  private rebuildFor(native: NativeLanguage): Promise<WasmEngine> {
    if (this.rebuilt?.native === native && this.enginePromise === this.rebuilt.engine) return this.rebuilt.engine;
    this.added.clear();
    this.nativePromise = Promise.resolve(native);
    const engine = this.build(native);
    this.enginePromise = engine;
    this.rebuilt = { native, engine };
    // A build that failed is tried again by the next call rather than remembered.
    engine.catch(() => {
      if (this.enginePromise === engine) this.enginePromise = null;
    });
    return engine;
  }

  /**
   * Forget the engine, its packs and the native language it was built for: the next call asks the
   * resolver again and builds anew (add-lingua-native-language-choice D2). The background drops its
   * two engines so when the reader chose another native language. The old engine is not freed here —
   * a call already under way may still hold it; the glue's finaliser frees it once unreachable.
   */
  drop(): void {
    this.enginePromise = null;
    this.nativePromise = null;
    this.added.clear();
    this.rebuilt = null;
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
    return (await this.engineHolding(changes)).applyStatusChanges(JSON.stringify(changes));
  }

  async exportCardOps(): Promise<CardOp[]> {
    return JSON.parse((await this.engine()).exportCardOps()) as CardOp[];
  }

  async applyCardOps(ops: CardOp[]): Promise<number> {
    return (await this.engineHolding(ops)).applyCardOps(JSON.stringify(ops));
  }

  // --- CEFR levels (add-lingua-cefr-levels) ---

  async exportDeclaredLevels(): Promise<DeclaredLevelOp[]> {
    return JSON.parse((await this.engine()).exportDeclaredLevels()) as DeclaredLevelOp[];
  }

  async applyDeclaredLevelChanges(changes: DeclaredLevelOp[]): Promise<number> {
    return (await this.engineHolding(changes)).applyDeclaredLevelChanges(JSON.stringify(changes));
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
    const engine = await this.engine();
    const grammar = JSON.parse(engine.wordGrammar(written, lemma, this.language)) as WordGrammar;
    // The rank comes with the grammar (add-lingua-card-frequency D3): one answer, under one bound.
    return { ...grammar, rank: engine.frequencyRank(lemma, this.language) ?? null };
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

  async levelsEstimated(): Promise<boolean> {
    return (await this.engine()).levelsEstimated(this.language);
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
