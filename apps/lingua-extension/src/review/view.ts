import { languageName } from "../analyzer/language-labels.ts";
import type { Rating, ReviewCard, ReviewSummary } from "../analyzer/port.ts";
import { review as enReview } from "../i18n/en/review.ts";
import { review as esReview } from "../i18n/es/review.ts";
import { review as frReview } from "../i18n/fr/review.ts";
import {
  DEFAULT_INTERFACE_LANGUAGE,
  formatCount,
  formatTime,
  type InterfaceLanguage,
  NODE_SLOT,
  plural,
  type PluralForms,
  renderAround,
} from "../i18n/index.ts";
import { withoutEdgePunctuation } from "../reading/selection.ts";
import { sameSpokenText, type Speaker } from "../reading/speech.ts";
import { type ReviewView, SESSION_CARDS } from "./session.ts";
import { keepable, keepCandidate, KEEP_MAX_UNITS, type ReviewTranslation } from "./translation.ts";

// The review render — one function every review host (the side panel and the injected drawer)
// calls to paint a ReviewView, wiring its controls to the given actions. Pure DOM, no styling of
// its own (the host provides the token sheet and review.css); no user-visible "lemma".
//
// The card reads like the page it came from (refine-lingua-review-session D6, D7): the sentence
// first, the word marked; the answer's space there from the start, so revealing it changes that
// space only; one action zone the same size before and after — « Afficher la réponse », then
// « Pas su » on the left and « Su » on the right, like the reader's page turns — and two links.
// Its words are the catalogue's `review` module in the interface language
// (localise-lingua-review-stats D1); the card's own words — the headword, the sentence, the
// dictionary form — say the studied language they are in.
// Keys are read on the review's own root, never on the page: Space or Enter reveals, the left
// and right arrows answer. With a speaker, the card can be heard before and after the reveal, as
// the word card is (refine-lingua-review-session D11).

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
  /** Start a practice session (add-lingua-review-practice); a host without it offers none. */
  practice?(): void;
  reveal(): void;
  grade(rating: Rating): void;
  markKnown(): void;
  ignore(): void;
  /** End the session under way (add-lingua-review-stop-session); a host without it offers no link. */
  end?(): void;
}

/** How a host has the review painted. */
export interface ReviewRenderOptions {
  /** The reader studies several languages, so each card says its own. */
  showLanguage?: boolean;
  /** The interface language, whose module the review speaks and whose plural forms its counts
   *  take — the copy derived from it, so the two cannot disagree; French when not given. */
  interfaceLanguage?: InterfaceLanguage;
  /**
   * Reads the card aloud, with the voices and rules of the word card (add-lingua-read-aloud).
   * Without one, or without a voice it may use, the card offers nothing to hear.
   */
  speaker?: Speaker;
  /**
   * The current card's machine translation, when its pack has no gloss and a model was asked
   * (add-lingua-review-translation D5): shown once revealed, never before. Without one, the answer
   * is today's.
   */
  translation?: ReviewTranslation;
  /**
   * « Traduction en ligne » on the revealed card (add-lingua-review-translation-fallback D2): its
   * button and where its request stands. Absent: no button.
   */
  online?: ReviewOnlineOffer;
  /**
   * On a revealed card with a gloss, the « Traduire la phrase » link asks for its translation
   * (add-lingua-review-translation-fallback D1). Absent: no link.
   */
  askTranslation?: () => void;
  /**
   * Keep a machine translation's word as the card's gloss (add-lingua-review-translation-fallback D3):
   * offered under a translation shown. Absent: no action.
   */
  keep?: (gloss: string) => void;
  /**
   * On a revealed card without a gloss that nothing can translate — no model ready, no online
   * button —, how the reader gets a translation, said in the no-translation note itself
   * (add-lingua-review-translation-fallback D12). Absent: the note alone.
   */
  hint?: ReviewTranslationHint;
}

/**
 * How the reader gets a translation for a card nothing can translate (D12), per host: turn on
 * « Traduction étendue » in Réglages (the extension), turn it on in the Safari extension (the app,
 * the setting off there), or download the app's copy of the models (the app, the setting on and its
 * models absent) — `download` re-offering the app's own offer.
 */
export type ReviewTranslationHint =
  { kind: "settings" } | { kind: "extension" } | { kind: "download"; download: () => void };

/**
 * The review's « Traduction en ligne » button, as the selection card's
 * (add-lingua-online-translation D8): offered, asking, answered, unavailable, or the daily limit
 * reached with when it resets (epoch milliseconds).
 */
export interface ReviewOnlineOffer {
  state: "offered" | "asking" | "answered" | "unavailable" | "limit";
  resetsAt?: number;
  press: () => void;
}

/** One listen button: what it speaks, under which key, and how it is labelled. */
interface Listen {
  key: "word" | "headword" | "sentence";
  text: string;
  label: string;
  aria: string;
}

/** A listen row, and how to repaint it when the speaker's state changes. */
interface ListenRow {
  el: HTMLElement;
  paint(): void;
}

/** What each root's last render subscribed to its speaker, undone by the next render. */
const unsubscribes = new WeakMap<HTMLElement, () => void>();

/**
 * Render `view` into `root`, replacing its contents. A root that held the focus keeps it. A text
 * being read that the new view no longer shows stops: another card silences the last one.
 */
export function renderReview(
  root: HTMLElement,
  view: ReviewView,
  actions: ReviewActions,
  opts: ReviewRenderOptions = {},
): void {
  const language = opts.interfaceLanguage ?? DEFAULT_INTERFACE_LANGUAGE;
  const copy = REVIEW_COPY[language];
  const scope = root.getRootNode() as Document | ShadowRoot;
  const focused = scope.activeElement;
  const hadFocus = focused === root || (focused !== null && root.contains(focused));
  unsubscribes.get(root)?.();
  unsubscribes.delete(root);
  root.replaceChildren();
  root.tabIndex = -1;
  // A property, not a listener: each render replaces the last one instead of adding another.
  root.onkeydown = (e) => onKey(e, root, view, actions);

  const card = view.phase === "reviewing" ? view.card : null;
  const playing = opts.speaker?.speaking() ?? null;
  if (playing && !(card ? listensOf(card, copy) : []).some((l) => l.key === playing.key && l.text === playing.text)) {
    opts.speaker?.stop();
  }

  const rows: ListenRow[] = [];
  if (view.phase === "idle") root.append(button(copy.start, "review-btn primary review-start", () => actions.start()));
  else if (view.phase === "done" || !card) root.append(ending(view, actions, language, copy));
  else root.append(cardView(card, view.showing ?? 1, view.practice ?? false, actions, opts, language, copy, rows));

  const speaker = opts.speaker;
  if (speaker && rows.length > 0) {
    unsubscribes.set(
      root,
      speaker.subscribe(() => {
        for (const row of rows) row.paint();
      }),
    );
  }

  if (hadFocus) root.focus({ preventScroll: true });
}

/** Space or Enter reveals; once revealed, ← answers a miss and → a recall. */
function onKey(e: KeyboardEvent, root: HTMLElement, view: ReviewView, actions: ReviewActions): void {
  const card = view.phase === "reviewing" ? view.card : null;
  if (!card || e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
  // The keep field's own keys: typing never answers the card.
  if ((e.target as HTMLElement | null)?.tagName === "INPUT") return;
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

function cardView(
  card: ReviewCard,
  showing: number,
  practice: boolean,
  actions: ReviewActions,
  opts: ReviewRenderOptions,
  language: InterfaceLanguage,
  copy: ReviewCopy,
  rows: ListenRow[],
): HTMLElement {
  // The card's words are the studied language's, inside a surface that says the interface's: they
  // say their own — an older card without one is English (localise-lingua-reading-surfaces D3).
  const studied = card.language ?? "en";
  const main = div("review-main");
  const head = div("review-head");
  // A card a miss brought back says so, where the count of cards remaining stands
  // (add-lingua-review-repeat-hint D2): the line keeps its place, nothing moves on an e-ink screen.
  // That says why it is there again, so it wins over practice's own note (add-lingua-review-practice).
  head.append(
    showing > 1
      ? note(copy.repeat(showing), "remaining review-repeat")
      : practice
        ? note(plural(language, card.remaining, copy.practiceRemaining), "remaining review-practice-note")
        : note(plural(language, card.remaining, copy.remaining), "remaining"),
  );
  // The card's language, when the reader studies several (add-lingua-language-stats-review D2).
  if (opts.showLanguage && card.language) head.append(note(languageName(language, card.language), "review-language"));
  // Ending the session, a link at the end of the same line (add-lingua-review-stop-session D1): no
  // block of its own, so nothing moves on an e-ink screen.
  if (actions.end) {
    const end = actions.end;
    const link = button(copy.end, "review-link review-end", () => end());
    link.setAttribute("aria-label", copy.endLabel);
    head.append(link);
  }
  main.append(head);

  const source = sourceLabel(card.source);
  if (source) main.append(line("source", source));
  const sentence = card.sentence.trim();
  const marked = markedIn(card);
  // No sentence (a word seeded from a level), or none holding the word: the word itself.
  const headword = cardHeadword(card);
  if (!marked) main.append(studiedLine("headword", headword, studied));
  if (sentence) main.append(sentenceView(sentence, marked, studied));
  // What there is to hear stays the same once revealed, so the reveal moves nothing here (D11).
  const listen = (listens: Listen[]): HTMLElement[] => {
    if (!opts.speaker) return [];
    const row = listenRow(opts.speaker, listens, copy);
    rows.push(row);
    return [row.el];
  };
  main.append(...listen(frontListens(card, copy)));

  // The answer's space, there from the start: revealing fills it and moves nothing else.
  const answer = div("review-answer");
  answer.setAttribute("aria-live", "polite");
  if (card.revealed) {
    // The dictionary form, when the sentence shows another one — both read without their edge
    // punctuation, so `four-poster bed,` is not offered as the form of `four-poster bed`.
    const otherForm = marked !== null && marked.text.toLowerCase() !== headword.toLowerCase();
    if (otherForm) answer.append(studiedLine("dictionary-form", headword, studied));
    if (card.gloss) {
      answer.append(line("gloss", card.gloss));
      // Under the gloss: the translation the reader asked for, else the link asking it (D1).
      const asked = opts.translation ? askedTranslation(opts.translation, copy) : null;
      if (asked) answer.append(asked);
      else if (opts.askTranslation) {
        const ask = opts.askTranslation;
        answer.append(button(copy.translateSentence, "review-link review-ask-translation", () => ask()));
      }
    } else {
      answer.append(missingGloss(card, opts.translation, opts.hint, copy));
    }
    if (opts.online) answer.append(onlineRow(opts.online, language, copy));
    if (opts.keep && opts.translation?.state === "translated") {
      answer.append(keepRow(opts.translation.translation, opts.translation.alone, opts.keep, copy));
    }
    if (otherForm) answer.append(...listen([headwordListen(headword, copy)]));
  }
  main.append(answer);

  const answers = div("review-actions");
  if (!card.revealed) {
    answers.append(button(copy.reveal, "review-btn primary review-reveal", () => actions.reveal()));
  } else {
    answers.append(
      button(copy.missed, "review-btn review-miss", () => actions.grade("again")),
      button(copy.recalled, "review-btn primary review-hit", () => actions.grade("good")),
    );
  }
  const links = div("review-links");
  links.append(
    button(copy.markKnown, "review-link review-known", () => actions.markKnown()),
    button(copy.hide, "review-link review-hide", () => actions.ignore()),
  );
  const bar = div("review-bar");
  bar.append(answers, links);

  const wrap = div("review-reviewing");
  wrap.append(main, bar);
  return wrap;
}

/** The end of a session: what it did, and « Encore 10 » while another session would hold cards. */
function ending(view: ReviewView, actions: ReviewActions, language: InterfaceLanguage, copy: ReviewCopy): HTMLElement {
  const box = div("review-done");
  const s: ReviewSummary | null = view.summary;
  if (!s || s.reviewed + s.known + s.hidden === 0) {
    // Ended before any answer: it ended, nothing to count — never « nothing due » above « Encore 10 »
    // (add-lingua-review-stop-session D3).
    box.append(view.ended ? line("done-title", copy.sessionDone) : note(copy.nothingDue));
  } else {
    box.append(line("done-title", copy.sessionDone));
    const list = document.createElement("ul");
    list.className = "review-done-stats";
    // Each count's figure in bold, where its message puts it.
    const item = (n: number, forms: PluralForms): void => {
      if (n === 0) return;
      const li = document.createElement("li");
      const b = document.createElement("b");
      b.textContent = formatCount(language, n);
      renderAround(li, plural(language, n, forms, NODE_SLOT), b);
      list.append(li);
    };
    item(s.reviewed, copy.reviewed);
    item(s.recovered, copy.recovered);
    item(s.holding, copy.holding);
    item(s.known, copy.known);
    item(s.hidden, copy.hidden);
    box.append(list);
  }
  if (view.moreDue) {
    const more = copy.more(formatCount(language, SESSION_CARDS));
    box.append(button(more, "review-btn primary review-more", () => actions.start()));
  } else if (view.practiceAvailable && actions.practice) {
    // Nothing due, cards already reviewed: practise anyway, no schedule moved (add-lingua-review-practice).
    const practise = actions.practice;
    box.append(button(copy.practice, "review-btn primary review-practice", () => practise()));
  }
  return box;
}

/**
 * A card's word read without its edge punctuation (fix-lingua-selection-card-punctuation D4): a card
 * captured as `four-poster bed,` is shown, heard and compared as `four-poster bed`, and never
 * rewritten. A word of punctuation alone is shown as it is stored.
 */
const readWord = (word: string): string => withoutEdgePunctuation(word) || word.trim();

/** The card's headword as the front reads it: without its edge punctuation. */
export function cardHeadword(card: ReviewCard): string {
  return readWord(card.headword);
}

/**
 * The card's word as its sentence shows it — the form met, then the dictionary form, each without
 * its edge punctuation, so the mark leaves a captured comma outside — or null without a sentence
 * holding it. The review's translation marks the same place.
 */
export function markedIn(card: ReviewCard): MarkedWord | null {
  const sentence = card.sentence.trim();
  return sentence ? markWord(sentence, readWord(card.surface), cardHeadword(card)) : null;
}

/**
 * In the place of a missing gloss: the machine translation that came, « Traduction en cours… » while
 * it is on its way, else the no-translation note (add-lingua-review-translation D2, D5) — saying, when
 * nothing can translate the card, how to get a translation (add-lingua-review-translation-fallback D12).
 */
function missingGloss(
  card: ReviewCard,
  translation: ReviewTranslation | undefined,
  hint: ReviewTranslationHint | undefined,
  copy: ReviewCopy,
): HTMLElement {
  if (translation?.state === "pending") return note(copy.translating, "review-translating");
  if (translation?.state === "translated") return translationBlock(translation, copy);
  const missing = note(noGloss(card, copy), "review-no-gloss");
  if (hint?.kind === "download") {
    // The same line, its action a small text control: nothing moves but the line itself.
    const { download } = hint;
    missing.append(
      " ",
      button(copy.hintDownload, "review-link review-hint-download", () => download()),
    );
  } else if (hint) {
    missing.append(" ", hint.kind === "settings" ? copy.hintSettings : copy.hintExtension);
  }
  return missing;
}

/** Under a gloss, what the reader asked: the translation, or that it is on its way; nothing when none came. */
function askedTranslation(translation: ReviewTranslation, copy: ReviewCopy): HTMLElement | null {
  if (translation.state === "pending") return note(copy.translating, "review-translating");
  return translation.state === "translated" ? translationBlock(translation, copy) : null;
}

/** A machine translation under its label — the engine's or the online service's — its marks in `<mark>`s. */
function translationBlock(
  translation: Extract<ReviewTranslation, { state: "translated" }>,
  copy: ReviewCopy,
): HTMLElement {
  const { translation: t, alone, online } = translation;
  const block = div("review-translation");
  const label = line(
    "translation-label",
    online
      ? alone
        ? copy.onlineTranslationAloneLabel
        : copy.onlineTranslationLabel
      : alone
        ? copy.translationAloneLabel
        : copy.translationLabel,
  );
  // Text nodes only: the sentence came from a page and through the engine.
  const sentence = div("review-translation-sentence");
  let at = 0;
  for (const { start, end } of [...t.marks].sort((a, b) => a.start - b.start)) {
    if (start < at || end > t.sentence.length) continue; // overlapping or out of range: skipped, never thrown
    const mark = document.createElement("mark");
    mark.textContent = t.sentence.slice(start, end);
    sentence.append(document.createTextNode(t.sentence.slice(at, start)), mark);
    at = end;
  }
  sentence.append(document.createTextNode(t.sentence.slice(at)));
  block.append(label, sentence);
  return block;
}

/**
 * The « Traduction en ligne » button and its note, as the selection card shows them: pressable only
 * while offered — one press per card — then what became of the request. Text nodes only.
 */
function onlineRow(offer: ReviewOnlineOffer, language: InterfaceLanguage, copy: ReviewCopy): HTMLElement {
  const row = div("review-online");
  const b = button(copy.onlineTranslate, "review-link review-online-btn", () => {
    if (b.disabled) return;
    b.disabled = true;
    offer.press();
  });
  b.disabled = offer.state !== "offered";
  const status = div("review-online-note");
  status.setAttribute("role", "status");
  status.textContent =
    offer.state === "asking"
      ? copy.onlineTranslating
      : offer.state === "unavailable"
        ? copy.onlineUnavailable
        : offer.state === "limit"
          ? copy.onlineLimit(formatTime(language, new Date(offer.resetsAt ?? Date.now())))
          : "";
  status.hidden = !status.textContent;
  row.append(b, status);
  return row;
}

/**
 * Keeping the translation's word as the card's gloss (add-lingua-review-translation-fallback D3). A
 * candidate of one mark that may be kept is named — « Garder « se relâcher » » — and kept in one tap,
 * « Modifier » beside it; otherwise — no mark, several marks, too long — « Garder une traduction… »
 * opens the field, prefilled with the candidate or empty. The field replaces the row in place, on the
 * reader's tap only.
 */
function keepRow(
  translation: { sentence: string; marks: { start: number; end: number }[] },
  alone: boolean,
  keep: (gloss: string) => void,
  copy: ReviewCopy,
): HTMLElement {
  const row = div("review-keep");
  const candidate = keepCandidate(translation, alone);
  const openField = (): void => row.replaceChildren(keepField(candidate?.text ?? "", keep, copy, () => fill()));
  const fill = (): void => {
    if (candidate && !candidate.split && keepable(candidate.text)) {
      const { text } = candidate;
      row.replaceChildren(
        button(copy.keep(text), "review-link review-keep-btn", () => keep(text)),
        button(copy.keepEdit, "review-link review-keep-edit", openField),
      );
    } else {
      row.replaceChildren(button(copy.keepOther, "review-link review-keep-other", openField));
    }
  };
  fill();
  return row;
}

/** The one-line field: « Enregistrer » disabled while the trimmed text is empty or too long. */
function keepField(prefill: string, keep: (gloss: string) => void, copy: ReviewCopy, cancel: () => void): HTMLElement {
  const form = document.createElement("form");
  form.className = "review-keep-field";
  const input = document.createElement("input");
  input.type = "text";
  input.className = "review-keep-input";
  input.value = prefill;
  input.setAttribute("aria-label", copy.keepFieldLabel);
  // Room to correct a candidate over the bound, never past it unnoticed: « Enregistrer » says so.
  input.maxLength = KEEP_MAX_UNITS * 2;
  const save = document.createElement("button");
  save.type = "submit";
  save.className = "review-link review-keep-save";
  save.textContent = copy.keepSave;
  const update = (): void => {
    save.disabled = !keepable(input.value);
  };
  input.addEventListener("input", update);
  update();
  form.addEventListener("submit", (e) => {
    e.preventDefault();
    if (keepable(input.value)) keep(input.value.trim());
  });
  form.append(input, save, button(copy.keepCancel, "review-link review-keep-cancel", cancel));
  queueMicrotask(() => input.focus({ preventScroll: true }));
  return form;
}

/** An expression is not a word: its missing translation says so. */
function noGloss(card: ReviewCard, copy: ReviewCopy): string {
  return /\s/u.test(cardHeadword(card)) ? copy.noGlossExpression : copy.noGloss;
}

/**
 * What the front offers to hear, the same before and after the reveal: the word as the front
 * shows it — marked in its sentence, else above it — then the sentence, unless it is the word.
 */
function frontListens(card: ReviewCard, copy: ReviewCopy): Listen[] {
  const word = (markedIn(card)?.text ?? cardHeadword(card)).trim();
  if (!word) return [];
  const listens: Listen[] = [
    /\s/u.test(word)
      ? { key: "word", text: word, label: copy.listenExpression, aria: copy.listenExpressionLabel }
      : { key: "word", text: word, label: copy.listenWord, aria: copy.listenWordLabel },
  ];
  const sentence = card.sentence.trim();
  if (sentence && !sameSpokenText(sentence, word)) {
    listens.push({ key: "sentence", text: sentence, label: copy.listenSentence, aria: copy.listenSentenceLabel });
  }
  return listens;
}

/** Once revealed, the dictionary form beside the form met (add-lingua-dictionary-form-voice D1). */
function headwordListen(headword: string, copy: ReviewCopy): Listen {
  return {
    key: "headword",
    text: headword,
    label: copy.listenForm(headword),
    aria: copy.listenDictionaryFormLabel(headword),
  };
}

/** Every text the card may be reading: one that is not is stopped when the card changes. */
function listensOf(card: ReviewCard, copy: ReviewCopy): Listen[] {
  return [...frontListens(card, copy), headwordListen(cardHeadword(card), copy)];
}

/**
 * A listen row, painted from the speaker's state — no voice it may use, no button; the text being
 * read offers to stop — and repainted on its every change (a voice list announced late, an
 * utterance ending).
 */
function listenRow(speaker: Speaker, listens: Listen[], copy: ReviewCopy): ListenRow {
  const el = div("review-listen");
  const paint = (): void => {
    el.replaceChildren();
    const offered = speaker.available() ? listens : [];
    el.hidden = offered.length === 0;
    const playing = speaker.speaking();
    for (const listen of offered) {
      const on = playing?.key === listen.key && playing.text === listen.text;
      const b = button(on ? copy.stop : listen.label, on ? "review-listen-btn speaking" : "review-listen-btn", () => {
        if (on) speaker.stop();
        else speaker.speak(listen.key, listen.text);
      });
      b.setAttribute("aria-label", on ? copy.stopLabel : listen.aria);
      el.append(b);
    }
  };
  paint();
  return { el, paint };
}

/** The sentence as text, the word in a mark: built from text nodes only — it came from a page. */
function sentenceView(sentence: string, marked: MarkedWord | null, studied: string): HTMLElement {
  const p = document.createElement("p");
  p.className = "review-sentence";
  p.lang = studied;
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

/** A line of the card's own words, saying the studied language they are in. */
function studiedLine(kind: string, text: string, studied: string): HTMLElement {
  const d = line(kind, text);
  d.lang = studied;
  return d;
}

function line(kind: string, text: string): HTMLElement {
  const d = div(`review-${kind}`);
  d.textContent = text;
  return d;
}
