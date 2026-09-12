import type { CardOp, LinguaPort, NewCard, Rating, ReviewCard, StatusChangeIn, StatusOp } from "./port.ts";
import type { CefrLevel, LemmaStatus, LevelRow, PageAnalysis, SeedOrder } from "./types.ts";

// The Chromium LinguaPort implementation: the lingua-core WASM module instantiated
// lazily in the content script's isolated world (design D2). This is the only place
// that touches the wasm-pack output; everything else consumes the LinguaPort seam.
// The engine holds the whole lingua-core LinguaState (knowledge + deck + FSRS);
// persistence across contexts is backup → chrome.storage.local → restore.
//
// MV3 loading notes: a classic content script cannot statically import the wasm-pack
// ES module, and init()'s bare auto-fetch is unreliable under chrome-extension://, so
// we dynamic-import the glue by its extension URL and hand init() the explicit .wasm
// Response. All three files (glue, _bg.wasm, pack) are web_accessible_resources.

interface WasmEngine {
  setCalibration(threshold: number): void;
  calibration(): number;
  setStatus(lemma: string, status: string): void;
  analyse(blocks: string[]): string;
  gloss(lemma: string): string | undefined;
  trackedCount(): number;
  addCard(
    lemma: string,
    surface: string,
    sentence: string,
    url: string,
    gloss: string | null | undefined,
    capturedAt: number,
  ): void;
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
  notice(): string;
  licences(): string;
  setStatusAt(lemma: string, status: string, atMs: number): void;
  exportStatusOps(): string;
  applyStatusChanges(json: string): number;
  exportCardOps(): string;
  applyCardOps(json: string): number;
  setDeclaredLevel(level: string): void;
  declaredLevel(): string | undefined;
  hasLevels(): boolean;
  levelLadder(): string;
  recordExposures(lemmas: string[], source: string, atMs: number): void;
  promoteByExposure(thresholdDays: number, atMs: number): number;
  seedLevel(level: string, count: number, order: string, at: number): number;
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
const PACK_PATH = "assets/pack.lingua";

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

export class WasmAnalyzerPort implements LinguaPort {
  private enginePromise: Promise<WasmEngine> | null = null;

  constructor(private readonly loadGlue: GlueLoader = dynamicGlue) {}

  /** Instantiated once per tab, on the first call. */
  private engine(): Promise<WasmEngine> {
    return (this.enginePromise ??= this.build());
  }

  private async build(): Promise<WasmEngine> {
    const mod = await this.loadGlue();
    await mod.default(await fetch(chrome.runtime.getURL(WASM_PATH)));
    const packBytes = new Uint8Array(await (await fetch(chrome.runtime.getURL(PACK_PATH))).arrayBuffer());
    // Throws if the pack is malformed or built for an incompatible analyzer_version.
    return new mod.LinguaEngine(packBytes);
  }

  async analyse(blocks: string[]): Promise<PageAnalysis> {
    return JSON.parse((await this.engine()).analyse(blocks)) as PageAnalysis;
  }

  async setCalibration(threshold: number): Promise<void> {
    (await this.engine()).setCalibration(threshold);
  }

  async calibration(): Promise<number> {
    return (await this.engine()).calibration();
  }

  async setStatus(lemma: string, status: LemmaStatus | null): Promise<void> {
    (await this.engine()).setStatus(lemma, status ?? CLEAR);
  }

  async gloss(lemma: string): Promise<string | undefined> {
    return (await this.engine()).gloss(lemma);
  }

  async trackedCount(): Promise<number> {
    return (await this.engine()).trackedCount();
  }

  async addCard(card: NewCard): Promise<void> {
    (await this.engine()).addCard(card.lemma, card.surface, card.sentence, card.url, card.gloss, card.capturedAt);
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

  async notice(): Promise<string> {
    return (await this.engine()).notice();
  }

  async licences(): Promise<string[]> {
    return JSON.parse((await this.engine()).licences()) as string[];
  }

  async setStatusAt(lemma: string, status: LemmaStatus | null, atMs: number): Promise<void> {
    (await this.engine()).setStatusAt(lemma, status ?? CLEAR, atMs);
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

  async setDeclaredLevel(level: CefrLevel | null): Promise<void> {
    (await this.engine()).setDeclaredLevel(level ?? "");
  }

  async declaredLevel(): Promise<CefrLevel | null> {
    return ((await this.engine()).declaredLevel() as CefrLevel | undefined) ?? null;
  }

  async hasLevels(): Promise<boolean> {
    return (await this.engine()).hasLevels();
  }

  async levelLadder(): Promise<LevelRow[]> {
    return JSON.parse((await this.engine()).levelLadder()) as LevelRow[];
  }

  async recordExposures(lemmas: string[], source: string, atMs: number): Promise<void> {
    (await this.engine()).recordExposures(lemmas, source, atMs);
  }

  async promoteByExposure(thresholdDays: number, atMs: number): Promise<number> {
    return (await this.engine()).promoteByExposure(thresholdDays, atMs);
  }

  async seedLevel(level: CefrLevel, count: number, order: SeedOrder, at: number): Promise<number> {
    return (await this.engine()).seedLevel(level, count, order, at);
  }
}
