import type { LinguaPort } from "../analyzer/port.ts";
import { drawer as frDrawer } from "../i18n/fr/drawer.ts";
import { DEFAULT_INTERFACE_LANGUAGE, type InterfaceLanguage } from "../i18n/index.ts";
import { mountReview, type ReviewPage } from "../review/review-page.ts";
import { type AsyncStorageArea } from "../state/storage.ts";
import { mountStats } from "../stats/view.ts";
import { requestSync } from "../sync/messages.ts";
import { mountSettings, type SettingsView } from "./settings-view.ts";
import type { Speaker } from "./speech.ts";
import { followSurfaceLook } from "./surface-look.ts";

// The injected in-page panel: a closed-shadow overlay with Révision / Statistiques /
// Réglages, so the reader never has to LEAVE the page it is reading (design D1, extended).
// Every view is the SAME module the native side panel renders — mountReview, mountStats,
// mountSettings — so the content is identical across the two hosts. On Firefox (where a page
// element cannot open the sidebar) this is THE panel the HUD and popup open; on Chromium the
// native side panel is used instead and this stays the Alt+Shift+D micro-review.

export type DrawerView = "review" | "stats" | "settings";

/** The drawer's copy: the catalogue's `drawer` module, in the interface language (its French the default). */
export type DrawerCopy = typeof frDrawer;

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
  /** The page's speaker, whose voices Réglages lists. */
  speaker?: Speaker;
  /** Follow the reader's colours and text size (surface-look); off: the drawer as designed. */
  followLook?: boolean;
  /** The language of the page or book the drawer opens on, where Révision opens
   *  (refine-lingua-review-language D2). */
  pageLanguage?: () => Promise<string | null>;
  /** The interface language, said by the host's `lang` (localise-lingua-reading-surfaces D3); French when not given. */
  language?: InterfaceLanguage;
  /** The drawer's copy in that language; the French module when not given. */
  copy?: DrawerCopy;
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
    const copy = opts.copy ?? frDrawer;
    this.host = document.createElement("div");
    this.host.id = "cymbra-lingua-drawer-host";
    this.host.setAttribute("data-cymbra-lingua-skip", "");
    this.host.lang = opts.language ?? DEFAULT_INTERFACE_LANGUAGE;
    const root = this.host.attachShadow({ mode: "closed" });
    if (opts.followLook) followSurfaceLook(this.host, opts.area);
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
    addTab("review", copy.review);
    addTab("stats", copy.stats);
    addTab("settings", copy.settings);
    const close = document.createElement("button");
    close.className = "drawer-close";
    close.textContent = copy.closeIcon;
    close.setAttribute("aria-label", copy.close);
    close.addEventListener("click", () => this.hide());
    head.append(tabsRow, close);

    this.lost = document.createElement("div");
    this.lost.className = "drawer-lost";
    this.lost.textContent = copy.sessionLost;
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
    this.reviewPage?.pageChanged(); // an opening: Révision opens in the page's language (D3)
    await this.switchTo(view);
  }

  /** The document's language changed under the drawer: Révision follows it, now if it is shown. */
  async pageChanged(): Promise<void> {
    this.reviewPage?.pageChanged();
    if (this.open && this.current === "review") await this.switchTo("review");
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

  /** Leave the page: its reading session is taken down (add-lingua-native-language-choice D3). */
  destroy(): void {
    this.hide();
    this.host.remove();
  }

  private async switchTo(view: DrawerView): Promise<void> {
    this.current = view;
    this.reviewBody.hidden = view !== "review";
    this.statsBody.hidden = view !== "stats";
    this.settingsBody.hidden = view !== "settings";
    for (const [v, b] of this.tabs) b.classList.toggle("active", v === view);
    if (view === "review") {
      this.reviewPage ??= mountReview(this.reviewBody, this.opts.port, this.opts.store, {
        now: this.opts.now,
        prefs: this.opts.area,
        pageLanguage: this.opts.pageLanguage,
        interfaceLanguage: this.opts.language,
      });
      await this.reviewPage.refresh();
    } else if (view === "stats") {
      await mountStats(this.statsBody, this.opts.port, this.opts.store, undefined, this.opts.language);
    } else {
      this.settings ??= mountSettings(this.settingsBody, this.opts.port, this.opts.area, {
        persist: this.opts.onChange,
        store: this.opts.store,
        speaker: this.opts.speaker,
        // The language the reading session handed the drawer (localise-lingua-settings D1).
        interfaceLanguage: this.opts.language,
      });
      await this.settings.refresh();
    }
  }
}
