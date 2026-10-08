import { languageName } from "../analyzer/language-labels.ts";
import type { Rating } from "../analyzer/port.ts";
import { review as enReview } from "../i18n/en/review.ts";
import { review as esReview } from "../i18n/es/review.ts";
import { review as frReview } from "../i18n/fr/review.ts";
import { DEFAULT_INTERFACE_LANGUAGE, type InterfaceLanguage, NODE_SLOT, plural, renderAround } from "../i18n/index.ts";
import type { ReviewView } from "./session.ts";

// The review render — one function both surfaces (side panel and injected drawer) call
// to paint a ReviewView, wiring its controls to the given actions. Pure DOM, no styling
// of its own (the host provides the token sheet); no user-visible "lemma". Its words are the
// catalogue's `review` module in the interface language (localise-lingua-review-stats D1).

/** The review's copy: the catalogue's `review` module, in the interface language (its French the default). */
export type ReviewCopy = typeof frReview;

/** The review's copy by interface language: the one place that holds all three. */
const REVIEW_COPY: Record<InterfaceLanguage, ReviewCopy> = { fr: frReview, en: enReview, es: esReview };

/** The review's module for the interface language — the render's, and the page's around it. */
export function reviewCopy(language: InterfaceLanguage): ReviewCopy {
  return REVIEW_COPY[language];
}

export interface ReviewActions {
  start(): void;
  reveal(): void;
  grade(rating: Rating): void;
  markKnown(): void;
}

export interface ReviewRenderOptions {
  /** The reader studies several languages, so each card says its own. */
  showLanguage?: boolean;
  /** The interface language, whose module the review speaks and whose plural forms its counts
   *  take — the copy derived from it, so the two cannot disagree; French when not given. */
  interfaceLanguage?: InterfaceLanguage;
}

/** The grades, in the order the card shows them; each is labelled by the copy's entry of its name. */
const GRADES: readonly Rating[] = ["again", "hard", "good", "easy"];

/** Render `view` into `root`, replacing its contents. */
export function renderReview(
  root: HTMLElement,
  view: ReviewView,
  actions: ReviewActions,
  opts: ReviewRenderOptions = {},
): void {
  const language = opts.interfaceLanguage ?? DEFAULT_INTERFACE_LANGUAGE;
  const copy = REVIEW_COPY[language];
  root.replaceChildren();

  if (view.phase === "idle") {
    root.append(button(copy.start, () => actions.start(), true));
    return;
  }

  if (view.phase === "done" || !view.card) {
    root.append(note(copy.nothingDue));
    return;
  }

  const card = view.card;
  // The card's words are the studied language's, inside a surface that says the interface's (the
  // drawer's host, the side panel's page): they say their own, as the engine counts them — an older
  // card without one is English (localise-lingua-reading-surfaces D3).
  const studied = card.language ?? "en";
  root.append(note(plural(language, card.remaining, copy.remaining), "remaining"));
  // The card's language, when the reader studies several (add-lingua-language-stats-review D2).
  if (opts.showLanguage && card.language) root.append(note(languageName(card.language), "review-language"));
  root.append(headword(card.headword, studied));

  if (!card.revealed) {
    root.append(button(copy.reveal, () => actions.reveal(), true));
    return;
  }

  if (card.gloss) root.append(line("gloss", card.gloss));
  if (card.sentence) {
    const sentence = line("sentence", "");
    const words = document.createElement("span");
    words.lang = studied;
    words.textContent = card.sentence;
    renderAround(sentence, copy.sentence(NODE_SLOT), words);
    root.append(sentence);
  }
  const source = sourceLabel(card.source);
  if (source) root.append(line("source", source));

  const grades = document.createElement("div");
  grades.className = "review-grades";
  for (const rating of GRADES) grades.append(button(copy[rating], () => actions.grade(rating), rating === "good"));
  root.append(
    grades,
    button(copy.markKnown, () => actions.markKnown(), false),
  );
}

/**
 * Where a card's word was met, short: a page by its site, a book by its title and chapter
 * (add-lingua-reader). The source is a label the device kept, not a link — the page may be
 * gone, the book deleted, and the card keeps it all the same.
 */
export function sourceLabel(source: string | undefined): string {
  if (!source) return "";
  try {
    const url = new URL(source);
    if (url.protocol === "http:" || url.protocol === "https:") return url.hostname.replace(/^www\./, "");
  } catch {
    // Not an address: a book's title and chapter, shown as they are.
  }
  return source;
}

function button(label: string, onClick: () => void, primary: boolean): HTMLButtonElement {
  const b = document.createElement("button");
  b.className = primary ? "review-btn primary" : "review-btn";
  b.textContent = label;
  b.addEventListener("click", onClick);
  return b;
}

function note(text: string, extra = ""): HTMLElement {
  const d = document.createElement("div");
  d.className = extra ? `review-note ${extra}` : "review-note";
  d.textContent = text;
  return d;
}

function headword(text: string, language: string): HTMLElement {
  const d = document.createElement("div");
  d.className = "review-headword";
  d.lang = language;
  d.textContent = text;
  return d;
}

function line(kind: string, text: string): HTMLElement {
  const d = document.createElement("div");
  d.className = `review-${kind}`;
  d.textContent = text;
  return d;
}
