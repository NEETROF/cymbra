import { languageName } from "../analyzer/language-labels.ts";
import type { Rating, ReviewCard, ReviewSummary } from "../analyzer/port.ts";
import { type ReviewView, SESSION_CARDS } from "./session.ts";

// The review render — one function every review host (the side panel and the injected drawer)
// calls to paint a ReviewView, wiring its controls to the given actions. Pure DOM, no styling of
// its own (the host provides the token sheet and review.css); no user-visible "lemma".
//
// The card reads like the page it came from (refine-lingua-review-session D6, D7): the sentence
// first, the word marked; the answer's space there from the start, so revealing it changes that
// space only; one action zone the same size before and after — « Afficher la réponse », then
// « Pas su » on the left and « Su » on the right, like the reader's page turns — and two links.
// Keys are read on the review's own root, never on the page: Space or Enter reveals, the left
// and right arrows answer.

export interface ReviewActions {
  start(): void;
  reveal(): void;
  grade(rating: Rating): void;
  markKnown(): void;
  ignore(): void;
}

/**
 * Render `view` into `root`, replacing its contents. `showLanguage`: the reader studies several
 * languages, so each card says its own. A root that held the focus keeps it.
 */
export function renderReview(
  root: HTMLElement,
  view: ReviewView,
  actions: ReviewActions,
  opts: { showLanguage?: boolean } = {},
): void {
  const scope = root.getRootNode() as Document | ShadowRoot;
  const focused = scope.activeElement;
  const hadFocus = focused === root || (focused !== null && root.contains(focused));
  root.replaceChildren();
  root.tabIndex = -1;
  // A property, not a listener: each render replaces the last one instead of adding another.
  root.onkeydown = (e) => onKey(e, root, view, actions);

  if (view.phase === "idle") root.append(button("Réviser", "review-btn primary review-start", () => actions.start()));
  else if (view.phase === "done" || !view.card) root.append(ending(view, actions));
  else root.append(cardView(view.card, actions, opts));

  if (hadFocus) root.focus({ preventScroll: true });
}

/** Space or Enter reveals; once revealed, ← answers « Pas su » and → « Su ». */
function onKey(e: KeyboardEvent, root: HTMLElement, view: ReviewView, actions: ReviewActions): void {
  const card = view.phase === "reviewing" ? view.card : null;
  if (!card || e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
  let act: (() => void) | null = null;
  if (!card.revealed && (e.key === " " || e.key === "Enter")) {
    // A link or another button keeps its own Space and Enter.
    const target = e.target as HTMLElement | null;
    if (target !== root && target?.tagName === "BUTTON" && !target.classList.contains("review-reveal")) return;
    act = () => actions.reveal();
  } else if (card.revealed && e.key === "ArrowLeft") {
    act = () => actions.grade("again");
  } else if (card.revealed && e.key === "ArrowRight") {
    act = () => actions.grade("good");
  }
  if (!act) return;
  e.preventDefault();
  e.stopPropagation();
  act();
}

function cardView(card: ReviewCard, actions: ReviewActions, opts: { showLanguage?: boolean }): HTMLElement {
  const main = div("review-main");
  const head = div("review-head");
  head.append(note(`${card.remaining} carte(s) à revoir`, "remaining"));
  // The card's language, when the reader studies several (add-lingua-language-stats-review D2).
  if (opts.showLanguage && card.language) head.append(note(languageName(card.language), "review-language"));
  main.append(head);

  const source = sourceLabel(card.source);
  if (source) main.append(line("source", source));
  const sentence = card.sentence.trim();
  const marked = sentence ? markWord(sentence, card.surface, card.headword) : null;
  // No sentence (a word seeded from a level), or none holding the word: the word itself.
  if (!marked) main.append(line("headword", card.headword));
  if (sentence) main.append(sentenceView(sentence, marked));

  // The answer's space, there from the start: revealing fills it and moves nothing else.
  const answer = div("review-answer");
  answer.setAttribute("aria-live", "polite");
  if (card.revealed) {
    // The dictionary form, when the sentence shows another one.
    if (marked && marked.text.toLowerCase() !== card.headword.toLowerCase())
      answer.append(line("dictionary-form", card.headword));
    answer.append(card.gloss ? line("gloss", card.gloss) : note("Pas de traduction pour ce mot.", "review-no-gloss"));
  }
  main.append(answer);

  const answers = div("review-actions");
  if (!card.revealed) {
    answers.append(button("Afficher la réponse", "review-btn primary review-reveal", () => actions.reveal()));
  } else {
    answers.append(
      button("Pas su", "review-btn review-miss", () => actions.grade("again")),
      button("Su", "review-btn primary review-hit", () => actions.grade("good")),
    );
  }
  const links = div("review-links");
  links.append(
    button("Je connais", "review-link review-known", () => actions.markKnown()),
    document.createTextNode(" · "),
    button("Ne plus me le montrer", "review-link review-hide", () => actions.ignore()),
  );
  const bar = div("review-bar");
  bar.append(answers, links);

  const wrap = div("review-reviewing");
  wrap.append(main, bar);
  return wrap;
}

/** The end of a session: what it did, and « Encore 10 » while another session would hold cards. */
function ending(view: ReviewView, actions: ReviewActions): HTMLElement {
  const box = div("review-done");
  const s: ReviewSummary | null = view.summary;
  if (!s || s.reviewed + s.known + s.hidden === 0) {
    box.append(note("Rien à réviser pour l'instant."));
  } else {
    box.append(line("done-title", "Séance terminée"));
    const list = document.createElement("ul");
    list.className = "review-done-stats";
    const item = (n: number, one: string, many: string): void => {
      if (n === 0) return;
      const li = document.createElement("li");
      const b = document.createElement("b");
      b.textContent = String(n);
      li.append(b, document.createTextNode(` ${n === 1 ? one : many}`));
      list.append(li);
    };
    item(s.reviewed, "mot revu", "mots revus");
    item(s.recovered, "rattrapé en route", "rattrapés en route");
    item(s.holding, "tient maintenant plus d'un mois", "tiennent maintenant plus d'un mois");
    item(s.known, "classé connu", "classés connus");
    item(s.hidden, "masqué", "masqués");
    box.append(list);
  }
  if (view.moreDue)
    box.append(button(`Encore ${SESSION_CARDS}`, "review-btn primary review-more", () => actions.start()));
  return box;
}

/** The sentence as text, the word in a mark: built from text nodes only — it came from a page. */
function sentenceView(sentence: string, marked: MarkedWord | null): HTMLElement {
  const p = document.createElement("p");
  p.className = "review-sentence";
  if (!marked) {
    p.textContent = sentence;
    return p;
  }
  const mark = document.createElement("mark");
  mark.className = "review-word";
  mark.textContent = marked.text;
  p.append(document.createTextNode(marked.before), mark, document.createTextNode(marked.after));
  return p;
}

export interface MarkedWord {
  before: string;
  text: string;
  after: string;
}

/**
 * The first of `forms` found in `sentence` as a whole word, letter case aside — the form met,
 * then the dictionary form — split around it; null when none is there.
 */
export function markWord(sentence: string, ...forms: string[]): MarkedWord | null {
  for (const form of forms) {
    const f = form.trim();
    if (!f) continue;
    const found = new RegExp(`(?<![\\p{L}\\p{N}])${escapeRegExp(f)}(?![\\p{L}\\p{N}])`, "iu").exec(sentence);
    if (found) {
      return {
        before: sentence.slice(0, found.index),
        text: found[0],
        after: sentence.slice(found.index + found[0].length),
      };
    }
  }
  return null;
}

function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
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

function button(label: string, className: string, onClick: () => void): HTMLButtonElement {
  const b = document.createElement("button");
  b.type = "button";
  b.className = className;
  b.textContent = label;
  b.addEventListener("click", onClick);
  return b;
}

function div(className: string): HTMLDivElement {
  const d = document.createElement("div");
  d.className = className;
  return d;
}

function note(text: string, extra = ""): HTMLElement {
  const d = div(extra ? `review-note ${extra}` : "review-note");
  d.textContent = text;
  return d;
}

function line(kind: string, text: string): HTMLElement {
  const d = div(`review-${kind}`);
  d.textContent = text;
  return d;
}
