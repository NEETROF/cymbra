import type { LinguaPort } from "../analyzer/port.ts";
import { ReviewController, type ReviewView } from "../review/session.ts";
import { type ReviewActions, renderReview } from "../review/view.ts";

// The injected review drawer: a collapsible closed-shadow panel for micro-reviews
// without leaving the page (design D1). It shares the ReviewController + renderReview
// with the side panel — two rendering hosts over one review logic. State changes
// (grade, mark-known) persist through `onChange` so the side panel and reading badge
// update via storage.onChanged. On Safari (no side panel API) this is the sole
// in-browser review surface.

export interface DrawerOptions {
  /** Combined token sheet + review styles + drawer styles, injected into the shadow. */
  css: string;
  port: LinguaPort;
  /** Epoch-seconds clock (Date.now()/1000 in production). */
  now: () => number;
  /** Persist after a state-changing action (backup → storage). */
  onChange: () => Promise<void>;
}

export class Drawer {
  private readonly host: HTMLElement;
  private readonly panel: HTMLElement;
  private readonly body: HTMLElement;
  private readonly controller: ReviewController;
  private readonly actions: ReviewActions;
  private open = false;

  constructor(private readonly opts: DrawerOptions) {
    this.controller = new ReviewController(opts.port, opts.now);

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
    const title = document.createElement("div");
    title.className = "drawer-title";
    title.textContent = "Révision";
    const close = document.createElement("button");
    close.className = "drawer-close";
    close.textContent = "×";
    close.setAttribute("aria-label", "Fermer");
    close.addEventListener("click", () => this.hide());
    head.append(title, close);

    this.body = document.createElement("div");
    this.panel.append(head, this.body);
    root.append(style, this.panel);

    this.actions = {
      start: () => void this.run(() => this.controller.start(), false),
      reveal: () => void this.run(() => this.controller.reveal(), false),
      grade: (rating) => void this.run(() => this.controller.grade(rating), true),
      markKnown: () => void this.run(() => this.controller.markKnown(), true),
    };
  }

  /** Open the drawer (starting a fresh session) or close it. */
  async toggle(): Promise<void> {
    if (!this.host.isConnected) document.documentElement.appendChild(this.host);
    this.open = !this.open;
    this.panel.hidden = !this.open;
    if (this.open) await this.run(() => this.controller.start(), false);
  }

  hide(): void {
    this.open = false;
    this.panel.hidden = true;
  }

  private async run(produce: () => Promise<ReviewView>, persist: boolean): Promise<void> {
    const view = await produce();
    if (persist) await this.opts.onChange();
    renderReview(this.body, view, this.actions);
  }
}
