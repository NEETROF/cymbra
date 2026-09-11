import type { LemmaStatus } from "../analyzer/types.ts";

// The on-page word popup: a closed shadow root (isolated from page CSS and JS), showing
// the dictionary form, the form as seen, the pack gloss, a plain-language rarity note,
// and the three reading gestures. It owns no classification or persistence — it emits a
// Gesture and the content script does the rest (persist, re-sync the engine, repaint,
// broadcast to other tabs). The internal word "lemma" appears in NO user-visible string.
//
// The card view (createCard) is separated from the shadow host so the rendering and
// gesture wiring can be unit-tested without a closed shadow root (which is unreachable
// from outside by design).

/** A gesture the user made on a word or phrase. */
export interface Gesture {
  /** The dictionary form (single word) or the lowercased phrase (expression). */
  lemma: string;
  /** The surface text as seen. */
  surface: string;
  /** The source sentence, carried onto a created card. */
  sentence: string;
  status: LemmaStatus;
}

export interface WordPopupContent {
  /** Headword — the dictionary form, shown with no "lemma" label. */
  headword: string;
  /** The surface text as seen (for the "forme vue" line when it differs). */
  surface: string;
  /** The native-language gloss, or null when the pack has none. */
  gloss: string | null;
  /** Plain-language rarity note (never a bare number). */
  rarity: string;
  /** The source sentence for a created card. */
  sentence: string;
  /** Whether this is a multi-word expression (hides "Je connais"). */
  expression?: boolean;
  /** Anchor rectangle in viewport coordinates. */
  rect: { left: number; bottom: number };
}

/** A card view: a detached element tree plus show/hide, independent of any shadow root. */
export interface CardView {
  readonly el: HTMLElement;
  show(content: WordPopupContent, onGesture: (g: Gesture) => void): void;
  hide(): void;
  visible(): boolean;
}

/** Build the card view (no shadow root involved — testable in isolation). */
export function createCard(): CardView {
  const el = div("card");
  el.hidden = true;
  el.addEventListener("click", (e) => e.stopPropagation());

  const headwordEl = div("headword");
  const seenEl = div("seen");
  const rarityEl = div("rarity");
  const glossEl = div("gloss");
  const actionsEl = div("actions");
  el.append(headwordEl, seenEl, rarityEl, glossEl, actionsEl);

  let current: WordPopupContent | null = null;

  function button(
    label: string,
    status: LemmaStatus,
    primary: boolean,
    onGesture: (g: Gesture) => void,
  ): HTMLButtonElement {
    const b = document.createElement("button");
    b.textContent = label;
    if (primary) b.classList.add("primary");
    b.addEventListener("click", () => {
      if (!current) return;
      onGesture({ lemma: current.headword, surface: current.surface, sentence: current.sentence, status });
      view.hide();
    });
    return b;
  }

  const view: CardView = {
    el,
    visible: () => !el.hidden,
    hide() {
      el.hidden = true;
      current = null;
    },
    show(content, onGesture) {
      current = content;
      headwordEl.textContent = content.headword;

      const differs = !!content.surface && content.surface.toLowerCase() !== content.headword.toLowerCase();
      seenEl.textContent = differs ? `forme vue : « ${content.surface} »` : "";
      seenEl.hidden = !differs;

      rarityEl.textContent = content.rarity;

      if (content.gloss) {
        glossEl.textContent = content.gloss;
        glossEl.classList.remove("empty");
      } else {
        glossEl.textContent = "Pas de traduction dans le pack.";
        glossEl.classList.add("empty");
      }

      actionsEl.replaceChildren();
      if (!content.expression) actionsEl.append(button("Je connais", "known", false, onGesture));
      actionsEl.append(button("+ Deck", "learning", true, onGesture), button("Ignorer", "ignored", false, onGesture));

      el.style.left = `${clamp(content.rect.left, 8, viewportWidth() - 296)}px`;
      el.style.top = `${content.rect.bottom + 8}px`;
      el.hidden = false;
    },
  };

  // A close affordance (clicking off the popup also dismisses it — see content.ts).
  const closeEl = document.createElement("button");
  closeEl.className = "close";
  closeEl.textContent = "✕"; // ✕
  closeEl.setAttribute("aria-label", "Fermer");
  closeEl.addEventListener("click", () => view.hide());
  el.append(closeEl);

  return view;
}

export interface WordPopupOptions {
  /** Combined token sheet + popup styles, injected into the shadow root. */
  css: string;
  /** Called when the user picks a gesture. */
  onGesture: (gesture: Gesture) => void;
}

export class WordPopup {
  /** The host element in the page (excluded from scanning by its id). */
  readonly host: HTMLElement;
  private readonly view: CardView = createCard();

  constructor(private readonly opts: WordPopupOptions) {
    this.host = document.createElement("div");
    this.host.id = "cymbra-lingua-host";
    this.host.setAttribute("data-cymbra-lingua-skip", "");
    const root = this.host.attachShadow({ mode: "closed" });
    const style = document.createElement("style");
    style.textContent = opts.css;
    root.append(style, this.view.el);
  }

  private attach(): void {
    if (!this.host.isConnected) document.documentElement.appendChild(this.host);
  }

  visible(): boolean {
    return this.view.visible();
  }

  /** Whether an event target is inside the popup (used to ignore self-clicks). */
  contains(target: EventTarget | null): boolean {
    return target === this.host || (target instanceof Node && this.host.contains(target));
  }

  hide(): void {
    this.view.hide();
  }

  show(content: WordPopupContent): void {
    this.attach();
    this.view.show(content, this.opts.onGesture);
  }
}

function div(className: string): HTMLElement {
  const e = document.createElement("div");
  e.className = className;
  return e;
}

function clamp(x: number, lo: number, hi: number): number {
  const top = hi < lo ? lo : hi;
  return Math.max(lo, Math.min(top, x));
}

function viewportWidth(): number {
  return document.documentElement.clientWidth || 1024;
}
