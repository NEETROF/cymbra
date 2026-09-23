import type { MarkedTranslation } from "../translate/markup.ts";
import type { LemmaStatus } from "../analyzer/types.ts";
import { isTouchPrimary } from "../state/platform.ts";

// The on-page word popup: a closed shadow root (isolated from page CSS and JS), showing
// the dictionary form, the form as seen, the pack gloss, a plain-language rarity note,
// and the three reading gestures. It owns no classification or persistence — it emits a
// Gesture and the content script does the rest (persist, re-sync the engine, repaint,
// broadcast to other tabs). The internal word "lemma" appears in NO user-visible string.
//
// The card view (createCard) is separated from the shadow host so the rendering and
// gesture wiring can be unit-tested without a closed shadow root (which is unreachable
// from outside by design).
//
// A card that waits for the engine is shown twice: pending (no action), then complete.
// There is no in-place update — the only thing that ever changes a card is `show` — so
// the gesture key is always the one on screen, and the view's generation, bumped by every
// show and every hide, tells the caller whether an answer still has a card to land on.

/** A gesture the user made on a word or phrase. */
export interface Gesture {
  /** The dictionary form (single word) or the lowercased phrase (expression). */
  lemma: string;
  /** The surface text as seen. */
  surface: string;
  /** The source sentence, carried onto a created card. */
  sentence: string;
  /** The chosen status, or `null` to clear it ("Remettre à apprendre"). */
  status: LemmaStatus | null;
  /** Whether the card was a multi-word expression (its gloss is never asked of the pack). */
  expression: boolean;
  /** The answer the card showed, so an expression's gloss reaches the card it creates. */
  gloss: string | null;
}

/** One word-by-word row: a dictionary form the reader does not know, with its pack gloss. */
export interface GlossRow {
  form: string;
  gloss: string;
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
  /** The word's current status (drives which actions are offered); null = new/unknown. */
  status?: LemmaStatus | null;
  /** Anchor rectangle in viewport coordinates (the word's box). */
  rect: { left: number; top: number; bottom: number };
  /** The card is waiting for the engine: a waiting line in place of the answer, no action. */
  pending?: boolean;
  /**
   * A translation is still on its way. On a pending card it says what is being waited for — the
   * pack answers in milliseconds, so a card that also asked the engine is waiting for THAT. On a
   * completed card it says the pack's answer is not the last word: the engine is slower (seconds,
   * on a cold device) and its answer will replace it.
   */
  translating?: boolean;
  /** Word-by-word rows, shown under their label instead of the gloss line when non-empty. */
  rows?: GlossRow[];
  /** A complete card that offers nothing to press (its key never arrived). */
  noActions?: boolean;
  /**
   * The reader's sentence, machine-translated, with where their selection landed — computed
   * for display only (add-lingua-translation-engine). No gesture carries it, so it can never
   * reach a card: only dictionary data is stored.
   */
  translation?: MarkedTranslation | null;
}

/** A card view: a detached element tree plus show/hide, independent of any shadow root. */
export interface CardView {
  readonly el: HTMLElement;
  /** Render `content` and return the card's new generation. */
  show(content: WordPopupContent, onGesture: (g: Gesture) => void): number;
  hide(): void;
  visible(): boolean;
  /** A counter bumped by every show and every hide — the close button and a gesture included. */
  generation(): number;
}

const ROWS_LABEL = "Mot à mot — ce n'est pas une traduction de l'expression.";
const TRANSLATION_LABEL = "Dans votre phrase — traduction automatique";
const WAITING = "Recherche dans le pack…";
const TRANSLATING = "Traduction en cours…";
const NO_GLOSS = "Pas de traduction dans le pack.";
const NO_GLOSS_EXPRESSION = "Pas de traduction dans le pack pour cette expression.";

/** Build the card view (no shadow root involved — testable in isolation). */
export function createCard(): CardView {
  const el = div("card");
  el.hidden = true;
  el.addEventListener("click", (e) => e.stopPropagation());

  const headwordEl = div("headword");
  const seenEl = div("seen");
  const rarityEl = div("rarity");
  const glossEl = div("gloss");
  const translationEl = div("translation");
  const actionsEl = div("actions");
  el.append(headwordEl, seenEl, rarityEl, glossEl, translationEl, actionsEl);

  let current: WordPopupContent | null = null;
  let generation = 0;

  function button(
    label: string,
    status: LemmaStatus | null,
    primary: boolean,
    onGesture: (g: Gesture) => void,
  ): HTMLButtonElement {
    const b = document.createElement("button");
    b.textContent = label;
    if (primary) b.classList.add("primary");
    b.addEventListener("click", () => {
      if (!current) return;
      onGesture({
        lemma: current.headword,
        surface: current.surface,
        sentence: current.sentence,
        status,
        expression: !!current.expression,
        gloss: current.gloss,
      });
      view.hide();
    });
    return b;
  }

  /**
   * The answer slot: the waiting line, the labelled rows, the gloss, or the no-gloss note. A
   * translation is a better answer than word-by-word rows, so with one the rows never show —
   * and neither does the note that the pack has no translation, which would sit, untrue,
   * right above one.
   */
  function renderAnswer(content: WordPopupContent): void {
    glossEl.replaceChildren();
    glossEl.classList.remove("empty", "waiting");
    glossEl.hidden = false;
    if (content.pending) {
      glossEl.textContent = content.translating ? TRANSLATING : WAITING;
      glossEl.classList.add("waiting");
      return;
    }
    renderPackAnswer(content);
    // The pack has answered and the engine has not. Say so under its answer: a reader looking at
    // word-by-word rows must know a translation is still coming, so the rows are never mistaken
    // for the last word on their selection.
    if (content.translating && !content.translation) {
      const note = div("translating-note");
      note.textContent = TRANSLATING;
      glossEl.hidden = false;
      glossEl.append(note);
    }
  }

  /** What the pack alone has to say: the translated sentence's own gloss, rows, or neither. */
  function renderPackAnswer(content: WordPopupContent): void {
    if (content.translation) {
      if (content.gloss) glossEl.textContent = content.gloss;
      else glossEl.hidden = true;
      return;
    }
    if (content.rows && content.rows.length > 0) {
      const label = div("rows-label");
      label.textContent = ROWS_LABEL;
      glossEl.append(label);
      for (const r of content.rows) {
        const row = div("row");
        row.textContent = `${r.form} → ${r.gloss}`;
        glossEl.append(row);
      }
      return;
    }
    if (content.gloss) {
      glossEl.textContent = content.gloss;
      return;
    }
    glossEl.textContent = content.expression ? NO_GLOSS_EXPRESSION : NO_GLOSS;
    glossEl.classList.add("empty");
  }

  /**
   * The reader's sentence, translated, their selection's place marked. Built from text nodes
   * only: the sentence came from the page and through the engine, so as markup it could carry
   * anything the page held.
   */
  function renderTranslation(content: WordPopupContent): void {
    translationEl.replaceChildren();
    const t = content.pending ? null : content.translation;
    translationEl.hidden = !t;
    if (!t) return;
    const label = div("translation-label");
    label.textContent = TRANSLATION_LABEL;
    const sentence = div("translation-sentence");
    let at = 0;
    for (const { start, end } of [...t.marks].sort((a, b) => a.start - b.start)) {
      if (start < at || end > t.sentence.length) continue; // overlapping or out of range: skip, never throw
      sentence.append(document.createTextNode(t.sentence.slice(at, start)));
      const mark = document.createElement("mark");
      mark.textContent = t.sentence.slice(start, end);
      sentence.append(mark);
      at = end;
    }
    sentence.append(document.createTextNode(t.sentence.slice(at)));
    translationEl.append(label, sentence);
  }

  const view: CardView = {
    el,
    visible: () => !el.hidden,
    generation: () => generation,
    hide() {
      generation++;
      el.hidden = true;
      current = null;
    },
    show(content, onGesture) {
      generation++;
      current = content;
      headwordEl.textContent = content.headword;

      const differs = !!content.surface && content.surface.toLowerCase() !== content.headword.toLowerCase();
      seenEl.textContent = differs ? `forme vue : « ${content.surface} »` : "";
      seenEl.hidden = !differs;

      rarityEl.textContent = content.rarity;

      renderAnswer(content);
      renderTranslation(content);

      // Actions depend on the word's current status: hide the one it already is. Offer
      // "Remettre à apprendre" (clear) only for an IGNORED word — ignored is always an
      // explicit decision, and clearing withdraws it so the word is highlighted again. A
      // displayed "Known" may be merely presumed by level/frequency, so no clear there
      // ("+ Deck" is the real "I want to learn this" for such a word). A pending card
      // offers nothing yet — its key may change with the answer — and so does a complete
      // card whose key never arrived.
      actionsEl.replaceChildren();
      if (!content.pending && !content.noActions) {
        const st = content.status ?? null;
        if (!content.expression && st !== "known") actionsEl.append(button("Je connais", "known", false, onGesture));
        if (st !== "learning") actionsEl.append(button("+ Deck", "learning", true, onGesture));
        if (st !== "ignored") actionsEl.append(button("Ignorer", "ignored", false, onGesture));
        if (st === "ignored") actionsEl.append(button("Remettre à apprendre", null, false, onGesture));
      }
      actionsEl.hidden = actionsEl.childElementCount === 0;

      el.hidden = false; // reveal first so the card can be measured, then position it
      positionCard(el, content.rect);
      return generation;
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

  /** Render `content` and return the card's new generation (see `generation`). */
  show(content: WordPopupContent): number {
    this.attach();
    return this.view.show(content, this.opts.onGesture);
  }

  /** The card view's generation: an answer for an earlier one has no card to land on. */
  generation(): number {
    return this.view.generation();
  }
}

function div(className: string): HTMLElement {
  const e = document.createElement("div");
  e.className = className;
  return e;
}

/** Clearance (px) left for the platform's own selection callout (Copier / Chercher /
 *  Traduire) on the FLIPPED branch only.
 *
 *  Measured on an iPhone: iOS has the same placement preference this card does — below the
 *  selection when there is room, flipped above near the bottom of the viewport. So the two
 *  DO collide in the common case, and a gutter on the below branch would fix it. That was
 *  tried and rejected on device: pushing the card ~56 px down detaches it from the words it
 *  describes, which costs more than the overlap does — the callout is one tap from gone,
 *  and it only really shows over the taller multi-word card. The flipped branch keeps its
 *  gutter because there the card would otherwise sit directly ON the bar with nothing to
 *  dismiss it first. */
const CALLOUT_GUTTER = 44;

/**
 * Place the fixed card fully within the viewport: just below the word, flipped ABOVE it
 * when there isn't room below, and finally clamped so it is never clipped. Near the bottom
 * of the page an un-flipped card showed only partially and — being `position: fixed` —
 * could not be scrolled into view; the flip + clamp fix that. On a touch device the flip
 * also clears the platform's selection callout (see CALLOUT_GUTTER — the un-flipped branch
 * deliberately does not). Measured after the card is revealed so its
 * real height/width drive the placement.
 */
function positionCard(el: HTMLElement, rect: { left: number; top: number; bottom: number }): void {
  const r = el.getBoundingClientRect();
  el.style.left = `${clamp(rect.left, 8, viewportWidth() - r.width - 8)}px`;
  let top = rect.bottom + 8; // prefer just below the word (where no callout sits)
  if (top + r.height > viewportHeight() - 8) {
    const gutter = isTouchPrimary() ? CALLOUT_GUTTER : 0; // no room below → flip above the callout
    top = rect.top - 8 - gutter - r.height;
  }
  el.style.top = `${clamp(top, 8, viewportHeight() - r.height - 8)}px`; // keep it fully on screen
}

function clamp(x: number, lo: number, hi: number): number {
  const top = hi < lo ? lo : hi;
  return Math.max(lo, Math.min(top, x));
}

function viewportWidth(): number {
  return document.documentElement.clientWidth || window.innerWidth || 1024;
}

function viewportHeight(): number {
  return document.documentElement.clientHeight || window.innerHeight || 768;
}
