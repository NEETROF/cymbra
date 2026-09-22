import { describe, expect, it } from "vitest";
import type { AnalyzedToken, PhraseGloss, PhraseMatch, PhrasePart, PhraseToken } from "@/analyzer/types.ts";
import {
  ANSWER_TIMEOUT_MS,
  type CardSurface,
  type Clock,
  clickIsOnWord,
  decideClick,
  MAX_ROWS,
  type PageHit,
  rarityText,
  rowGloss,
  rowsFor,
  wholeSelectionMatch,
  SelectionCards,
  type SelectionCardPorts,
  type SelectionInput,
} from "@/reading/selection-card.ts";
import type { WordPopupContent } from "@/reading/wordpopup.ts";

// The scenarios of specs/lingua-browser-extension/spec.md, driven through fakes: a card
// surface that records every show and lets a test bump the generation (a close, a scroll,
// another card), a clock the test fires by hand, and ports whose promises the test resolves
// or rejects when it decides the engine answers.

const RECT = { left: 10, top: 20, bottom: 40 };
const CALIBRATION = 3000;

const tok = (over: Partial<PhraseToken> & Pick<PhraseToken, "surface" | "lemma" | "class">): PhraseToken => ({
  gloss: null,
  function_word: false,
  ...over,
});
const match = (over: Partial<PhraseMatch> & Pick<PhraseMatch, "start" | "end" | "key" | "gloss">): PhraseMatch => ({
  class: "Unknown",
  ...over,
});
const part = (over: Partial<PhrasePart> & Pick<PhrasePart, "lemma" | "class">): PhrasePart => ({
  gloss: null,
  function_word: false,
  ...over,
});
const pageToken = (
  over: Partial<AnalyzedToken> & Pick<AnalyzedToken, "surface" | "lemma" | "class">,
): AnalyzedToken => ({
  block: 0,
  start: 0,
  end: 0,
  gloss: null,
  ...over,
});
const selection = (text: string): SelectionInput => ({ text, sentence: `They ${text} on Friday.`, rect: RECT });
const hitOf = (token: AnalyzedToken): PageHit => ({ token, rect: RECT, sentence: `They ${token.surface} on Friday.` });

interface Deferred<T> {
  resolve: (value: T) => void;
  reject: (reason: unknown) => void;
}

function fakePorts() {
  const phraseGloss: Array<Deferred<PhraseGloss> & { text: string }> = [];
  const gloss: Array<Deferred<string | undefined> & { lemma: string }> = [];
  const ports: SelectionCardPorts = {
    phraseGloss: (text) =>
      new Promise<PhraseGloss>((resolve, reject) => {
        phraseGloss.push({ text, resolve, reject });
      }),
    gloss: (lemma) =>
      new Promise<string | undefined>((resolve, reject) => {
        gloss.push({ lemma, resolve, reject });
      }),
  };
  return { ports, phraseGloss, gloss };
}

function fakeSurface() {
  const shows: WordPopupContent[] = [];
  let generation = 0;
  const surface: CardSurface = {
    show: (content) => {
      shows.push(content);
      return ++generation;
    },
    generation: () => generation,
  };
  return {
    surface,
    shows,
    last: () => shows.at(-1)!,
    /** The reader closed, scrolled or otherwise hid the card: the view bumps on a hide too. */
    hide: () => ++generation,
  };
}

function fakeClock() {
  const timers = new Map<number, { fn: () => void; ms: number }>();
  let next = 0;
  const clock: Clock = {
    setTimeout: (fn, ms) => {
      timers.set(++next, { fn, ms });
      return next;
    },
    clearTimeout: (handle) => {
      timers.delete(handle as number);
    },
  };
  return {
    clock,
    armed: () => [...timers.values()].map((t) => t.ms),
    /** The bounded wait elapses. */
    elapse: () => {
      const due = [...timers.entries()];
      timers.clear();
      for (const [, t] of due) t.fn();
    },
  };
}

/** The gesture a card's button emits, as `createCard` builds it from the content on screen. */
function gesture(content: WordPopupContent) {
  return {
    lemma: content.headword,
    surface: content.surface,
    sentence: content.sentence,
    status: null,
    expression: !!content.expression,
    gloss: content.gloss,
  };
}

/** Let the engine's answer (a promise) reach the card. */
const settle = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

function harness() {
  const ports = fakePorts();
  const view = fakeSurface();
  const clock = fakeClock();
  const cards = new SelectionCards(ports.ports, view.surface, { calibration: () => CALIBRATION, clock: clock.clock });
  return { cards, ...ports, ...view, ...clock };
}

describe("a selected word resolves to its dictionary form", () => {
  it("An inflected word in a block the page analysis skipped", async () => {
    const h = harness();
    h.cards.openForSelection(selection("endeavors"), null);
    expect(h.last()).toMatchObject({
      headword: "endeavors",
      surface: "endeavors",
      rarity: "Sélection.",
      pending: true,
    });
    expect(h.phraseGloss[0]!.text).toBe("endeavors");

    h.phraseGloss[0]!.resolve({
      tokens: [tok({ surface: "endeavors", lemma: "endeavor", class: "Unknown", gloss: "effort" })],
    });
    await settle();

    expect(h.shows).toHaveLength(2);
    expect(h.last()).toEqual({
      headword: "endeavor",
      surface: "endeavors",
      gloss: "effort",
      rarity: rarityText("Unknown", CALIBRATION),
      sentence: "They endeavors on Friday.",
      status: null,
      rect: RECT,
      rows: undefined,
    });
  });

  it("A contraction in a block the page analysis skipped", async () => {
    const h = harness();
    h.cards.openForSelection(selection("don't"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "don't", lemma: "do", class: "Known", gloss: "faire", function_word: true }),
        tok({ surface: "n't", lemma: "not", class: "Known", function_word: true }),
      ],
    });
    await settle();
    expect(h.last()).toMatchObject({ headword: "do", surface: "don't", gloss: "faire", status: "known" });
  });

  it("A compound the pack lists", () => {
    const h = harness();
    const token = pageToken({ surface: "well-known", lemma: "well-known", class: "Unknown", gloss: "bien connu" });
    h.cards.openForSelection(selection("well-known"), hitOf(token));
    expect(h.shows).toHaveLength(1);
    expect(h.last()).toEqual({
      headword: "well-known",
      surface: "well-known",
      gloss: "bien connu",
      rarity: rarityText("Unknown", CALIBRATION),
      sentence: "They well-known on Friday.",
      status: null, // new to the reader: "Je connais" is offered
      rect: RECT,
    });
    expect(h.phraseGloss).toHaveLength(0);
    expect(h.gloss).toHaveLength(0);
  });

  it("A name", async () => {
    const h = harness();
    h.cards.openForSelection(selection("Jenkins"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [tok({ surface: "Jenkins", lemma: "jenkin", class: "ProperNounOutOfLexicon" })],
    });
    await settle();
    expect(h.last()).toEqual({
      headword: "Jenkins",
      surface: "Jenkins",
      gloss: null,
      rarity: "Sélection.",
      sentence: "They Jenkins on Friday.",
      rect: RECT,
    });
    expect(h.last().expression).toBeFalsy();
    expect(h.last().noActions).toBeFalsy();
  });

  it("A name: no token at all gives the same card", async () => {
    const h = harness();
    h.cards.openForSelection(selection("B2B"), null);
    h.phraseGloss[0]!.resolve({ tokens: [] });
    await settle();
    expect(h.last()).toMatchObject({ headword: "B2B", surface: "B2B", gloss: null, rarity: "Sélection." });
    expect(h.last().noActions).toBeFalsy();
  });

  it("Selecting with the mouse", () => {
    const h = harness();
    h.cards.gestureStarted(); // mousedown
    expect(h.cards.gestureOpenedCard()).toBe(false);
    h.cards.openForSelection(selection("endeavors"), null); // the capture flushed on mouseup
    expect(h.cards.gestureOpenedCard()).toBe(true);
    // The click that ends the gesture lands on no painted word and leaves the card alone.
    expect(decideClick({ gestureOpenedCard: true, hitClass: null, isLink: false, altKey: false })).toEqual({
      card: "leave",
      stop: false,
      cancel: false,
    });
    // The next gesture starts clean: a plain click on nothing hides the card again.
    h.cards.gestureStarted();
    expect(h.cards.gestureOpenedCard()).toBe(false);
    expect(decideClick({ gestureOpenedCard: false, hitClass: null, isLink: false, altKey: false }).card).toBe("hide");
  });

  it("Double-clicking an unknown word inside a link", () => {
    // The first click opens the card and blocks the link, as a plain click does.
    expect(decideClick({ gestureOpenedCard: false, hitClass: "Unknown", isLink: true, altKey: false })).toEqual({
      card: "open",
      stop: true,
      cancel: true,
    });
    // The second ends the selection gesture that opened the card: it opens and hides nothing,
    // and still does not follow the link.
    expect(decideClick({ gestureOpenedCard: true, hitClass: "Unknown", isLink: true, altKey: false })).toEqual({
      card: "leave",
      stop: true,
      cancel: true,
    });
  });
});

describe("word-by-word gloss is a labelled last resort", () => {
  it("A free combination of words", async () => {
    const h = harness();
    h.cards.openForSelection(selection("a compelling argument"), null);
    expect(h.last()).toMatchObject({
      headword: "a compelling argument",
      rarity: "Expression — la carte gardera sa phrase d’origine.",
      expression: true,
      pending: true,
    });
    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "a", lemma: "a", class: "Known", gloss: "un", function_word: true }),
        tok({ surface: "compelling", lemma: "compelling", class: "Unknown", gloss: "convaincant" }),
        tok({ surface: "argument", lemma: "argument", class: "Known", gloss: "argument" }),
      ],
    });
    await settle();
    expect(h.last()).toEqual({
      headword: "a compelling argument",
      surface: "a compelling argument",
      gloss: null,
      rarity: "Expression — la carte gardera sa phrase d’origine.",
      sentence: "They a compelling argument on Friday.",
      rect: RECT,
      expression: true,
      rows: [{ form: "compelling", gloss: "convaincant" }],
    });
  });

  it("Nothing worth showing", async () => {
    const h = harness();
    h.cards.openForSelection(selection("put up with"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "put", lemma: "put", class: "Known", gloss: "mettre" }),
        tok({ surface: "up", lemma: "up", class: "Unknown", gloss: "haut", function_word: true }),
        tok({ surface: "with", lemma: "with", class: "Unknown", gloss: "avec", function_word: true }),
      ],
    });
    await settle();
    expect(h.last()).toMatchObject({ expression: true, gloss: null, rows: [] });
  });

  it("A compound the lexicon does not list, on an analysed page", async () => {
    const h = harness();
    const token = pageToken({ surface: "error-prone", lemma: "error-prone", class: "Unknown", gloss: null });
    h.cards.openForSelection(selection("error-prone"), hitOf(token));
    expect(h.last()).toMatchObject({ headword: "error-prone", status: null, pending: true });
    expect(h.phraseGloss[0]!.text).toBe("error-prone");

    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({
          surface: "error-prone",
          lemma: "error-prone",
          class: "Unknown",
          parts: [
            part({ lemma: "error", class: "Known", gloss: "erreur" }),
            part({ lemma: "prone", class: "Unknown", gloss: "enclin" }),
          ],
        }),
      ],
    });
    await settle();
    expect(h.last()).toEqual({
      headword: "error-prone",
      surface: "error-prone",
      gloss: null,
      rarity: rarityText("Unknown", CALIBRATION),
      sentence: "They error-prone on Friday.",
      status: null,
      rect: RECT,
      rows: [{ form: "prone", gloss: "enclin" }],
    });
  });

  it("A compound inside a phrase", async () => {
    const h = harness();
    h.cards.openForSelection(selection("an error-prone approach"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "an", lemma: "a", class: "Known", gloss: "un", function_word: true }),
        tok({
          surface: "error-prone",
          lemma: "error-prone",
          class: "Unknown",
          parts: [
            part({ lemma: "error", class: "Known", gloss: "erreur" }),
            part({ lemma: "prone", class: "Unknown", gloss: "enclin" }),
          ],
        }),
        tok({ surface: "approach", lemma: "approach", class: "Known", gloss: "approche" }),
      ],
    });
    await settle();
    expect(h.last().rows).toEqual([{ form: "prone", gloss: "enclin" }]);
  });

  it("A compound the reader has marked known", async () => {
    const h = harness();
    h.cards.openForSelection(selection("an error-prone approach"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "an", lemma: "a", class: "Known", gloss: "un", function_word: true }),
        tok({
          surface: "error-prone",
          lemma: "error-prone",
          class: "Known",
          parts: [
            part({ lemma: "error", class: "Known", gloss: "erreur" }),
            part({ lemma: "prone", class: "Unknown", gloss: "enclin" }),
          ],
        }),
        tok({ surface: "approach", lemma: "approach", class: "Known", gloss: "approche" }),
      ],
    });
    await settle();
    expect(h.last().rows).toEqual([]);
  });

  it("The rows never reach a card", async () => {
    const h = harness();
    const gloss = await h.cards.cardGloss({
      lemma: "a compelling argument",
      surface: "a compelling argument",
      sentence: "s",
      status: "learning",
      expression: true,
      gloss: null,
    });
    expect(gloss).toBeNull();
    expect(h.gloss).toHaveLength(0); // the pack is not even asked
  });

  it("A long selection", async () => {
    const h = harness();
    const words = ["alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel"];
    h.cards.openForSelection(selection(words.join(" ")), null);
    h.phraseGloss[0]!.resolve({
      tokens: words.map((w) => tok({ surface: w, lemma: w, class: "Unknown", gloss: `g-${w}` })),
    });
    await settle();
    expect(h.last().rows).toHaveLength(MAX_ROWS);
    expect(h.last().rows!.map((r) => r.form)).toEqual(words.slice(0, MAX_ROWS));
  });
});

describe("a known word shows its gloss", () => {
  it("Opening a known word", async () => {
    const h = harness();
    const token = pageToken({ surface: "cities", lemma: "city", class: "Known", gloss: null });
    h.cards.openForToken(hitOf(token));
    expect(h.last()).toMatchObject({ headword: "city", surface: "cities", status: "known", pending: true });
    expect(h.gloss[0]!.lemma).toBe("city");

    h.gloss[0]!.resolve("ville");
    await settle();
    expect(h.last()).toEqual({
      headword: "city",
      surface: "cities",
      gloss: "ville",
      rarity: rarityText("Known", CALIBRATION),
      sentence: "They cities on Friday.",
      status: "known",
      rect: RECT,
    });
    expect(h.phraseGloss).toHaveLength(0);
  });

  it("Opening a known word: an ignored word asks the pack too", async () => {
    const h = harness();
    h.cards.openForToken(hitOf(pageToken({ surface: "ok", lemma: "ok", class: "Ignored" })));
    expect(h.gloss[0]!.lemma).toBe("ok");
    h.gloss[0]!.resolve(undefined);
    await settle();
    expect(h.last()).toMatchObject({ status: "ignored", gloss: null });
    expect(h.last().pending).toBeFalsy();
  });

  it("opens a highlighted word complete at once, its gloss having come with the page analysis", () => {
    const h = harness();
    const token = pageToken({ surface: "Seldom", lemma: "seldom", class: "Learning", gloss: "rarement" });
    h.cards.openForToken(hitOf(token));
    expect(h.shows).toHaveLength(1);
    expect(h.last()).toEqual({
      headword: "seldom",
      surface: "Seldom",
      gloss: "rarement",
      rarity: rarityText("Learning", CALIBRATION),
      sentence: "They Seldom on Friday.",
      status: "learning",
      rect: RECT,
    });
    expect(h.armed()).toEqual([]);
  });

  it("asks the phrase gloss for a hyphenated page token the analysis gave no gloss, whatever its class", async () => {
    const h = harness();
    const token = pageToken({ surface: "well-known", lemma: "well-known", class: "Known", gloss: null });
    h.cards.openForToken(hitOf(token));
    expect(h.phraseGloss[0]!.text).toBe("well-known");
    expect(h.gloss).toHaveLength(0);
    h.phraseGloss[0]!.resolve({
      tokens: [tok({ surface: "well-known", lemma: "well-known", class: "Known", gloss: "bien connu" })],
    });
    await settle();
    expect(h.last()).toMatchObject({ headword: "well-known", gloss: "bien connu", status: "known", rows: [] });
  });

  it("never rows a listed compound against itself, even when the page token is stale", async () => {
    // The page token still says Ignored (gloss withheld) while the engine, restored by a sync
    // in the meantime, now classes the compound Unknown: its own gloss is the answer, not a
    // word-by-word row of itself under the « Mot à mot » label.
    const h = harness();
    h.cards.openForToken(
      hitOf(pageToken({ surface: "well-known", lemma: "well-known", class: "Ignored", gloss: null })),
    );
    h.phraseGloss[0]!.resolve({
      tokens: [tok({ surface: "well-known", lemma: "well-known", class: "Unknown", gloss: "bien connu" })],
    });
    await settle();
    expect(h.last()).toMatchObject({ headword: "well-known", gloss: "bien connu", rows: [] });
  });
});

describe("a card that waits for the engine", () => {
  it("An engine that takes time to wake", async () => {
    const h = harness();
    h.cards.openForSelection(selection("gave up early"), null);
    // Visible at once, pending, before the engine has said anything.
    expect(h.shows).toHaveLength(1);
    expect(h.last().pending).toBe(true);
    expect(h.armed()).toEqual([ANSWER_TIMEOUT_MS]);

    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "gave", lemma: "give", class: "Learning", gloss: "donner" }),
        tok({ surface: "up", lemma: "up", class: "Known", gloss: "haut", function_word: true }),
        tok({ surface: "early", lemma: "early", class: "Unknown", gloss: "tôt" }),
      ],
    });
    await settle();
    // The answer and the actions arrive together: one complete show, no longer pending.
    expect(h.shows).toHaveLength(2);
    expect(h.last().pending).toBeFalsy();
    expect(h.last().noActions).toBeFalsy();
    expect(h.last().rows).toEqual([
      { form: "give", gloss: "donner" },
      { form: "early", gloss: "tôt" },
    ]);
  });

  it("An answer that arrives too late", async () => {
    const h = harness();
    h.cards.openForSelection(selection("seldom ship"), null);
    h.cards.openForSelection(selection("seldom ship on"), null); // the reader extended the selection
    expect(h.shows).toHaveLength(2);

    h.phraseGloss[0]!.resolve({
      tokens: [tok({ surface: "seldom", lemma: "seldom", class: "Unknown", gloss: "rarement" })],
    });
    await settle();
    expect(h.shows).toHaveLength(2); // the first answer lands nowhere

    h.phraseGloss[1]!.resolve({
      tokens: [tok({ surface: "ship", lemma: "ship", class: "Unknown", gloss: "expédier" })],
    });
    await settle();
    expect(h.shows).toHaveLength(3);
    expect(h.last()).toMatchObject({ headword: "seldom ship on", rows: [{ form: "ship", gloss: "expédier" }] });
  });

  it("Another card opened in the meantime", async () => {
    const h = harness();
    h.cards.openForSelection(selection("seldom ship"), null);
    const word = pageToken({ surface: "conundrum", lemma: "conundrum", class: "Unknown", gloss: "énigme" });
    h.cards.openForToken(hitOf(word)); // the reader clicked a highlighted word
    expect(h.last().headword).toBe("conundrum");

    h.phraseGloss[0]!.resolve({
      tokens: [tok({ surface: "seldom", lemma: "seldom", class: "Unknown", gloss: "rarement" })],
    });
    await settle();
    expect(h.shows).toHaveLength(2);
    expect(h.last().headword).toBe("conundrum"); // left as it was
  });

  it("A pending card closed by the reader", async () => {
    const h = harness();
    h.cards.openForSelection(selection("seldom ship"), null);
    h.hide(); // the close button
    h.phraseGloss[0]!.resolve({
      tokens: [tok({ surface: "seldom", lemma: "seldom", class: "Unknown", gloss: "rarement" })],
    });
    await settle();
    expect(h.shows).toHaveLength(1); // no card reopens
  });

  it("An engine that does not answer, for a selection of several words", async () => {
    const h = harness();
    h.cards.openForSelection(selection("seldom ship"), null);
    h.elapse();
    expect(h.shows).toHaveLength(2);
    expect(h.last()).toEqual({
      headword: "seldom ship",
      surface: "seldom ship",
      gloss: null,
      rarity: "Expression — la carte gardera sa phrase d’origine.",
      sentence: "They seldom ship on Friday.",
      rect: RECT,
      expression: true,
      rows: [],
    });

    // The engine failing outright completes the same card.
    const failed = harness();
    failed.cards.openForSelection(selection("seldom ship"), null);
    failed.phraseGloss[0]!.reject(new Error("event page gone"));
    await settle();
    expect(failed.last()).toMatchObject({ expression: true, gloss: null, rows: [] });
    expect(failed.last().noActions).toBeFalsy();
  });

  it("An engine that does not answer, for a known word", async () => {
    const h = harness();
    h.cards.openForToken(hitOf(pageToken({ surface: "cities", lemma: "city", class: "Known" })));
    h.elapse();
    expect(h.last()).toEqual({
      headword: "city",
      surface: "cities",
      gloss: null,
      rarity: rarityText("Known", CALIBRATION),
      sentence: "They cities on Friday.",
      status: "known",
      rect: RECT,
    });

    const failed = harness();
    failed.cards.openForToken(hitOf(pageToken({ surface: "cities", lemma: "city", class: "Known" })));
    failed.gloss[0]!.reject(new Error("event page gone"));
    await settle();
    expect(failed.last()).toMatchObject({ headword: "city", status: "known", gloss: null });
    expect(failed.last().noActions).toBeFalsy();
  });

  it("An engine that does not answer, for a hyphenated page token", async () => {
    const h = harness();
    h.cards.openForToken(hitOf(pageToken({ surface: "error-prone", lemma: "error-prone", class: "Unknown" })));
    h.elapse();
    expect(h.last()).toMatchObject({ headword: "error-prone", status: null, gloss: null });
    expect(h.last().rows).toBeUndefined();
    expect(h.last().noActions).toBeFalsy();
  });

  it("An engine that does not answer, for a word outside the page analysis", async () => {
    const h = harness();
    h.cards.openForSelection(selection("endeavors"), null);
    h.elapse();
    expect(h.last()).toEqual({
      headword: "endeavors",
      surface: "endeavors",
      gloss: null,
      rarity: "Sélection.",
      sentence: "They endeavors on Friday.",
      rect: RECT,
      noActions: true,
    });

    const failed = harness();
    failed.cards.openForSelection(selection("endeavors"), null);
    failed.phraseGloss[0]!.reject(new Error("event page gone"));
    await settle();
    expect(failed.last()).toMatchObject({ headword: "endeavors", noActions: true });
  });

  it("An answer that arrives after the fallback", async () => {
    const h = harness();
    h.cards.openForSelection(selection("endeavors"), null);
    h.elapse();
    expect(h.last().noActions).toBe(true);

    h.phraseGloss[0]!.resolve({
      tokens: [tok({ surface: "endeavors", lemma: "endeavor", class: "Unknown", gloss: "effort" })],
    });
    await settle();
    expect(h.shows).toHaveLength(2);
    expect(h.last().noActions).toBe(true); // left as the fallback made it
  });

  it("settles once: the answer first clears the timer, which then has nothing to do", async () => {
    const h = harness();
    h.cards.openForSelection(selection("endeavors"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [tok({ surface: "endeavors", lemma: "endeavor", class: "Unknown", gloss: "effort" })],
    });
    await settle();
    expect(h.armed()).toEqual([]); // cleared on the answer
    h.elapse();
    expect(h.shows).toHaveLength(2);
    expect(h.last().headword).toBe("endeavor");
  });

  it("settles once: a rejection after the timer is dropped", async () => {
    const h = harness();
    h.cards.openForSelection(selection("seldom ship"), null);
    h.elapse();
    h.phraseGloss[0]!.reject(new Error("late"));
    await settle();
    expect(h.shows).toHaveLength(2);
  });

  it("drops a fallback for a card the reader has left", () => {
    const h = harness();
    h.cards.openForSelection(selection("seldom ship"), null);
    h.hide(); // Escape, a scroll, the reader switched off
    h.elapse();
    expect(h.shows).toHaveLength(1);
  });
});

describe("an expression is the card's answer", () => {
  /** `gave up`, as the analyser and the pack answer it. */
  const gaveUp: PhraseGloss = {
    tokens: [
      tok({ surface: "gave", lemma: "give", class: "Known", gloss: "donner" }),
      tok({ surface: "up", lemma: "up", class: "Known", gloss: "haut", function_word: true }),
    ],
    expressions: [match({ start: 0, end: 2, key: "give up", gloss: "Abandonner; Se rendre" })],
  };

  it("A phrasal verb whose words the reader knows", async () => {
    const h = harness();
    h.cards.openForSelection(selection("put up with"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "put", lemma: "put", class: "Known", gloss: "mettre" }),
        tok({ surface: "up", lemma: "up", class: "Known", gloss: "haut", function_word: true }),
        tok({ surface: "with", lemma: "with", class: "Known", gloss: "avec", function_word: true }),
      ],
      expressions: [match({ start: 0, end: 3, key: "put up with", gloss: "Supporter, subir; Faire avec" })],
    });
    await settle();
    expect(h.last()).toMatchObject({ headword: "put up with", gloss: "Supporter, subir; Faire avec" });
    expect(h.last().rows).toBeUndefined(); // the gloss line, not a list of its words
  });

  it("An inflected expression makes one card", async () => {
    const h = harness();
    h.cards.openForSelection(selection("gave up"), null);
    h.phraseGloss[0]!.resolve(gaveUp);
    await settle();
    // The card is keyed by the headword, which `onGesture` lowercases into the card's key:
    // selecting `give up` later reaches the same one.
    expect(h.last().headword).toBe("give up");
    expect(await h.cards.cardGloss({ ...gesture(h.last()), status: "learning" })).toBe("Abandonner; Se rendre");
  });

  it("The form as seen", async () => {
    const h = harness();
    h.cards.openForSelection(selection("gave up"), null);
    h.phraseGloss[0]!.resolve(gaveUp);
    await settle();
    expect(h.last()).toMatchObject({ headword: "give up", surface: "gave up" });
  });

  it("offers a word's actions, « Je connais » included", async () => {
    const h = harness();
    h.cards.openForSelection(selection("gave up"), null);
    h.phraseGloss[0]!.resolve(gaveUp);
    await settle();
    expect(h.last().expression).toBe(false); // an expression the pack keys is a word to the card
    expect(h.last().status).toBeNull();
  });

  it("carries the status the reader gave the expression", async () => {
    const h = harness();
    h.cards.openForSelection(selection("gave up"), null);
    h.phraseGloss[0]!.resolve({
      ...gaveUp,
      expressions: [match({ start: 0, end: 2, key: "give up", gloss: "Abandonner", class: "Learning" })],
    });
    await settle();
    expect(h.last().status).toBe("learning");
  });

  it("An expression inside a phrase", async () => {
    const h = harness();
    h.cards.openForSelection(selection("a compelling starting point"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "a", lemma: "a", class: "Known", gloss: "un", function_word: true }),
        tok({ surface: "compelling", lemma: "compel", class: "Unknown", gloss: "Contraindre" }),
        tok({ surface: "starting", lemma: "start", class: "Unknown", gloss: "Commencer" }),
        tok({ surface: "point", lemma: "point", class: "Unknown", gloss: "Point" }),
      ],
      expressions: [match({ start: 2, end: 4, key: "start point", gloss: "Point de départ" })],
    });
    await settle();
    expect(h.last().rows).toEqual([
      { form: "compel", gloss: "Contraindre" },
      { form: "start point", gloss: "Point de départ" },
    ]);
    expect(h.last().gloss).toBeNull(); // still the expression card, not an answer of its own
  });

  it("An expression the reader has settled", async () => {
    const h = harness();
    h.cards.openForSelection(selection("a compelling starting point"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "a", lemma: "a", class: "Known", gloss: "un", function_word: true }),
        tok({ surface: "compelling", lemma: "compel", class: "Unknown", gloss: "Contraindre" }),
        tok({ surface: "starting", lemma: "start", class: "Unknown", gloss: "Commencer" }),
        tok({ surface: "point", lemma: "point", class: "Unknown", gloss: "Point" }),
      ],
      expressions: [match({ start: 2, end: 4, key: "start point", gloss: "Point de départ", class: "Known" })],
    });
    await settle();
    expect(h.last().rows).toEqual([{ form: "compel", gloss: "Contraindre" }]);
  });

  it("An idiom is not taken apart", async () => {
    const h = harness();
    h.cards.openForSelection(selection("raining cats and dogs"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "raining", lemma: "rain", class: "Unknown", gloss: "Pleuvoir" }),
        tok({ surface: "cats", lemma: "cat", class: "Unknown", gloss: "Chat" }),
        tok({ surface: "and", lemma: "and", class: "Known", gloss: "et", function_word: true }),
        tok({ surface: "dogs", lemma: "dog", class: "Unknown", gloss: "Chien" }),
      ],
      expressions: [match({ start: 0, end: 4, key: "rain cat and dog", gloss: "Pleuvoir à verse" })],
    });
    await settle();
    expect(h.last()).toMatchObject({ headword: "rain cat and dog", gloss: "Pleuvoir à verse" });
    expect(h.last().rows).toBeUndefined();
  });

  it("No expression", async () => {
    const h = harness();
    h.cards.openForSelection(selection("a compelling argument"), null);
    h.phraseGloss[0]!.resolve({
      tokens: [
        tok({ surface: "a", lemma: "a", class: "Known", gloss: "un", function_word: true }),
        tok({ surface: "compelling", lemma: "compel", class: "Unknown", gloss: "Contraindre" }),
        tok({ surface: "argument", lemma: "argument", class: "Known", gloss: "argument" }),
      ],
    });
    await settle();
    expect(h.last()).toMatchObject({ expression: true, gloss: null });
    expect(h.last().rows).toEqual([{ form: "compel", gloss: "Contraindre" }]);
  });
});

describe("wholeSelectionMatch", () => {
  it("takes a match covering every token", () => {
    const answer: PhraseGloss = {
      tokens: [
        tok({ surface: "gave", lemma: "give", class: "Known" }),
        tok({ surface: "up", lemma: "up", class: "Known" }),
      ],
      expressions: [match({ start: 0, end: 2, key: "give up", gloss: "Abandonner" })],
    };
    expect(wholeSelectionMatch(answer)?.key).toBe("give up");
  });

  it("refuses one that covers only part of them", () => {
    const answer: PhraseGloss = {
      tokens: [
        tok({ surface: "a", lemma: "a", class: "Known" }),
        tok({ surface: "starting", lemma: "start", class: "Unknown" }),
        tok({ surface: "point", lemma: "point", class: "Unknown" }),
      ],
      expressions: [match({ start: 1, end: 3, key: "start point", gloss: "Point de départ" })],
    };
    expect(wholeSelectionMatch(answer)).toBeNull();
  });

  it("refuses an answer with no token at all", () => {
    expect(wholeSelectionMatch({ tokens: [], expressions: [] })).toBeNull();
  });
});

describe("clickIsOnWord", () => {
  const word = (): DOMRect[] => [{ left: 100, right: 160, top: 40, bottom: 58 } as DOMRect];

  it("takes a click inside the word", () => {
    expect(clickIsOnWord(130, 50, word())).toBe(true);
  });

  it("takes a click a couple of pixels off its edge", () => {
    expect(clickIsOnWord(162, 59, word())).toBe(true);
  });

  it("refuses a click in the margin beside the line, where the caret would snap to the word", () => {
    expect(clickIsOnWord(40, 50, word())).toBe(false);
    expect(clickIsOnWord(400, 50, word())).toBe(false);
  });

  it("refuses a click above or below the line", () => {
    expect(clickIsOnWord(130, 10, word())).toBe(false);
    expect(clickIsOnWord(130, 200, word())).toBe(false);
  });

  it("takes a click on either box of a word broken across two lines", () => {
    const wrapped = [
      { left: 600, right: 640, top: 40, bottom: 58 },
      { left: 20, right: 60, top: 60, bottom: 78 },
    ] as DOMRect[];
    expect(clickIsOnWord(30, 70, wrapped)).toBe(true);
    expect(clickIsOnWord(300, 70, wrapped)).toBe(false);
  });

  it("refuses a word with no box at all", () => {
    expect(clickIsOnWord(130, 50, [])).toBe(false);
  });
});

describe("rowGloss", () => {
  it("keeps the first sense, so a row never ends mid-word on the reducer's cut", () => {
    // The real pack's `start`, cut at 80 characters: three senses, the last one truncated.
    expect(rowGloss("Commencement, début, inauguration; Commencer, débuter, initier, entamer; Procédu")).toBe(
      "Commencement, début, inauguration",
    );
  });

  it("returns a one-sense gloss unchanged", () => {
    expect(rowGloss("Astreindre, contraindre, forcer")).toBe("Astreindre, contraindre, forcer");
  });

  it("skips a sense that says the definition is missing", () => {
    expect(rowGloss("Définition manquante ou à compléter; Erreur, faute")).toBe("Erreur, faute");
  });

  it("gives no row when every sense is missing its definition", () => {
    expect(rowGloss("Définition manquante ou à compléter")).toBeNull();
  });
});

describe("rowsFor", () => {
  it("shows the first sense of a gloss, not the whole entry", () => {
    expect(
      rowsFor([
        tok({ surface: "prone", lemma: "prone", class: "Unknown", gloss: "Susceptible, enclin; À plat ventre" }),
      ]),
    ).toEqual([{ form: "prone", gloss: "Susceptible, enclin" }]);
  });

  it("leaves out a word whose only gloss says the definition is missing", () => {
    expect(
      rowsFor([
        tok({ surface: "zorb", lemma: "zorb", class: "Unknown", gloss: "Définition manquante ou à compléter" }),
      ]),
    ).toEqual([]);
  });

  it("lists each dictionary form once, in reading order", () => {
    const rows = rowsFor([
      tok({ surface: "ships", lemma: "ship", class: "Unknown", gloss: "expédier" }),
      tok({ surface: "seldom", lemma: "seldom", class: "Learning", gloss: "rarement" }),
      tok({ surface: "ship", lemma: "ship", class: "Unknown", gloss: "expédier" }),
    ]);
    expect(rows).toEqual([
      { form: "ship", gloss: "expédier" },
      { form: "seldom", gloss: "rarement" },
    ]);
  });

  it("leaves out known and ignored words, function words, proper nouns and unglossed words", () => {
    expect(
      rowsFor([
        tok({ surface: "the", lemma: "the", class: "Unknown", gloss: "le", function_word: true }),
        tok({ surface: "city", lemma: "city", class: "Known", gloss: "ville" }),
        tok({ surface: "ok", lemma: "ok", class: "Ignored", gloss: "ok" }),
        tok({ surface: "Jenkins", lemma: "jenkin", class: "ProperNounOutOfLexicon", gloss: "jenkins" }),
        tok({ surface: "zorb", lemma: "zorb", class: "Unknown", gloss: null }),
      ]),
    ).toEqual([]);
  });

  it("replaces an unlisted compound by its parts, filtered like words", () => {
    const compound = tok({
      surface: "opt-in",
      lemma: "opt-in",
      class: "Unknown",
      parts: [
        part({ lemma: "opt", class: "Unknown", gloss: "choisir" }),
        part({ lemma: "in", class: "Unknown", gloss: "dans", function_word: true }),
      ],
    });
    expect(rowsFor([compound])).toEqual([{ form: "opt", gloss: "choisir" }]);
    expect(rowsFor([tok({ surface: "an", lemma: "a", class: "Known", function_word: true }), compound])).toEqual([
      { form: "opt", gloss: "choisir" },
    ]);
  });

  it("gives nothing for a compound the reader marked known or ignored, whatever its parts", () => {
    const parts = [part({ lemma: "opt", class: "Unknown", gloss: "choisir" })];
    expect(rowsFor([tok({ surface: "opt-in", lemma: "opt-in", class: "Known", parts })])).toEqual([]);
    expect(rowsFor([tok({ surface: "opt-in", lemma: "opt-in", class: "Ignored", parts })])).toEqual([]);
  });

  it("stops at the bound", () => {
    const tokens = Array.from({ length: MAX_ROWS + 3 }, (_, i) =>
      tok({ surface: `w${i}`, lemma: `w${i}`, class: "Unknown", gloss: `g${i}` }),
    );
    expect(rowsFor(tokens)).toHaveLength(MAX_ROWS);
  });
});

describe("decideClick", () => {
  it("opens the card of a painted word on a plain click, and stops the click", () => {
    expect(decideClick({ gestureOpenedCard: false, hitClass: "Unknown", isLink: false, altKey: false })).toEqual({
      card: "open",
      stop: true,
      cancel: false,
    });
    expect(decideClick({ gestureOpenedCard: false, hitClass: "Known", isLink: false, altKey: true })).toEqual({
      card: "open",
      stop: true,
      cancel: true,
    });
  });

  it("hides the card on a click that lands on no word, unless the click ends a gesture", () => {
    expect(decideClick({ gestureOpenedCard: false, hitClass: null, isLink: true, altKey: false }).card).toBe("hide");
    expect(decideClick({ gestureOpenedCard: true, hitClass: null, isLink: true, altKey: false }).card).toBe("leave");
  });

  it("lets a treated word inside a link follow its link on a plain click, and opens it on Alt", () => {
    expect(decideClick({ gestureOpenedCard: false, hitClass: "Learning", isLink: true, altKey: false })).toEqual({
      card: "leave",
      stop: false,
      cancel: false,
    });
    expect(decideClick({ gestureOpenedCard: false, hitClass: "Learning", isLink: true, altKey: true })).toEqual({
      card: "open",
      stop: true,
      cancel: true,
    });
  });

  it("neither opens nor hides on the click that ends a gesture, yet still cancels an untreated word's link", () => {
    expect(decideClick({ gestureOpenedCard: true, hitClass: "Unknown", isLink: false, altKey: false })).toEqual({
      card: "leave",
      stop: true,
      cancel: false,
    });
    expect(decideClick({ gestureOpenedCard: true, hitClass: "Unknown", isLink: true, altKey: false })).toEqual({
      card: "leave",
      stop: true,
      cancel: true,
    });
  });
});

describe("cardGloss", () => {
  it("asks the pack for a word's gloss by its lowercased dictionary form", async () => {
    const h = harness();
    const pending = h.cards.cardGloss({
      lemma: "Seldom",
      surface: "Seldom",
      sentence: "s",
      status: "learning",
      expression: false,
      gloss: null,
    });
    expect(h.gloss[0]!.lemma).toBe("seldom");
    h.gloss[0]!.resolve("rarement");
    expect(await pending).toBe("rarement");
  });

  it("gives null for a word the pack does not gloss", async () => {
    const h = harness();
    const pending = h.cards.cardGloss({
      lemma: "zorb",
      surface: "zorb",
      sentence: "s",
      status: "learning",
      expression: false,
      gloss: null,
    });
    h.gloss[0]!.resolve(undefined);
    expect(await pending).toBeNull();
  });

  it("never asks the pack for an expression, whatever it holds", async () => {
    const h = harness();
    expect(
      await h.cards.cardGloss({
        lemma: "well known",
        surface: "well known",
        sentence: "s",
        status: "learning",
        expression: true,
        gloss: null,
      }),
    ).toBeNull();
    expect(h.gloss).toHaveLength(0);
  });
});
