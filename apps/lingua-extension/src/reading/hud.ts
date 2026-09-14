// The in-page HUD: a discreet, expandable "percentage pill" anchored bottom-right of the
// page. Collapsed it shows just the known-word percentage; tapped it reveals the reader's
// actions — Réviser (the in-page deck), Capturer (a selection into the deck), Stats, and a
// gear that opens the extension's Réglages in the lateral panel. It is the touch-reachable
// home of the reader on the page itself, which matters most on mobile (no keyboard, awkward
// toolbar popup).
//
// Split like the word popup: a pure `createHud` factory building the DOM (testable via its
// returned `el`), wrapped by `LinguaHud` into a closed shadow root injected on the page.

export interface HudActions {
  /** Open the review deck (the in-page drawer). */
  onReview: () => void;
  /** Open the learning statistics (the lateral panel). */
  onStats: () => void;
  /** Open the extension's settings (the lateral Réglages panel). */
  onSettings: () => void;
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

  return {
    el,
    collapse,
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
