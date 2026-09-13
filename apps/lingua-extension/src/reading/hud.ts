import { CEFR_LEVELS, type CefrLevel } from "../analyzer/types.ts";

// The in-page HUD: a discreet, expandable "percentage pill" anchored bottom-right of the
// page. Collapsed it shows just the known-word percentage; tapped it reveals the reader's
// actions (Réviser / Capturer / Stats) and a settings popover (CEFR level + hide). It is
// the touch-reachable home of the reader on the page itself — the deck, capture and level
// without opening the toolbar popup, which matters most on mobile (no keyboard).
//
// Split like the word popup: a pure `createHud` factory building the DOM (testable via its
// returned `el`), wrapped by `LinguaHud` into a closed shadow root injected on the page.

export interface HudActions {
  /** Open the review deck (the in-page drawer). */
  onReview: () => void;
  /** Capture the current selection (word or phrase) into the deck. */
  onCapture: () => void;
  /** Open the learning statistics. */
  onStats: () => void;
  /** Declare the CEFR level (null = "Débutant", nothing presumed known). */
  onSetLevel: (level: CefrLevel | null) => void;
  /** Hide the HUD (persisted by the caller; re-shown from the popup). */
  onHide: () => void;
}

export interface HudState {
  /** Whether the page has analysable English — the HUD hides itself when it does not. */
  analysable: boolean;
  /** Known-word percentage, or null when it cannot be computed. */
  percent: number | null;
  /** Whether the pack carries CEFR levels (drives the level picker in settings). */
  hasLevels: boolean;
  /** The declared level, or null for "Débutant". */
  declaredLevel: CefrLevel | null;
}

export interface HudView {
  readonly el: HTMLElement;
  /** Reflect the latest reading state (percentage, level, whether to show at all). */
  update(state: HudState): void;
  /** Whether the pill is currently shown (not hidden for a non-analysable page). */
  visible(): boolean;
}

function button(cls: string, text: string, onClick: () => void, ariaLabel?: string): HTMLButtonElement {
  const b = document.createElement("button");
  b.className = cls;
  b.type = "button";
  b.textContent = text;
  if (ariaLabel) b.setAttribute("aria-label", ariaLabel);
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

  const settings = document.createElement("div");
  settings.className = "hud-settings";
  settings.hidden = true;

  const collapse = (): void => {
    row.hidden = true;
    settings.hidden = true;
  };

  // Settings popover: the CEFR level picker + "hide the bar". Anchored above the pill.
  const settingsLabel = document.createElement("div");
  settingsLabel.className = "hud-settings-label";
  settingsLabel.textContent = "Niveau d'anglais";
  const levels = document.createElement("div");
  levels.className = "hud-levels";
  const levelButtons = new Map<string, HTMLButtonElement>();
  const addLevel = (value: string, text: string): void => {
    const b = button("hud-lvl", text, () => {
      actions.onSetLevel((value as CefrLevel) || null);
      settings.hidden = true; // the reader re-renders us via update() with the new level
    });
    b.dataset.lvl = value;
    levels.append(b);
    levelButtons.set(value, b);
  };
  for (const lvl of CEFR_LEVELS) addLevel(lvl, lvl);
  addLevel("", "Débutant");
  const hideBtn = button("hud-hide", "Masquer la barre", () => actions.onHide());
  settings.append(settingsLabel, levels, hideBtn);

  // The pill: the always-visible percentage toggles the actions row.
  const pill = document.createElement("div");
  pill.className = "hud-pill";
  const pct = button("hud-pct", "—", () => {
    const willExpand = row.hidden;
    row.hidden = !willExpand;
    if (!willExpand) settings.hidden = true;
  });
  pct.setAttribute("aria-label", "Mots connus sur la page — ouvrir les actions");

  const review = button("hud-act", "Réviser", () => {
    actions.onReview();
    collapse();
  });
  const capture = button("hud-act", "Capturer", () => {
    actions.onCapture();
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
      settings.hidden = !settings.hidden;
    },
    "Paramètres",
  );
  const collapseBtn = button("hud-collapse", "⌄", collapse, "Réduire");
  row.append(review, capture, stats, gear, collapseBtn);
  pill.append(pct, row);
  el.append(settings, pill);

  return {
    el,
    visible: () => !el.hidden,
    update(state: HudState): void {
      el.hidden = !state.analysable;
      if (!state.analysable) {
        collapse();
        return;
      }
      pct.textContent = state.percent == null ? "—" : `${state.percent}%`;
      // The level picker only makes sense with a CEFR pack; otherwise settings offer just
      // "Masquer la barre".
      settingsLabel.hidden = !state.hasLevels;
      levels.hidden = !state.hasLevels;
      const current = state.declaredLevel ?? "";
      for (const [value, b] of levelButtons) b.classList.toggle("active", value === current);
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

  /** Attach the host to the page (idempotent). */
  mount(): void {
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
    if (!this.hidden && this.last) this.view.update(this.last);
  }

  /** Remove the host from the page. */
  destroy(): void {
    this.host.remove();
  }
}
