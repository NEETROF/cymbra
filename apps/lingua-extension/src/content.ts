import { createLinguaPort } from "./analyzer/create-port.ts";
import type { LinguaPort } from "./analyzer/port.ts";
import type { TokenClass } from "./analyzer/types.ts";
import { type Block, collectBlocks } from "./reading/blocks.ts";
import { Drawer } from "./reading/drawer.ts";
import { clear as clearHighlights, injectPageStyles, render } from "./reading/highlight.ts";
import { ReadingObservers } from "./reading/observer.ts";
import { findTokenAt, type ResolvedToken, resolveTokens, type ScanStats, statsFromAnalysis } from "./reading/scan.ts";
import { captureSelection, sentenceAround } from "./reading/selection.ts";
import { type Gesture, WordPopup } from "./reading/wordpopup.ts";
import { type AsyncStorageArea, hydrateEngine, ROOT_KEY, saveBackup } from "./state/storage.ts";
import drawerCss from "./styles/drawer.css";
import popupCss from "./styles/wordpopup.css";
import reviewCss from "./styles/review.css";
import tokensCss from "./styles/tokens.css";

// Content-script controller. Owns the analysis pass, the highlight paint, the word
// popup, the review drawer and the page↔storage plumbing. The authoritative state is
// the engine's lingua-core LinguaState, persisted as its backup string in
// chrome.storage.local; every context restores from it, so a gesture (or a review in
// the side panel/drawer) repaints every other via storage.onChanged. The DOM is walked
// only for dirty subtrees; analysis over the (cheap) block text is whole-document so
// the language gate and the percentage stay page-correct.

const storageArea: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

const nowSeconds = (): number => Math.floor(Date.now() / 1000);

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

const NOT_ANALYSABLE: ScanStats = statsFromAnalysis({
  analyzer_version: "",
  analysable: false,
  tokens: [],
  counted: 0,
  known: 0,
  percent: null,
});

class ReadingSession {
  private readonly port: LinguaPort = createLinguaPort();
  private readonly blocksByContainer = new Map<Element, Block>();
  private resolved: ResolvedToken[] = [];
  private stats: ScanStats = NOT_ANALYSABLE;
  private calibration = 3000;
  private readonly popup: WordPopup;
  private readonly drawer: Drawer;
  private readonly observers: ReadingObservers;

  constructor() {
    this.popup = new WordPopup({ css: `${tokensCss}\n${popupCss}`, onGesture: (g) => void this.onGesture(g) });
    this.drawer = new Drawer({
      css: `${tokensCss}\n${reviewCss}\n${drawerCss}`,
      port: this.port,
      now: nowSeconds,
      onChange: () => this.persist(),
    });
    this.observers = new ReadingObservers({ onRescan: (containers) => void this.refresh(containers) });
  }

  async start(): Promise<void> {
    injectPageStyles(tokensCss);
    await hydrateEngine(this.port, storageArea);
    this.calibration = await this.port.calibration();
    await this.refresh([document.body]);
    this.observers.start();
    document.addEventListener("click", (e) => this.onClick(e), true);
    chrome.storage.onChanged.addListener((changes, areaName) => {
      const root = changes[ROOT_KEY];
      if (areaName === "local" && root && typeof (root.newValue as { backup?: string })?.backup === "string") {
        void this.onExternalChange((root.newValue as { backup: string }).backup);
      }
    });
    chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
      if (msg?.type === "captureSelection") void this.onCaptureSelection();
      else if (msg?.type === "toggleDrawer") void this.drawer.toggle();
      else if (msg?.type === "setCalibration") void this.onSetCalibration(Number(msg.value));
      else if (msg?.type === "reset") void this.onReset();
      else if (msg?.type === "getStats") {
        void this.statsMessage().then(sendResponse);
        return true; // async response
      }
      return false;
    });
  }

  private async persist(): Promise<void> {
    await saveBackup(storageArea, await this.port.backup());
  }

  /** Re-walk the given dirty containers, then repaint from a fresh whole-doc analysis. */
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

  /** Re-analyse current block text (no DOM walk) and repaint. */
  private async repaint(): Promise<void> {
    const blocks = [...this.blocksByContainer.values()];
    if (blocks.length === 0) {
      this.resolved = [];
      this.stats = NOT_ANALYSABLE;
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
      rarity: rarityText(hit.token.class, this.calibration),
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
    if (g.status === "learning") {
      const gloss = (await this.port.gloss(key)) ?? null;
      await this.port.addCard({
        lemma: key,
        surface: g.surface,
        sentence: g.sentence,
        url: location.href,
        gloss,
        capturedAt: nowSeconds(),
      });
    } else {
      await this.port.setStatus(key, g.status);
    }
    await this.persist();
    await this.repaint();
  }

  private async onExternalChange(backup: string): Promise<void> {
    await this.port.restore(backup);
    this.calibration = await this.port.calibration();
    await this.repaint();
  }

  private async onSetCalibration(value: number): Promise<void> {
    if (!Number.isFinite(value)) return;
    await this.port.setCalibration(value);
    this.calibration = value;
    await this.persist();
    await this.repaint();
  }

  private async onReset(): Promise<void> {
    await this.port.reset();
    await this.port.setCalibration(3000);
    this.calibration = 3000;
    await this.persist();
    await this.repaint();
  }

  private async statsMessage() {
    const now = nowSeconds();
    return {
      analysable: this.stats.analysable,
      percent: this.stats.percent,
      counted: this.stats.counted,
      unknownOccurrences: this.stats.unknownOccurrences,
      distinctUnknown: this.stats.distinctUnknown,
      calibration: this.calibration,
      trackedCount: await this.port.trackedCount(),
      deckCount: await this.port.deckCount(),
      dueCount: await this.port.dueCount(now),
    };
  }
}

// The reader can arrive two ways — a registered content script (after the <all_urls>
// grant) or an activeTab executeScript from the popup — so guard against running twice.
const GUARD = "__cymbraLinguaReading";

async function bootstrap(): Promise<void> {
  const w = window as unknown as Record<string, boolean>;
  if (w[GUARD]) return;
  w[GUARD] = true;
  await new ReadingSession().start();
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => void bootstrap());
} else {
  void bootstrap();
}
