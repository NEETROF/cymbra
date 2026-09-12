import { resolveContentPort } from "./analyzer/create-port.ts";
import type { LinguaPort } from "./analyzer/port.ts";
import type { TokenClass } from "./analyzer/types.ts";
import { type Block, collectBlocks } from "./reading/blocks.ts";
import { Drawer } from "./reading/drawer.ts";
import { clear as clearHighlights, injectPageStyles, render } from "./reading/highlight.ts";
import { ReadingObservers } from "./reading/observer.ts";
import { findTokenAt, type ResolvedToken, resolveTokens, type ScanStats, statsFromAnalysis } from "./reading/scan.ts";
import { captureSelection, MAX_SELECTION_LENGTH, sentenceAround } from "./reading/selection.ts";
import { type Gesture, WordPopup } from "./reading/wordpopup.ts";
import { dailyRecorder, recordExposures, recordWordLearned, utcDay } from "./state/dailystats.ts";
import {
  type AsyncStorageArea,
  ENABLED_KEY,
  hydrateEngine,
  loadEnabled,
  ROOT_KEY,
  saveBackup,
} from "./state/storage.ts";
import { clearSyncCursors } from "./sync/sync.ts";
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
  private readonly blocksByContainer = new Map<Element, Block>();
  private resolved: ResolvedToken[] = [];
  private stats: ScanStats = NOT_ANALYSABLE;
  private calibration = 3000;
  /** Global master switch. When off the reader does not analyse, paint or pop up. */
  private enabled = true;
  /** Daily exposures are counted once per page load (a re-scan does not re-count). */
  private exposuresRecorded = false;
  private readonly popup: WordPopup;
  private readonly drawer: Drawer;
  private readonly observers: ReadingObservers;

  // The port is resolved before construction (`resolveContentPort`) so a CSP-blocked
  // page can hand us the messaging port instead of the in-content WASM engine.
  constructor(private readonly port: LinguaPort) {
    this.popup = new WordPopup({ css: `${tokensCss}\n${popupCss}`, onGesture: (g) => void this.onGesture(g) });
    this.drawer = new Drawer({
      css: `${tokensCss}\n${reviewCss}\n${drawerCss}`,
      port: this.port,
      now: nowSeconds,
      onChange: () => this.persist(),
      record: dailyRecorder(storageArea),
    });
    this.observers = new ReadingObservers({ onRescan: (containers) => void this.refresh(containers) });
  }

  async start(): Promise<void> {
    await hydrateEngine(this.port, storageArea);
    this.calibration = await this.port.calibration();
    this.enabled = await loadEnabled(storageArea);
    document.addEventListener("click", (e) => this.onClick(e), true);
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && this.popup.visible()) this.popup.hide();
    });
    // A multi-word mouse selection opens the whole-selection card directly (Alt+L too).
    document.addEventListener("mouseup", (e) => this.onMouseUp(e));
    chrome.storage.onChanged.addListener((changes, areaName) => {
      if (areaName !== "local") return;
      const root = changes[ROOT_KEY];
      if (root && typeof (root.newValue as { backup?: string })?.backup === "string") {
        void this.onExternalChange((root.newValue as { backup: string }).backup);
      }
      const toggled = changes[ENABLED_KEY];
      if (toggled) void this.onEnabledChange(toggled.newValue !== false);
    });
    chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
      if (msg?.type === "captureSelection") void this.onCaptureSelection();
      else if (msg?.type === "toggleDrawer") void this.drawer.toggle();
      else if (msg?.type === "setCalibration") void this.onSetCalibration(Number(msg.value));
      else if (msg?.type === "reset") void this.onReset(msg.scope === "partial" ? "partial" : "full");
      else if (msg?.type === "getStats") {
        void this.statsMessage().then(sendResponse);
        return true; // async response
      }
      return false;
    });

    if (this.enabled) await this.activate();
    else this.pushDisabledBadge();
  }

  /** Paint the page and begin watching it for changes (the reader's "on" state). */
  private async activate(): Promise<void> {
    injectPageStyles(tokensCss);
    await this.refresh([document.body]);
    this.observers.start();
  }

  /** React to the global toggle flipping in another context (popup, other tab). */
  private async onEnabledChange(enabled: boolean): Promise<void> {
    if (enabled === this.enabled) return;
    this.enabled = enabled;
    if (enabled) {
      await this.activate();
    } else {
      this.observers.stop();
      this.popup.hide();
      this.resolved = [];
      clearHighlights();
      this.pushDisabledBadge();
    }
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
    if (!this.enabled) {
      clearHighlights();
      return;
    }
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
    // Count studied-word exposures once per page load (§3 daily stats).
    if (this.stats.analysable && !this.exposuresRecorded && this.stats.counted > 0) {
      this.exposuresRecorded = true;
      void recordExposures(storageArea, utcDay(Date.now()), this.stats.counted);
    }
    // Re-assert the token sheet before painting: a single-page-app navigation
    // (GitHub's morphing) can strip our injected styles, which leaves highlights
    // unpainted even though clicks still resolve. injectPageStyles is idempotent
    // and self-healing, so this restores them on the first paint after a nav.
    injectPageStyles(tokensCss);
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

  /** Clear the badge for this tab: the reader is switched off here. */
  private pushDisabledBadge(): void {
    try {
      chrome.runtime.sendMessage({ type: "stats", pct: null, disabled: true });
    } catch {
      // The service worker may be asleep; nothing to show while disabled anyway.
    }
  }

  private onClick(e: MouseEvent): void {
    if (!this.enabled || this.popup.contains(e.target)) return;
    // A live phrase/compound selection is the whole-selection card's job (onMouseUp);
    // don't also open the single-word popup for whatever word the release landed on.
    const sel = window.getSelection();
    if (sel && !sel.isCollapsed && /[-\s]/.test(String(sel).trim())) return;
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

  /** A phrase or compound mouse selection opens the whole-selection card directly. A
   *  selection counts as one when it holds a space or a hyphen (so "repo-wide" is taken
   *  whole, not reduced to the word the release landed on). */
  private onMouseUp(e: MouseEvent): void {
    if (!this.enabled || this.popup.contains(e.target)) return;
    const sel = window.getSelection();
    const text = sel ? String(sel).trim().replace(/\s+/g, " ") : "";
    if (sel && !sel.isCollapsed && /[-\s]/.test(text) && text.length <= MAX_SELECTION_LENGTH) {
      void this.onCaptureSelection();
    }
  }

  private async onCaptureSelection(): Promise<void> {
    if (!this.enabled) return;
    const cap = captureSelection();
    if (!cap) return;
    const isPhrase = cap.text.includes(" ");
    const gloss = isPhrase ? null : ((await this.port.gloss(cap.text.toLowerCase())) ?? null);
    this.popup.show({
      headword: cap.text,
      surface: cap.text,
      gloss,
      rarity: isPhrase ? "Expression — la carte gardera sa phrase d’origine." : "Sélection.",
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
      // Stamp the change so it orders correctly in cross-device sync (LWW).
      await this.port.setStatusAt(key, g.status, Date.now());
      if (g.status === "known") void recordWordLearned(storageArea, utcDay(Date.now()));
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

  /**
   * `full` wipes everything (statuses, exposure, deck + FSRS); `partial` clears
   * statuses/calibration/level but KEEPS the deck and exposure. Both restore the
   * default calibration. The popup gates this behind an explicit scope choice and
   * a confirmation, so a single stray click can never wipe a deck.
   *
   * A `full` reset also clears the sync cursors, so the next sync re-pulls the
   * whole server state: for a signed-in user the statuses + deck re-download
   * (a repair), rather than being gone. A `partial` reset leaves the cursors
   * alone — it is a deliberate local clear of statuses, not a re-pull.
   */
  private async onReset(scope: "full" | "partial"): Promise<void> {
    if (scope === "partial") {
      await this.port.resetStatuses();
    } else {
      await this.port.reset();
      await clearSyncCursors(storageArea);
    }
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
  try {
    const port = await resolveContentPort();
    await new ReadingSession(port).start();
  } catch (e) {
    // Surface a legible failure rather than dying as a silent unhandled rejection,
    // and allow a retry on the next injection.
    console.error("[Cymbra Lingua] reader failed to start:", e);
    w[GUARD] = false;
  }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => void bootstrap());
} else {
  void bootstrap();
}
