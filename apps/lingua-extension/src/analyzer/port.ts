import type { LemmaStatus, PageAnalysis } from "./types.ts";

// The AnalyzerPort seam (design D2). The content script consumes analysis
// exclusively through this interface, never touching the WASM module directly.
// In this Chromium change the implementation is the WASM engine instantiated in
// the content script (see engine.ts); the later Firefox (WASM in the event page)
// and Safari (nativeMessaging) variants provide a different implementation of the
// SAME interface, with no change to the reading code. Tests inject a fake.

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
