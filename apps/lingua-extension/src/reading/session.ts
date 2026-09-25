import { createTranslatorPort } from "../translate/create-port.ts";
import type { LinguaPort } from "../analyzer/port.ts";
import { type CefrLevel, STUDIED_LANGUAGE } from "../analyzer/types.ts";
import { type Block, isElement, mergeBlocks } from "./blocks.ts";
import { Drawer, type DrawerView } from "./drawer.ts";
import { clear as clearHighlights, injectPageStyles, render } from "./highlight.ts";
import { ExposureTracker } from "./exposure-tracker.ts";
import { ReadingObservers } from "./observer.ts";
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
} from "./scan.ts";
import { type HudActions, type HudState, LinguaHud } from "./hud.ts";
import {
  type Capture,
  type CaptureKind,
  captureSelection,
  classifySelection,
  SelectionWatcher,
  sentenceForRange,
} from "./selection.ts";
import { clickIsOnWord, decideClick, type PageHit, SelectionCards } from "./selection-card.ts";
import { browserSpeechEngine, createSpeaker, type Speaker } from "./speech.ts";
import type { SurfaceCss } from "./surface-css.ts";
import { type Gesture, WordPopup } from "./wordpopup.ts";
import { recordExposures, recordWordLearned, utcDay } from "../state/dailystats.ts";
import { needsLevelChoice } from "../state/level-choice.ts";
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
} from "../state/storage.ts";
import { messagedArea, watchBackup } from "../state/store.ts";
import { requestSync } from "../sync/messages.ts";

// The reading controller, one implementation for its two hosts: the content script on a web
// page, and the reader page on each section of a book (add-lingua-reader D3). It owns the
// analysis pass, the highlight paint, the word popup, the review drawer and the
// page↔storage plumbing. The authoritative state is the engine's lingua-core LinguaState,
// persisted as its backup string in the background-owned store; every context restores from
// it, so a gesture (or a review in the side panel/drawer) repaints every other.
//
// Two documents are in play, and they are the same one on a web page:
//  - the READ document (`ReadingHost`) — walked, analysed, highlighted, listened to for
//    clicks and selections. A book section is its own document, in foliate-js's iframe.
//  - the SURFACE document — the global `document`, where the popup, the selection card, the
//    drawer and the HUD mount. The reader page's own chrome, on a book.
// The DOM is walked only for dirty subtrees; analysis over the (cheap) block text is
// whole-document so the language gate and the percentage stay page-correct.

/** A box, in some viewport's coordinates. */
export interface Box {
  left: number;
  top: number;
  bottom: number;
}

/** What a session reads: a document, its window, and how it sits among the surfaces. */
export interface ReadingHost {
  /** The document analysed, highlighted and listened to. */
  doc: Document;
  /** Its window: the selection, the highlight registry and the observers are its own. */
  win: Window & typeof globalThis;
  /** Paint the whole document at once (a book section) rather than by viewport window (a page). */
  paintWhole: boolean;
  /** Map a box in the read document's viewport to the surfaces' — identity on a web page, the
   *  iframe's offset added for a book section. */
  toSurface(box: Box): Box;
  /** The source a card captured here keeps on the device: the page address, or the book and
   *  section. Never sent (lingua-privacy). */
  source(): string;
  /** The tag reading exposures are recorded under. */
  exposureSource(): string;
}

/** The web page the content script runs in. */
export function pageHost(): ReadingHost {
  return {
    doc: document,
    win: window,
    paintWhole: false,
    toSurface: (box) => box,
    source: () => location.href,
    exposureSource: () => `reading:${location.hostname}`,
  };
}

/** What shows the reading state: the in-page HUD pill on a web page, the reader's toolbar on a book. */
export interface ReadingIndicator {
  /** Attach to the surface document, after a first successful paint. */
  mount(): void;
  update(state: HudState): void;
  setHidden(hidden: boolean): void;
}

export interface SessionOptions {
  css: SurfaceCss;
  /** Build the indicator, given the actions it offers. The in-page HUD pill by default. */
  indicator?: (actions: HudActions) => ReadingIndicator;
  /** Told to the popup with the figures, so it can tell a book from a page. */
  surface?: "page" | "book";
  /** Every paint of the read document settled — the reader reveals a section on its first. */
  onPainted?: () => void;
  /** A click on nothing — no word, no link, no card to dismiss: the reader turns the page. */
  onBlankClick?: (e: MouseEvent) => void;
  /** Drop a phrase's selection once a finger lifts from it (`SelectionWatcher`): Safari's build. */
  dropPhraseOnLift?: boolean;
}

/** The figures the popup asks for with `getStats`. */
export interface SessionStats {
  surface: "page" | "book";
  analysable: boolean;
  percent: number | null;
  counted: number;
  unknownOccurrences: number;
  distinctUnknown: number;
  calibration: number;
  declaredLevel: CefrLevel | null;
  hasLevels: boolean;
  needsLevel: boolean;
  trackedCount: number;
  deckCount: number;
  dueCount: number;
}

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

/** The text position under a point of a document's viewport, whichever API the engine has. */
export function caretAt(doc: Document, x: number, y: number): { node: Node; offset: number } | null {
  const d = doc as Document & {
    caretRangeFromPoint?: (x: number, y: number) => Range | null;
    caretPositionFromPoint?: (x: number, y: number) => { offsetNode: Node; offset: number } | null;
  };
  if (d.caretRangeFromPoint) {
    const r = d.caretRangeFromPoint(x, y);
    return r ? { node: r.startContainer, offset: r.startOffset } : null;
  }
  if (d.caretPositionFromPoint) {
    const p = d.caretPositionFromPoint(x, y);
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

export class ReadingSession {
  private readonly blocksByContainer = new Map<Element, Block>();
  private resolved: ResolvedToken[] = [];
  /** Per-container block+tokens, for the Alt-click reclassify path of non-painted words. */
  private clickable = new Map<Element, BlockTokens>();
  /** Debounces `selectionchange` into one capture, whatever the pointer. */
  private selection: SelectionWatcher;
  private stats: ScanStats = NOT_ANALYSABLE;
  private calibration = 3000;
  /** Whether the reader hid the in-page HUD pill (persisted, toggled from the popup / panel). */
  private hudHidden = false;
  /** Global master switch. When off the reader does not analyse, paint or pop up. */
  private enabled = true;
  /** Daily exposures are counted once per document read (a re-scan does not re-count). */
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
  private readonly indicator: ReadingIndicator;
  private indicatorMounted = false;
  private readonly observers: ReadingObservers;
  /** Viewport-gated reading exposure (slice 5c): lemmas whose block was actually read. */
  private readonly exposure: ExposureTracker;
  private readonly pendingExposure = new Set<string>();
  private exposureFlushTimer: ReturnType<typeof setTimeout> | null = null;
  /** The last backup we wrote, to ignore our own storage.onChanged echo. */
  private lastBackup: string | null = null;
  /** No level declared for a CEFR language yet: the HUD offers the choice (see refreshNeedsLevel). */
  private needsLevel = false;
  /** The document being read, or none (the reader's library, between two sections). */
  private host: ReadingHost | null = null;
  /** Removes the listeners of the read document when it is detached. */
  private hostListeners: AbortController | null = null;
  /** Where the popup, the drawer and the indicator live: this context's own document. */
  private readonly surfaceDoc: Document = document;
  private readonly surfaceWin: Window = window;

  // The port is resolved before construction (`resolveContentPort`) so a CSP-blocked
  // page can hand us the messaging port instead of the in-content WASM engine.
  constructor(
    private readonly port: LinguaPort,
    private readonly opts: SessionOptions,
  ) {
    this.popup = new WordPopup({
      css: opts.css.popup,
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
      css: opts.css.drawer,
      port: this.port,
      area: storageArea,
      store,
      now: nowSeconds,
      onChange: () => this.persist(),
      speaker: this.speaker,
    });
    const actions: HudActions = {
      onReview: () => this.openReviewSurface("review"),
      onStats: () => this.openReviewSurface("stats"),
      onSettings: () => this.openReviewSurface("settings"),
    };
    this.indicator = opts.indicator?.(actions) ?? new LinguaHud({ css: opts.css.hud, actions });
    this.observers = new ReadingObservers({ onRescan: (containers) => void this.refresh(containers) });
    this.exposure = new ExposureTracker((lemmas) => this.onExposed(lemmas));
    this.selection = this.watchSelection(this.surfaceWin);
  }

  /** Restore the engine, wire the surfaces, then read `host` (none: the reader's library). */
  async start(host: ReadingHost | null): Promise<void> {
    await hydrateEngine(this.port, store);
    this.calibration = await this.port.calibration();
    this.hudHidden = await loadHudHidden(storageArea);
    this.enabled = await loadEnabled(storageArea);
    await this.refreshNeedsLevel();
    this.surfaceDoc.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && this.popup.visible()) this.popup.hide();
    });
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
    const surface = this.surfaceDoc;
    surface.addEventListener("visibilitychange", () => {
      if (surface.visibilityState === "hidden" && this.pendingExposure.size > 0) void this.flushExposure();
      // Nothing keeps talking in a tab the reader left.
      if (surface.visibilityState === "hidden") this.speaker.stop();
      // Back on the tab (possibly after reading on another device): ask for a sync too.
      if (surface.visibilityState === "visible" && this.enabled) void requestSync("page");
    });
    // The word popup is position:fixed and anchored to a word's box; a scroll detaches it
    // (and near the page bottom it could sit half-off-screen). Dismiss it on scroll — a
    // re-click reopens it correctly placed. Capture so nested scroll containers count too.
    this.surfaceWin.addEventListener("scroll", () => this.dismiss(), { capture: true, passive: true });
    chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
      // An extension page hears every `runtime.sendMessage` of every tab's content script;
      // a book in the reader answers only what is addressed to it (tabs.sendMessage from the
      // popup or the background, which carries no tab).
      if (this.opts.surface === "book" && sender.tab) return false;
      // No settings travel here: Réglages, wherever they are shown, are `mountSettings` on
      // that surface's own engine port, and reach this page as a changed backup
      // (`onExternalChange`).
      if (msg?.type === "captureSelection") this.onCaptureSelection();
      else if (msg?.type === "toggleDrawer") void this.drawer.toggle();
      else if (msg?.type === "openDrawer") void this.drawer.openOn(drawerView(msg.view));
      else if (msg?.type === "getStats") {
        void this.statsMessage().then(sendResponse);
        return true; // async response
      }
      return false;
    });

    this.drawer.setSessionLost((await storageArea.get(SESSION_LOST_KEY))[SESSION_LOST_KEY] === true);

    if (host) await this.attach(host);
    else if (!this.enabled) this.pushDisabledBadge();
  }

  /**
   * Read `host` from now on: listen to it, and — when the reader is on — paint it and watch
   * it. Whatever was read before is detached first, so one session follows a book from
   * section to section.
   */
  async attach(host: ReadingHost): Promise<void> {
    this.detach();
    this.host = host;
    // The section's own AbortController: an `addEventListener` signal is of its window's realm.
    const listeners = new host.win.AbortController();
    this.hostListeners = listeners;
    const { doc } = host;
    const on = (type: string, fn: (e: Event) => void, opts: AddEventListenerOptions = {}): void =>
      doc.addEventListener(type, fn, { ...opts, signal: listeners.signal });
    this.selection = this.watchSelection(host.win);
    on("click", (e) => this.onClick(e as MouseEvent), { capture: true });
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
    on("selectionchange", () => this.selection.notify(), { passive: true });
    on("mousedown", pointerDown, pointer);
    on("touchstart", pointerDown, pointer);
    on("mouseup", () => this.selection.release(), pointer);
    on("touchend", () => this.selection.release("finger"), pointer);
    on("touchcancel", () => this.selection.release(), pointer);
    if (doc !== this.surfaceDoc) {
      // A book section has its own keyboard focus and scrolls in its own window (the
      // scrolled flow): the same dismissals as the surface document's.
      on("keydown", (e) => {
        if ((e as KeyboardEvent).key === "Escape") this.dismiss();
      });
      host.win.addEventListener("scroll", () => this.dismiss(), {
        capture: true,
        passive: true,
        signal: listeners.signal,
      });
    }

    if (this.enabled) {
      await this.activate();
    } else {
      this.pushDisabledBadge();
      this.opts.onPainted?.(); // nothing to paint: the host need not wait for it
    }
  }

  /** Stop reading the current document: its listeners, observers, highlights and figures go. */
  detach(): void {
    const host = this.host;
    if (!host) return;
    if (this.pendingExposure.size > 0) void this.flushExposure();
    this.hostListeners?.abort();
    this.hostListeners = null;
    this.selection.cancel();
    this.observers.stop();
    this.exposure.stop();
    this.popup.hide();
    clearHighlights({ doc: host.doc });
    this.host = null;
    this.blocksByContainer.clear();
    this.resolved = [];
    this.clickable.clear();
    this.stats = NOT_ANALYSABLE;
    this.exposuresRecorded = false;
    this.updateHud();
  }

  /** Hide the word popup, if it shows: its word moved (a scroll, a page turn). */
  dismiss(): void {
    if (this.popup.visible()) this.popup.hide();
  }

  private watchSelection(win: Pick<Window, "getSelection">): SelectionWatcher {
    return new SelectionWatcher({
      onCapture: (kind, cap) => this.onCapture(kind, cap),
      win,
      // Only where the card is shown in the callout's place: the reader switched on, the page
      // analysed. Anywhere else the platform's selection is left exactly as it is.
      dropPhraseOnLift: () =>
        (this.opts.dropPhraseOnLift ?? __TARGET__ === "safari") && this.enabled && this.stats.analysable,
    });
  }

  /** Paint the read document and begin watching it for changes (the reader's "on" state). */
  private async activate(): Promise<void> {
    const host = this.host;
    if (!host) return;
    injectPageStyles(this.opts.css.tokens, host.doc);
    await this.refresh([host.doc.body], { firstPaint: true });
    // Mount the indicator only after a successful first paint, so a failed init (which resets
    // the injection guard and lets a retry create a fresh session) leaves no orphan host.
    if (!this.indicatorMounted) {
      this.indicator.mount();
      this.indicatorMounted = true;
    }
    this.syncHud();
    this.observers.start(host.doc.body);
    this.exposure.start(host.win);
    // A page load asks for a sync; the background runs at most one a minute.
    void requestSync("page");
  }

  /** Reflect the HUD's current visibility: shown only while enabled and not user-hidden. */
  private syncHud(): void {
    this.indicator.setHidden(!this.enabled || this.hudHidden);
  }

  /** Push the current reading state into the in-page HUD pill. */
  private updateHud(): void {
    this.indicator.update({
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
      if (this.host) clearHighlights({ doc: this.host.doc });
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

  /**
   * Re-walk the given dirty containers, then repaint from a fresh whole-doc analysis — only when a
   * block changed (fix-lingua-dynamic-rescan D3): a player's clock ticking every second must not
   * re-analyse the page. The first paint always repaints: it is what sets the badge, the indicator
   * and `onPainted`, even for a page with no text.
   */
  private async refresh(dirtyRoots: Element[], opts: { firstPaint?: boolean } = {}): Promise<void> {
    const host = this.host;
    const changed = mergeBlocks(this.blocksByContainer, dirtyRoots);
    if (changed || opts.firstPaint) await this.repaint();
    // The document may have been detached while the analysis ran (a page turned to the next section).
    if (this.host === host) this.observers.track(this.blocksByContainer.keys());
  }

  /** Re-analyse current block text (no DOM walk) and repaint. */
  private async repaint(): Promise<void> {
    const host = this.host;
    if (!host) return;
    if (!this.enabled) {
      clearHighlights({ doc: host.doc });
      return;
    }
    const blocks = [...this.blocksByContainer.values()];
    if (blocks.length === 0) {
      this.resolved = [];
      this.clickable.clear();
      this.stats = NOT_ANALYSABLE;
      clearHighlights({ doc: host.doc });
      this.pushBadge();
      this.updateHud();
      this.opts.onPainted?.();
      return;
    }
    const analysis = await this.port.analyse(blocks.map((b) => b.text));
    // Another document replaced this one while the engine answered: its figures are stale.
    if (this.host !== host) return;
    this.resolved = resolveTokens(blocks, analysis);
    this.clickable = clickableByContainer(blocks, analysis);
    this.stats = statsFromAnalysis(analysis);
    // Track which blocks the reader actually sees, to confirm below-level words by reading.
    if (this.stats.analysable) this.exposure.track(lemmasByContainer(blocks, analysis));
    // Count studied-word exposures once per document read (§3 daily stats).
    if (this.stats.analysable && !this.exposuresRecorded && this.stats.counted > 0) {
      this.exposuresRecorded = true;
      void recordExposures(store, utcDay(Date.now()), this.stats.counted);
    }
    // Re-assert the token sheet before painting: a single-page-app navigation
    // (GitHub's morphing) can strip our injected styles, which leaves highlights
    // unpainted even though clicks still resolve. injectPageStyles is idempotent
    // and self-healing, so this restores them on the first paint after a nav.
    injectPageStyles(this.opts.css.tokens, host.doc);
    render(this.resolved, { doc: host.doc, window: !host.paintWhole });
    this.pushBadge();
    this.updateHud();
    this.opts.onPainted?.();
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
    const host = this.host;
    if (!host || !this.enabled || this.popup.contains(e.target)) return;
    // A live phrase/compound selection is the capture path's job; don't also open the
    // single-word popup for whatever word the release landed on, ahead of the debounce.
    const sel = host.win.getSelection();
    if (sel && !sel.isCollapsed && /[-\s]/.test(String(sel).trim())) return;
    const caret = caretAt(host.doc, e.clientX, e.clientY);
    // A plain click resolves only PAINTED words; a non-painted (Known/Ignored) word needs the
    // Alt/Option modifier, so a plain click never intercepts one (the page keeps it). The
    // caret snaps to the nearest text, so a click in the page's empty margin resolves to the
    // first or last word of a line: the word's own boxes decide whether it was really clicked.
    const near = caret ? this.hitAt(caret.node, caret.offset, e.altKey) : null;
    const hit = near && clickIsOnWord(e.clientX, e.clientY, near.range.getClientRects()) ? near : null;
    const target = e.target as Node | null;
    const isLink = isElement(target) && !!target.closest("a[href]");
    const cardWasOpen = this.popup.visible();
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
    else if (decision.card === "hide" && cardWasOpen) this.popup.hide();
    if (decision.stop) e.stopPropagation();
    // Suppress the default ONLY to block an untreated word's link, or for a deliberate
    // Alt-click — never otherwise, so a painted word inside a <label>/<summary>/<button>
    // keeps its native activation.
    if (decision.cancel) e.preventDefault();
    // A click on nothing at all — no word, no link, no card it closes — is the host's.
    if (!hit && !isLink && !cardWasOpen && decision.card === "hide") this.opts.onBlankClick?.(e);
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
      rect: this.toSurface({ left: rect.left, top: rect.top, bottom: rect.bottom }),
      sentence: sentenceForRange(hit.range),
    };
  }

  /** A box of the read document, where the surfaces are drawn. */
  private toSurface(box: Box): Box {
    return this.host ? this.host.toSurface(box) : box;
  }

  /** Hit-test a non-painted word (Known/Ignored) by resolving only its own block's tokens.
   *  Block+tokens come paired from the same analysis, so a since-changed DOM yields a clean
   *  miss (ranges over old nodes), never a wrong-word hit. */
  private reclassifyHit(node: Node, offset: number): ResolvedToken | null {
    let el: Element | null = isElement(node) ? node : node.parentElement;
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
    if (!this.enabled || !this.host) return;
    const hit = kind === "word" ? this.hitAt(cap.range.startContainer, cap.range.startOffset, true) : null;
    this.cards.openForSelection(
      { text: cap.text, sentence: cap.sentence, selection: cap.selection, rect: this.toSurface(cap.rect) },
      hit ? this.pageHit(hit) : null,
    );
  }

  /** The keyboard shortcut: same routing as a pointer selection, so both produce the
   *  same panel for the same selection. */
  private onCaptureSelection(): void {
    if (!this.host) return;
    const cap = captureSelection(undefined, this.host.win);
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
        url: this.host?.source() ?? "",
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
    if (this.exposureFlushTimer !== null) clearTimeout(this.exposureFlushTimer);
    this.exposureFlushTimer = null;
    if (this.pendingExposure.size === 0) return;
    const lemmas = [...this.pendingExposure];
    this.pendingExposure.clear();
    const now = Date.now();
    await this.port.recordExposures(lemmas, this.host?.exposureSource() ?? "reading:", now);
    const promoted = await this.port.promoteByExposure(EXPOSURE_PROMOTE_DAYS, now);
    await this.persist();
    if (promoted > 0) await this.repaint(); // words became known → refresh highlights
  }

  private async statsMessage(): Promise<SessionStats> {
    const now = nowSeconds();
    return {
      surface: this.opts.surface ?? "page",
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
