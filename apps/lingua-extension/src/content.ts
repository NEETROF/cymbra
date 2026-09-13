import { resolveContentPort } from "./analyzer/create-port.ts";
import type { LinguaPort } from "./analyzer/port.ts";
import type { CefrLevel, LemmaStatus, TokenClass } from "./analyzer/types.ts";
import { type Block, collectBlocks } from "./reading/blocks.ts";
import { Drawer } from "./reading/drawer.ts";
import { clear as clearHighlights, injectPageStyles, render } from "./reading/highlight.ts";
import { ExposureTracker } from "./reading/exposure-tracker.ts";
import { ReadingObservers } from "./reading/observer.ts";
import {
  type BlockTokens,
  clickableByContainer,
  findTokenAt,
  findTokenInBlock,
  lemmasByContainer,
  type ResolvedToken,
  resolveTokens,
  type ScanStats,
  statsFromAnalysis,
} from "./reading/scan.ts";
import { LinguaHud } from "./reading/hud.ts";
import { captureSelection, MAX_SELECTION_LENGTH, sentenceAround } from "./reading/selection.ts";
import { type Gesture, WordPopup } from "./reading/wordpopup.ts";
import { dailyRecorder, recordExposures, recordWordLearned, utcDay } from "./state/dailystats.ts";
import {
  type AsyncStorageArea,
  ENABLED_KEY,
  HUD_HIDDEN_KEY,
  hydrateEngine,
  loadEnabled,
  loadHudHidden,
  ROOT_KEY,
  saveBackup,
  saveHudHidden,
} from "./state/storage.ts";
import { clearSyncCursors } from "./sync/sync.ts";
import drawerCss from "./styles/drawer.css";
import hudCss from "./styles/hud.css";
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

/** The reader's current status for a token class, or null for a new/unknown word — drives
 *  which actions the popup offers when a word is reopened. */
function statusOfClass(cls: TokenClass): LemmaStatus | null {
  switch (cls) {
    case "Known":
      return "known";
    case "Ignored":
      return "ignored";
    case "Learning":
      return "learning";
    default:
      return null;
  }
}

/** Hold duration (ms) that turns a single-finger press into a reclassify long-press. */
const LONG_PRESS_MS = 500;
/** Finger travel (px) beyond which a press is treated as a scroll/drag, not a long-press. */
const LONG_PRESS_MOVE_TOL = 10;

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

/** Distinct read-days after which a below-level presumed word is confirmed known. */
const EXPOSURE_PROMOTE_DAYS = 4;
/** Debounce before flushing accumulated reading exposures to the engine. */
const EXPOSURE_FLUSH_MS = 2000;

class ReadingSession {
  private readonly blocksByContainer = new Map<Element, Block>();
  private resolved: ResolvedToken[] = [];
  /** Per-container block+tokens, for the Alt-click reclassify path of non-painted words. */
  private clickable = new Map<Element, BlockTokens>();
  /** In-flight long-press candidate (touch equivalent of Alt-click), or null. */
  private longPress: { x: number; y: number; timer: ReturnType<typeof setTimeout> } | null = null;
  /** True between a fired long-press and its touchend, so the compat click is swallowed. */
  private longPressFired = false;
  /** Ignore synthesised clicks until this epoch-ms (a long-press's compat click). */
  private suppressClickUntil = 0;
  /** While true, suppress the native long-press text-selection + context menu. */
  private suppressSelection = false;
  private stats: ScanStats = NOT_ANALYSABLE;
  private calibration = 3000;
  /** Cached CEFR facts for the HUD, so a repaint needs no extra port round-trips: whether
   *  the pack has levels (constant) and the declared level (refreshed where it changes). */
  private hasLevels = false;
  private declaredLevel: CefrLevel | null = null;
  /** Whether the reader hid the in-page HUD pill (persisted, toggled from the popup). */
  private hudHidden = false;
  /** Global master switch. When off the reader does not analyse, paint or pop up. */
  private enabled = true;
  /** Daily exposures are counted once per page load (a re-scan does not re-count). */
  private exposuresRecorded = false;
  private readonly popup: WordPopup;
  private readonly drawer: Drawer;
  private readonly hud: LinguaHud;
  private readonly observers: ReadingObservers;
  /** Viewport-gated reading exposure (slice 5c): lemmas whose block was actually read. */
  private readonly exposure: ExposureTracker;
  private readonly pendingExposure = new Set<string>();
  private exposureFlushTimer: ReturnType<typeof setTimeout> | null = null;
  /** The last backup we wrote, to ignore our own storage.onChanged echo. */
  private lastBackup: string | null = null;

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
    this.hud = new LinguaHud({
      css: `${tokensCss}\n${hudCss}`,
      actions: {
        onReview: () => void this.drawer.toggle(),
        onCapture: () => void this.onCaptureSelection(),
        onStats: () => this.openStats(),
        onSetLevel: (level) => void this.onSetLevel(level),
        onHide: () => void this.hideHud(),
      },
    });
    this.observers = new ReadingObservers({ onRescan: (containers) => void this.refresh(containers) });
    this.exposure = new ExposureTracker((lemmas) => this.onExposed(lemmas));
  }

  async start(): Promise<void> {
    await hydrateEngine(this.port, storageArea);
    this.calibration = await this.port.calibration();
    this.hasLevels = await this.port.hasLevels();
    this.declaredLevel = await this.port.declaredLevel();
    this.hudHidden = await loadHudHidden(storageArea);
    this.enabled = await loadEnabled(storageArea);
    document.addEventListener("click", (e) => this.onClick(e), true);
    // Long-press = the touch equivalent of Alt-click (reopen a marked word). touchstart/move/
    // cancel stay passive (they only arm/cancel the timer); touchend is non-passive so a
    // fired press can swallow its compatibility click. selectstart/contextmenu are suppressed
    // only while a long-press is firing, to kill the native text-selection + callout it raises.
    document.addEventListener("touchstart", (e) => this.onTouchStart(e), { capture: true, passive: true });
    document.addEventListener("touchmove", (e) => this.onTouchMove(e), { capture: true, passive: true });
    document.addEventListener("touchend", (e) => this.onTouchEnd(e), { capture: true });
    document.addEventListener("touchcancel", () => this.onTouchCancel(), { capture: true, passive: true });
    document.addEventListener(
      "selectstart",
      (e) => {
        if (this.suppressSelection) e.preventDefault();
      },
      true,
    );
    document.addEventListener(
      "contextmenu",
      (e) => {
        if (this.suppressSelection) e.preventDefault();
      },
      true,
    );
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
      const hudToggled = changes[HUD_HIDDEN_KEY];
      if (hudToggled) {
        this.hudHidden = hudToggled.newValue === true;
        this.syncHud();
      }
    });
    // Flush pending reading exposures before the tab is hidden / navigated away.
    document.addEventListener("visibilitychange", () => {
      if (document.visibilityState === "hidden" && this.pendingExposure.size > 0) void this.flushExposure();
    });
    // The word popup is position:fixed and anchored to a word's box; a scroll detaches it
    // (and near the page bottom it could sit half-off-screen). Dismiss it on scroll — a
    // re-click reopens it correctly placed. Capture so nested scroll containers count too.
    window.addEventListener(
      "scroll",
      () => {
        if (this.popup.visible()) this.popup.hide();
      },
      { capture: true, passive: true },
    );
    chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
      // The mutating commands acknowledge only AFTER their repaint has settled
      // `this.stats`, so the popup's follow-up `getStats` reads the new
      // percentage — not the pre-change one it would catch if we acked eagerly.
      if (msg?.type === "captureSelection") void this.onCaptureSelection();
      else if (msg?.type === "toggleDrawer") void this.drawer.toggle();
      else if (msg?.type === "setCalibration") {
        void this.onSetCalibration(Number(msg.value)).then(() => sendResponse(true));
        return true;
      } else if (msg?.type === "setLevel") {
        void this.onSetLevel((msg.value as CefrLevel) || null).then(() => sendResponse(true));
        return true;
      } else if (msg?.type === "reset") {
        void this.onReset(msg.scope === "partial" ? "partial" : "full").then(() => sendResponse(true));
        return true;
      } else if (msg?.type === "getStats") {
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
    this.hud.mount();
    this.syncHud();
    await this.refresh([document.body]);
    this.observers.start();
    this.exposure.start();
  }

  /** Reflect the HUD's current visibility: shown only while enabled and not user-hidden. */
  private syncHud(): void {
    this.hud.setHidden(!this.enabled || this.hudHidden);
  }

  /** Push the current reading state into the in-page HUD pill. */
  private updateHud(): void {
    this.hud.update({
      analysable: this.stats.analysable,
      percent: this.stats.percent,
      hasLevels: this.hasLevels,
      declaredLevel: this.declaredLevel,
    });
  }

  /** Open the learning statistics (background opens the page — no popup needed). */
  private openStats(): void {
    try {
      chrome.runtime.sendMessage({ type: "openStats" });
    } catch {
      // The service worker may be asleep; the user can retry.
    }
  }

  /** Hide the in-page HUD (persisted; the popup offers to show it again). */
  private async hideHud(): Promise<void> {
    this.hudHidden = true;
    await saveHudHidden(storageArea, true);
    this.syncHud();
  }

  /** React to the global toggle flipping in another context (popup, other tab). */
  private async onEnabledChange(enabled: boolean): Promise<void> {
    if (enabled === this.enabled) return;
    this.enabled = enabled;
    if (enabled) {
      await this.activate();
    } else {
      this.observers.stop();
      this.exposure.stop();
      if (this.exposureFlushTimer !== null) clearTimeout(this.exposureFlushTimer);
      this.exposureFlushTimer = null;
      this.pendingExposure.clear();
      this.popup.hide();
      this.resolved = [];
      this.clickable.clear();
      clearHighlights();
      this.syncHud();
      this.pushDisabledBadge();
    }
  }

  private async persist(): Promise<void> {
    const backup = await this.port.backup();
    this.lastBackup = backup; // so our own storage.onChanged echo is ignored
    await saveBackup(storageArea, backup);
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
      this.clickable.clear();
      this.stats = NOT_ANALYSABLE;
      clearHighlights();
      this.pushBadge();
      this.updateHud();
      return;
    }
    const analysis = await this.port.analyse(blocks.map((b) => b.text));
    this.resolved = resolveTokens(blocks, analysis);
    this.clickable = clickableByContainer(blocks, analysis);
    this.stats = statsFromAnalysis(analysis);
    // Track which blocks the reader actually sees, to confirm below-level words by reading.
    if (this.stats.analysable) this.exposure.track(lemmasByContainer(blocks, analysis));
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
    this.updateHud();
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
    if (Date.now() < this.suppressClickUntil) {
      // The compatibility click a long-press just fired: swallow it whole, so a long-pressed
      // link cannot navigate on the fallback click either.
      e.preventDefault();
      e.stopPropagation();
      return;
    }
    // A live phrase/compound selection is the whole-selection card's job (onMouseUp);
    // don't also open the single-word popup for whatever word the release landed on.
    const sel = window.getSelection();
    if (sel && !sel.isCollapsed && /[-\s]/.test(String(sel).trim())) return;
    const caret = caretAt(e.clientX, e.clientY);
    if (!caret) {
      if (this.popup.visible()) this.popup.hide();
      return;
    }
    // A plain click resolves only PAINTED words; a non-painted (Known/Ignored) word needs the
    // Alt/Option modifier, so a plain click never intercepts one (the page keeps it).
    const hit = this.hitAt(caret.node, caret.offset, e.altKey);
    if (!hit) {
      if (this.popup.visible()) this.popup.hide();
      return;
    }
    const isLink = e.target instanceof Element && !!e.target.closest("a[href]");
    // Only an UNTREATED word (Unknown) blocks its link — "tant qu'un mot n'a pas été traité".
    // A treated (Learning/decked) word that is a link follows the link on a plain click; its
    // popup stays reachable via Alt-click / long-press. Alt-click always reclassifies.
    if (!e.altKey && hit.token.class === "Learning" && isLink) return;
    this.showPopup(hit);
    e.stopPropagation();
    // Suppress the default ONLY to block an untreated word's link, or for a deliberate
    // Alt-click — never otherwise, so a painted word inside a <label>/<summary>/<button>
    // keeps its native activation.
    if (e.altKey || isLink) e.preventDefault();
  }

  /**
   * Long-press = the touch equivalent of Alt-click: press-and-hold ONE finger on a word to
   * reopen it (painted or a marked Known/Ignored word) and reclassify. One finger is
   * precise (a word is a small target); two fingers can't sit on one word. The catch is
   * that a long-press also raises the OS text-selection + callout, which `fireLongPress`
   * suppresses. Cancelled by movement, a second finger, an early lift, or touchcancel.
   */
  private onTouchStart(e: TouchEvent): void {
    this.cancelLongPress();
    if (!this.enabled || this.popup.contains(e.target) || e.touches.length !== 1) return;
    const t = e.touches[0];
    if (!t) return;
    const x = t.clientX;
    const y = t.clientY;
    this.longPress = { x, y, timer: setTimeout(() => this.fireLongPress(x, y), LONG_PRESS_MS) };
  }

  private onTouchMove(e: TouchEvent): void {
    const lp = this.longPress;
    if (!lp) return;
    const t = e.touches[0];
    // A moved finger is a scroll/drag, and a second finger is a pinch — neither is a press.
    if (e.touches.length !== 1 || !t || Math.hypot(t.clientX - lp.x, t.clientY - lp.y) > LONG_PRESS_MOVE_TOL) {
      this.cancelLongPress();
    }
  }

  private onTouchEnd(e: TouchEvent): void {
    if (this.longPressFired) {
      // The press already opened the popup; swallow the compatibility click this touchend
      // would synthesise (else it would re-hit onClick and, for a marked word, dismiss it).
      this.longPressFired = false;
      e.preventDefault();
      e.stopPropagation();
    }
    this.cancelLongPress(); // an early lift (before the timer) is a normal tap, not a press
  }

  private onTouchCancel(): void {
    this.longPressFired = false;
    this.cancelLongPress();
  }

  private cancelLongPress(): void {
    if (this.longPress) {
      clearTimeout(this.longPress.timer);
      this.longPress = null;
    }
  }

  /** The hold completed: open the word's popup and suppress the native selection/callout. */
  private fireLongPress(x: number, y: number): void {
    this.longPress = null;
    if (!this.enabled || !this.openWordAt(x, y, true)) return;
    this.longPressFired = true; // touchend will swallow the compat click
    this.suppressClickUntil = Date.now() + 700; // fallback if the browser clicks anyway
    this.suppressSelection = true; // kill the long-press text-selection + context menu
    setTimeout(() => {
      this.suppressSelection = false;
    }, 1000);
    window.getSelection()?.removeAllRanges();
  }

  /**
   * The token at a caret: a PAINTED word (Learning/Unknown) directly, or — when
   * `allowReclassify` (the deliberate Alt-click / long-press path) — a non-painted
   * (Known/Ignored) word via an on-demand block hit-test. Null when nothing is there.
   */
  private hitAt(node: Node, offset: number, allowReclassify: boolean): ResolvedToken | null {
    return findTokenAt(this.resolved, node, offset) ?? (allowReclassify ? this.reclassifyHit(node, offset) : null);
  }

  /** Show the word popup for a resolved hit (status-aware actions), anchored to its box. */
  private showPopup(hit: ResolvedToken): void {
    const rect = hit.range.getBoundingClientRect();
    this.popup.show({
      headword: hit.token.lemma,
      surface: hit.token.surface,
      gloss: hit.token.gloss,
      rarity: rarityText(hit.token.class, this.calibration),
      sentence: sentenceAround(hit.range.startContainer, hit.token.surface),
      status: statusOfClass(hit.token.class),
      rect: { left: rect.left, top: rect.top, bottom: rect.bottom },
    });
  }

  /** Open the popup for the word at viewport (x, y) (the long-press path always reclassifies).
   *  Returns whether a word was found; dismisses any open popup on a miss. */
  private openWordAt(x: number, y: number, allowReclassify: boolean): boolean {
    const caret = caretAt(x, y);
    const hit = caret && this.hitAt(caret.node, caret.offset, allowReclassify);
    if (!hit) {
      if (this.popup.visible()) this.popup.hide();
      return false;
    }
    this.showPopup(hit);
    return true;
  }

  /** Hit-test a non-painted word (Known/Ignored) by resolving only its own block's tokens.
   *  Block+tokens come paired from the same analysis, so a since-changed DOM yields a clean
   *  miss (ranges over old nodes), never a wrong-word hit. */
  private reclassifyHit(node: Node, offset: number): ResolvedToken | null {
    let el: Element | null = node instanceof Element ? node : node.parentElement;
    while (el && !this.clickable.has(el)) el = el.parentElement;
    const entry = el && this.clickable.get(el);
    if (!entry) return null;
    return findTokenInBlock(entry.block, entry.tokens, node, offset);
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
      // Promoting a word that was in the deck (learning) to known/ignored must retire its
      // card so it stops coming due — a word you now treat as known/ignored shouldn't keep
      // being reviewed. No-op when there is no card. (Clearing → "à apprendre" keeps it.)
      if (g.status === "known" || g.status === "ignored") await this.port.retireCard(key, nowSeconds());
    }
    await this.persist();
    await this.repaint();
  }

  private async onExternalChange(backup: string): Promise<void> {
    if (backup === this.lastBackup) return; // our own write echoed back — nothing to do
    await this.port.restore(backup);
    this.calibration = await this.port.calibration();
    this.declaredLevel = await this.port.declaredLevel(); // a sync may have changed the level
    await this.repaint();
  }

  /** A block was read (visible past the dwell): queue its lemmas and throttle a flush. */
  private onExposed(lemmas: string[]): void {
    for (const lemma of lemmas) this.pendingExposure.add(lemma);
    if (this.exposureFlushTimer !== null) return; // a flush is already scheduled
    this.exposureFlushTimer = setTimeout(() => void this.flushExposure(), EXPOSURE_FLUSH_MS);
  }

  /**
   * Record the read lemmas as exposures, then confirm any below-level presumed word
   * that has now been read on enough distinct days. Off the render path (throttled);
   * only repaints when a promotion actually changed a word's status.
   */
  private async flushExposure(): Promise<void> {
    this.exposureFlushTimer = null;
    if (this.pendingExposure.size === 0) return;
    const lemmas = [...this.pendingExposure];
    this.pendingExposure.clear();
    const now = Date.now();
    await this.port.recordExposures(lemmas, `reading:${location.hostname}`, now);
    const promoted = await this.port.promoteByExposure(EXPOSURE_PROMOTE_DAYS, now);
    await this.persist();
    if (promoted > 0) await this.repaint(); // words became known → refresh highlights
  }

  private async onSetCalibration(value: number): Promise<void> {
    if (!Number.isFinite(value)) return;
    await this.port.setCalibration(value);
    this.calibration = value;
    await this.persist();
    await this.repaint();
  }

  /**
   * Declare (or clear, with `null` = "débutant / from zero") the reader's CEFR
   * level. With a level in play the frequency calibration must presume nothing —
   * the level is the only source of presumed-known (design: no silent
   * presumption) — so it is pinned to 0. Below-level words then stop being
   * highlighted; at/above stay highlighted.
   */
  private async onSetLevel(level: CefrLevel | null): Promise<void> {
    // Stamp the decision so it wins cross-device last-write-wins when it syncs.
    await this.port.setDeclaredLevelAt(level, Date.now());
    await this.port.setCalibration(0);
    this.calibration = 0;
    this.declaredLevel = level;
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
    // Option B: with CEFR data, presume nothing until the reader picks a level —
    // keep frequency calibration at 0 and let the popup re-prompt for a level.
    // Without CEFR data, restore the default frequency calibration.
    const cal = (await this.port.hasLevels()) ? 0 : 3000;
    await this.port.setCalibration(cal);
    this.calibration = cal;
    this.declaredLevel = await this.port.declaredLevel(); // a reset clears the declared level
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
      declaredLevel: await this.port.declaredLevel(),
      hasLevels: await this.port.hasLevels(),
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
