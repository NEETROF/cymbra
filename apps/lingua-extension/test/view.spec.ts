import { beforeEach, describe, expect, it, vi } from "vitest";
import { createSpeaker, type VoiceInfo } from "@/reading/speech.ts";
import type { ReviewView } from "@/review/session.ts";
import { markWord, type ReviewActions, renderReview, sourceLabel } from "@/review/view.ts";
import { makeFakeSpeech } from "./helpers.ts";

let root: HTMLElement;
let actions: ReviewActions;

beforeEach(() => {
  document.body.innerHTML = "<div id='r'></div><input id='page'>";
  root = document.getElementById("r")!;
  actions = { start: vi.fn(), reveal: vi.fn(), grade: vi.fn(), markKnown: vi.fn(), ignore: vi.fn() };
});

function button(label: string): HTMLButtonElement {
  const b = [...root.querySelectorAll("button")].find((el) => el.textContent === label);
  if (!b) throw new Error(`no button "${label}"`);
  return b as HTMLButtonElement;
}

const card = (over: Partial<NonNullable<ReviewView["card"]>> = {}) => ({
  headword: "seldom",
  surface: "seldom",
  sentence: "They seldom ship.",
  gloss: "rarement",
  revealed: false,
  remaining: 3,
  ...over,
});

const reviewing = (over: Partial<NonNullable<ReviewView["card"]>> = {}): ReviewView => ({
  phase: "reviewing",
  card: card(over),
  summary: null,
  moreDue: false,
});

const done = (summary: ReviewView["summary"], moreDue = false): ReviewView => ({
  phase: "done",
  card: null,
  summary,
  moreDue,
});

const key = (target: EventTarget, k: string): KeyboardEvent => {
  const e = new KeyboardEvent("keydown", { key: k, bubbles: true, cancelable: true });
  target.dispatchEvent(e);
  return e;
};

describe("renderReview — the card's language", () => {
  it("says the card's language for a reader of several languages", () => {
    renderReview(root, reviewing({ language: "es" }), actions, { showLanguage: true });
    expect(root.querySelector(".review-language")?.textContent).toBe("Espagnol");
  });

  it("says nothing of it for a reader of one language", () => {
    renderReview(root, reviewing({ language: "en" }), actions);
    expect(root.querySelector(".review-language")).toBeNull();
  });
});

describe("renderReview — the card (refine-lingua-review-session)", () => {
  it("idle shows a start button wired to start()", () => {
    renderReview(root, { phase: "idle", card: null, summary: null, moreDue: false }, actions);
    button("Réviser").click();
    expect(actions.start).toHaveBeenCalledOnce();
  });

  it("puts the sentence on the front with the word marked, the gloss hidden", () => {
    renderReview(root, reviewing(), actions);
    const sentence = root.querySelector(".review-sentence")!;
    expect(sentence.textContent).toBe("They seldom ship.");
    expect(sentence.querySelector("mark.review-word")!.textContent).toBe("seldom");
    expect(root.querySelector(".review-headword")).toBeNull(); // the sentence holds the word
    expect(root.textContent).not.toContain("rarement");
    button("Afficher la réponse").click();
    expect(actions.reveal).toHaveBeenCalledOnce();
  });

  it("reserves the answer's space before the reveal, and fills that space only", () => {
    renderReview(root, reviewing(), actions);
    const before = [...root.querySelectorAll("[class^='review-']")].map((e) => e.className);
    expect(root.querySelector(".review-answer")!.childElementCount).toBe(0);
    renderReview(root, reviewing({ revealed: true }), actions);
    const after = [...root.querySelectorAll("[class^='review-']")].map((e) => e.className);
    // Same blocks, in the same order, around the answer; only the action zone's buttons change.
    const frame = (names: string[]) => names.filter((n) => !/gloss|dictionary-form|btn/.test(n));
    expect(frame(after)).toEqual(frame(before));
    expect(root.querySelector(".review-answer .review-gloss")!.textContent).toBe("rarement");
  });

  it("scenario: a word met in a book shows the book, the sentence, and its gloss only once revealed", () => {
    const book = "The Hound of the Baskervilles · I: Mr. Sherlock Holmes";
    renderReview(root, reviewing({ source: book }), actions);
    expect(root.querySelector(".review-source")!.textContent).toBe(book);
    expect(root.querySelector(".review-gloss")).toBeNull();
    renderReview(root, reviewing({ source: "", revealed: true }), actions);
    expect(root.querySelector(".review-source")).toBeNull();
    expect(root.querySelector(".review-gloss")!.textContent).toBe("rarement");
  });

  it("scenario: a card without a sentence shows the word alone", () => {
    renderReview(root, reviewing({ sentence: "" }), actions);
    expect(root.querySelector(".review-headword")!.textContent).toBe("seldom");
    expect(root.querySelector(".review-sentence")).toBeNull();
  });

  it("marks the form met, and gives the dictionary form with the answer when it differs", () => {
    renderReview(
      root,
      reviewing({ headword: "grin", surface: "grinning", sentence: "He couldn't stop grinning." }),
      actions,
    );
    expect(root.querySelector("mark.review-word")!.textContent).toBe("grinning");
    renderReview(
      root,
      reviewing({ headword: "grin", surface: "grinning", sentence: "He couldn't stop grinning.", revealed: true }),
      actions,
    );
    expect(root.querySelector(".review-dictionary-form")!.textContent).toBe("grin");
  });

  it("shows the word above a sentence that does not hold it", () => {
    renderReview(root, reviewing({ sentence: "A sentence about something else." }), actions);
    expect(root.querySelector(".review-headword")!.textContent).toBe("seldom");
    expect(root.querySelector("mark")).toBeNull();
  });

  it("says so when a card has no gloss, an expression as an expression", () => {
    renderReview(root, reviewing({ gloss: null, revealed: true }), actions);
    expect(root.querySelector(".review-no-gloss")?.textContent).toBe("Pas de traduction pour ce mot.");
    renderReview(
      root,
      reviewing({ headword: "such as those made", surface: "such as those made", gloss: null, revealed: true }),
      actions,
    );
    expect(root.querySelector(".review-no-gloss")?.textContent).toBe("Pas de traduction pour cette expression.");
  });

  it("scenario: remembered and forgotten — two answers on a revealed card, left « Pas su », right « Su »", () => {
    renderReview(root, reviewing({ revealed: true }), actions);
    const answers = [...root.querySelectorAll(".review-actions button")].map((b) => b.textContent);
    expect(answers).toEqual(["Pas su", "Su"]);
    expect(root.textContent).not.toMatch(/Difficile|Facile|Correct/);
    button("Su").click();
    expect(actions.grade).toHaveBeenCalledWith("good");
    button("Pas su").click();
    expect(actions.grade).toHaveBeenCalledWith("again");
  });

  it("offers « Je connais » and « Ne plus me le montrer » on every card", () => {
    renderReview(root, reviewing(), actions);
    button("Je connais").click();
    expect(actions.markKnown).toHaveBeenCalledOnce();
    button("Ne plus me le montrer").click();
    expect(actions.ignore).toHaveBeenCalledOnce();
  });

  it("never parses a sentence or a gloss as markup", () => {
    const sentence = 'They <img src=x onerror="alert(1)"> seldom ship.';
    renderReview(root, reviewing({ sentence, gloss: "<b>rarement</b>", revealed: true }), actions);
    expect(root.querySelector("img")).toBeNull();
    expect(root.querySelector(".review-gloss b")).toBeNull();
    expect(root.querySelector(".review-sentence")!.textContent).toBe(sentence);
  });

  it("never renders the word 'lemma'/'lemme'", () => {
    renderReview(root, reviewing({ revealed: true }), actions);
    expect(root.textContent ?? "").not.toMatch(/lemm/i);
  });
});

describe("renderReview — the end of a session", () => {
  it("scenario: end of a session says what it did and offers more while cards remain", () => {
    renderReview(root, done({ reviewed: 10, recovered: 2, holding: 3, known: 0, hidden: 1 }, true), actions);
    expect(root.querySelector(".review-done-title")!.textContent).toBe("Séance terminée");
    const lines = [...root.querySelectorAll(".review-done-stats li")].map((li) => li.textContent);
    expect(lines).toEqual([
      "10 mots revus",
      "2 rattrapés en route",
      "3 tiennent maintenant plus d'un mois",
      "1 masqué",
    ]);
    button("Encore 10").click();
    expect(actions.start).toHaveBeenCalledOnce();
  });

  it("scenario: nothing left to review offers no « Encore 10 »", () => {
    renderReview(root, done({ reviewed: 1, recovered: 0, holding: 1, known: 0, hidden: 0 }), actions);
    expect([...root.querySelectorAll(".review-done-stats li")].map((li) => li.textContent)).toEqual([
      "1 mot revu",
      "1 tient maintenant plus d'un mois",
    ]);
    expect(root.querySelectorAll("button")).toHaveLength(0);
  });

  it("says there was nothing to review when the session held no card", () => {
    renderReview(root, done(null), actions);
    expect(root.textContent).toContain("Rien à réviser");
    expect(root.querySelectorAll("button")).toHaveLength(0);
  });
});

describe("renderReview — keys (refine-lingua-review-session D7)", () => {
  it("scenario: with a keyboard, Space reveals, then the right arrow answers « Su »", () => {
    renderReview(root, reviewing(), actions);
    expect(key(root, " ").defaultPrevented).toBe(true);
    expect(actions.reveal).toHaveBeenCalledOnce();
    renderReview(root, reviewing({ revealed: true }), actions);
    key(root, "ArrowRight");
    expect(actions.grade).toHaveBeenCalledWith("good");
    key(root, "ArrowLeft");
    expect(actions.grade).toHaveBeenCalledWith("again");
  });

  it("leaves a link's own Enter to the link", () => {
    renderReview(root, reviewing(), actions);
    key(button("Je connais"), "Enter");
    expect(actions.reveal).not.toHaveBeenCalled();
  });

  it("scenario: keys on the page being read act on the page, not on the review", () => {
    renderReview(root, reviewing(), actions);
    const page = document.getElementById("page")!;
    expect(key(page, " ").defaultPrevented).toBe(false);
    renderReview(root, reviewing({ revealed: true }), actions);
    key(page, "ArrowRight");
    expect(actions.reveal).not.toHaveBeenCalled();
    expect(actions.grade).not.toHaveBeenCalled();
  });

  it("keeps the focus in the review across a render", () => {
    renderReview(root, reviewing(), actions);
    root.focus();
    renderReview(root, reviewing({ revealed: true }), actions);
    expect(document.activeElement).toBe(root);
  });
});

describe("renderReview — read aloud (refine-lingua-review-session D11)", () => {
  const samantha: VoiceInfo = {
    name: "Samantha",
    lang: "en-US",
    localService: true,
    default: false,
    voiceURI: "Samantha",
  };

  function speaking(voices: VoiceInfo[] = [samantha]) {
    const fake = makeFakeSpeech(voices);
    return { fake, speaker: createSpeaker(fake.engine, "en", fake.preference) };
  }

  const labels = (scope: Element | null): string[] =>
    [...(scope?.querySelectorAll(".review-listen button") ?? [])].map((b) => b.textContent ?? "");
  const front = (): Element | null => root.querySelector(".review-main > .review-listen");
  const inAnswer = (): Element | null => root.querySelector(".review-answer");

  it("offers to hear the word and its sentence before the reveal, and the same after it", () => {
    const { speaker } = speaking();
    renderReview(root, reviewing(), actions, { speaker });
    expect(labels(front())).toEqual(["▶ Mot", "▶ Phrase"]);
    expect(labels(inAnswer())).toEqual([]);

    renderReview(root, reviewing({ revealed: true }), actions, { speaker });
    expect(labels(front())).toEqual(["▶ Mot", "▶ Phrase"]);
  });

  it("reads the word as the front shows it, and the sentence", () => {
    const { fake, speaker } = speaking();
    renderReview(root, reviewing({ surface: "Seldom", sentence: "Seldom do they ship." }), actions, { speaker });
    button("▶ Mot").click();
    expect(fake.spoken.map((u) => u.text)).toEqual(["Seldom"]);
    button("▶ Phrase").click();
    expect(fake.spoken.map((u) => u.text)).toEqual(["Seldom", "Seldom do they ship."]);
  });

  it("hears an expression as one, the one shown above a sentence that does not hold it", () => {
    const { fake, speaker } = speaking();
    const view = reviewing({
      headword: "such as those made",
      surface: "such as those made",
      sentence: "Its claws are adapted to killing prey such as mice.",
      gloss: null,
    });
    renderReview(root, view, actions, { speaker });
    expect(labels(front())).toEqual(["▶ Expression", "▶ Phrase"]);
    button("▶ Expression").click();
    expect(fake.spoken[0]?.text).toBe("such as those made");
  });

  it("once revealed, offers the dictionary form in the answer when the sentence shows another one", () => {
    const { fake, speaker } = speaking();
    const grinning = { headword: "grin", surface: "grinning", sentence: "He couldn't stop grinning." };
    renderReview(root, reviewing(grinning), actions, { speaker });
    expect(labels(inAnswer())).toEqual([]); // the dictionary form is part of the answer
    renderReview(root, reviewing({ ...grinning, revealed: true }), actions, { speaker });
    expect(labels(inAnswer())).toEqual(["▶ grin"]);
    button("▶ grin").click();
    expect(fake.spoken[0]?.text).toBe("grin");
  });

  it("turns the button being read into « ■ Arrêter », which stops it", () => {
    const { fake, speaker } = speaking();
    renderReview(root, reviewing(), actions, { speaker });
    button("▶ Phrase").click();
    expect(labels(front())).toEqual(["▶ Mot", "■ Arrêter"]);
    button("■ Arrêter").click();
    expect(speaker.speaking()).toBeNull();
    expect(fake.cancels()).toBeGreaterThan(0);
    expect(labels(front())).toEqual(["▶ Mot", "▶ Phrase"]);
  });

  it("keeps reading through the reveal, and stops when another card comes", () => {
    const { speaker } = speaking();
    renderReview(root, reviewing(), actions, { speaker });
    button("▶ Phrase").click();
    renderReview(root, reviewing({ revealed: true }), actions, { speaker });
    expect(speaker.speaking()?.text).toBe("They seldom ship.");

    renderReview(root, reviewing({ headword: "dwell", surface: "dwells", sentence: "It dwells here." }), actions, {
      speaker,
    });
    expect(speaker.speaking()).toBeNull();
  });

  it("offers nothing to hear without a speaker or without a voice, and adds it when voices come late", () => {
    renderReview(root, reviewing(), actions);
    expect(root.querySelector(".review-listen")).toBeNull();

    const { fake, speaker } = speaking([]);
    renderReview(root, reviewing(), actions, { speaker });
    expect((front() as HTMLElement).hidden).toBe(true);
    fake.list([samantha]);
    expect((front() as HTMLElement).hidden).toBe(false);
    expect(labels(front())).toEqual(["▶ Mot", "▶ Phrase"]);
  });

  it("leaves the speaker alone once the card it painted is gone", () => {
    const { fake, speaker } = speaking([]);
    renderReview(root, reviewing(), actions, { speaker });
    const stale = front() as HTMLElement;
    renderReview(root, done(null), actions, { speaker });
    fake.list([samantha]);
    expect(stale.hidden).toBe(true); // the old row was unsubscribed, not repainted
  });
});

describe("markWord", () => {
  it("finds the form as a whole word, letter case aside, or the dictionary form, or nothing", () => {
    expect(markWord("They Seldom ship.", "seldom")).toEqual({ before: "They ", text: "Seldom", after: " ship." });
    expect(markWord("He couldn't stop grinning.", "grinned", "grinning")?.text).toBe("grinning");
    expect(markWord("A seldomly used word.", "seldom")).toBeNull(); // part of another word
    expect(markWord("Él dijo «año» y se fue.", "año")?.text).toBe("año");
    expect(markWord("a compelling starting point here", "compelling starting point")?.before).toBe("a ");
    expect(markWord("What (if) anything?", "(if)")?.text).toBe("(if)"); // no regex from the word
    expect(markWord("Nothing.", "", "  ")).toBeNull();
  });
});

describe("sourceLabel", () => {
  it("names a page by its site, keeps a book's title and chapter, and nothing as nothing", () => {
    expect(sourceLabel("https://www.theguardian.com/world/2026/sep/01/x")).toBe("theguardian.com");
    expect(sourceLabel("http://example.org/")).toBe("example.org");
    expect(sourceLabel("Emma · Chapter 3")).toBe("Emma · Chapter 3");
    expect(sourceLabel("mailto:someone@example.org")).toBe("mailto:someone@example.org");
    expect(sourceLabel("")).toBe("");
    expect(sourceLabel(undefined)).toBe("");
  });
});
