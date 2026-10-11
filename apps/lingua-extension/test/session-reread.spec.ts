import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LanguagePort } from "@/analyzer/port.ts";
import type { NativeLanguage, PageAnalysis, PhraseGloss, StudiedLanguage, WordGrammar } from "@/analyzer/types.ts";
import { HL_LEARNING, HL_UNKNOWN } from "@/reading/highlight.ts";
import type { ResolvedToken } from "@/reading/scan.ts";
import {
  GUARDED,
  otherStudiedLanguages,
  paintReread,
  primaryLanguage,
  type ReadingHost,
  ReadingSession,
  type SessionStats,
} from "@/reading/session.ts";
import { type Gesture, WordPopup, type WordPopupContent } from "@/reading/wordpopup.ts";
import { makeFakePort } from "./helpers.ts";

// read-lingua-card-in-another-studied-language, the session's side (tasks 2.1–2.3, 4.2): which
// documents re-read a word, what a gesture on a re-read card records and where, how the document
// paints the words it re-read, and when it forgets them. The engine is the fake port, answering per
// language from the tables below; the document is a book section in its own iframe.

const packs = vi.hoisted(() => ({ shipped: ["en-fr", "es-fr", "fr-en", "es-en", "fr-es", "en-es"] }));
vi.mock("@/analyzer/pairs.ts", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/analyzer/pairs.ts")>();
  return {
    ...actual,
    readingLanguage: (port: Parameters<typeof actual.readingLanguage>[0]) =>
      actual.readingLanguage(port, packs.shipped),
    acceptedLanguages: (port: Parameters<typeof actual.acceptedLanguages>[0]) =>
      actual.acceptedLanguages(port, packs.shipped),
  };
});

class FakeHighlight extends Set<Range> {
  constructor(...ranges: Range[]) {
    super(ranges);
  }
}

const CSS_TEXT = { tokens: "", popup: "", drawer: "", hud: "" };
const PAGE = "<p>Le verbe hablaba se conjugue.</p>";

/** A book section declaring `lang`, with its own window and highlight registry. */
function section(html: string, lang: string | null) {
  const frame = document.createElement("iframe");
  document.body.append(frame);
  const doc = frame.contentDocument!;
  const win = frame.contentWindow! as Window & typeof globalThis;
  if (lang) doc.documentElement.setAttribute("lang", lang);
  doc.body.innerHTML = html;
  const registry = new Map<string, FakeHighlight>();
  Object.assign(win, { CSS: { highlights: registry }, Highlight: FakeHighlight });
  const box = { left: 0, top: 0, bottom: 0, right: 0, width: 0, height: 0, x: 0, y: 0, toJSON: () => ({}) };
  (win as unknown as { Range: typeof Range }).Range.prototype.getBoundingClientRect = () => box;
  const host: ReadingHost = {
    doc,
    win,
    paintWhole: true,
    toSurface: (b) => b,
    source: () => "A book",
    exposureSource: () => "reading:book",
  };
  const painted = (name: string) => [...(registry.get(name) ?? [])].map((r) => r.toString());
  /** Select `word` in the first paragraph and lift the pointer: the selection is captured. */
  const select = (word: string) => {
    const node = doc.querySelector("p")!.firstChild!;
    const at = node.textContent!.indexOf(word);
    const range = doc.createRange();
    range.setStart(node, at);
    range.setEnd(node, at + word.length);
    const sel = win.getSelection()!;
    sel.removeAllRanges();
    sel.addRange(range);
    doc.body.dispatchEvent(new win.Event("mouseup", { bubbles: true }));
  };
  return { host, painted, select };
}

/** The French analysis of `PAGE`: « hablaba » unknown, nothing known of it. */
function frenchAnalysis(blocks: string[]): PageAnalysis {
  const at = new TextEncoder().encode(blocks[0]!.slice(0, blocks[0]!.indexOf("hablaba"))).length;
  return {
    analyzer_version: "1",
    analysable: true,
    tokens: [{ block: 0, start: at, end: at + 7, surface: "hablaba", lemma: "hablaba", class: "Unknown", gloss: null }],
    counted: 5,
    known: 4,
    percent: 80,
  };
}

const EMPTY: WordGrammar = { gloss: null, senses: [], readings: [], others: [], pieces: [], rank: null };

/**
 * A reader of `native` who studies `studied`, the document read in `documentLanguage`. Spanish knows
 * « hablaba » (`hablar`), with the status the reader gave it there; French knows nothing of it.
 */
async function reader(opts: {
  native?: NativeLanguage;
  studied?: StudiedLanguage[];
  documentLanguage?: StudiedLanguage;
  analysable?: boolean;
}) {
  const { port, calls } = makeFakePort();
  await port.setStudiedLanguages(opts.studied ?? ["es", "fr"]);
  port.nativeLanguage = async () => opts.native ?? "en";
  port.detectLanguage = async () => opts.documentLanguage ?? "fr";
  const spanish = new Map<string, "Known" | "Learning" | "Ignored">();
  const asked: { language: StudiedLanguage; text: string }[] = [];
  const statuses: [StudiedLanguage, string, string | null][] = [];
  const cards: [StudiedLanguage, string, string | null][] = [];
  const glosses: [StudiedLanguage, string][] = [];
  type View = LanguagePort & { language: StudiedLanguage };
  port.analyse = async function (this: View, blocks) {
    if (opts.analysable === false)
      return { analyzer_version: "1", analysable: false, tokens: [], counted: 0, known: 0, percent: null };
    return frenchAnalysis(blocks);
  };
  port.phraseGloss = async function (this: View, text): Promise<PhraseGloss> {
    asked.push({ language: this.language, text });
    const word = text.toLowerCase();
    if (this.language === "es" && (word === "hablaba" || word === "demande")) {
      const lemma = word === "hablaba" ? "hablar" : "demandar";
      return {
        tokens: [
          {
            surface: text,
            lemma,
            class: spanish.get(lemma) ?? "Unknown",
            gloss: word === "hablaba" ? "to talk, to speak" : "to sue",
            function_word: false,
          },
        ],
      };
    }
    return { tokens: [{ surface: text, lemma: word, class: "Unknown", gloss: null, function_word: false }] };
  };
  port.wordGrammar = async function (this: View, _written, lemma): Promise<WordGrammar> {
    if (this.language === "es" && lemma === "hablar") return { ...EMPTY, gloss: "to talk, to speak", rank: 181 };
    return EMPTY;
  };
  port.gloss = async function (this: View, lemma) {
    glosses.push([this.language, lemma]);
    return this.language === "es" && lemma === "hablar" ? "to talk, to speak" : undefined;
  };
  port.setStatusAt = async function (this: View, lemma, status) {
    statuses.push([this.language, lemma, status]);
    if (this.language === "es" && status)
      spanish.set(lemma, status === "known" ? "Known" : status === "learning" ? "Learning" : "Ignored");
  };
  port.addCard = async function (this: View, card) {
    cards.push([this.language, card.lemma, card.gloss]);
    if (this.language === "es") spanish.set(card.lemma, "Learning");
  };
  const shows: WordPopupContent[] = [];
  const show = vi.spyOn(WordPopup.prototype, "show");
  show.mockImplementation(function (this: WordPopup, content: WordPopupContent) {
    shows.push(content);
    return shows.length;
  });
  vi.spyOn(WordPopup.prototype, "generation").mockImplementation(() => shows.length);
  const session = new ReadingSession(port, { css: CSS_TEXT, surface: "book" });
  await session.start(null);
  return { session, port, calls, asked, statuses, cards, glosses, shows, spanish };
}

/** The private seams the browser drives. */
type Driven = {
  onGesture(g: Gesture): Promise<void>;
  repaint(): Promise<void>;
  statsMessage(): Promise<SessionStats>;
  showPopup(hit: ResolvedToken): void;
  resolved: ResolvedToken[];
};

const gestureOn = (content: WordPopupContent, status: Gesture["status"]): Gesture => ({
  lemma: content.headword,
  surface: content.surface,
  sentence: content.sentence,
  status,
  expression: false,
  gloss: content.gloss,
  ...(content.readIn ? { readIn: content.readIn } : {}),
});

beforeEach(() => {
  vi.stubGlobal("CSS", { highlights: new Map() });
  vi.stubGlobal("Highlight", FakeHighlight);
  vi.stubGlobal("chrome", {
    runtime: {
      sendMessage: vi.fn(async (msg: { type?: string }) => (msg?.type === "store:get" ? { items: {} } : undefined)),
      onMessage: { addListener: () => {}, removeListener: () => {} },
    },
    storage: {
      local: { get: async () => ({}), set: async () => {} },
      onChanged: { addListener: () => {}, removeListener: () => {} },
    },
  });
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
  document.body.innerHTML = "";
  document.documentElement.querySelectorAll("[data-cymbra-lingua-skip]").forEach((el) => el.remove());
});

/** The daily counts the session wrote through the store, the latest last. */
function dailyWrites(): Record<string, Record<string, unknown>>[] {
  const send = chrome.runtime.sendMessage as unknown as {
    mock: { calls: [{ type?: string; items?: Record<string, unknown> }][] };
  };
  return send.mock.calls
    .map(([message]) => message)
    .filter((message) => message.type === "store:set" && message.items?.["cymbra-lingua-daily-v4"])
    .map((message) => message.items!["cymbra-lingua-daily-v4"] as Record<string, Record<string, unknown>>);
}

describe("a Spanish word on a French page, for an English speaker who studies Spanish and French", () => {
  async function opened() {
    const r = await reader({});
    const page = section(PAGE, "fr");
    await r.session.attach(page.host);
    expect(page.painted(HL_UNKNOWN)).toEqual(["hablaba"]);
    page.select("hablaba");
    await vi.waitFor(() => expect(r.shows.at(-1)?.readIn).toBe("es"));
    return { ...r, page, card: r.shows.at(-1)! };
  }

  it("opens the Spanish card of hablar", async () => {
    const { card, asked, session } = await opened();
    expect(card).toMatchObject({ headword: "hablar", surface: "hablaba", gloss: "to talk, to speak", language: "es" });
    expect(asked).toContainEqual({ language: "es", text: "hablaba" });
    session.detach();
  });

  it("Known in Spanish: recorded under Spanish, nothing French, and no longer highlighted", async () => {
    const { session, page, card, statuses } = await opened();
    const driven = session as unknown as Driven;
    const before = await driven.statsMessage();
    await driven.onGesture(gestureOn(card, "known"));
    expect(statuses).toEqual([["es", "hablar", "known"]]);
    expect(page.painted(HL_UNKNOWN)).toEqual([]);
    expect(page.painted(HL_LEARNING)).toEqual([]);
    // The page's figures are the French analysis's.
    const after = await driven.statsMessage();
    expect([after.percent, after.counted, after.unknownOccurrences]).toEqual([
      before.percent,
      before.counted,
      before.unknownOccurrences,
    ]);
    await vi.waitFor(() => expect(dailyWrites()).toHaveLength(1));
    expect(Object.values(dailyWrites()[0]!)).toEqual([
      { es: { exposures: 0, unknownSeen: 0, wordsLearned: 1, reviews: 0, native: "en" } },
    ]);
    session.detach();
  });

  it("Added to the deck: a Spanish card with the Spanish gloss, painted as being learnt", async () => {
    const { session, page, card, cards, glosses, shows } = await opened();
    await (session as unknown as Driven).onGesture(gestureOn(card, "learning"));
    expect(cards).toEqual([["es", "hablar", "to talk, to speak"]]);
    expect(glosses).toEqual([["es", "hablar"]]);
    expect(page.painted(HL_LEARNING)).toEqual(["hablaba"]);
    expect(page.painted(HL_UNKNOWN)).toEqual([]);
    // Opened again, it is still a word French does not know: its Spanish card, now in the deck.
    const driven = session as unknown as Driven;
    driven.showPopup(driven.resolved[0]!);
    await vi.waitFor(() => expect(shows.at(-1)).toMatchObject({ readIn: "es", status: "learning" }));
    session.detach();
  });

  it("ignored in Spanish: unpainted; brought back to learn: painted unknown again", async () => {
    const { session, page, card, spanish } = await opened();
    const driven = session as unknown as Driven;
    await driven.onGesture(gestureOn(card, "ignored"));
    expect(page.painted(HL_UNKNOWN)).toEqual([]);
    spanish.delete("hablar");
    await driven.repaint();
    expect(page.painted(HL_UNKNOWN)).toEqual(["hablaba"]);
    session.detach();
  });

  it("The next document: hablaba is painted by the French analysis, as before", async () => {
    const { session, card, port } = await opened();
    await (session as unknown as Driven).onGesture(gestureOn(card, "known"));
    const phraseGloss = vi.spyOn(port, "phraseGloss");
    const next = section(PAGE, "fr");
    await session.attach(next.host);
    expect(next.painted(HL_UNKNOWN)).toEqual(["hablaba"]);
    expect(phraseGloss).not.toHaveBeenCalled(); // nothing remembered: nothing asked at paint
    session.detach();
  });

  it("a gesture on a card of the document's language records it there, as before", async () => {
    const { session, statuses } = await opened();
    await (session as unknown as Driven).onGesture({
      lemma: "conjuguer",
      surface: "conjugue",
      sentence: "Le verbe hablaba se conjugue.",
      status: "known",
      expression: false,
      gloss: null,
    });
    expect(statuses).toEqual([["fr", "conjuguer", "known"]]);
    session.detach();
  });

  it("is stopped with every voice it opened", async () => {
    const { session } = await opened();
    session.stop();
  });
});

describe("the documents that re-read nothing", () => {
  it("A Catalan page: no Spanish card", async () => {
    const r = await reader({});
    const page = section(PAGE, "ca");
    await r.session.attach(page.host);
    page.select("hablaba");
    await vi.waitFor(() => expect(r.shows.length).toBeGreaterThan(0));
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(r.asked.filter((a) => a.language === "es")).toEqual([]);
    expect(r.shows.at(-1)!.readIn).toBeUndefined();
    r.session.detach();
  });

  it("A French speaker's French page (not analysable): « demande » and hablaba keep their card", async () => {
    const r = await reader({ native: "fr", studied: ["en", "es"], documentLanguage: "en", analysable: false });
    const page = section("<p>Il demande si hablaba est espagnol.</p>", "fr");
    await r.session.attach(page.host);
    for (const word of ["demande", "hablaba"]) {
      page.select(word);
      await vi.waitFor(() => expect(r.shows.at(-1)?.pending).not.toBe(true));
    }
    expect(r.asked.filter((a) => a.language === "es")).toEqual([]);
    expect(r.shows.every((card) => card.readIn === undefined)).toBe(true);
    r.session.detach();
  });

  it("a reader of one language: nothing asked of another", async () => {
    const r = await reader({ studied: ["fr"] });
    const page = section(PAGE, "fr");
    await r.session.attach(page.host);
    page.select("hablaba");
    await vi.waitFor(() => expect(r.shows.length).toBeGreaterThan(0));
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(r.asked.filter((a) => a.language !== "fr")).toEqual([]);
    r.session.detach();
  });
});

describe("otherStudiedLanguages (D1, D7)", () => {
  const base = {
    languages: ["es", "fr"] as StudiedLanguage[],
    language: "fr" as StudiedLanguage,
    analysable: true,
    declared: [] as (string | null)[],
    native: "en" as NativeLanguage,
  };

  it("is the reader's other languages, in the profile's order", () => {
    expect(otherStudiedLanguages(base)).toEqual(["es"]);
    expect(otherStudiedLanguages({ ...base, languages: ["fr", "es", "en"] as StudiedLanguage[] })).toEqual([
      "es",
      "en",
    ]);
    expect(otherStudiedLanguages({ ...base, declared: ["fr", null] })).toEqual(["es"]);
  });

  it("is none on a document not read in a studied language, or declaring the native language", () => {
    expect(otherStudiedLanguages({ ...base, analysable: false })).toEqual([]);
    expect(otherStudiedLanguages({ ...base, declared: [null, "en"] })).toEqual([]);
    expect(
      otherStudiedLanguages({ ...base, native: "fr", languages: ["en", "es"], language: "en", declared: ["fr"] }),
    ).toEqual([]);
  });

  it("never reads Spanish on a Catalan, Galician or Occitan page, nor French on a Catalan, Occitan or Romanian one", () => {
    for (const declared of ["ca", "gl", "oc", "glg", "cat", "oci"]) {
      expect(otherStudiedLanguages({ ...base, declared: [primaryLanguage(declared)] }), declared).toEqual([]);
    }
    for (const declared of ["ro", "ron", "rum", "ca", "oc"]) {
      expect(
        otherStudiedLanguages({ ...base, language: "es", declared: [primaryLanguage(declared)] }),
        declared,
      ).toEqual([]);
    }
    // Galician is not kept out of French.
    expect(otherStudiedLanguages({ ...base, language: "es", declared: ["gl"] })).toEqual(["fr"]);
    expect(GUARDED.en).toBeUndefined();
  });

  it("reads the guards' three-letter codes", () => {
    expect(["cat", "glg", "oci", "ron", "rum"].map(primaryLanguage)).toEqual(["ca", "gl", "oc", "ro", "ro"]);
  });
});

describe("paintReread (D6)", () => {
  const resolved = (surface: string, cls: "Unknown" | "Learning" = "Unknown"): ResolvedToken => ({
    token: { block: 0, start: 0, end: 1, surface, lemma: surface.toLowerCase(), class: cls, gloss: null },
    range: document.createRange(),
  });

  it("paints a re-read word by its status in its language, and every other as the analysis did", () => {
    const tokens = [resolved("Hablaba"), resolved("comía"), resolved("vivía"), resolved("dormía"), resolved("otro")];
    const painted = paintReread(
      tokens,
      new Map([
        ["hablaba", "Known"],
        ["comía", "Ignored"],
        ["vivía", "Learning"],
        ["dormía", "ProperNounOutOfLexicon"],
      ]),
    );
    expect(painted.map((r) => [r.token.surface, r.token.class])).toEqual([
      ["vivía", "Learning"],
      ["dormía", "Unknown"],
      ["otro", "Unknown"],
    ]);
    expect(painted[2]).toBe(tokens[4]);
  });

  it("returns the analysis's paint untouched with nothing remembered", () => {
    const tokens = [resolved("hablaba")];
    expect(paintReread(tokens, new Map())).toBe(tokens);
  });
});
