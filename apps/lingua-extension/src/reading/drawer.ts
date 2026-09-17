import type { LinguaPort } from "../analyzer/port.ts";
import { mountReview, type ReviewPage } from "../review/review-page.ts";
import { type AsyncStorageArea } from "../state/storage.ts";
import { mountStats } from "../stats/view.ts";
import { requestSync } from "../sync/messages.ts";
import { mountSettings, type SettingsView } from "./settings-view.ts";

// The injected in-page panel: a closed-shadow overlay with Révision / Statistiques /
// Réglages, so the reader never has to LEAVE the page it is reading (design D1, extended).
// Every view is the SAME module the native side panel renders — mountReview, mountStats,
// mountSettings — so the content is identical across the two hosts. On Firefox (where a page
// element cannot open the sidebar) this is THE panel the HUD and popup open; on Chromium the
// native side panel is used instead and this stays the Alt+Shift+D micro-review.

export type DrawerView = "review" | "stats" | "settings";

export interface DrawerOptions {
  /** Combined token sheet + review + stats + settings + drawer styles, for the shadow. */
  css: string;
  port: LinguaPort;
  /** Preferences surface (the HUD toggle, the last-sync time): chrome.storage.local. */
  area: AsyncStorageArea;
  /** The reader's data (deck, statistics, cursors), owned by the background. */
  store: AsyncStorageArea;
  /** Epoch-seconds clock (Date.now()/1000 in production). */
  now: () => number;
  /** Persist after a state-changing settings action (backup → storage). */
  onChange: () => Promise<void>;
}

export class Drawer {
  private readonly host: HTMLElement;
  private readonly panel: HTMLElement;
  private readonly lost: HTMLElement;
  private readonly reviewBody: HTMLElement;
  private readonly statsBody: HTMLElement;
  private readonly settingsBody: HTMLElement;
  private readonly tabs: Map<DrawerView, HTMLButtonElement> = new Map();
  private reviewPage: ReviewPage | null = null;
  private settings: SettingsView | null = null;
  private open = false;
  private current: DrawerView = "review";

  constructor(private readonly opts: DrawerOptions) {
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

    this.lost = document.createElement("div");
    this.lost.className = "drawer-lost";
    this.lost.textContent = "Session expirée — reconnecte-toi depuis le menu de l'extension pour synchroniser.";
    this.lost.hidden = true;

    this.reviewBody = document.createElement("div");
    this.statsBody = document.createElement("div");
    this.statsBody.hidden = true;
    this.settingsBody = document.createElement("div");
    this.settingsBody.hidden = true;
    this.panel.append(head, this.lost, this.reviewBody, this.statsBody, this.settingsBody);
    root.append(style, this.panel);
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
    void requestSync("surface");
    await this.switchTo(view);
  }

  /** Show (or hide) the banner for a session the server refused. */
  setSessionLost(lost: boolean): void {
    this.lost.hidden = !lost;
  }

  /**
   * Redraw the open view after the engine changed under it (a reading gesture, a sync pull).
   * Révision keeps a review under way: its page only redraws the current card.
   */
  async refresh(): Promise<void> {
    if (this.open) await this.switchTo(this.current);
  }

  hide(): void {
    this.open = false;
    this.panel.hidden = true;
  }

  private async switchTo(view: DrawerView): Promise<void> {
    this.current = view;
    this.reviewBody.hidden = view !== "review";
    this.statsBody.hidden = view !== "stats";
    this.settingsBody.hidden = view !== "settings";
    for (const [v, b] of this.tabs) b.classList.toggle("active", v === view);
    if (view === "review") {
      this.reviewPage ??= mountReview(this.reviewBody, this.opts.port, this.opts.store, { now: this.opts.now });
      await this.reviewPage.refresh();
    } else if (view === "stats") {
      await mountStats(this.statsBody, this.opts.port, this.opts.store);
    } else {
      this.settings ??= mountSettings(this.settingsBody, this.opts.port, this.opts.area, {
        persist: this.opts.onChange,
        store: this.opts.store,
      });
      await this.settings.refresh();
    }
  }
}
