import { beforeEach, describe, expect, it, vi } from "vitest";
import { createSpeaker, type VoiceInfo } from "@/reading/speech.ts";
import type { ReviewView } from "@/review/session.ts";
import { markWord, type ReviewActions, type ReviewOnlineOffer, renderReview, sourceLabel } from "@/review/view.ts";
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

describe("renderReview — a card a miss brought back (add-lingua-review-repeat-hint)", () => {
  it("says the count of cards remaining on a first showing", () => {
    renderReview(root, reviewing(), actions);
    expect(root.querySelector(".remaining")?.textContent).toBe("3 carte(s) à revoir");
    expect(root.querySelector(".review-repeat")).toBeNull();
    renderReview(root, { ...reviewing(), showing: 1 }, actions);
    expect(root.querySelector(".review-repeat")).toBeNull();
  });

  it("says its second and third showing in its place", () => {
    renderReview(root, { ...reviewing(), showing: 2 }, actions);
    expect(root.querySelector(".remaining")?.textContent).toBe("À revoir · 2ᵉ passage");
    expect(root.querySelectorAll(".remaining")).toHaveLength(1);
    renderReview(root, { ...reviewing({ revealed: true }), showing: 3 }, actions);
    expect(root.querySelector(".review-repeat")?.textContent).toBe("À revoir · 3ᵉ passage");
  });

  it("speaks the interface language", () => {
    renderReview(root, { ...reviewing(), showing: 2 }, actions, { interfaceLanguage: "en" });
    expect(root.querySelector(".review-repeat")?.textContent).toBe("Again · 2nd time");
    renderReview(root, { ...reviewing(), showing: 3 }, actions, { interfaceLanguage: "en" });
    expect(root.querySelector(".review-repeat")?.textContent).toBe("Again · 3rd time");
    renderReview(root, { ...reviewing(), showing: 4 }, actions, { interfaceLanguage: "en" });
    expect(root.querySelector(".review-repeat")?.textContent).toBe("Again · 4th time");
    renderReview(root, { ...reviewing(), showing: 2 }, actions, { interfaceLanguage: "es" });
    expect(root.querySelector(".review-repeat")?.textContent).toBe("Otra vez · 2.ª vez");
  });

  it("wins over practice's note: it says why the card is there again", () => {
    renderReview(root, { ...reviewing(), practice: true, showing: 2 }, actions);
    expect(root.querySelector(".remaining")?.textContent).toBe("À revoir · 2ᵉ passage");
    expect(root.querySelector(".review-practice-note")).toBeNull();
  });
});

describe("renderReview — practice (add-lingua-review-practice)", () => {
  it("offers practice where nothing is due, when the host can start it", () => {
    const practice = vi.fn();
    const view: ReviewView = { ...done(null), practiceAvailable: true };
    renderReview(root, view, { ...actions, practice });
    expect(root.textContent).toContain("Rien à réviser pour l'instant.");
    const button = root.querySelector<HTMLButtonElement>(".review-practice");
    expect(button?.textContent).toBe("S'entraîner quand même");
    button?.click();
    expect(practice).toHaveBeenCalledOnce();
    // A host without the action, or nothing to practise: no button.
    renderReview(root, view, actions);
    expect(root.querySelector(".review-practice")).toBeNull();
    renderReview(root, done(null), { ...actions, practice });
    expect(root.querySelector(".review-practice")).toBeNull();
  });

  it("offers « Encore 10 » rather than practice while cards are due", () => {
    renderReview(root, { ...done(null, true), practiceAvailable: true }, { ...actions, practice: vi.fn() });
    expect(root.querySelector(".review-more")).not.toBeNull();
    expect(root.querySelector(".review-practice")).toBeNull();
  });

  it("says a practice card is practice, in place of the count of cards remaining", () => {
    renderReview(root, { ...reviewing(), practice: true }, actions);
    expect(root.querySelector(".remaining")?.textContent).toBe("Entraînement · 3 carte(s)");
    renderReview(root, { ...reviewing({ remaining: 1 }), practice: true }, actions, { interfaceLanguage: "en" });
    expect(root.querySelector(".review-practice-note")?.textContent).toBe("Practice · 1 card");
    renderReview(root, reviewing(), actions);
    expect(root.querySelector(".review-practice-note")).toBeNull();
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

// fix-lingua-selection-card-punctuation D4: the front reads a card's word without its edge
// punctuation, and never rewrites the card.
describe("renderReview — a card's word without its edge punctuation", () => {
  const comma = {
    headword: "four-poster bed,",
    surface: "four-poster bed,",
    sentence: "He slept in a four-poster bed, alone.",
    gloss: "Lit à baldaquin",
  };

  it("scenario: a card captured with its comma — marked without it, no dictionary form beside it", () => {
    const view = reviewing({ ...comma, revealed: true });
    renderReview(root, view, actions);
    expect(root.querySelector("mark.review-word")!.textContent).toBe("four-poster bed");
    expect(root.querySelector(".review-sentence")!.textContent).toBe("He slept in a four-poster bed, alone.");
    expect(root.querySelector(".review-dictionary-form")).toBeNull();
    expect(root.querySelector(".review-gloss")!.textContent).toBe("Lit à baldaquin");
    expect(view.card!.headword).toBe("four-poster bed,"); // the card keeps its stored headword
  });

  it("heads a card whose sentence does not hold it by the trimmed headword, and hears it so", () => {
    const fake = makeFakeSpeech([
      { name: "Samantha", lang: "en-US", localService: true, default: false, voiceURI: "Samantha" },
    ]);
    const speaker = createSpeaker(fake.engine, "en", fake.preference);
    renderReview(root, reviewing({ ...comma, sentence: "" }), actions, { speaker });
    expect(root.querySelector(".review-headword")!.textContent).toBe("four-poster bed");
    button("▶ Expression").click();
    expect(fake.spoken[0]?.text).toBe("four-poster bed");
  });

  it("tests the trimmed headword for the no-translation note", () => {
    renderReview(root, reviewing({ headword: "harbour,", surface: "harbour,", gloss: null, revealed: true }), actions);
    expect(root.querySelector(".review-no-gloss")?.textContent).toBe("Pas de traduction pour ce mot.");
    expect(root.querySelector("mark")).toBeNull();
    expect(root.querySelector(".review-headword")!.textContent).toBe("harbour");
  });

  it("compares the dictionary form with the form met once both are trimmed", () => {
    const grin = { headword: "grin.", surface: "grinning,", sentence: "He couldn't stop grinning, then left." };
    renderReview(root, reviewing({ ...grin, revealed: true }), actions);
    expect(root.querySelector("mark.review-word")!.textContent).toBe("grinning");
    expect(root.querySelector(".review-dictionary-form")!.textContent).toBe("grin");
  });

  it("shows a word of punctuation alone as it is stored", () => {
    renderReview(root, reviewing({ headword: "—", surface: "—", sentence: "" }), actions);
    expect(root.querySelector(".review-headword")!.textContent).toBe("—");
  });

  it("scenario: a card without punctuation — the front is as before", () => {
    renderReview(root, reviewing({ revealed: true }), actions);
    expect(root.querySelector("mark.review-word")!.textContent).toBe("seldom");
    expect(root.querySelector(".review-dictionary-form")).toBeNull();
    expect(root.querySelector(".review-gloss")!.textContent).toBe("rarement");
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

describe("renderReview — a card translated at review (add-lingua-review-translation)", () => {
  const expression = {
    headword: "such as those made",
    surface: "such as those made",
    sentence: "Rules such as those made here hold.",
    gloss: null,
  };
  const translated = (sentence: string, marks: { start: number; end: number }[], alone = false) => ({
    state: "translated" as const,
    translation: { sentence, marks },
    alone,
  });

  it("An expression in review: nothing of it before the reveal, the machine translation after", () => {
    const translation = translated("Des règles comme celles faites ici tiennent.", [{ start: 11, end: 34 }]);
    renderReview(root, reviewing(expression), actions, { translation });
    expect(root.querySelector(".review-translation")).toBeNull();
    expect(root.textContent).not.toContain("Des règles");

    renderReview(root, reviewing({ ...expression, revealed: true }), actions, { translation });

    expect(root.querySelector(".review-translation-label")?.textContent).toBe(
      "Dans votre phrase — traduction automatique",
    );
    expect(root.querySelector(".review-translation mark")?.textContent).toBe("comme celles faites ici");
    expect(root.querySelector(".review-translation-sentence")?.textContent).toBe(
      "Des règles comme celles faites ici tiennent.",
    );
    expect(root.querySelector(".review-no-gloss")).toBeNull();
  });

  it("Ready at the reveal: the translation is in the answer's space, nothing added after", () => {
    renderReview(root, reviewing({ ...expression, revealed: true }), actions, {
      translation: translated("Des règles.", []),
    });
    const answer = root.querySelector(".review-answer")!;
    expect(answer.children).toHaveLength(1);
    expect(answer.querySelector(".review-translation")).not.toBeNull();
    expect(root.querySelector(".review-translating")).toBeNull();
  });

  it("Still on its way: says a translation is coming, in place of the no-translation note", () => {
    renderReview(root, reviewing({ ...expression, revealed: true }), actions, { translation: { state: "pending" } });
    expect(root.querySelector(".review-translating")?.textContent).toBe("Traduction en cours…");
    expect(root.querySelector(".review-no-gloss")).toBeNull();
  });

  it("none came: the no-translation note, as before the engine existed", () => {
    renderReview(root, reviewing({ ...expression, revealed: true }), actions, { translation: { state: "none" } });
    expect(root.querySelector(".review-no-gloss")?.textContent).toBe("Pas de traduction pour cette expression.");
    expect(root.querySelector(".review-translation, .review-translating")).toBeNull();
  });

  it("A sentence that does not hold the expression: labelled as a machine translation of it", () => {
    renderReview(root, reviewing({ ...expression, sentence: "Other words.", revealed: true }), actions, {
      translation: translated("comme celles faites", [], true),
    });
    expect(root.querySelector(".review-translation-label")?.textContent).toBe("Traduction automatique");
  });

  it("shows a translation holding markup as text, and skips a mark out of range", () => {
    renderReview(root, reviewing({ ...expression, revealed: true }), actions, {
      translation: translated("<img src=x onerror=alert(1)> ok", [
        { start: 0, end: 4 },
        { start: 30, end: 90 },
      ]),
    });
    const sentence = root.querySelector(".review-translation-sentence")!;
    expect(sentence.querySelector("img")).toBeNull();
    expect(sentence.textContent).toBe("<img src=x onerror=alert(1)> ok");
    expect(sentence.querySelectorAll("mark")).toHaveLength(1);
  });

  it("speaks the interface language: English labels for an English interface", () => {
    renderReview(root, reviewing({ ...expression, revealed: true }), actions, {
      interfaceLanguage: "en",
      translation: { state: "pending" },
    });
    expect(root.querySelector(".review-translating")?.textContent).toBe("Translating…");
  });
});

// add-lingua-review-translation-fallback D1–D3, D10: the link, the online button and keeping.
describe("renderReview — keeping a translation, the link and the online button", () => {
  const slacken = {
    headword: "slacken",
    surface: "slacken",
    sentence: "He felt his grip slacken.",
    gloss: null,
    revealed: true,
  };
  const translated = (sentence: string, marks: { start: number; end: number }[], alone = false) => ({
    state: "translated" as const,
    translation: { sentence, marks },
    alone,
  });
  const SE_RELACHER = translated("Il sentit sa prise se relâcher.", [{ start: 19, end: 30 }]);
  const field = (): HTMLInputElement => root.querySelector<HTMLInputElement>(".review-keep-input")!;
  const type = (value: string): void => {
    field().value = value;
    field().dispatchEvent(new Event("input"));
  };

  it("One tap: names the marked word and keeps it, « Modifier » beside it", () => {
    const keep = vi.fn();
    renderReview(root, reviewing(slacken), actions, { translation: SE_RELACHER, keep });
    expect(button("Modifier")).toBeTruthy();
    button("Garder « se relâcher »").click();
    expect(keep).toHaveBeenCalledWith("se relâcher");
  });

  // (marks, alone, the action's label) — a candidate named, or the field's path.
  const CANDIDATES: [string, { start: number; end: number }[], boolean, string][] = [
    [
      "Il remettait toujours le travail à plus tard.",
      [
        { start: 3, end: 12 },
        { start: 33, end: 44 },
      ],
      false,
      "Garder une traduction…",
    ],
    ["comme celles faites", [], true, "Garder « comme celles faites »"],
    ["Il sentit sa prise se relâcher.", [], false, "Garder une traduction…"],
    ["Il sentit sa prise se relâcher.", [{ start: 40, end: 50 }], false, "Garder une traduction…"],
    [`Il dit ${"très ".repeat(20)}fort.`, [{ start: 7, end: 107 }], false, "Garder une traduction…"],
  ];

  it("names a one-mark candidate or an expression alone whole; several marks, none or too long: the field", () => {
    for (const [sentence, marks, alone, label] of CANDIDATES) {
      renderReview(root, reviewing(slacken), actions, {
        translation: translated(sentence, marks, alone),
        keep: vi.fn(),
      });
      expect(button(label), sentence).toBeTruthy();
    }
  });

  it("No mark: an empty field, « Enregistrer » disabled until a translation is typed, then kept", () => {
    const keep = vi.fn();
    renderReview(root, reviewing(slacken), actions, {
      translation: translated("Il sentit sa prise faiblir.", []),
      keep,
    });
    button("Garder une traduction…").click();
    expect(field().value).toBe("");
    for (const [value, enabled] of [
      ["", false],
      ["   ", false],
      ["x".repeat(81), false],
      [" faiblir ", true],
    ] as const) {
      type(value);
      expect(button("Enregistrer").disabled, JSON.stringify(value)).toBe(!enabled);
    }
    button("Enregistrer").click();
    expect(keep).toHaveBeenCalledWith("faiblir");
  });

  // (translation, its marks, the field prefilled with) — several marks never kept in one tap.
  const SPLIT: [string, { start: number; end: number }[], string][] = [
    [
      "Il remettait toujours le travail à plus tard.",
      [
        { start: 3, end: 12 },
        { start: 33, end: 44 },
      ],
      "remettait … à plus tard",
    ],
    // The iPhone spike's answer for « colloquially referred to as »: a non-word and a spurious second mark.
    [
      "Ce phénomène est facilitamment appelé une ruée vers le sucre",
      [
        { start: 17, end: 37 },
        { start: 47, end: 51 },
      ],
      "facilitamment appelé … vers",
    ],
  ];

  it("A split mark: never one tap — « Garder une traduction… » opens the field prefilled, a correction is saved", () => {
    for (const [sentence, marks, prefilled] of SPLIT) {
      const keep = vi.fn();
      renderReview(root, reviewing(slacken), actions, { translation: translated(sentence, marks), keep });
      expect(root.querySelector(".review-keep-btn"), sentence).toBeNull();
      expect(root.querySelector(".review-keep-edit"), sentence).toBeNull();
      button("Garder une traduction…").click();
      expect(field().value).toBe(prefilled);
      button("Annuler").click();
      button("Garder une traduction…").click();
      type("remettre à plus tard");
      button("Enregistrer").click();
      expect(keep).toHaveBeenCalledWith("remettre à plus tard");
    }
  });

  it("One mark: « Modifier » opens the field prefilled, « Annuler » comes back", () => {
    renderReview(root, reviewing(slacken), actions, { translation: SE_RELACHER, keep: vi.fn() });
    button("Modifier").click();
    expect(field().value).toBe("se relâcher");
    button("Annuler").click();
    expect(button("Garder « se relâcher »")).toBeTruthy();
  });

  it("typing in the field never answers the card", () => {
    renderReview(root, reviewing(slacken), actions, { translation: SE_RELACHER, keep: vi.fn() });
    button("Modifier").click();
    key(field(), "ArrowLeft");
    key(field(), "ArrowRight");
    expect(actions.grade).not.toHaveBeenCalled();
  });

  it("shows a translation holding markup as text, its candidate too", () => {
    renderReview(root, reviewing(slacken), actions, {
      translation: translated("Il <b>se</b> relâche.", [{ start: 3, end: 12 }]),
      keep: vi.fn(),
    });
    expect(root.querySelector(".review-translation b")).toBeNull();
    expect(root.querySelector(".review-translation mark")?.textContent).toBe("<b>se</b>");
    expect(button("Garder « <b>se</b> »")).toBeTruthy();
  });

  it("offers no keep action without a translation shown, nor without the host's keep", () => {
    renderReview(root, reviewing(slacken), actions, { translation: { state: "pending" }, keep: vi.fn() });
    expect(root.querySelector(".review-keep")).toBeNull();
    renderReview(root, reviewing(slacken), actions, { translation: SE_RELACHER });
    expect(root.querySelector(".review-keep")).toBeNull();
  });

  it("A word the pack glosses: the link beside the gloss, the asked translation under it", () => {
    const ask = vi.fn();
    renderReview(root, reviewing({ revealed: true }), actions, { askTranslation: ask });
    expect(root.querySelector(".review-gloss")?.textContent).toBe("rarement");
    button("Traduire la phrase").click();
    expect(ask).toHaveBeenCalledOnce();

    renderReview(root, reviewing({ revealed: true }), actions, { translation: { state: "pending" } });
    expect(root.querySelector(".review-translating")?.textContent).toBe("Traduction en cours…");
    renderReview(root, reviewing({ revealed: true }), actions, { translation: SE_RELACHER });
    expect(root.querySelector(".review-gloss")?.textContent).toBe("rarement");
    expect(root.querySelector(".review-translation-label")?.textContent).toBe(
      "Dans votre phrase — traduction automatique",
    );
    renderReview(root, reviewing({ revealed: true }), actions, { translation: { state: "none" } });
    expect(root.querySelector(".review-translation, .review-translating, .review-no-gloss")).toBeNull();
  });

  it("No model: no link, as before", () => {
    renderReview(root, reviewing({ revealed: true }), actions);
    expect(root.querySelector(".review-ask-translation")).toBeNull();
  });

  // (state, pressable, the note)
  const ONLINE: [ReviewOnlineOffer["state"], boolean, string][] = [
    ["offered", true, ""],
    ["asking", false, "Traduction en ligne…"],
    ["answered", false, ""],
    ["unavailable", false, "Traduction en ligne indisponible pour le moment."],
    ["limit", false, "Limite quotidienne atteinte — de nouveau disponible à"],
  ];

  it("the online button: pressable only while offered, its note as the selection card's", () => {
    for (const [state, pressable, note] of ONLINE) {
      const press = vi.fn();
      renderReview(root, reviewing(slacken), actions, { online: { state, resetsAt: Date.now(), press } });
      const b = button("Traduction en ligne");
      expect(b.disabled, state).toBe(!pressable);
      expect(root.querySelector(".review-online-note")?.textContent ?? "").toContain(note);
      b.click();
      b.click();
      expect(press, state).toHaveBeenCalledTimes(pressable ? 1 : 0);
    }
  });

  it("labels an online translation as one", () => {
    for (const [alone, label] of [
      [false, "Dans votre phrase — traduction automatique en ligne"],
      [true, "Traduction automatique en ligne"],
    ] as const) {
      renderReview(root, reviewing(slacken), actions, { translation: { ...translated("x", [], alone), online: true } });
      expect(root.querySelector(".review-translation-label")?.textContent).toBe(label);
    }
  });
});
