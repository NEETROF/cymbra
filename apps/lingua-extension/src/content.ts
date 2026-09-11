import { WasmAnalyzerPort } from "./analyzer/engine.ts";
import type { AnalyzerPort } from "./analyzer/port.ts";
import type { TokenClass } from "./analyzer/types.ts";
import { type Block, collectBlocks } from "./reading/blocks.ts";
import { clear as clearHighlights, injectPageStyles, render } from "./reading/highlight.ts";
import { ReadingObservers } from "./reading/observer.ts";
import { findTokenAt, type ResolvedToken, resolveTokens, type ScanStats, statsFromAnalysis } from "./reading/scan.ts";
import { captureSelection, sentenceAround } from "./reading/selection.ts";
import { type Gesture, WordPopup } from "./reading/wordpopup.ts";
import { type AsyncStorageArea, type LinguaState, loadState, migrate, ROOT_KEY, saveState } from "./state/storage.ts";
import { applyStatus, deck, knownCount } from "./state/status.ts";
import popupCss from "./styles/wordpopup.css";
import tokensCss from "./styles/tokens.css";

// Content-script controller. Owns the analysis pass, the highlight paint, the word
// popup and the page↔storage plumbing. The DOM is walked only for dirty subtrees on
// mutation; analysis over the (cheap) block text is whole-document so the language
// gate and the percentage stay page-correct. State lives in chrome.storage.local; a
// gesture in one tab repaints every other via storage.onChanged.

const storageArea: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

function caretAt(x: number, y: number): { node: Node; offset: number } | null {
  const doc = document as Document & {
    caretRangeFromPoint?: (x: number, y: number) => Range | null;
    caretPositionFromPoint?: (x: number, y: number) => { offsetNode: Node; offset: number } | null;
  };
  if (doc.caretRangeFromPoint) {
    const r = doc.caretRangeFromPoint(x, y);
    return r ? { node: r.startContainer, offset: r.startOffset } : null;
  }
  if (doc.caretPositionFromPoint) {
    const p = doc.caretPositionFromPoint(x, y);
    return p ? { node: p.offsetNode, offset: p.offset } : null;
  }
  return null;
}

function rarityText(cls: TokenClass, calibration: number): string {
  if (cls === "Learning") return "Dans ton deck — en cours d'apprentissage.";
  return `Peu fréquent — au-delà de tes ${calibration.toLocaleString("fr-FR")} mots les plus courants.`;
}

class ReadingSession {
  private readonly port: AnalyzerPort = new WasmAnalyzerPort();
  private readonly blocksByContainer = new Map<Element, Block>();
  private resolved: ResolvedToken[] = [];
  private stats: ScanStats = statsFromAnalysis({
    analyzer_version: "",
    analysable: false,
    tokens: [],
    counted: 0,
    known: 0,
    percent: null,
  });
  private state: LinguaState;
  private readonly popup: WordPopup;
  private readonly observers: ReadingObservers;

  constructor(state: LinguaState) {
    this.state = state;
    this.popup = new WordPopup({ css: `${tokensCss}\n${popupCss}`, onGesture: (g) => void this.onGesture(g) });
    this.observers = new ReadingObservers({ onRescan: (containers) => void this.refresh(containers) });
  }

  async start(): Promise<void> {
    injectPageStyles(tokensCss);
    await this.syncPort(this.state);
    await this.refresh([document.body]);
    this.observers.start();
    document.addEventListener("click", (e) => this.onClick(e), true);
    chrome.storage.onChanged.addListener((changes, areaName) => {
      if (areaName === "local" && changes[ROOT_KEY]) void this.onExternalChange(migrate(changes[ROOT_KEY].newValue));
    });
    chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
      if (msg?.type === "captureSelection") void this.onCaptureSelection();
      else if (msg?.type === "getStats") sendResponse(this.statsMessage());
      return false;
    });
  }

  /** Push all stored statuses + calibration into the engine's in-memory state. */
  private async syncPort(state: LinguaState): Promise<void> {
    await this.port.setCalibration(state.calibration);
    for (const [lemma, status] of Object.entries(state.statuses)) await this.port.setStatus(lemma, status);
  }

  /** Re-walk the given dirty containers, re-analyse the whole document, repaint. */
  private async refresh(dirtyRoots: Element[]): Promise<void> {
    for (const root of dirtyRoots) {
      for (const c of [...this.blocksByContainer.keys()]) {
        if (c === root || root.contains(c) || !c.isConnected) this.blocksByContainer.delete(c);
      }
      for (const b of collectBlocks(root)) this.blocksByContainer.set(b.container, b);
    }
    for (const c of [...this.blocksByContainer.keys()]) if (!c.isConnected) this.blocksByContainer.delete(c);
    await this.repaint();
    this.observers.track(this.blocksByContainer.keys());
  }

  /** Re-analyse current block text (no DOM walk) and repaint — used after a status change. */
  private async repaint(): Promise<void> {
    const blocks = [...this.blocksByContainer.values()];
    if (blocks.length === 0) {
      this.resolved = [];
      this.stats = statsFromAnalysis({
        analyzer_version: "",
        analysable: false,
        tokens: [],
        counted: 0,
        known: 0,
        percent: null,
      });
      clearHighlights();
      this.pushBadge();
      return;
    }
    const analysis = await this.port.analyse(blocks.map((b) => b.text));
    this.resolved = resolveTokens(blocks, analysis);
    this.stats = statsFromAnalysis(analysis);
    render(this.resolved);
    this.pushBadge();
  }

  private pushBadge(): void {
    try {
      chrome.runtime.sendMessage({ type: "stats", pct: this.stats.analysable ? this.stats.percent : null });
    } catch {
      // The service worker may be asleep; the badge refreshes on the next pass.
    }
  }

  private onClick(e: MouseEvent): void {
    if (this.popup.contains(e.target)) return;
    const caret = caretAt(e.clientX, e.clientY);
    if (!caret) {
      if (this.popup.visible()) this.popup.hide();
      return;
    }
    const hit = findTokenAt(this.resolved, caret.node, caret.offset);
    if (!hit) {
      if (this.popup.visible()) this.popup.hide();
      return;
    }
    const rect = hit.range.getBoundingClientRect();
    this.popup.show({
      headword: hit.token.lemma,
      surface: hit.token.surface,
      gloss: hit.token.gloss,
      rarity: rarityText(hit.token.class, this.state.calibration),
      sentence: sentenceAround(hit.range.startContainer, hit.token.surface),
      rect: { left: rect.left, bottom: rect.bottom },
    });
    e.stopPropagation();
  }

  private async onCaptureSelection(): Promise<void> {
    const cap = captureSelection();
    if (!cap) return;
    const isPhrase = cap.text.includes(" ");
    const gloss = isPhrase ? null : ((await this.port.gloss(cap.text.toLowerCase())) ?? null);
    this.popup.show({
      headword: cap.text,
      surface: cap.text,
      gloss,
      rarity: isPhrase ? "Expression — la phrase part avec la carte." : "Sélection.",
      sentence: cap.sentence,
      rect: cap.rect,
      expression: isPhrase,
    });
  }

  private async onGesture(g: Gesture): Promise<void> {
    const key = g.lemma.toLowerCase();
    this.state = applyStatus(this.state, key, g.status, {
      surface: g.surface,
      sentence: g.sentence,
      createdAt: Date.now(),
    });
    await saveState(storageArea, this.state);
    await this.port.setStatus(key, g.status);
    await this.repaint();
  }

  private async onExternalChange(next: LinguaState): Promise<void> {
    // Re-sync the engine to the other tab's statuses (added, changed and removed).
    const removed = Object.keys(this.state.statuses).filter((l) => !(l in next.statuses));
    for (const lemma of removed) await this.port.setStatus(lemma, null);
    this.state = next;
    await this.syncPort(next);
    await this.repaint();
  }

  private statsMessage() {
    return {
      analysable: this.stats.analysable,
      percent: this.stats.percent,
      counted: this.stats.counted,
      unknownOccurrences: this.stats.unknownOccurrences,
      distinctUnknown: this.stats.distinctUnknown,
      calibration: this.state.calibration,
      knownCount: knownCount(this.state),
      deckCount: deck(this.state).length,
    };
  }
}

// The reader can arrive two ways — a registered content script (after the <all_urls>
// grant) or an activeTab executeScript from the popup — so guard against running twice
// in one page.
const GUARD = "__cymbraLinguaReading";

async function bootstrap(): Promise<void> {
  const w = window as unknown as Record<string, boolean>;
  if (w[GUARD]) return;
  w[GUARD] = true;
  const state = await loadState(storageArea);
  const session = new ReadingSession(state);
  await session.start();
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => void bootstrap());
} else {
  void bootstrap();
}
