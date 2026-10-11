import { beforeEach, describe, expect, it, vi } from "vitest";
import type { StudiedLanguage } from "@/analyzer/types.ts";
import { createCard, type Gesture, WordPopup, type WordPopupContent } from "@/reading/wordpopup.ts";
import { createSpeaker, type Speaker, type VoiceInfo } from "@/reading/speech.ts";
import { makeFakeSpeech } from "./helpers.ts";

// read-lingua-card-in-another-studied-language, the card view (tasks 3.1, 4.3): a card read in another
// studied language names that language in one line before its frequency line, in the interface
// language; its word and dictionary form are heard with that language's voice, its sentence with the
// document's; its gesture carries the language. A card without it is the card it was.

beforeEach(() => {
  document.body.innerHTML = "";
});

const voice = (name: string, lang: string): VoiceInfo => ({
  name,
  lang,
  localService: true,
  default: false,
  voiceURI: name,
});
const THOMAS = voice("Thomas", "fr-FR");
const MONICA = voice("Monica", "es-ES");

/** The Spanish card of « hablaba », met on a French page. */
const reread = (over: Partial<WordPopupContent> = {}): WordPopupContent => ({
  headword: "hablar",
  surface: "hablaba",
  gloss: "to talk, to speak",
  rarity: "Courant — parmi les 1 000 mots les plus fréquents.",
  sentence: "Le verbe hablaba se conjugue ainsi.",
  status: null,
  rect: { left: 40, top: 60, bottom: 80 },
  language: "es",
  readIn: "es",
  ...over,
});

/** A French card, as every card read in the document's language. */
const french = (): WordPopupContent => ({
  headword: "conjuguer",
  surface: "conjugue",
  gloss: "to conjugate",
  rarity: "Peu fréquent — au-delà des 5 000 mots les plus fréquents.",
  sentence: "Le verbe hablaba se conjugue ainsi.",
  status: null,
  rect: { left: 40, top: 60, bottom: 80 },
  language: "fr",
});

function speakers() {
  const fake = makeFakeSpeech([THOMAS, MONICA]);
  const document = createSpeaker(fake.engine, "fr", fake.preference);
  const others = new Map<StudiedLanguage, Speaker>();
  const speakerFor = vi.fn((studied: StudiedLanguage) => {
    let s = others.get(studied);
    if (!s) {
      s = createSpeaker(fake.engine, studied, fake.preference);
      others.set(studied, s);
    }
    return s;
  });
  return { fake, document, speakerFor, others };
}

const lineOf = (card: { el: HTMLElement }) => card.el.querySelector(".read-in");
const labels = (card: { el: HTMLElement }): string[] =>
  [...card.el.querySelectorAll(".listen button")].map((b) => b.textContent ?? "");
function button(card: { el: HTMLElement }, label: string): HTMLButtonElement {
  const b = [...card.el.querySelectorAll("button")].find((el) => el.textContent === label);
  if (!b) throw new Error(`no button "${label}"`);
  return b as HTMLButtonElement;
}

describe("the language line (D4)", () => {
  it("names the language before the frequency line, in each interface language", () => {
    for (const [language, line] of [
      ["fr", "Mot espagnol"],
      ["en", "Spanish word"],
      ["es", "Palabra española"],
    ] as const) {
      const card = createCard(undefined, language);
      card.show(reread(), () => {});
      expect(lineOf(card)?.textContent, language).toBe(line);
      expect(lineOf(card)?.nextElementSibling?.className).toBe("rarity");
    }
  });

  it("names each studied language with the noun's agreement", () => {
    const card = createCard(undefined, "es");
    card.show(reread({ readIn: "fr", language: "fr" }), () => {});
    expect(lineOf(card)?.textContent).toBe("Palabra francesa");
    card.show(reread({ readIn: "en", language: undefined }), () => {});
    expect(lineOf(card)?.textContent).toBe("Palabra inglesa");
    const fr = createCard(undefined, "fr");
    fr.show(reread({ readIn: "fr", language: "fr" }), () => {});
    expect(lineOf(fr)?.textContent).toBe("Mot français");
  });

  it("a card without it renders the tree it always did, after a re-read card too", () => {
    const plain = createCard();
    plain.show(french(), () => {});
    const before = plain.el.outerHTML;
    expect(lineOf(plain)).toBeNull();

    const { document: doc, speakerFor } = speakers();
    const card = createCard(doc, "fr", speakerFor);
    const reference = createCard(doc, "fr");
    reference.show(french(), () => {});
    card.show(reread(), () => {});
    card.show(french(), () => {});
    expect(lineOf(card)).toBeNull();
    expect(card.el.outerHTML).toBe(reference.el.outerHTML);
    expect(before).toBe(createCardHtml());
  });

  it("carries the language on the card's gesture, and on no other", () => {
    const spy = vi.fn<(g: Gesture) => void>();
    const card = createCard();
    card.show(reread(), spy);
    button(card, "Je connais").click();
    expect(spy).toHaveBeenLastCalledWith({
      lemma: "hablar",
      surface: "hablaba",
      sentence: "Le verbe hablaba se conjugue ainsi.",
      status: "known",
      expression: false,
      gloss: "to talk, to speak",
      readIn: "es",
    });
    card.show(french(), spy);
    button(card, "+ Deck").click();
    expect(spy.mock.lastCall![0]).not.toHaveProperty("readIn");
  });
});

/** A plain French card's tree, built afresh. */
function createCardHtml(): string {
  const card = createCard();
  card.show(french(), () => {});
  return card.el.outerHTML;
}

describe("The word's voice (D4)", () => {
  it("speaks the word and its dictionary form with the card language's voice, the sentence with the document's", () => {
    const { fake, document: doc, speakerFor } = speakers();
    const card = createCard(doc, "fr", speakerFor);
    window.document.body.append(card.el);
    card.show(reread(), () => {});
    expect(speakerFor).toHaveBeenCalledWith("es");
    expect(labels(card)).toEqual(["▶ hablaba", "▶ hablar", "▶ Phrase"]);
    button(card, "▶ hablaba").click();
    expect(fake.spoken.at(-1)).toMatchObject({ text: "hablaba", voice: MONICA });
    expect(labels(card)).toEqual(["■ Arrêter", "▶ hablar", "▶ Phrase"]);
    button(card, "▶ hablar").click();
    expect(fake.spoken.at(-1)).toMatchObject({ text: "hablar", voice: MONICA });
    button(card, "▶ Phrase").click();
    expect(fake.spoken.at(-1)).toMatchObject({ text: "Le verbe hablaba se conjugue ainsi.", voice: THOMAS });
    // One utterance at a time: the Spanish voice fell silent.
    expect(labels(card)).toEqual(["▶ hablaba", "▶ hablar", "■ Arrêter"]);
    button(card, "■ Arrêter").click();
    expect(doc.speaking()).toBeNull();
  });

  it("stops the word's voice from its button, and when the card closes or another word opens", () => {
    const { document: doc, speakerFor, others } = speakers();
    const card = createCard(doc, "fr", speakerFor);
    card.show(reread(), () => {});
    button(card, "▶ hablaba").click();
    button(card, "■ Arrêter").click();
    expect(others.get("es")!.speaking()).toBeNull();
    button(card, "▶ hablaba").click();
    card.hide();
    expect(others.get("es")!.speaking()).toBeNull();
    card.show(reread(), () => {});
    button(card, "▶ hablar").click();
    card.show(french(), () => {});
    expect(others.get("es")!.speaking()).toBeNull();
  });

  it("redraws the row when the other language's voice ends", () => {
    const { fake, document: doc, speakerFor } = speakers();
    const card = createCard(doc, "fr", speakerFor);
    card.show(reread(), () => {});
    button(card, "▶ hablaba").click();
    fake.spoken.at(-1)!.done(null);
    expect(labels(card)).toEqual(["▶ hablaba", "▶ hablar", "▶ Phrase"]);
  });

  it("offers only the sentence when no voice of the card's language is on the device", () => {
    const fake = makeFakeSpeech([THOMAS]);
    const doc = createSpeaker(fake.engine, "fr", fake.preference);
    const card = createCard(doc, "fr", (studied) => createSpeaker(fake.engine, studied, fake.preference));
    card.show(reread(), () => {});
    expect(labels(card)).toEqual(["▶ Phrase"]);
  });

  it("without a speaker per language, the card's own reads the word", () => {
    const { fake, document: doc } = speakers();
    const card = createCard(doc, "fr");
    card.show(reread(), () => {});
    button(card, "▶ hablaba").click();
    expect(fake.spoken.at(-1)).toMatchObject({ text: "hablaba", voice: THOMAS });
  });

  it("the popup hands its speaker per language to its card", () => {
    const { fake, document: doc, speakerFor } = speakers();
    const popup = new WordPopup({ css: "", onGesture: () => {}, speaker: doc, speakerFor });
    popup.show(reread());
    expect(speakerFor).toHaveBeenCalledWith("es");
    expect(fake.spoken).toHaveLength(0);
  });
});
