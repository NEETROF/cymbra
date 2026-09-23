import { createTranslatorPort } from "./translate/create-port.ts";
import { resolveContentPort } from "./analyzer/create-port.ts";
import type { LinguaPort } from "./analyzer/port.ts";
import { type CefrLevel, STUDIED_LANGUAGE } from "./analyzer/types.ts";
import { type Block, collectBlocks } from "./reading/blocks.ts";
import { Drawer, type DrawerView } from "./reading/drawer.ts";
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
import {
  type Capture,
  type CaptureKind,
  captureSelection,
  classifySelection,
  SelectionWatcher,
  sentenceForRange,
} from "./reading/selection.ts";
import { clickIsOnWord, decideClick, type PageHit, SelectionCards } from "./reading/selection-card.ts";
import { browserSpeechEngine, createSpeaker, type Speaker } from "./reading/speech.ts";
import { type Gesture, WordPopup } from "./reading/wordpopup.ts";
import { recordExposures, recordWordLearned, utcDay } from "./state/dailystats.ts";
import { needsLevelChoice } from "./state/level-choice.ts";
import {
  type AsyncStorageArea,
  ENABLED_KEY,
  HUD_HIDDEN_KEY,
  hydrateEngine,
  loadEnabled,
  loadHudHidden,
  saveBackup,
  SESSION_LOST_KEY,
  storedVoicePreference,
} from "./state/storage.ts";
import { messagedArea, watchBackup } from "./state/store.ts";
import { requestSync } from "./sync/messages.ts";
import { clearSyncCursors } from "./sync/sync.ts";
import drawerCss from "./styles/drawer.css";
import hudCss from "./styles/hud.css";
import popupCss from "./styles/wordpopup.css";
import reviewCss from "./styles/review.css";
import statsCss from "./stats/stats.css";
import settingsCss from "./styles/settings.css";
import tokensCss from "./styles/tokens.css";

// Content-script controller. Owns the analysis pass, the highlight paint, the word
// popup, the review drawer and the page↔storage plumbing. The authoritative state is
// the engine's lingua-core LinguaState, persisted as its backup string in
// chrome.storage.local; every context restores from it, so a gesture (or a review in
// the side panel/drawer) repaints every other via storage.onChanged. The DOM is walked
// only for dirty subtrees; analysis over the (cheap) block text is whole-document so
// the language gate and the percentage stay page-correct.

/** The reader's data (backup, statistics), owned by the background (design D1/D2). */
const store: AsyncStorageArea = messagedArea();

/** Preferences and marks, which every surface must read before any round-trip. */
const storageArea: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

const nowSeconds = (): number => Math.floor(Date.now() / 1000);

/** Coerce an untrusted message payload to a drawer view (defaults to review). */
function drawerView(v: unknown): DrawerView {
  return v === "stats" || v === "settings" ? v : "review";
}

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
  /** Debounces `selectionchange` into one capture, whatever the pointer. */
  private readonly selection: SelectionWatcher;
  private stats: ScanStats = NOT_ANALYSABLE;
  private calibration = 3000;
  /** Whether the reader hid the in-page HUD pill (persisted, toggled from the popup / panel). */
  private hudHidden = false;
  /** Global master switch. When off the reader does not analyse, paint or pop up. */
  private enabled = true;
  /** Daily exposures are counted once per page load (a re-scan does not re-count). */
  private exposuresRecorded = false;
  private readonly popup: WordPopup;
  /** Reads a card's selection and sentence aloud, with a voice on this device only. */
  private readonly speaker: Speaker = createSpeaker(
    browserSpeechEngine(),
    STUDIED_LANGUAGE,
    storedVoicePreference(storageArea),
  );
  /** What a selection or a click opens — every decision lives there, tested; this class only wires it. */
  private readonly cards: SelectionCards;
  private readonly drawer: Drawer;
  private readonly hud: LinguaHud;
  private readonly observers: ReadingObservers;
  /** Viewport-gated reading exposure (slice 5c): lemmas whose block was actually read. */
  private readonly exposure: ExposureTracker;
  private readonly pendingExposure = new Set<string>();
  private exposureFlushTimer: ReturnType<typeof setTimeout> | null = null;
  /** The last backup we wrote, to ignore our own storage.onChanged echo. */
  private lastBackup: string | null = null;
  /** No level declared for a CEFR language yet: the HUD offers the choice (see refreshNeedsLevel). */
  private needsLevel = false;

  // The port is resolved before construction (`resolveContentPort`) so a CSP-blocked
  // page can hand us the messaging port instead of the in-content WASM engine.
  constructor(private readonly port: LinguaPort) {
    this.popup = new WordPopup({
      css: `${tokensCss}\n${popupCss}`,
      onGesture: (g) => void this.onGesture(g),
      speaker: this.speaker,
    });
    this.cards = new SelectionCards(
      this.port,
      { show: (content) => this.popup.show(content), generation: () => this.popup.generation() },
      // The translator is none in every shipped build; a development build that side-loads a
      // model gets the messaging port, which sends the request off this thread.
      { calibration: () => this.calibration, translator: createTranslatorPort() },
    );
    this.drawer = new Drawer({
      css: `${tokensCss}\n${reviewCss}\n${statsCss}\n${settingsCss}\n${drawerCss}`,
      port: this.port,
      area: storageArea,
      store,
      now: nowSeconds,
      onChange: () => this.persist(),
      speaker: this.speaker,
    });
    this.hud = new LinguaHud({
      css: `${tokensCss}\n${hudCss}`,
      actions: {
        onReview: () => this.openReviewSurface("review"),
        onStats: () => this.openReviewSurface("stats"),
        onSettings: () => this.openReviewSurface("settings"),
      },
    });
    this.observers = new ReadingObservers({ onRescan: (containers) => void this.refresh(containers) });
    this.exposure = new ExposureTracker((lemmas) => this.onExposed(lemmas));
    this.selection = new SelectionWatcher({ onCapture: (kind, cap) => this.onCapture(kind, cap) });
  }

  async start(): Promise<void> {
    await hydrateEngine(this.port, store);
    this.calibration = await this.port.calibration();
    this.hudHidden = await loadHudHidden(storageArea);
    this.enabled = await loadEnabled(storageArea);
    await this.refreshNeedsLevel();
    document.addEventListener("click", (e) => this.onClick(e), true);
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && this.popup.visible()) this.popup.hide();
    });
    // The selection is the capture gesture, on every pointer: a mouse drag, shift-arrows and
    // the platform's own press-and-hold / handle drag all emit `selectionchange`. A pointer
    // lift only flushes the pending debounce early — it is not a second capture path, which
    // is what the touch-less `mouseup` wiring used to be. The reader adds no gesture of its
    // own here: on iOS, press-and-hold IS the selection gesture and cannot be shared. A
    // pointer going down also starts a gesture for the cards: the click that ends it must
    // leave alone the card the capture opened (`decideClick`).
    const pointer = { capture: true, passive: true } as const;
    const pointerDown = (): void => {
      this.cards.gestureStarted();
      this.selection.hold();
    };
    document.addEventListener("selectionchange", () => this.selection.notify(), { passive: true });
    document.addEventListener("mousedown", pointerDown, pointer);
    document.addEventListener("touchstart", pointerDown, pointer);
    document.addEventListener("mouseup", () => this.selection.release(), pointer);
    document.addEventListener("touchend", () => this.selection.release(), pointer);
    document.addEventListener("touchcancel", () => this.selection.release(), pointer);
    // The deck and statuses, when another surface changed them.
    watchBackup(store, (backup) => void this.onExternalChange(backup));
    chrome.storage.onChanged.addListener((changes, areaName) => {
      if (areaName !== "local") return;
      const toggled = changes[ENABLED_KEY];
      if (toggled) void this.onEnabledChange(toggled.newValue !== false);
      const hudToggled = changes[HUD_HIDDEN_KEY];
      if (hudToggled) {
        this.hudHidden = hudToggled.newValue === true;
        this.syncHud();
      }
      const lost = changes[SESSION_LOST_KEY];
      if (lost) this.drawer.setSessionLost(lost.newValue === true);
    });
    // Flush pending reading exposures before the tab is hidden / navigated away.
    document.addEventListener("visibilitychange", () => {
      if (document.visibilityState === "hidden" && this.pendingExposure.size > 0) void this.flushExposure();
      // Nothing keeps talking in a tab the reader left.
      if (document.visibilityState === "hidden") this.speaker.stop();
      // Back on the tab (possibly after reading on another device): ask for a sync too.
      if (document.visibilityState === "visible" && this.enabled) void requestSync("page");
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
      if (msg?.type === "captureSelection") this.onCaptureSelection();
      else if (msg?.type === "toggleDrawer") void this.drawer.toggle();
      else if (msg?.type === "openDrawer") void this.drawer.openOn(drawerView(msg.view));
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

    this.drawer.setSessionLost((await storageArea.get(SESSION_LOST_KEY))[SESSION_LOST_KEY] === true);

    if (this.enabled) await this.activate();
    else this.pushDisabledBadge();
  }

  /** Paint the page and begin watching it for changes (the reader's "on" state). */
  private async activate(): Promise<void> {
    injectPageStyles(tokensCss);
    await this.refresh([document.body]);
    // Mount the HUD only after a successful first paint, so a failed init (which resets
    // the injection guard and lets a retry create a fresh session) leaves no orphan host.
    this.hud.mount();
    this.syncHud();
    this.observers.start();
    this.exposure.start();
    // A page load asks for a sync; the background runs at most one a minute.
    void requestSync("page");
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
      needsLevel: this.needsLevel,
    });
  }

  /** Open a panel view, staying in the page as much as possible:
   *   - Firefox and Safari: the in-page drawer (a page element cannot open Firefox's
   *     sidebar; Safari has no panel API).
   *   - Chromium: the native Side Panel, which docks beside the page (via the background). */
  private openReviewSurface(view: DrawerView): void {
    if (__REVIEW_IN_PAGE__) void this.drawer.openOn(view);
    else this.openPanel(view);
  }

  /** Ask the background to open the Chromium Side Panel on a given view (it keeps the
   *  content-script click's user gesture); a tab is the rare fallback. */
  private openPanel(view: DrawerView): void {
    try {
      chrome.runtime.sendMessage({ type: "openPanel", view });
    } catch {
      // The service worker may be asleep; the user can retry.
    }
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
    await saveBackup(store, backup);
    // A level picked in the drawer's settings lands here (our own echo is ignored below).
    await this.refreshNeedsLevel();
    this.updateHud();
    // Push what just changed. The background debounces a storage change too, but its timer
    // dies with a suspended Safari event page; this request keeps the page alive until the
    // exchange is over (it is throttled, so a burst of gestures still makes one run).
    void requestSync("surface");
  }

  /** Whether the reader still has to choose a level (« Débutant » counts as a choice). */
  private async refreshNeedsLevel(): Promise<void> {
    this.needsLevel = await needsLevelChoice(this.port);
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
      void recordExposures(store, utcDay(Date.now()), this.stats.counted);
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
    // A live phrase/compound selection is the capture path's job; don't also open the
    // single-word popup for whatever word the release landed on, ahead of the debounce.
    const sel = window.getSelection();
    if (sel && !sel.isCollapsed && /[-\s]/.test(String(sel).trim())) return;
    const caret = caretAt(e.clientX, e.clientY);
    // A plain click resolves only PAINTED words; a non-painted (Known/Ignored) word needs the
    // Alt/Option modifier, so a plain click never intercepts one (the page keeps it). The
    // caret snaps to the nearest text, so a click in the page's empty margin resolves to the
    // first or last word of a line: the word's own boxes decide whether it was really clicked.
    const near = caret ? this.hitAt(caret.node, caret.offset, e.altKey) : null;
    const hit = near && clickIsOnWord(e.clientX, e.clientY, near.range.getClientRects()) ? near : null;
    const isLink = e.target instanceof Element && !!e.target.closest("a[href]");
    // Only an UNTREATED word (Unknown) blocks its link — "tant qu'un mot n'a pas été traité".
    // A treated (Learning/decked) word that is a link follows the link on a plain click; its
    // popup stays reachable by selecting it (Alt-click on a mouse). Alt-click reclassifies.
    // The click that ends a selection gesture opens and hides nothing: the capture already
    // opened the card it asked for.
    const decision = decideClick({
      gestureOpenedCard: this.cards.gestureOpenedCard(),
      hitClass: hit?.token.class ?? null,
      isLink,
      altKey: e.altKey,
    });
    if (decision.card === "open" && hit) this.showPopup(hit);
    else if (decision.card === "hide" && this.popup.visible()) this.popup.hide();
    if (decision.stop) e.stopPropagation();
    // Suppress the default ONLY to block an untreated word's link, or for a deliberate
    // Alt-click — never otherwise, so a painted word inside a <label>/<summary>/<button>
    // keeps its native activation.
    if (decision.cancel) e.preventDefault();
  }

  /**
   * The token at a caret: a PAINTED word (Learning/Unknown) directly, or — when
   * `allowReclassify` (the deliberate Alt-click / single-word selection path) — a
   * non-painted (Known/Ignored) word via an on-demand block hit-test. Null otherwise.
   */
  private hitAt(node: Node, offset: number, allowReclassify: boolean): ResolvedToken | null {
    return findTokenAt(this.resolved, node, offset) ?? (allowReclassify ? this.reclassifyHit(node, offset) : null);
  }

  /** Show the word popup for a resolved hit (status-aware actions), anchored to its box. */
  private showPopup(hit: ResolvedToken): void {
    this.cards.openForToken(this.pageHit(hit));
  }

  /** What the cards need from a resolved hit: the token, its box and its sentence, read off the range. */
  private pageHit(hit: ResolvedToken): PageHit {
    const rect = hit.range.getBoundingClientRect();
    return {
      token: hit.token,
      rect: { left: rect.left, top: rect.top, bottom: rect.bottom },
      sentence: sentenceForRange(hit.range),
    };
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

  /** A settled selection, whatever the pointer that made it. One word resolves through the
   *  same hit-test Alt-click uses, so the popup offers the actions matching its real status
   *  (an already-known word is not offered "Je connais" again); a word the page analysis
   *  never saw is read by the analyser instead; several words open the whole-selection
   *  card. The cards decide, this only hands them the hit. */
  private onCapture(kind: CaptureKind, cap: Capture): void {
    if (!this.enabled) return;
    const hit = kind === "word" ? this.hitAt(cap.range.startContainer, cap.range.startOffset, true) : null;
    this.cards.openForSelection(
      { text: cap.text, sentence: cap.sentence, selection: cap.selection, rect: cap.rect },
      hit ? this.pageHit(hit) : null,
    );
  }

  /** The keyboard shortcut: same routing as a pointer selection, so both produce the
   *  same panel for the same selection. */
  private onCaptureSelection(): void {
    const cap = captureSelection();
    if (!cap) return;
    this.onCapture(classifySelection(cap.text), cap);
  }

  private async onGesture(g: Gesture): Promise<void> {
    const key = g.lemma.toLowerCase();
    if (g.status === "learning") {
      const gloss = await this.cards.cardGloss(g);
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
      if (g.status === "known") void recordWordLearned(store, utcDay(Date.now()));
      // Promoting a word that was in the deck (learning) to known/ignored must retire its
      // card so it stops coming due — a word you now treat as known/ignored shouldn't keep
      // being reviewed. No-op when there is no card. (Clearing → "à apprendre" keeps it.)
      if (g.status === "known" || g.status === "ignored") await this.port.retireCard(key, nowSeconds());
    }
    await this.persist();
    await this.repaint();
    // An open drawer shows the deck and stats this gesture just changed.
    await this.drawer.refresh();
  }

  private async onExternalChange(backup: string): Promise<void> {
    if (backup === this.lastBackup) return; // our own write echoed back — nothing to do
    await this.port.restore(backup);
    this.calibration = await this.port.calibration();
    await this.refreshNeedsLevel(); // a level picked in another tab or the popup
    await this.repaint();
    await this.drawer.refresh(); // e.g. cards a sync just pulled
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
      await clearSyncCursors(store);
    }
    // Option B: with CEFR data, presume nothing until the reader picks a level —
    // keep frequency calibration at 0 and let the popup re-prompt for a level.
    // Without CEFR data, restore the default frequency calibration.
    const cal = (await this.port.hasLevels()) ? 0 : 3000;
    await this.port.setCalibration(cal);
    this.calibration = cal;
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
      needsLevel: await needsLevelChoice(this.port),
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
