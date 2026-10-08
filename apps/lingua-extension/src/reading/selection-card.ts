import type {
  AnalyzedToken,
  LemmaStatus,
  PhraseGloss,
  PhraseMatch,
  PhraseToken,
  StudiedLanguage,
  TokenClass,
  WordGrammar,
} from "../analyzer/types.ts";
import { selection as frSelection } from "../i18n/fr/selection.ts";
import { DEFAULT_INTERFACE_LANGUAGE, formatNumber, type InterfaceLanguage } from "../i18n/index.ts";
import type { MarkedTranslation, Span } from "../translate/markup.ts";
import type { TranslationRequest, TranslatorPort } from "../translate/port.ts";
import type { Gesture, GlossRow, WordPopupContent } from "./wordpopup.ts";

/** The selection card's copy: the catalogue's `selection` module, in the interface language (its French the default). */
export type SelectionCopy = typeof frSelection;

// What a selection opens, decided in one place with no DOM and injected ports, so that
// content.ts — excluded from coverage — stays the thin caller. A selection holding
// whitespace opens the expression card; one without opens the word card of its page token
// when there is one, or of the first token the analyser finds in it. Word by word is the
// last resort: the pack glosses of the words the reader does not know, labelled as not being
// a translation, never stored.
//
// A card that needs the engine opens pending, then completes exactly once — with the answer,
// or with a fallback when the engine fails or stays silent — and only if the card view still
// shows the card that asked: its generation is the whole staleness story.
//
// The translation engine, when the reader has it (« Traduction étendue », a model on the device),
// answers a selection of several words — and a single word only where the pack is silent: no gloss,
// and not a proper noun outside the lexicon, whose translation is noise. Even then the word is
// translated in its sentence, marked, never alone (add-lingua-translation-delivery D7). Without
// the engine, every card is exactly what it was before it existed.
//
// Every word card also asks its word's grammar (add-lingua-word-grammar): what the form is, what
// else it may be, and its gloss by part of speech. A card that already holds its gloss — a
// highlighted word — waits for that answer only briefly and is never drawn pending: past the
// bound it is exactly the card it was before, and a late answer lands nowhere.

/** How long a pending card waits for the engine before its fallback completes it. */
export const ANSWER_TIMEOUT_MS = 3000;
/**
 * How long a word card waits for its grammar before completing without it. Short: the card holds
 * its gloss already, and an engine that is awake answers in milliseconds — this only bounds a
 * cold one (Safari suspends the event page), which then costs the grammar, never the card.
 */
export const GRAMMAR_WAIT_MS = 250;
/** The most word-by-word rows a card shows. */
export const MAX_ROWS = 6;

/**
 * How long the card keeps its offer to upgrade open. The card no longer waits for the engine
 * before answering — the pack's answer is shown as soon as it arrives and the translation
 * replaces it when it lands — so this is not a wait the reader sits through; it is when the
 * card stops expecting one. Set above the engine's own start bound (START_TIMEOUT_MS), which
 * is what a cold engine is really bounded by: measured at 4.8 s on a Galaxy Tab S6 Lite, far
 * past anything the reader should have been made to wait for.
 */
export const TRANSLATION_WAIT_MS = 15_000;

/** The reader's current status for a token class, or null for a new/unknown word — drives
 *  which actions the popup offers when a word is reopened. */
export function statusOfClass(cls: TokenClass): LemmaStatus | null {
  switch (cls) {
    case "Known":
      return "known";
    case "Ignored":
      return "ignored";
    case "Learning":
      return "learning";
    default:
      return null;
  }
}

/**
 * How common a word is, in plain language (add-lingua-card-frequency D1, D4): the band of its
 * dictionary form's rank in the pack, whatever the reader's level — a word they are learning says so
 * instead. `null`: the pack does not rank it, so it lies beyond every band. No rank yet (a pending
 * card, or none in time): no line, rather than a guess. The bands are `copy`'s, their counts
 * written as `language` writes a number — French as before, « 20 000 » with its narrow no-break
 * space (localise-lingua-reading-surfaces D4).
 */
export function rarityText(
  cls: TokenClass,
  rank: number | null | undefined,
  copy: SelectionCopy = frSelection,
  language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE,
): string {
  const count = (n: number): string => formatNumber(language, n);
  if (cls === "Learning") return copy.inDeck;
  if (rank === undefined) return "";
  if (rank === null || rank > 20_000) return copy.rare(count(20_000));
  if (rank > 5_000) return copy.uncommon(count(5_000));
  if (rank > 1_000) return copy.fairlyCommon(count(5_000));
  if (rank > 100) return copy.common(count(1_000));
  return copy.veryCommon(count(100));
}

/** The engine calls a card may need. */
export interface SelectionCardPorts {
  phraseGloss(text: string): Promise<PhraseGloss>;
  gloss(lemma: string): Promise<string | undefined>;
  wordGrammar(written: string, lemma: string): Promise<WordGrammar>;
}

/** The card view, as this module sees it: show returns a generation, and every show or hide bumps it. */
export interface CardSurface {
  show(content: WordPopupContent): number;
  generation(): number;
}

/** Timers, injected so a test settles a request by hand. */
export interface Clock {
  setTimeout(fn: () => void, ms: number): unknown;
  clearTimeout(handle: unknown): void;
}

/** A page token under the selection, with what the content script read off the DOM range. */
export interface PageHit {
  token: AnalyzedToken;
  rect: WordPopupContent["rect"];
  sentence: string;
  /** The word as it stands on the page — `don't` for its `do` — when the DOM range gave it. */
  written?: string;
  /** Where the token sits in `sentence`, found by position — what the translator marks. */
  selection?: Span | null;
}

/** A settled selection: its text, its source sentence and its anchor box. */
export interface SelectionInput {
  text: string;
  sentence: string;
  /** Where the selection sits in `sentence`, found by position — what the translator marks. */
  selection?: Span | null;
  rect: WordPopupContent["rect"];
}

/** A click, as much of it as the rule needs. */
export interface ClickInput {
  /** A capture opened a card since the last pointer-down — this click ends that gesture. */
  gestureOpenedCard: boolean;
  /** The class of the painted (or Alt-resolved) word under the click, or null for none. */
  hitClass: TokenClass | null;
  isLink: boolean;
  altKey: boolean;
}

export interface ClickDecision {
  card: "open" | "hide" | "leave";
  /** Stop the click reaching the page. */
  stop: boolean;
  /** Suppress the default: an untreated word's link, or a deliberate Alt-click. */
  cancel: boolean;
}

const DEFAULT_CLOCK: Clock = {
  setTimeout: (fn, ms) => setTimeout(fn, ms),
  clearTimeout: (handle) => clearTimeout(handle as ReturnType<typeof setTimeout>),
};

/**
 * A gloss the reducer left with nothing in it: the Wiktionary entry had no definition (older
 * packs). It matches no gloss of any edition the packs read today, each reducer leaving its
 * edition's placeholders out — measured over every committed gloss by
 * `test/row-gloss-tables.spec.ts` (add-lingua-english-card-wording D4).
 */
const EMPTY_SENSE = /définition manquante/i;

/** The longest sense a row shows: what a sense held before the pack kept them whole. */
export const ROW_SENSE_CHARS = 80;

/**
 * What a row shows of a pack gloss: its FIRST sense, senses being separated by `;`.
 *
 * A full gloss carries up to eight whole senses, which the word card pages. One sense is the
 * price of a dictionary; eight stacked under one another are unreadable, and the reader is
 * scanning the row for the meaning in THIS phrase, not reading the entry. A sense longer than
 * `ROW_SENSE_CHARS` is cut after a whole word, with an ellipsis. Senses whose text says the
 * definition is missing are skipped; a gloss made only of those gives no row. The word card of a
 * single word still shows the whole gloss.
 */
export function rowGloss(gloss: string, copy: SelectionCopy = frSelection): string | null {
  for (const sense of gloss.split(";")) {
    const text = sense.trim();
    if (text && !EMPTY_SENSE.test(text)) return cutAtWord(text, ROW_SENSE_CHARS, copy);
  }
  return null;
}

/**
 * `text` within `max` characters, the ellipsis included, ending on a whole word when it can. The
 * trailing set strips the separators and the opening marks a cut can leave behind; it gains an
 * opening mark of an edition only when a committed row would end on one (none does today, the
 * English Wiktionary's “ and ‘ included: `test/row-gloss-tables.spec.ts` measures every pair's
 * glosses), and a closing mark is never stripped — ’ is also the apostrophe
 * (add-lingua-english-card-wording D4). Every French row is held byte for byte by that spec's
 * snapshot.
 */
function cutAtWord(text: string, max: number, copy: SelectionCopy): string {
  if (text.length <= max) return text;
  const room = max - 1;
  const space = text.lastIndexOf(" ", room);
  const cut = space > room / 2 ? text.slice(0, space) : text.slice(0, room);
  return copy.truncated(cut.replace(/[\s,;:(«[\-–—/]+$/, ""));
}

/** Whether the reader has settled this class: a known or ignored word needs no row. */
function settled(cls: TokenClass): boolean {
  return cls === "Known" || cls === "Ignored";
}

/**
 * The word-by-word rows of a glossed text (design D5). The candidates are the tokens; a
 * compound the lexicon does not list is replaced by its parts, and an expression of the pack
 * by itself — unless the reader marked the compound or the expression known or ignored, in
 * which case it gives nothing, its words included. A candidate makes a row when the reader
 * does not know it (unknown or learning), it is not a function word and the pack glosses it
 * with something to show; one row per dictionary form, in reading order, at most MAX_ROWS.
 */
export function rowsFor(
  tokens: PhraseToken[],
  matches: readonly PhraseMatch[] = [],
  copy: SelectionCopy = frSelection,
): GlossRow[] {
  const rows: GlossRow[] = [];
  const seen = new Set<string>();
  const add = (form: string, gloss: string | null): boolean => {
    if (!gloss || seen.has(form)) return false;
    const text = rowGloss(gloss, copy);
    if (!text) return false;
    seen.add(form);
    rows.push({ form, gloss: text });
    return rows.length >= MAX_ROWS;
  };
  for (let i = 0; i < tokens.length;) {
    // An expression covering these tokens answers for them all: it takes their place in the
    // list, and a settled one leaves nothing behind — the reader has dealt with it.
    const match = matches.find((m) => m.start === i);
    if (match) {
      if (!settled(match.class) && add(match.key, match.gloss)) return rows;
      i = match.end;
      continue;
    }
    const token = tokens[i]!;
    const candidates = token.parts && token.parts.length > 0 ? (settled(token.class) ? [] : token.parts) : [token];
    for (const c of candidates) {
      if (c.class !== "Unknown" && c.class !== "Learning") continue;
      if (c.function_word) continue;
      if (add(c.lemma, c.gloss)) return rows;
    }
    i += 1;
  }
  return rows;
}

/** The expression covering the whole text, when one does: the card's answer and its key. */
export function wholeSelectionMatch(answer: PhraseGloss): PhraseMatch | null {
  const match = answer.expressions?.find((m) => m.start === 0 && m.end === answer.tokens.length);
  return answer.tokens.length > 0 && match ? match : null;
}

/** How far outside a word's box a click still counts as being on it (px). */
const CLICK_SLACK = 3;

/**
 * Whether the click actually landed ON the word, and not merely near it.
 *
 * `caretRangeFromPoint` snaps to the nearest text position, so a click in a page's empty
 * margin — above, below or beside the column — resolves to the first or last word of a
 * line and used to open that word's card. The reader dismisses a card by clicking the empty
 * space; getting a card for a word they never pointed at is the opposite of that. The word's
 * own boxes (a line-wrapped word has several) are the truth, with a few pixels of slack so
 * clicking the edge of a letter still counts.
 */
export function clickIsOnWord(x: number, y: number, rects: Iterable<DOMRect>): boolean {
  for (const r of rects) {
    if (
      x >= r.left - CLICK_SLACK &&
      x <= r.right + CLICK_SLACK &&
      y >= r.top - CLICK_SLACK &&
      y <= r.bottom + CLICK_SLACK
    ) {
      return true;
    }
  }
  return false;
}

/**
 * What a click does to the card (design D4). A click on nothing hides the card — unless it
 * ends the gesture that opened one, in which case it leaves cards alone; so does a click on
 * a word after such a gesture, while still being stopped and, inside a link, cancelled. A
 * treated (Learning) word inside a link keeps its link on a plain click, as today.
 */
export function decideClick(input: ClickInput): ClickDecision {
  if (input.hitClass === null) {
    return { card: input.gestureOpenedCard ? "leave" : "hide", stop: false, cancel: false };
  }
  // Only an UNTREATED word (Unknown) blocks its link — "tant qu'un mot n'a pas été traité".
  if (!input.altKey && input.hitClass === "Learning" && input.isLink) {
    return { card: "leave", stop: false, cancel: false };
  }
  return { card: input.gestureOpenedCard ? "leave" : "open", stop: true, cancel: input.altKey || input.isLink };
}

/** Whether a page token needs the phrase gloss: a hyphenated form the page analysis gave no gloss. */
function needsPhraseGloss(token: AnalyzedToken): boolean {
  return token.surface.includes("-") && token.gloss === null;
}

export class SelectionCards {
  private readonly clock: Clock;
  /** The cards' copy and the language its figures are written in (localise-lingua-reading-surfaces D1, D4). */
  private readonly copy: SelectionCopy;
  private readonly interfaceLanguage: InterfaceLanguage;
  /** A capture opened a card since the last pointer-down (see `decideClick`). */
  private openedByCapture = false;

  constructor(
    private readonly ports: SelectionCardPorts,
    private readonly surface: CardSurface,
    private readonly opts: {
      clock?: Clock;
      /**
       * The translation engine as it is right now for the document's language — asked per card,
       * since the reader can turn it on or off, and its models can arrive or go, while the page is
       * open. Null: no engine.
       */
      translator?: () => TranslatorPort | null;
      /**
       * The studied language the document is read in: what a translation is asked in
       * (generalise-lingua-translation-model-state D5). English when not given.
       */
      language?: () => StudiedLanguage;
      /** The interface language the cards' figures are written in; French when not given. */
      interfaceLanguage?: InterfaceLanguage;
      /** The cards' copy in that language; the French module when not given. */
      copy?: SelectionCopy;
    },
  ) {
    this.clock = opts.clock ?? DEFAULT_CLOCK;
    this.copy = opts.copy ?? frSelection;
    this.interfaceLanguage = opts.interfaceLanguage ?? DEFAULT_INTERFACE_LANGUAGE;
  }

  /** The frequency line of a card, in the interface language. */
  private rarity(cls: TokenClass, rank: number | null | undefined): string {
    return rarityText(cls, rank, this.copy, this.interfaceLanguage);
  }

  /** A settled selection: the expression card, the page token's word card, or the analyser's. */
  openForSelection(sel: SelectionInput, hit: PageHit | null): void {
    this.openedByCapture = true;
    if (/\s/.test(sel.text)) this.openExpression(sel);
    else if (hit) this.openForToken(hit);
    else this.openLooseWord(sel);
  }

  /**
   * The word card of a page token. A token that needs nothing more opens complete, exactly
   * as before; what it can lack is fetched by one follow-up, and that card opens pending:
   * a hyphenated form with no gloss of its own asks the phrase gloss, whose single token
   * brings its gloss whatever its class and the parts of an unlisted compound; a known or
   * ignored word asks its gloss, which the page analysis withholds.
   */
  openForToken(hit: PageHit): void {
    const { token } = hit;
    const written = hit.written?.trim() || token.surface;
    const base: WordPopupContent = {
      headword: token.lemma,
      surface: token.surface,
      gloss: null,
      // No rank yet: it comes with the grammar (add-lingua-card-frequency D3).
      rarity: this.rarity(token.class, undefined),
      sentence: hit.sentence,
      status: statusOfClass(token.class),
      rect: hit.rect,
      ...this.languageOfCard(),
      ...(written !== token.surface ? { written } : {}),
    };
    const inSentence = (card: WordPopupContent, cls: TokenClass) =>
      this.wordEngine(card, cls, {
        sentence: hit.sentence,
        selection: hit.selection ?? null,
        language: this.language(),
      });
    if (needsPhraseGloss(token)) {
      this.request(
        { ...base, pending: true },
        () =>
          this.ports.phraseGloss(token.surface).then(async (answer) => ({
            answer,
            grammar: answer.tokens[0] ? await this.grammarBounded(written, answer.tokens[0].lemma) : null,
          })),
        (reply) => {
          const first = reply?.answer.tokens[0];
          // Only an unlisted compound rows itself: a listed one, whatever its class, has its
          // own gloss, which is the better answer — never a word-by-word row of itself.
          return first
            ? inSentence(
                {
                  ...base,
                  gloss: first.gloss,
                  rarity: this.rarity(first.class, reply.grammar?.rank),
                  rows: first.parts?.length ? rowsFor([first], reply.answer.expressions, this.copy) : [],
                  ...grammarOf(reply.grammar),
                },
                first.class,
              )
            : inSentence(base, token.class);
        },
      );
    } else if (token.class === "Known" || token.class === "Ignored") {
      // The page analysis withholds a known word's gloss: the grammar answer brings it.
      this.request(
        { ...base, pending: true },
        () => this.ports.wordGrammar(written, token.lemma),
        (answer) =>
          inSentence(
            {
              ...base,
              gloss: answer?.gloss ?? null,
              rarity: this.rarity(token.class, answer?.rank),
              ...grammarOf(answer),
            },
            token.class,
          ),
      );
    } else {
      this.completeWithGrammar({ ...base, gloss: token.gloss }, written, token.lemma, token.class, (card) =>
        inSentence(card, token.class),
      );
    }
  }

  /**
   * Complete a card that holds its gloss with its word's grammar: once the grammar arrives, or
   * once GRAMMAR_WAIT_MS has passed without it — whichever comes first — and never drawn pending
   * meanwhile. Nothing is shown if another card was shown, or this one hidden, in between.
   */
  private completeWithGrammar(
    card: WordPopupContent,
    written: string,
    lemma: string,
    cls: TokenClass,
    finish: (card: WordPopupContent) => PendingCard,
  ): void {
    const asked = this.surface.generation();
    void this.grammarBounded(written, lemma).then((grammar) => {
      if (this.surface.generation() !== asked) return;
      this.show(finish({ ...card, rarity: this.rarity(cls, grammar?.rank), ...grammarOf(grammar) }));
    });
  }

  /** The word's grammar, or null when the engine fails or stays silent past GRAMMAR_WAIT_MS. */
  private grammarBounded(written: string, lemma: string): Promise<WordGrammar | null> {
    return new Promise((resolve) => {
      let done = false;
      const finish = (grammar: WordGrammar | null): void => {
        if (done) return;
        done = true;
        this.clock.clearTimeout(timer);
        resolve(grammar);
      };
      const timer = this.clock.setTimeout(() => finish(null), GRAMMAR_WAIT_MS);
      new Promise<WordGrammar>((ok) => ok(this.ports.wordGrammar(written, lemma))).then(finish, () => finish(null));
    });
  }

  /** The gloss a created card carries: none for an expression, the pack's for a word. */
  async cardGloss(g: Gesture): Promise<string | null> {
    // A key holding a space is an expression, which the single-lemma port cannot answer:
    // its gloss is the one the card showed, dictionary data worth keeping. A phrase the
    // pack does not know shows no gloss, so it carries none.
    if (g.lemma.includes(" ")) return g.gloss ?? null;
    return (await this.ports.gloss(g.lemma.toLowerCase())) ?? null;
  }

  /** A pointer went down: whatever a capture opens from here on, the click that follows leaves alone. */
  gestureStarted(): void {
    this.openedByCapture = false;
  }

  gestureOpenedCard(): boolean {
    return this.openedByCapture;
  }

  /** Several words: the expression card, keyed by the text, with word-by-word rows as the answer. */
  private openExpression(sel: SelectionInput): void {
    const base: WordPopupContent = {
      headword: sel.text,
      surface: sel.text,
      gloss: null,
      rarity: this.copy.expressionKind,
      sentence: sel.sentence,
      rect: sel.rect,
      expression: true,
      // The selection's words say the document's language, as a word's card does.
      ...this.languageOfCard(),
    };
    const translator = this.translator();
    // Asked now, answered whenever. The two are no longer raced: a cold engine costs seconds on
    // a slow device (4.8 s, measured), and the reader must not sit through that to see what the
    // pack knows. The pack's answer is shown as soon as it lands, saying a translation is on its
    // way, and the translation replaces it when it arrives.
    const later = translator
      ? this.translateBounded(translator, {
          sentence: sel.sentence,
          selection: sel.selection ?? null,
          language: this.language(),
        })
      : null;
    this.request(
      { ...base, pending: true, translating: !!later },
      () => this.ports.phraseGloss(sel.text).catch(() => null),
      (answer) => ({ ...expressionCard(base, answer, this.copy), translating: !!later, later }),
    );
  }

  /**
   * A single word's card, and whether the engine is asked for it: only when there is one, the
   * pack has no gloss to show, the word is not a proper noun outside the lexicon, and its place in
   * its sentence is known — it is marked there, never translated alone. A word the pack glosses
   * keeps its dictionary card.
   */
  private wordEngine(card: WordPopupContent, cls: TokenClass, request: TranslationRequest): PendingCard {
    const translator = this.translator();
    if (!translator || card.gloss || cls === "ProperNounOutOfLexicon") return card;
    if (!request.selection || !request.sentence.trim()) return card;
    return { ...card, translating: true, later: this.translateBounded(translator, request) };
  }

  private translator(): TranslatorPort | null {
    return this.opts.translator?.() ?? null;
  }

  /** The language a translation is asked in: the document's. */
  private language(): StudiedLanguage {
    return this.opts.language?.() ?? "en";
  }

  /**
   * The language a card names its forms in (add-lingua-spanish-word-card): said only when it is
   * not English, so an English card's content is what it always was.
   */
  private languageOfCard(): Pick<WordPopupContent, "language"> {
    const language = this.language();
    return language === "en" ? {} : { language };
  }

  /** The engine's answer for this request, or null — never later than TRANSLATION_WAIT_MS. */
  private translateBounded(translator: TranslatorPort, request: TranslationRequest): Promise<MarkedTranslation | null> {
    return new Promise((resolve) => {
      const timer = this.clock.setTimeout(() => resolve(null), TRANSLATION_WAIT_MS);
      const done = (translation: MarkedTranslation | null): void => {
        this.clock.clearTimeout(timer);
        resolve(translation);
      };
      translator.translate(request).then(
        (result) => done(result.kind === "translated" ? result.translation : null),
        () => done(null),
      );
    });
  }

  /**
   * One word no page token covers: the analyser reads it and its first token decides, as a
   * page token would have — headword, status and gloss from the token, so the card is keyed
   * by the dictionary form, never by the text as written. A proper noun outside the lexicon
   * (whose dictionary form is an artefact) or no token at all gives the raw-text card of
   * before, under the text as written. Without an answer the card has no key: it offers
   * nothing to press.
   */
  private openLooseWord(sel: SelectionInput): void {
    const raw: WordPopupContent = {
      headword: sel.text,
      surface: sel.text,
      gloss: null,
      rarity: this.copy.selectionKind,
      sentence: sel.sentence,
      rect: sel.rect,
    };
    const written = sel.text.trim();
    this.request(
      { ...raw, pending: true },
      () =>
        this.ports.phraseGloss(sel.text).then(async (answer) => {
          const t = answer.tokens[0];
          const named = t && t.class !== "ProperNounOutOfLexicon";
          return { answer, grammar: named ? await this.grammarBounded(written, t.lemma) : null };
        }),
      (reply) => {
        if (!reply) return { ...raw, noActions: true };
        const { answer } = reply;
        const t = answer.tokens[0];
        if (!t || t.class === "ProperNounOutOfLexicon") return raw;
        return this.wordEngine(
          {
            headword: t.lemma,
            surface: t.surface,
            gloss: t.gloss,
            rarity: this.rarity(t.class, reply.grammar?.rank),
            sentence: sel.sentence,
            status: statusOfClass(t.class),
            rect: sel.rect,
            ...this.languageOfCard(),
            rows: t.parts?.length ? rowsFor([t], answer.expressions, this.copy) : undefined,
            ...(written !== t.surface ? { written } : {}),
            ...grammarOf(reply.grammar),
          },
          t.class,
          { sentence: sel.sentence, selection: sel.selection ?? null, language: this.language() },
        );
      },
    );
  }

  /**
   * Show `pending`, ask the engine, and complete the card exactly once — by the answer, its
   * rejection or the timer, whichever comes first (the fallback gets `null`) — and only if
   * the card view still shows the pending card: a card opened for something else, closed
   * or already completed makes the answer land nowhere.
   */
  private request<T>(
    pending: WordPopupContent,
    ask: () => Promise<T>,
    complete: (answer: T | null) => PendingCard,
  ): void {
    const generation = this.surface.show(pending);
    let settled = false;
    const settle = (answer: T | null): void => {
      if (settled) return;
      settled = true;
      this.clock.clearTimeout(timer);
      if (this.surface.generation() !== generation) return;
      this.show(complete(answer));
    };
    const timer = this.clock.setTimeout(() => settle(null), ANSWER_TIMEOUT_MS);
    new Promise<T>((resolve) => resolve(ask())).then(
      (answer) => settle(answer),
      () => settle(null),
    );
  }

  /**
   * Show a completed card and, when a translation is on its way, upgrade it once it lands — or
   * once it is known there is none. Showing bumps the generation: the upgrade is measured against
   * THIS card, so a card the reader has since replaced, closed or acted on is never written over.
   */
  private show({ later, ...card }: PendingCard): void {
    const shown = this.surface.show(card);
    if (!later) return;
    void later.then(
      (translation) => {
        if (this.surface.generation() !== shown) return;
        // A translation is a better answer than word-by-word rows, so they are not kept. Without
        // one, the card simply stops saying a translation is coming.
        this.surface.show(
          translation ? { ...card, rows: undefined, translating: false, translation } : { ...card, translating: false },
        );
      },
      () => undefined,
    );
  }
}

/** A completed card, and the translation that will upgrade it, when one was asked. */
type PendingCard = WordPopupContent & { later?: Promise<MarkedTranslation | null> | null };

/**
 * The grammar a card shows, as a spreadable field: nothing at all without an answer, or with one
 * that has nothing to say — no reading, no other word, no pieces, no part of speech to name — so
 * such a card is exactly the card it was before the grammar existed.
 */
function grammarOf(grammar: WordGrammar | null | undefined): Pick<WordPopupContent, "grammar"> {
  if (!grammar) return {};
  const says =
    grammar.readings.length > 0 ||
    grammar.others.length > 0 ||
    grammar.pieces.length > 1 ||
    grammar.senses.some((group) => group.tag !== undefined);
  return says ? { grammar } : {};
}

/** The card an expression gets from the pack alone — exactly what it was before the engine. */
function expressionCard(base: WordPopupContent, answer: PhraseGloss | null, copy: SelectionCopy): WordPopupContent {
  if (!answer) return { ...base, rows: [] };
  const whole = wholeSelectionMatch(answer);
  // An expression the pack knows is the answer, not a list of its words: the card is keyed by
  // its dictionary form — so `gave up` and `give up` are one card — and offers a word's
  // actions, the knowledge model treating it as a lemma of its own.
  if (whole) {
    return {
      ...base,
      expression: false,
      headword: whole.key,
      gloss: whole.gloss,
      status: statusOfClass(whole.class),
    };
  }
  return { ...base, rows: rowsFor(answer.tokens, answer.expressions, copy) };
}
