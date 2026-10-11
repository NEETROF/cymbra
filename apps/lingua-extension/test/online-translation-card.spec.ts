import { describe, expect, it, vi } from "vitest";
import type { PhraseGloss, PhraseToken, WordGrammar } from "@/analyzer/types.ts";
import { type Clock, type SelectionCardPorts, type SelectionInput, SelectionCards } from "@/reading/selection-card.ts";
import { createCard, type Gesture, type WordPopupContent } from "@/reading/wordpopup.ts";
import {
  MAX_SENTENCE_CHARS,
  type OnlineTranslationRequest,
  type OnlineTranslationResult,
  translateOnline,
} from "@/translate/online.ts";
import type { TranslationResult, TranslatorPort } from "@/translate/port.ts";
import type { MarkedTranslation } from "@/translate/markup.ts";

// The card scenarios of the lingua-translation delta (add-lingua-online-translation): the selection
// cards drive a REAL card view (createCard), so the button is pressed as the reader presses it, and
// what reaches the service is recorded by a fake `translateOnline` the test answers by hand.

const RECT = { left: 10, top: 20, bottom: 40 };
const SENTENCE = "He kept putting off the dentist.";
const SEL: SelectionInput = { text: "putting off", sentence: SENTENCE, selection: { start: 8, end: 19 }, rect: RECT };
const ONLINE: MarkedTranslation = {
  sentence: "Il remettait toujours le dentiste à plus tard.",
  marks: [
    { start: 3, end: 12 },
    { start: 34, end: 45 },
  ],
};
const ENGINE: MarkedTranslation = {
  sentence: "Il continuait à repousser le dentiste.",
  marks: [{ start: 16, end: 25 }],
};

const tok = (over: Partial<PhraseToken> & Pick<PhraseToken, "surface" | "lemma" | "class">): PhraseToken => ({
  gloss: null,
  function_word: false,
  ...over,
});
const ROWS_ANSWER: PhraseGloss = {
  tokens: [
    tok({ surface: "putting", lemma: "put", class: "Unknown", gloss: "Mettre" }),
    tok({ surface: "off", lemma: "off", class: "Unknown", gloss: "Éteint" }),
  ],
};

const flush = async (): Promise<void> => {
  for (let i = 0; i < 20; i++) await Promise.resolve();
};

interface Deferred<T> {
  resolve: (value: T) => void;
  reject: (reason: unknown) => void;
}

/** Timers that never fire on their own: a test that wants the bound elapses it by hand. */
const clock: Clock = { setTimeout: () => 0, clearTimeout: () => {} };

function setup(opts: { online?: boolean | (() => boolean); engine?: boolean; language?: "en" | "es" | "fr" } = {}) {
  const phraseGloss: Array<Deferred<PhraseGloss>> = [];
  const ports: SelectionCardPorts = {
    phraseGloss: () => new Promise((resolve, reject) => phraseGloss.push({ resolve, reject })),
    gloss: async () => undefined,
    wordGrammar: async (): Promise<WordGrammar> => ({
      gloss: null,
      senses: [],
      readings: [],
      others: [],
      pieces: [],
      rank: 3000,
    }),
  };
  const engineAsked: Array<Deferred<TranslationResult>> = [];
  const translator: TranslatorPort = {
    translate: () => new Promise((resolve, reject) => engineAsked.push({ resolve, reject })),
  };
  const sent: Array<Deferred<OnlineTranslationResult> & { request: OnlineTranslationRequest }> = [];
  const view = createCard();
  document.body.replaceChildren(view.el);
  const shows: WordPopupContent[] = [];
  const gestures: Gesture[] = [];
  const online = opts.online ?? true;
  const cards = new SelectionCards(
    ports,
    {
      show: (content) => {
        shows.push(content);
        return view.show(content, (g) => gestures.push(g));
      },
      generation: () => view.generation(),
    },
    {
      clock,
      translator: () => (opts.engine ? translator : null),
      ...(opts.language ? { language: () => opts.language! } : {}),
      online: typeof online === "function" ? online : () => online,
      translateOnline: (request) =>
        new Promise((resolve, reject) => {
          sent.push({ request, resolve, reject });
        }),
    },
  );
  const button = (): HTMLButtonElement | null => view.el.querySelector<HTMLButtonElement>(".online button");
  const press = (): void => button()!.click();
  const note = (): string => view.el.querySelector<HTMLElement>(".online-note")?.textContent ?? "";
  const shown = (): string => view.el.querySelector(".translation-sentence")?.textContent ?? "";
  const label = (): string => view.el.querySelector(".translation-label")?.textContent ?? "";
  return { cards, view, phraseGloss, engineAsked, sent, shows, gestures, button, press, note, shown, label };
}

/** Open the card for « putting off » and let the pack answer with word-by-word rows. */
async function openPhrase(s: ReturnType<typeof setup>, sel: SelectionInput = SEL): Promise<void> {
  s.cards.openForSelection(sel, null);
  await flush();
  s.phraseGloss[0]!.resolve(ROWS_ANSWER);
  await flush();
}

describe("The online translation button appears only when chosen", () => {
  it("Setting ticked, a phrase selected: the pack's answer, then the engine's translation, and the button", async () => {
    const s = setup({ engine: true });
    await openPhrase(s);
    expect(s.view.el.textContent).toContain("Mot à mot");
    expect(s.button()?.textContent).toBe("Traduction en ligne");
    expect(s.button()?.disabled).toBe(false);
    s.engineAsked[0]!.resolve({ kind: "translated", translation: ENGINE });
    await flush();
    expect(s.shown()).toBe(ENGINE.sentence);
    expect(s.button()?.textContent).toBe("Traduction en ligne");
  });

  it("Setting not ticked: no button, and nothing leaves the device", async () => {
    const s = setup({ online: false });
    await openPhrase(s);
    expect(s.button()).toBeNull();
    expect((s.view.el.querySelector(".online") as HTMLElement).hidden).toBe(true);
    expect(s.sent).toEqual([]);
  });

  it("No access (the flag off, or signed out): the same answer as unticked — no button", async () => {
    // The offer is one boolean: the setting ticked AND the account's access (translate/online.ts).
    const s = setup({ online: () => false });
    await openPhrase(s);
    expect(s.button()).toBeNull();
  });

  it("No model on the device: the pack's answer and the button", async () => {
    const s = setup({ engine: false });
    await openPhrase(s);
    expect(s.view.el.textContent).toContain("Mot à mot");
    expect(s.button()).not.toBeNull();
  });

  it("A single word: the word's dictionary card, without the button", async () => {
    const s = setup();
    s.cards.openForSelection(
      { text: "dentist", sentence: SENTENCE, selection: { start: 24, end: 31 }, rect: RECT },
      null,
    );
    await flush();
    s.phraseGloss[0]?.resolve({
      tokens: [tok({ surface: "dentist", lemma: "dentist", class: "Unknown", gloss: "Dentiste" })],
    });
    await flush();
    expect(s.shows.length).toBeGreaterThan(0);
    expect(s.shows.every((c) => !c.online)).toBe(true);
    expect(s.button()).toBeNull();
  });

  it("The pending card offers nothing to press yet", async () => {
    const s = setup();
    s.cards.openForSelection(SEL, null);
    await flush();
    expect(s.shows.at(-1)!.pending).toBe(true);
    expect(s.button()).toBeNull();
  });

  it("asks per card: ticked mid-page, the next card has the button", async () => {
    let offered = false;
    const s = setup({ online: () => offered });
    await openPhrase(s);
    expect(s.button()).toBeNull();
    offered = true;
    s.cards.openForSelection(SEL, null);
    await flush();
    s.phraseGloss[1]!.resolve(ROWS_ANSWER);
    await flush();
    expect(s.button()).not.toBeNull();
  });

  it("finds the selection in its sentence when the page did not place it, and offers nothing when it cannot", async () => {
    const s = setup();
    await openPhrase(s, { ...SEL, selection: undefined });
    s.press();
    expect(s.sent[0]!.request.selection).toEqual({ start: 8, end: 19 });

    const lost = setup();
    await openPhrase(lost, { ...SEL, text: "gave up", selection: null });
    expect(lost.button()).toBeNull();
  });
});

describe("An online translation is shown in place of the engine's, as a machine translation", () => {
  it("sends the sentence once, from the document's language, on the press only", async () => {
    const s = setup({ language: "es" });
    await openPhrase(s);
    expect(s.sent).toEqual([]);
    s.press();
    expect(s.sent.map((x) => x.request)).toEqual([
      { sentence: SENTENCE, selection: { start: 8, end: 19 }, source: "es" },
    ]);
  });

  it("While the answer is on its way: says so, keeps what it shows, and the button is disabled", async () => {
    const s = setup({ engine: true });
    await openPhrase(s);
    s.engineAsked[0]!.resolve({ kind: "translated", translation: ENGINE });
    await flush();
    s.press();
    await flush();
    expect(s.note()).toBe("Traduction en ligne…");
    expect(s.shown()).toBe(ENGINE.sentence);
    expect(s.button()?.disabled).toBe(true);
  });

  it("The answer lands: the online translation under its label, the selection marked, in place of the engine's", async () => {
    const s = setup({ engine: true });
    await openPhrase(s);
    s.engineAsked[0]!.resolve({ kind: "translated", translation: ENGINE });
    await flush();
    s.press();
    s.sent[0]!.resolve({ kind: "translated", translation: ONLINE });
    await flush();
    expect(s.shown()).toBe(ONLINE.sentence);
    expect([...s.view.el.querySelectorAll(".translation-sentence mark")].map((m) => m.textContent)).toEqual([
      "remettait",
      "à plus tard",
    ]);
    expect(s.label()).toBe("Dans votre phrase — traduction automatique en ligne");
    expect(s.label()).not.toMatch(/Google|Gemini|Anthropic|Claude/);
    expect(s.note()).toBe("");
    expect(s.button()?.disabled).toBe(true);
  });

  it("An answer before the engine's: the engine's, when it lands, does not replace it", async () => {
    const s = setup({ engine: true });
    await openPhrase(s);
    s.press();
    s.sent[0]!.resolve({ kind: "translated", translation: ONLINE });
    await flush();
    expect(s.view.el.textContent).not.toContain("Mot à mot");
    s.engineAsked[0]!.resolve({ kind: "translated", translation: ENGINE });
    await flush();
    expect(s.shown()).toBe(ONLINE.sentence);
    expect(s.shows.at(-1)!.translating).toBe(false);
  });

  it("The reader moved on — closed the card, or opened another: nothing is shown in its place", async () => {
    const closed = setup();
    await openPhrase(closed);
    closed.press();
    closed.view.hide();
    const before = closed.shows.length;
    closed.sent[0]!.resolve({ kind: "translated", translation: ONLINE });
    await flush();
    expect(closed.shows.length).toBe(before);
    expect(closed.view.visible()).toBe(false);

    const another = setup();
    await openPhrase(another);
    another.press();
    another.cards.openForSelection({ ...SEL, text: "the dentist", selection: { start: 20, end: 31 } }, null);
    await flush();
    another.phraseGloss[1]!.resolve(ROWS_ANSWER);
    await flush();
    another.sent[0]!.resolve({ kind: "translated", translation: ONLINE });
    await flush();
    expect(another.shown()).toBe("");
    expect(another.shows.at(-1)!.headword).toBe("the dentist");
  });

  it("A card the reader acted on is not written over", async () => {
    const s = setup();
    await openPhrase(s);
    s.press();
    const deck = [...s.view.el.querySelectorAll<HTMLButtonElement>(".actions button")].find(
      (b) => b.textContent === "+ Deck",
    )!;
    deck.click();
    const before = s.shows.length;
    s.sent[0]!.resolve({ kind: "translated", translation: ONLINE });
    await flush();
    expect(s.shows.length).toBe(before);
    // Only dictionary data reaches the card: no gesture carries the online translation.
    expect(JSON.stringify(s.gestures)).not.toContain(ONLINE.sentence);
  });

  it("A card made after an online translation holds no translation", async () => {
    const s = setup();
    await openPhrase(s);
    s.press();
    s.sent[0]!.resolve({ kind: "translated", translation: ONLINE });
    await flush();
    [...s.view.el.querySelectorAll<HTMLButtonElement>(".actions button")]
      .find((b) => b.textContent === "+ Deck")!
      .click();
    expect(s.gestures).toHaveLength(1);
    expect(JSON.stringify(s.gestures[0])).not.toContain("remettait");
    expect(s.gestures[0]!.sentence).toBe(SENTENCE);
  });

  it("A press on a card already replaced sends nothing", async () => {
    const s = setup();
    await openPhrase(s);
    const offer = s.shows.at(-1)!.online!;
    s.view.hide();
    offer.press();
    expect(s.sent).toEqual([]);
  });

  it("Markup in the answer: shown as text, no element created from it", async () => {
    const s = setup();
    await openPhrase(s);
    s.press();
    const sentence = 'Il <img src=x onerror="alert(1)"> <b>remettait</b> &amp; plus.';
    const at = sentence.indexOf("<b>");
    s.sent[0]!.resolve({ kind: "translated", translation: { sentence, marks: [{ start: at, end: at + 19 }] } });
    await flush();
    const shown = s.view.el.querySelector(".translation-sentence")!;
    expect(shown.textContent).toBe(sentence);
    expect(shown.querySelector("img, b")).toBeNull();
    expect(shown.querySelectorAll("mark")).toHaveLength(1);
  });

  it("Pressing twice on the same card sends once", async () => {
    const s = setup();
    await openPhrase(s);
    const offer = s.shows.at(-1)!.online!;
    s.press();
    s.press();
    offer.press(); // the offer the card was first shown with, pressed again
    s.shows.at(-1)!.online!.press();
    expect(s.sent).toHaveLength(1);
  });
});

describe("An unavailable online translation costs the reader nothing", () => {
  it("No provider answers: the card keeps the engine's translation and says online translation is unavailable", async () => {
    const s = setup({ engine: true });
    await openPhrase(s);
    s.engineAsked[0]!.resolve({ kind: "translated", translation: ENGINE });
    await flush();
    s.press();
    s.sent[0]!.resolve({ kind: "unavailable" });
    await flush();
    expect(s.shown()).toBe(ENGINE.sentence);
    expect(s.label()).toBe("Dans votre phrase — traduction automatique");
    expect(s.note()).toBe("Traduction en ligne indisponible pour le moment.");
    expect(s.button()?.disabled).toBe(true);
  });

  it("No engine either: the pack's answer stays, with the message — and a send that fails is the same", async () => {
    const s = setup();
    await openPhrase(s);
    s.press();
    s.sent[0]!.reject(new Error("Receiving end does not exist."));
    await flush();
    expect(s.view.el.textContent).toContain("Mot à mot");
    expect(s.note()).toBe("Traduction en ligne indisponible pour le moment.");
  });

  it("The daily limit is reached: says so, and when it resets, in the interface language", async () => {
    const s = setup();
    await openPhrase(s);
    s.press();
    const resetsAt = new Date(2026, 9, 12, 14, 5).getTime();
    s.sent[0]!.resolve({ kind: "limit", resetsAt });
    await flush();
    expect(s.note()).toBe("Limite quotidienne atteinte — de nouveau disponible à 14:05.");
    expect(s.view.el.textContent).toContain("Mot à mot");
  });

  it("A 1 001-character sentence: unavailable, and no message reaches the background", async () => {
    const send = vi.fn(async () => ({ kind: "translated" }));
    const view = createCard();
    const long = `${"a".repeat(MAX_SENTENCE_CHARS - 11)} putting off`; // 1 001 characters
    expect(long.length).toBe(MAX_SENTENCE_CHARS + 1);
    const phrase: PhraseGloss = ROWS_ANSWER;
    const cards = new SelectionCards(
      {
        phraseGloss: async () => phrase,
        gloss: async () => undefined,
        wordGrammar: async () => ({ gloss: null, senses: [], readings: [], others: [], pieces: [], rank: null }),
      },
      { show: (c) => view.show(c, () => {}), generation: () => view.generation() },
      // The real page-side function, with its message seam recording what would leave the page.
      { clock, online: () => true, translateOnline: (request) => translateOnline(request, send) },
    );
    cards.openForSelection(
      { text: "putting off", sentence: long, selection: { start: 990, end: 1001 }, rect: RECT },
      null,
    );
    await flush();
    view.el.querySelector<HTMLButtonElement>(".online button")!.click();
    await flush();
    expect(send).not.toHaveBeenCalled();
    expect(view.el.querySelector(".online-note")?.textContent).toBe("Traduction en ligne indisponible pour le moment.");
  });
});

describe("the card's copy follows the interface language", () => {
  it("An English reader: the button, the waiting line, the label and the messages in English", async () => {
    const view = createCard(undefined, "en");
    const press = vi.fn();
    const base: WordPopupContent = {
      headword: "putting off",
      surface: "putting off",
      gloss: null,
      rarity: "",
      sentence: SENTENCE,
      rect: RECT,
      expression: true,
    };
    view.show({ ...base, online: { state: "offered", press } }, () => {});
    const button = view.el.querySelector<HTMLButtonElement>(".online button")!;
    expect(button.textContent).toBe("Online translation");
    button.click();
    button.click();
    expect(press).toHaveBeenCalledTimes(1);
    view.show({ ...base, online: { state: "asking", press } }, () => {});
    expect(view.el.querySelector(".online-note")?.textContent).toBe("Online translation…");
    view.show(
      { ...base, translation: ONLINE, onlineTranslation: true, online: { state: "answered", press } },
      () => {},
    );
    expect(view.el.querySelector(".translation-label")?.textContent).toBe(
      "In your sentence — online machine translation",
    );
    view.show(
      { ...base, online: { state: "limit", resetsAt: new Date(2026, 9, 12, 14, 5).getTime(), press } },
      () => {},
    );
    expect(view.el.querySelector(".online-note")?.textContent).toBe(
      "Daily limit reached — available again at 2:05 PM.",
    );
  });

  it("A Spanish reader: « Traducción en línea »", () => {
    const view = createCard(undefined, "es");
    view.show(
      {
        headword: "x y",
        surface: "x y",
        gloss: null,
        rarity: "",
        sentence: "x y",
        rect: RECT,
        online: { state: "unavailable", press: () => {} },
      },
      () => {},
    );
    expect(view.el.querySelector(".online button")?.textContent).toBe("Traducción en línea");
    expect(view.el.querySelector(".online-note")?.textContent).toBe("Traducción en línea no disponible por ahora.");
  });

  it("A pressed button keeps the reader's selection on the page", () => {
    const view = createCard();
    view.show(
      {
        headword: "x y",
        surface: "x y",
        gloss: null,
        rarity: "",
        sentence: "x y",
        rect: RECT,
        online: { state: "offered", press: () => {} },
      },
      () => {},
    );
    const button = view.el.querySelector<HTMLButtonElement>(".online button")!;
    for (const type of ["pointerdown", "mousedown"]) {
      const e = new Event(type, { cancelable: true });
      button.dispatchEvent(e);
      expect(e.defaultPrevented).toBe(true);
    }
  });
});
