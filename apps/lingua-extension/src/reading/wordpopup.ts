import type { MarkedTranslation, Span } from "../translate/markup.ts";
import type { LemmaStatus, StudiedLanguage, WordGrammar } from "../analyzer/types.ts";
import type { card as frCard } from "../i18n/fr/card.ts";
import {
  DEFAULT_INTERFACE_LANGUAGE,
  formatCount,
  type InterfaceLanguage,
  NODE_SLOT,
  renderAround,
} from "../i18n/index.ts";
import { glossPages, pageText, type GlossPage } from "./gloss-pages.ts";
import { readingCopy } from "./reading-copy.ts";
import { isTouchPrimary } from "../state/platform.ts";
import { sameSpokenText, type Speaker, type Speaking } from "./speech.ts";
import { followSurfaceLook } from "./surface-look.ts";

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
  /**
   * The word's grammar, when the engine answered it (add-lingua-word-grammar): what the form is,
   * what else it may be, the pieces of a split word, and the gloss laid out by part of speech.
   * Display only — a created card stores the flat gloss, exactly as before.
   */
  grammar?: WordGrammar | null;
  /** The word as it stands on the page (`don't` for its `do`); defaults to `surface`. */
  written?: string;
  /**
   * Where the selection sits in `sentence`, as [start, end) offsets — written only where known
   * (add-lingua-french-read-aloud D5): a French card reads an elided piece with the word the page
   * writes it against, which it finds after this span.
   */
  selection?: Span | null;
  /**
   * The studied language of the document the word was met in, whose forms the grammar lines name
   * (add-lingua-spanish-word-card) and which the card's words of the document say in `lang`
   * (localise-lingua-reading-surfaces); a card without one names English forms, as every card did.
   */
  language?: StudiedLanguage;
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

/** The card's copy: the catalogue's `card` module, in the interface language (its French the default). */
export type CardCopy = typeof frCard;

/** One listen button: what it speaks, under which key, and how it is labelled. */
interface Listen {
  key: "selection" | "headword" | "sentence";
  text: string;
  label: string;
  aria: string;
}

/** The keys the card speaks under: what closing it or opening another word silences. */
const CARD_KEYS: ReadonlySet<string> = new Set<Listen["key"]>(["selection", "headword", "sentence"]);

/**
 * The language of the words the card shows from the document — the headword, the form seen, the
 * word-by-word forms: the document's, which a card without one names English (`language`'s rule).
 * The host says the interface language (D3), so these say their own, for the voices, the
 * hyphenation and the spell-check that read them.
 */
function studiedLanguageOf(content: WordPopupContent): StudiedLanguage {
  return content.language ?? "en";
}

/** A word of the document, in its language, inside a line of the interface's. */
function studiedWord(text: string, language: string): HTMLElement {
  const word = document.createElement("span");
  word.lang = language;
  word.textContent = text;
  return word;
}

/**
 * Whether the card shows a form seen apart from its dictionary form, case aside: it then says
 * « forme vue » and offers to hear both forms (add-lingua-dictionary-form-voice D1).
 */
function seenDiffers(content: WordPopupContent): boolean {
  return !!content.surface && content.surface.toLowerCase() !== content.headword.toLowerCase();
}

/** A letter of the page, an accent written apart included. */
const LETTER = /[\p{L}\p{M}]/u;
/** An apostrophe, straight or typographic. */
const APOSTROPHE = /['\u2019]/u;
/** A form written with its apostrophe last: an elided piece (`l’`, `qu'`). */
const ELIDED = /['\u2019]$/u;

/** A text as the engine reads it: letter case, Unicode normalisation and the apostrophe's form aside. */
function engineForm(text: string): string {
  return text
    .normalize("NFC")
    .replace(/\u2019/gu, "'")
    .toLowerCase();
}

/**
 * The word an elided piece leans on, as the page writes it with the piece: from the selection's
 * start through the letters that follow it in its sentence, carrying on across an apostrophe
 * followed by a letter (« jusqu'aujourd'hui »), up to a space, a hyphen, a digit or punctuation
 * (« Qu'est » of « Qu'est-ce »). Null unless the selection is glued to what follows — a letter, or
 * an apostrophe and a letter — which holds whether the piece's span takes its apostrophe (`l'` |
 * `homme`) or leaves it to the next one (`l` | `'homme`).
 */
function gluedWord(content: WordPopupContent): string | null {
  const span = content.selection;
  const text = content.sentence;
  if (!span || span.start < 0 || span.end <= span.start || span.end > text.length) return null;
  const letterAt = (i: number): boolean => i < text.length && LETTER.test(text.charAt(i));
  const gluedAt = (i: number): boolean => letterAt(i) || (APOSTROPHE.test(text.charAt(i)) && letterAt(i + 1));
  if (!gluedAt(span.end)) return null;
  let end = span.end;
  while (gluedAt(end)) end += letterAt(end) ? 1 : 2;
  return text.slice(span.start, end);
}

/** What a single word's button reads, and whether it is an elided piece read with its word. */
interface Heard {
  text: string;
  leaning: boolean;
}

/**
 * What a single word's button reads (add-lingua-french-read-aloud D5). Read alone by a voice, a
 * French elided piece is a letter's name (`l'` is « elle »), so when the speaker reads French the
 * button reads what the page writes: an elided piece with the word it leans on (`l'` → « l'homme »);
 * else a piece of a word the analysis split, the word as written (`à` → « au ») — but never an
 * elided piece alone (`l’ homme` typed with a space), whose form seen is the word it stands for when
 * the analysis writes it so (`Le`); else the form seen, as every card reads in English and Spanish
 * (`del` still reads its piece).
 */
function heardWord(content: WordPopupContent, seen: string, speakerLanguage: string): Heard {
  if (speakerLanguage !== "fr") return { text: seen, leaning: false };
  const glued = gluedWord(content);
  if (glued) return { text: glued, leaning: true };
  const written = content.written?.trim();
  const asWritten = !!written && !ELIDED.test(written) && engineForm(written) !== engineForm(seen);
  return { text: asWritten ? written : seen, leaning: false };
}

/**
 * The texts a card offers to hear: the selection as seen — with the dictionary form beside it when
 * the card shows another form seen, each button saying what it reads (D2) — then its sentence unless
 * it is what the word button reads. A French speaker hears a single word as `heardWord` says.
 */
function listensFor(content: WordPopupContent, copy: CardCopy, speakerLanguage: string): Listen[] {
  const selection = (content.surface || content.headword).trim();
  if (!selection) return [];
  const headword = content.headword.trim();
  const listens: Listen[] = [];
  // Several words are a selection, whether or not the pack knows the expression (« animal doméstico »).
  const several = content.expression || /\s/u.test(selection);
  const word = several ? { text: selection, leaning: false } : heardWord(content, selection, speakerLanguage);
  const heard = word.text;
  // An elided piece read with its word shows a form other than its dictionary one, whatever text the
  // analysis gives its token — `l'`, or the word it stands for, `Le` (add-lingua-french-tokenisation).
  const leans = word.leaning && heard.toLowerCase() !== headword.toLowerCase();
  if (several) {
    listens.push({ key: "selection", text: selection, label: copy.listenSelection, aria: copy.listenSelectionLabel });
  } else if ((seenDiffers(content) || leans) && headword) {
    listens.push(
      {
        key: "selection",
        text: heard,
        label: copy.listenForm(heard),
        aria: copy.listenSeenFormLabel(heard),
      },
      {
        key: "headword",
        text: headword,
        label: copy.listenForm(headword),
        aria: copy.listenDictionaryFormLabel(headword),
      },
    );
  } else {
    listens.push({ key: "selection", text: heard, label: copy.listenWord, aria: copy.listenWordLabel });
  }
  const sentence = content.sentence.trim();
  if (sentence && !sameSpokenText(sentence, heard)) {
    listens.push({ key: "sentence", text: sentence, label: copy.listenSentence, aria: copy.listenSentenceLabel });
  }
  return listens;
}

/** Build the card view (no shadow root involved — testable in isolation). With a `speaker`, the
 *  card offers to hear the selection and its sentence (add-lingua-read-aloud). Its labels, its
 *  figures and its grammar lines are written in `language` — the catalogue's `card` module and
 *  grammar renderer of that one language (localise-lingua-reading-surfaces D1,
 *  generalise-lingua-card-wording D2), so they cannot disagree; without one, or with a language the
 *  catalogue lacks, the French. */
export function createCard(speaker?: Speaker, language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE): CardView {
  const { card: copy, grammar } = readingCopy(language);
  const el = div("card");
  el.hidden = true;
  el.addEventListener("click", (e) => e.stopPropagation());

  const headwordEl = div("headword");
  const seenEl = div("seen");
  const rarityEl = div("rarity");
  const listenEl = div("listen");
  listenEl.hidden = true;
  // Below the listen row and above the actions: a pending card offers the listen buttons, and
  // what its answer adds here moves nothing the reader can press (add-lingua-word-grammar D6).
  const grammarEl = div("grammar");
  grammarEl.hidden = true;
  const glossEl = div("gloss");
  const translationEl = div("translation");
  const actionsEl = div("actions");
  el.append(headwordEl, seenEl, rarityEl, listenEl, grammarEl, glossEl, translationEl, actionsEl);

  let current: WordPopupContent | null = null;
  let generation = 0;
  // A paged gloss: its pages, the one on screen, and what a card created from it stores — the
  // first page, one text, as the pack writes a gloss (null: the gloss as the content holds it).
  let pages: GlossPage[] = [];
  let pageIndex = 0;
  let storedGloss: string | null = null;
  let paged: { pageEl: HTMLElement; nav: PageNav } | null = null;

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
        gloss: storedGloss ?? current.gloss,
      });
      view.hide();
    });
    return b;
  }

  /**
   * The listen row, from the speaker's state: rebuilt on every show and on every change of the
   * speaker — a voice list announced late, an utterance ending — so the card on screen always
   * says what pressing does. It does not wait for the engine: a pending card offers it too.
   */
  function renderListen(): void {
    listenEl.replaceChildren();
    const listens = speaker && current && speaker.available() ? listensFor(current, copy, speaker.lang) : [];
    listenEl.hidden = listens.length === 0;
    if (!speaker) return;
    const playing = speaker.speaking();
    for (const listen of listens) {
      const on = playing?.key === listen.key && playing.text === listen.text;
      const b = document.createElement("button");
      b.type = "button";
      b.textContent = on ? copy.stop : listen.label;
      b.setAttribute("aria-label", on ? copy.stopLabel : listen.aria);
      b.classList.toggle("speaking", on);
      // Keep the reader's selection (and on a touch device the platform's callout) as it was:
      // a button's default on press is to take the focus and collapse the page selection.
      b.addEventListener("pointerdown", (e) => e.preventDefault());
      b.addEventListener("mousedown", (e) => e.preventDefault());
      b.addEventListener("click", () => {
        if (on) speaker.stop();
        else speaker.speak(listen.key, listen.text);
      });
      listenEl.append(b);
    }
  }

  /** Whether the speaker is reading one of this card's texts (a settings preview is not). */
  function cardSpeaking(): Speaking | null {
    const playing = speaker?.speaking() ?? null;
    return playing && CARD_KEYS.has(playing.key) ? playing : null;
  }

  speaker?.subscribe(() => {
    if (!el.hidden) renderListen();
  });

  /**
   * The answer slot: the waiting line, the labelled rows, the gloss, or the no-gloss note. A
   * translation is a better answer than word-by-word rows, so with one the rows never show —
   * and neither does the note that the pack has no translation, which would sit, untrue,
   * right above one.
   */
  function renderAnswer(content: WordPopupContent): void {
    glossEl.replaceChildren();
    glossEl.classList.remove("empty", "waiting");
    pages = [];
    pageIndex = 0;
    storedGloss = null;
    paged = null;
    glossEl.hidden = false;
    if (content.pending) {
      glossEl.textContent = content.translating ? copy.translating : copy.waiting;
      glossEl.classList.add("waiting");
      return;
    }
    renderPackAnswer(content);
    // The pack has answered and the engine has not. Say so under its answer: a reader looking at
    // word-by-word rows must know a translation is still coming, so the rows are never mistaken
    // for the last word on their selection.
    if (content.translating && !content.translation) {
      const note = div("translating-note");
      note.textContent = copy.translating;
      glossEl.hidden = false;
      glossEl.append(note);
    }
  }

  /**
   * What the form is, what else it may be, and the pieces of a split word — one line each, the
   * words of the studied language set apart. Built from text nodes only.
   */
  function renderGrammar(content: WordPopupContent): void {
    grammarEl.replaceChildren();
    const studied = studiedLanguageOf(content);
    const lines =
      content.grammar && !content.pending && !content.expression
        ? grammar.grammarLines(
            content.grammar,
            content.headword,
            content.surface,
            content.written ?? content.surface,
            studied,
          )
        : [];
    grammarEl.hidden = lines.length === 0;
    for (const line of lines) {
      const lineEl = div("grammar-line");
      for (const segment of line) {
        if (typeof segment === "string") {
          lineEl.append(document.createTextNode(segment));
        } else {
          // A word of the document says its language, inside the interface's line (D5).
          const word = document.createElement("em");
          word.lang = studied;
          word.textContent = segment.word;
          lineEl.append(word);
        }
      }
      grammarEl.append(lineEl);
    }
  }

  /**
   * The gloss, laid out by part of speech when the grammar answered it: one line per part of
   * speech, its name first. The groups are the same senses, so they are only used when they
   * make up exactly the gloss the card holds. A gloss longer than a page shows one page at a time,
   * with a control to move between them.
   */
  function renderGloss(content: WordPopupContent, gloss: string): void {
    pages = glossPages(gloss, content.grammar?.senses ?? []);
    pageIndex = 0;
    storedGloss = pages.length > 1 ? pageText(pages[0]!) : null;
    const pageEl = div("gloss-page");
    glossEl.append(pageEl);
    const nav = pages.length > 1 ? pageNav(pageEl) : null;
    if (nav) paged = { pageEl, nav };
    renderPage(pageEl, nav);
  }

  /**
   * Give a paged card the size of its largest page, so that moving between pages moves nothing:
   * a shorter page would shrink the card under the reader's pointer, shift the paging buttons and,
   * on a card flipped above its word, the whole card. Measured on the card as shown, every page
   * in turn: the widest first, then the tallest at that width.
   */
  function holdPageSize(): void {
    if (!paged) return;
    const { pageEl, nav } = paged;
    let width = 0;
    for (pageIndex = 0; pageIndex < pages.length; pageIndex++) {
      renderPage(pageEl, nav);
      width = Math.max(width, el.getBoundingClientRect().width);
    }
    if (width > 0) el.style.minWidth = `${Math.ceil(width)}px`;
    let height = 0;
    for (pageIndex = 0; pageIndex < pages.length; pageIndex++) {
      renderPage(pageEl, nav);
      height = Math.max(height, pageEl.getBoundingClientRect().height);
    }
    if (height > 0) pageEl.style.minHeight = `${Math.ceil(height)}px`;
    pageIndex = 0;
    renderPage(pageEl, nav);
  }

  /** The page on screen, and the paging control's state. */
  function renderPage(pageEl: HTMLElement, nav: PageNav | null): void {
    pageEl.replaceChildren();
    for (const group of pages[pageIndex] ?? []) {
      const text = group.senses.join(copy.senseSeparator);
      if (!group.tagged) {
        pageEl.append(document.createTextNode(text));
        continue;
      }
      const groupEl = div("sense-group");
      const heading = grammar.senseHeading(group.tag);
      if (heading) {
        const pos = document.createElement("span");
        pos.className = "pos";
        pos.textContent = heading;
        groupEl.append(pos, document.createTextNode(" "));
      }
      groupEl.append(document.createTextNode(text));
      pageEl.append(groupEl);
    }
    if (!nav) return;
    nav.previous.disabled = pageIndex === 0;
    nav.next.disabled = pageIndex === pages.length - 1;
    nav.count.textContent = `${formatCount(language, pageIndex + 1)}/${formatCount(language, pages.length)}`;
  }

  /** The control that moves between a gloss's pages: previous, « n/m », next. */
  function pageNav(pageEl: HTMLElement): PageNav {
    const navEl = div("gloss-nav");
    const count = document.createElement("span");
    count.className = "page-count";
    const nav: PageNav = {
      previous: pageButton(copy.previousPageIcon, copy.previousPage, -1),
      next: pageButton(copy.nextPageIcon, copy.nextPage, 1),
      count,
    };
    navEl.append(nav.previous, count, nav.next);
    glossEl.append(navEl);
    return nav;

    function pageButton(label: string, aria: string, step: number): HTMLButtonElement {
      const b = document.createElement("button");
      b.type = "button";
      b.textContent = label;
      b.setAttribute("aria-label", aria);
      // As the listen buttons: keep the reader's selection on the page.
      b.addEventListener("pointerdown", (e) => e.preventDefault());
      b.addEventListener("mousedown", (e) => e.preventDefault());
      b.addEventListener("click", () => {
        const next = pageIndex + step;
        if (next < 0 || next >= pages.length) return;
        pageIndex = next;
        renderPage(pageEl, nav);
        if (current) positionCard(el, current.rect);
      });
      return b;
    }
  }

  /** What the pack alone has to say: the translated sentence's own gloss, rows, or neither. */
  function renderPackAnswer(content: WordPopupContent): void {
    if (content.translation) {
      if (content.gloss) renderGloss(content, content.gloss);
      else glossEl.hidden = true;
      return;
    }
    if (content.rows && content.rows.length > 0) {
      const label = div("rows-label");
      label.textContent = copy.rowsLabel;
      glossEl.append(label);
      for (const r of content.rows) {
        // The form is the document's, the gloss the reader's: the form says its language.
        const row = div("row");
        renderAround(row, copy.row(NODE_SLOT, r.gloss), studiedWord(r.form, studiedLanguageOf(content)));
        glossEl.append(row);
      }
      return;
    }
    if (content.gloss) {
      renderGloss(content, content.gloss);
      return;
    }
    glossEl.textContent = content.expression ? copy.noGlossExpression : copy.noGloss;
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
    label.textContent = copy.translationLabel;
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
      // No speech outlives the control that stops it.
      if (cardSpeaking()) speaker?.stop();
    },
    show(content, onGesture) {
      generation++;
      el.style.minWidth = "";
      // Another word silences the card; the same selection completing with its answer does not.
      const playing = cardSpeaking();
      if (playing && !listensFor(content, copy, speaker?.lang ?? "").some((l) => l.text === playing.text)) {
        speaker?.stop();
      }
      current = content;
      const studied = studiedLanguageOf(content);
      headwordEl.textContent = content.headword;
      headwordEl.lang = studied;

      const differs = seenDiffers(content);
      if (differs) renderAround(seenEl, copy.seenForm(NODE_SLOT), studiedWord(content.surface, studied));
      else seenEl.replaceChildren();
      seenEl.hidden = !differs;

      rarityEl.textContent = content.rarity;
      // No rank yet, no line (add-lingua-card-frequency D4).
      rarityEl.hidden = !content.rarity;

      renderListen();
      renderGrammar(content);
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
        if (!content.expression && st !== "known") actionsEl.append(button(copy.known, "known", false, onGesture));
        if (st !== "learning") actionsEl.append(button(copy.addToDeck, "learning", true, onGesture));
        if (st !== "ignored") actionsEl.append(button(copy.ignore, "ignored", false, onGesture));
        if (st === "ignored") actionsEl.append(button(copy.relearn, null, false, onGesture));
      }
      actionsEl.hidden = actionsEl.childElementCount === 0;

      el.hidden = false; // reveal first so the card can be measured, then position it
      holdPageSize();
      positionCard(el, content.rect);
      return generation;
    },
  };

  // A close affordance (clicking off the popup also dismisses it — see content.ts).
  const closeEl = document.createElement("button");
  closeEl.className = "close";
  closeEl.textContent = copy.closeIcon;
  closeEl.setAttribute("aria-label", copy.close);
  closeEl.addEventListener("click", () => view.hide());
  el.append(closeEl);

  return view;
}

export interface WordPopupOptions {
  /** Combined token sheet + popup styles, injected into the shadow root. */
  css: string;
  /** Called when the user picks a gesture. */
  onGesture: (gesture: Gesture) => void;
  /** Reads the selection and its sentence aloud; without one the card has no listen row. */
  speaker?: Speaker;
  /** Follow the reader's colours and text size (surface-look); off: the card as designed. */
  followLook?: boolean;
  /**
   * The interface language, said by the host's `lang` (localise-lingua-reading-surfaces D3), whose
   * `card` module and grammar renderer the card reads (`reading-copy.ts`); French when not given.
   */
  language?: InterfaceLanguage;
}

export class WordPopup {
  /** The host element in the page (excluded from scanning by its id). */
  readonly host: HTMLElement;
  private readonly view: CardView;
  /** Stops following the reader's look (`destroy`). */
  private readonly unfollowLook: () => void = () => {};

  constructor(private readonly opts: WordPopupOptions) {
    this.view = createCard(opts.speaker, opts.language);
    this.host = document.createElement("div");
    this.host.id = "cymbra-lingua-host";
    this.host.setAttribute("data-cymbra-lingua-skip", "");
    this.host.lang = opts.language ?? DEFAULT_INTERFACE_LANGUAGE;
    const root = this.host.attachShadow({ mode: "closed" });
    const style = document.createElement("style");
    style.textContent = opts.css;
    root.append(style, this.view.el);
    if (opts.followLook) this.unfollowLook = followSurfaceLook(this.host);
  }

  /** Leave the page for good: its reading session is taken down (add-lingua-native-language-choice D3). */
  destroy(): void {
    this.unfollowLook();
    this.host.remove();
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

/** The parts of a paging control a page change updates. */
interface PageNav {
  previous: HTMLButtonElement;
  next: HTMLButtonElement;
  count: HTMLElement;
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
