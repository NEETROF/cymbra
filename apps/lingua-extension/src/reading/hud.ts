// The in-page HUD: a discreet, expandable "percentage pill", bottom-right of the page until
// the reader drags it elsewhere (it then rests against the nearer side edge, at the height it
// was dropped). Collapsed it shows just the known-word percentage; tapped it reveals the reader's
// actions — Réviser (the in-page deck), Capturer (a selection into the deck), Stats, and a
// gear that opens the extension's Réglages in the lateral panel. It is the touch-reachable
// home of the reader on the page itself, which matters most on mobile (no keyboard, awkward
// toolbar popup).
//
// Split like the word popup: a pure `createHud` factory building the DOM (testable via its
// returned `el`), wrapped by `LinguaHud` into a closed shadow root injected on the page.

import { DEFAULT_HUD_POSITION, type HudPosition } from "../state/storage.ts";

export interface HudActions {
  /** Open the review deck (the in-page drawer). */
  onReview: () => void;
  /** Open the learning statistics (the lateral panel). */
  onStats: () => void;
  /** Open the extension's settings (the lateral Réglages panel). */
  onSettings: () => void;
  /** The reader dropped the pill somewhere else: where it came to rest, to remember. */
  onMoved: (position: HudPosition) => void;
}

export interface HudState {
  /** Whether the page has analysable English — the HUD hides itself when it does not. */
  analysable: boolean;
  /** Known-word percentage, or null when it cannot be computed. */
  percent: number | null;
  /**
   * No level declared yet for a language with CEFR data: every word reads as unknown, so the
   * pill offers the choice right away. Needed where no first-run page opens (Safari).
   */
  needsLevel?: boolean;
}

export interface HudView {
  readonly el: HTMLElement;
  /** Reflect the latest reading state (percentage, whether to show at all). */
  update(state: HudState): void;
  /** Collapse the actions row back to the bare pill. */
  collapse(): void;
  /** Whether the pill is currently shown (not hidden for a non-analysable page). */
  visible(): boolean;
  /** Place the pill at a remembered position (no glide: nothing was dragged here). */
  setPosition(position: HudPosition): void;
}

/** Travel, in CSS pixels, under which a press on the pill is a tap rather than a drag. */
export const DRAG_THRESHOLD_PX = 6;

interface Box {
  left: number;
  top: number;
  width: number;
  height: number;
}

/**
 * Where a pill dropped at `pill` (its on-screen box) comes to rest: against the nearer side
 * edge, at the height it was dropped, as a fraction of `column` — the band it travels in.
 */
export function restingPosition(pill: Box, column: Box, viewportWidth: number): HudPosition {
  const side = pill.left + pill.width / 2 < viewportWidth / 2 ? "left" : "right";
  const travel = column.height - pill.height;
  const y = travel > 0 ? Math.min(1, Math.max(0, (pill.top - column.top) / travel)) : 1;
  return { side, y: Math.round(y * 1000) / 1000 };
}

function button(cls: string, text: string, onClick: () => void, ariaLabel?: string): HTMLButtonElement {
  const b = document.createElement("button");
  b.className = cls;
  b.type = "button";
  b.textContent = text;
  if (ariaLabel) b.setAttribute("aria-label", ariaLabel);
  // Preserve any page text selection so "Capturer" can read it: a plain mousedown on a
  // button collapses the document selection before the click handler runs. click still fires.
  b.addEventListener("mousedown", (e) => e.preventDefault());
  b.addEventListener("click", onClick);
  return b;
}

/** Build the HUD DOM and its behaviour, independent of the shadow host (so it is testable). */
export function createHud(actions: HudActions): HudView {
  const el = document.createElement("div");
  el.className = "hud";
  el.hidden = true;

  const row = document.createElement("div");
  row.className = "hud-actions";
  row.hidden = true;

  const collapse = (): void => {
    row.hidden = true;
  };

  // The pill: the always-visible percentage toggles the actions row.
  const pill = document.createElement("div");
  pill.className = "hud-pill";
  const pct = button("hud-pct", "—", () => {
    row.hidden = !row.hidden;
  });
  pct.setAttribute("aria-label", "Mots connus sur la page — ouvrir les actions");

  const review = button("hud-act", "Réviser", () => {
    actions.onReview();
    collapse();
  });
  const stats = button("hud-act", "Stats", () => {
    actions.onStats();
    collapse();
  });
  const gear = button(
    "hud-gear",
    "⚙",
    () => {
      actions.onSettings();
      collapse();
    },
    "Réglages",
  );
  const collapseBtn = button("hud-collapse", "⌄", collapse, "Réduire");
  row.append(review, stats, gear, collapseBtn);
  // Outside the collapsible row: visible on the bare pill until a level is declared.
  const level = button("hud-level", "Choisis ton niveau", () => {
    actions.onSettings();
    collapse();
  });
  level.hidden = true;
  pill.append(pct, level, row);
  el.append(pill);

  // The side and height are the stylesheet's to apply (hud.css), so the pill follows a resize
  // or a rotation on its own; a drag only ever writes these two.
  const place = (position: HudPosition): void => {
    el.dataset.side = position.side;
    el.style.setProperty("--hud-y", String(position.y));
  };
  place(DEFAULT_HUD_POSITION);

  // Dragging: a press that travels past the threshold moves the pill instead of tapping it,
  // and on release it glides to the nearer side edge. Pointer events cover the mouse and the
  // finger alike (hud.css sets `touch-action: none`, so a finger drags instead of scrolling).
  // The pointer is captured only once it is a drag: captured from the press, a plain tap's
  // click would land on the pill instead of the button under it.
  let press: { id: number; x: number; y: number; box: Box } | null = null;
  let dragging = false;
  // The click that ends a drag must not also toggle the actions, or fire the one it was
  // released on. Reset by the next press, since a finger's drag produces no click at all.
  let swallowClick = false;

  /** Glide from where the pill was dropped to where it rests (FLIP: first, last, invert, play). */
  const settle = (position: HudPosition): void => {
    const from = pill.getBoundingClientRect();
    // Transitions off BEFORE the drag offset is dropped: measuring forces a style pass, which
    // would otherwise start the glide from the drop point and report ITS first frame (the
    // drop point again) as the resting place — the pill would jump back to where the drag began.
    pill.style.transition = "none";
    pill.classList.remove("dragging");
    pill.style.translate = "";
    place(position);
    const to = pill.getBoundingClientRect();
    pill.style.translate = `${from.left - to.left}px ${from.top - to.top}px`;
    void pill.offsetWidth; // commit the inverted frame, or the glide below would not start from it
    pill.style.transition = "";
    pill.style.translate = "";
  };

  const onMove = (e: PointerEvent): void => {
    if (!press || e.pointerId !== press.id) return;
    const dx = e.clientX - press.x;
    const dy = e.clientY - press.y;
    if (!dragging) {
      if (Math.hypot(dx, dy) < DRAG_THRESHOLD_PX) return;
      dragging = true;
      pill.classList.add("dragging");
      try {
        pill.setPointerCapture(e.pointerId);
      } catch {
        // The pointer is already gone; the window listeners still see the rest of the drag.
      }
    }
    // Follow the pointer, the whole pill staying on screen.
    const { box } = press;
    const x = Math.min(Math.max(dx, -box.left), window.innerWidth - box.left - box.width);
    const y = Math.min(Math.max(dy, -box.top), window.innerHeight - box.top - box.height);
    pill.style.translate = `${x}px ${y}px`;
  };

  const onEnd = (e: PointerEvent): void => {
    if (!press || e.pointerId !== press.id) return;
    press = null;
    window.removeEventListener("pointermove", onMove, true);
    window.removeEventListener("pointerup", onEnd, true);
    window.removeEventListener("pointercancel", onEnd, true);
    if (!dragging) return;
    dragging = false;
    swallowClick = true;
    const position = restingPosition(pill.getBoundingClientRect(), el.getBoundingClientRect(), window.innerWidth);
    settle(position);
    actions.onMoved(position);
  };

  pill.addEventListener("pointerdown", (e) => {
    swallowClick = false;
    if (press || e.button !== 0) return;
    press = { id: e.pointerId, x: e.clientX, y: e.clientY, box: pill.getBoundingClientRect() };
    // On the window, capturing: a fast mouse leaves the pill before it counts as a drag, and a
    // page handler stopping propagation must not strand the drag.
    window.addEventListener("pointermove", onMove, true);
    window.addEventListener("pointerup", onEnd, true);
    window.addEventListener("pointercancel", onEnd, true);
  });
  // A drag from the pill's rim must not start a page selection (its buttons already prevent it).
  pill.addEventListener("mousedown", (e) => e.preventDefault());
  pill.addEventListener(
    "click",
    (e) => {
      if (!swallowClick) return;
      swallowClick = false;
      e.stopPropagation();
      e.preventDefault();
    },
    true,
  );

  return {
    el,
    collapse,
    setPosition: place,
    visible: () => !el.hidden,
    update(state: HudState): void {
      el.hidden = !state.analysable;
      if (!state.analysable) {
        collapse();
        return;
      }
      pct.textContent = state.percent == null ? "—" : `${state.percent}%`;
      level.hidden = !state.needsLevel;
    },
  };
}

export interface HudOptions {
  /** Token sheet + hud styles, injected into the shadow root. */
  css: string;
  actions: HudActions;
}

/** The HUD mounted into a closed shadow root on the page (mirrors Drawer / WordPopup). */
export class LinguaHud {
  private readonly host: HTMLElement;
  private readonly view: HudView;
  private hidden = false;
  private last: HudState | null = null;

  constructor(opts: HudOptions) {
    this.host = document.createElement("div");
    this.host.id = "cymbra-lingua-hud-host";
    this.host.setAttribute("data-cymbra-lingua-skip", ""); // the reader must not analyse itself
    const root = this.host.attachShadow({ mode: "closed" });
    const style = document.createElement("style");
    style.textContent = opts.css;
    this.view = createHud(opts.actions);
    root.append(style, this.view.el);
  }

  /** Attach the host to the page (idempotent; clears any orphan host from a failed retry). */
  mount(): void {
    const existing = document.getElementById("cymbra-lingua-hud-host");
    if (existing && existing !== this.host) existing.remove();
    if (!this.host.isConnected) document.documentElement.appendChild(this.host);
    this.applyHidden();
  }

  /** Reflect the latest reading state; a no-op paint while hidden (re-applied on show). */
  update(state: HudState): void {
    this.last = state;
    if (!this.hidden) this.view.update(state);
  }

  /** Place the pill where the reader left it (a stored position, or one dragged in another tab). */
  setPosition(position: HudPosition): void {
    this.view.setPosition(position);
  }

  /** Hide/show the whole HUD (reader disabled, or the user chose to hide it). */
  setHidden(hidden: boolean): void {
    this.hidden = hidden;
    this.applyHidden();
  }

  private applyHidden(): void {
    this.host.style.display = this.hidden ? "none" : "";
    if (!this.hidden) {
      this.view.collapse(); // re-show as the bare pill, never mid-expansion
      if (this.last) this.view.update(this.last);
    }
  }

  /** Remove the host from the page. */
  destroy(): void {
    this.host.remove();
  }
}
