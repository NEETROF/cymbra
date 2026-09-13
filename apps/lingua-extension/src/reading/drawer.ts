import type { LinguaPort } from "../analyzer/port.ts";
import { ReviewController, type ReviewView } from "../review/session.ts";
import { type ReviewActions, renderReview } from "../review/view.ts";
import { mountSettings, type SettingsView } from "./settings-view.ts";
import type { AsyncStorageArea } from "../state/storage.ts";
import { mountStats } from "../stats/view.ts";

// The injected in-page panel: a closed-shadow overlay with Révision / Statistiques /
// Réglages, so the reader never has to LEAVE the page it is reading (design D1, extended).
// It shares its logic with the native side panel — renderReview, mountStats and
// mountSettings are the same code, rendered into a second host. On Firefox (where a page
// element cannot open the sidebar) this is THE panel the HUD and popup open; on Chromium
// the native side panel is used instead and this stays the Alt+Shift+D micro-review.

export type DrawerView = "review" | "stats" | "settings";

export interface DrawerOptions {
  /** Combined token sheet + review + drawer + stats + settings styles, for the shadow. */
  css: string;
  port: LinguaPort;
  /** chrome.storage.local surface (for stats + the HUD-toggle in settings). */
  area: AsyncStorageArea;
  /** Epoch-seconds clock (Date.now()/1000 in production). */
  now: () => number;
  /** Persist after a state-changing action (backup → storage). */
  onChange: () => Promise<void>;
  /** Daily-stats hook forwarded to the review controller (grade / mark-known). */
  record?: (event: "review" | "learned") => void;
}

export class Drawer {
  private readonly host: HTMLElement;
  private readonly panel: HTMLElement;
  private readonly reviewBody: HTMLElement;
  private readonly statsBody: HTMLElement;
  private readonly settingsBody: HTMLElement;
  private readonly tabs: Map<DrawerView, HTMLButtonElement> = new Map();
  private controller: ReviewController;
  private readonly actions: ReviewActions;
  private settings: SettingsView | null = null;
  private open = false;

  constructor(private readonly opts: DrawerOptions) {
    this.controller = new ReviewController(opts.port, opts.now, opts.record);

    this.host = document.createElement("div");
    this.host.id = "cymbra-lingua-drawer-host";
    this.host.setAttribute("data-cymbra-lingua-skip", "");
    const root = this.host.attachShadow({ mode: "closed" });
    const style = document.createElement("style");
    style.textContent = opts.css;

    this.panel = document.createElement("div");
    this.panel.className = "drawer";
    this.panel.hidden = true;

    const head = document.createElement("div");
    head.className = "drawer-head";
    const tabsRow = document.createElement("div");
    tabsRow.className = "drawer-views";
    const addTab = (view: DrawerView, label: string): void => {
      const b = document.createElement("button");
      b.type = "button";
      b.textContent = label;
      b.addEventListener("click", () => void this.switchTo(view));
      tabsRow.append(b);
      this.tabs.set(view, b);
    };
    addTab("review", "Révision");
    addTab("stats", "Stats");
    addTab("settings", "Réglages");
    const close = document.createElement("button");
    close.className = "drawer-close";
    close.textContent = "×";
    close.setAttribute("aria-label", "Fermer");
    close.addEventListener("click", () => this.hide());
    head.append(tabsRow, close);

    this.reviewBody = document.createElement("div");
    this.statsBody = document.createElement("div");
    this.statsBody.hidden = true;
    this.settingsBody = document.createElement("div");
    this.settingsBody.hidden = true;
    this.panel.append(head, this.reviewBody, this.statsBody, this.settingsBody);
    root.append(style, this.panel);

    this.actions = {
      start: () => void this.run(() => this.controller.start(), false),
      reveal: () => void this.run(() => this.controller.reveal(), false),
      grade: (rating) => void this.run(() => this.controller.grade(rating), true),
      markKnown: () => void this.run(() => this.controller.markKnown(), true),
    };
  }

  /** Keyboard (Alt+Shift+D): toggle the panel open/closed on the review view. */
  async toggle(): Promise<void> {
    if (this.open) {
      this.hide();
      return;
    }
    await this.openOn("review");
  }

  /** Open the panel on a specific view (the HUD / popup entry points). */
  async openOn(view: DrawerView): Promise<void> {
    if (!this.host.isConnected) document.documentElement.appendChild(this.host);
    this.open = true;
    this.panel.hidden = false;
    await this.switchTo(view);
  }

  hide(): void {
    this.open = false;
    this.panel.hidden = true;
  }

  private async switchTo(view: DrawerView): Promise<void> {
    this.reviewBody.hidden = view !== "review";
    this.statsBody.hidden = view !== "stats";
    this.settingsBody.hidden = view !== "settings";
    for (const [v, b] of this.tabs) b.classList.toggle("active", v === view);
    if (view === "review") {
      await this.run(() => this.controller.start(), false);
    } else if (view === "stats") {
      await mountStats(this.statsBody, this.opts.port, this.opts.area);
    } else {
      this.settings ??= mountSettings(this.settingsBody, this.opts.port, this.opts.area, {
        persist: this.opts.onChange,
        onReset: async () => {
          // A reset wiped the deck: rebuild the review controller so its view is current.
          this.controller = new ReviewController(this.opts.port, this.opts.now, this.opts.record);
          await this.run(() => this.controller.start(), false);
        },
      });
      await this.settings.refresh();
    }
  }

  private async run(produce: () => Promise<ReviewView>, persist: boolean): Promise<void> {
    const view = await produce();
    if (persist) await this.opts.onChange();
    renderReview(this.reviewBody, view, this.actions);
  }
}
