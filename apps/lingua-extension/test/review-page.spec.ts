import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LinguaPort } from "@/analyzer/port.ts";
import { inSettings, mountReview, type ReviewOnlineSource } from "@/review/review-page.ts";
import type { ReviewTranslationHint } from "@/review/view.ts";
import { loadDailyStats, utcDay } from "@/state/dailystats.ts";
import { localDayStart } from "@/review/session.ts";
import type { SpeechEngine, VoiceInfo } from "@/reading/speech.ts";
import {
  type AsyncStorageArea,
  NEW_WORDS_PER_DAY_KEY,
  REVIEW_LANGUAGE_KEY,
  ROOT_KEY,
  STORAGE_VERSION,
} from "@/state/storage.ts";
import { STORE_CHANGED_KEY } from "@/state/store.ts";
import type { TranslatorSource } from "@/translate/create-port.ts";
import type { OnlineTranslationRequest, OnlineTranslationResult } from "@/translate/online.ts";
import { TRANSLATION_WAIT_MS, type TranslationRequest, type TranslationResult } from "@/translate/port.ts";
import { type FakeCard, makeFakePort, makeFakeSpeech } from "./helpers.ts";

// The Révision page as a WHOLE: the summary, the FSRS widget and the reaction to a state written
// by another surface (backup, restore and the pack's credits live in Réglages › Données). The render itself is
// pinned in view.spec.ts and the session in session.spec.ts — what is pinned here is the
// mounting: what the page asks the port, what it saves, and when it rebuilds itself.

// The bundle's pairs are a build-time constant: the pairs the page consults are set per test.
const packs = vi.hoisted(() => ({ shipped: ["en-fr"] }));
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

const NOW_SECONDS = 1_770_000_000; // the injected clock (epoch seconds)
const NOW_MS = NOW_SECONDS * 1000; // Date.now(), which only the daily recorder reads

const DECK: FakeCard[] = [
  { headword: "seldom", surface: "seldom", sentence: "They seldom ship.", gloss: "rarement" },
  { headword: "dwell", surface: "dwells", sentence: "It dwells in the north.", gloss: "demeurer" },
];

type Area = AsyncStorageArea & { raw: Record<string, unknown> };

function fakeArea(): Area {
  const raw: Record<string, unknown> = {};
  return {
    raw,
    async get(keys) {
      const list = keys == null ? Object.keys(raw) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const k of list) if (k in raw) out[k] = raw[k];
      return out;
    },
    async set(items) {
      Object.assign(raw, items);
    },
  };
}

/** What the store would announce to every surface after a write (see `state/store.ts`). */
type ChangeListener = (changes: Record<string, { newValue?: unknown }>, areaName: string) => void;
const listeners: ChangeListener[] = [];
let rev = 0;

function announce(keys: string[]): void {
  rev += 1;
  for (const listener of listeners) listener({ [STORE_CHANGED_KEY]: { newValue: { rev, keys } } }, "local");
}

beforeEach(() => {
  packs.shipped = ["en-fr"];
  listeners.length = 0;
  document.body.replaceChildren();
  vi.spyOn(Date, "now").mockReturnValue(NOW_MS);
  vi.stubGlobal("chrome", {
    storage: {
      onChanged: {
        addListener: (fn: ChangeListener) => void listeners.push(fn),
      },
    },
  });
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

interface Host {
  /** What the host says the page beside the review is read in (refine-lingua-review-language D2). */
  pageLanguage?: () => Promise<string | null>;
  /** The preferences area: the last language chosen in the review (D3), the daily allowance. */
  prefs?: Area;
  /** What reads the card aloud (refine-lingua-review-session D11); none under jsdom by default. */
  speech?: SpeechEngine | null;
  /** The translator its host hands it (add-lingua-review-translation D4); none by default. */
  translator?: TranslatorSource;
  /** Its « Traduction en ligne » (add-lingua-review-translation-fallback D2); none by default. */
  online?: ReviewOnlineSource;
  /** How a card nothing can translate gets a translation (add-lingua-review-translation-fallback D12). */
  translationHint?: (language: string) => ReviewTranslationHint | null;
  /** Beside a book, its words still to sort and opening their list (add-lingua-chapter-recap D6). */
  toSort?: () => Promise<number>;
  openToSort?: () => void;
}

function mount(port: LinguaPort, area: Area = fakeArea(), host: Host = {}) {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const prefs = host.prefs ?? fakeArea();
  const page = mountReview(container, port, area, {
    now: () => NOW_SECONDS,
    prefs,
    pageLanguage: host.pageLanguage,
    speech: host.speech,
    translator: host.translator,
    online: host.online,
    translationHint: host.translationHint,
    toSort: host.toSort,
    openToSort: host.openToSort,
  });
  return { page, container, area, prefs };
}

/**
 * Let the page's floating promises settle (every handler is fire-and-forget), over several rounds.
 */
const settle = async (): Promise<void> => {
  for (let i = 0; i < 3; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

function button(container: HTMLElement, label: string): HTMLButtonElement {
  const found = [...container.querySelectorAll("button")].find((b) => b.textContent === label);
  if (!found) throw new Error(`no button "${label}"`);
  return found;
}

const text = (container: HTMLElement, selector: string): string => container.querySelector(selector)?.textContent ?? "";

describe("Révision — the page", () => {
  it("shows the deck and what a review would hold today, asked on the injected clock", async () => {
    // The clock is a seam precisely so the page never reads Date.now() for the schedule.
    const asked: unknown[] = [];
    const { port } = makeFakePort();
    const prefs = fakeArea();
    prefs.raw[NEW_WORDS_PER_DAY_KEY] = 5;
    const m = mount(
      {
        ...port,
        deckCount: async () => 12,
        dueCount: async () => 40, // every new word: no longer the figure shown
        reviewableCount: async (now, _languages, allowance) => {
          asked.push([now, allowance]);
          return 3;
        },
      },
      fakeArea(),
      { prefs },
    );

    await m.page.refresh();

    // add-lingua-comeback-hints D1: the new words beyond the day's allowance are not counted.
    expect(text(m.container, ".summary")).toBe("12 carte(s) · 3 à revoir");
    expect(asked).toEqual([[NOW_SECONDS, { newPerDay: 5, dayStart: localDayStart(NOW_SECONDS) }]]);
    expect(button(m.container, "Réviser")).toBeTruthy();
  });

  it("starts a session and hands the first card over without its answer", async () => {
    const m = mount(makeFakePort(DECK).port);
    await m.page.refresh();

    button(m.container, "Réviser").click();
    await settle();

    expect(text(m.container, ".review-sentence")).toBe("They seldom ship.");
    expect(text(m.container, ".review-word")).toBe("seldom");
    expect(text(m.container, ".remaining")).toBe("2 carte(s) à revoir");
    expect(m.container.textContent).not.toContain("rarement"); // the answer is still hidden
    expect(m.page.reviewing()).toBe(true);
  });

  it("reveals the answer only when the reader asks for it", async () => {
    const m = mount(makeFakePort(DECK).port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    button(m.container, "Afficher la réponse").click();
    await settle();

    expect(text(m.container, ".review-gloss")).toBe("rarement");
    expect(text(m.container, ".review-sentence")).toBe("They seldom ship.");
  });

  it("grades a card, saves the state and counts the review for the day", async () => {
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();

    button(m.container, "Su").click();
    await settle();

    expect(calls.grades).toEqual(["good"]);
    // A grade changes the engine, so the whole state is written back for every surface.
    expect(m.area.raw[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "{}" });
    expect((await loadDailyStats(m.area))[utcDay(NOW_MS)]).toEqual({
      en: { exposures: 0, unknownSeen: 0, wordsLearned: 0, reviews: 1, native: "fr" },
    });
    expect(text(m.container, ".review-word")).toBe("dwells"); // moved on to the next card
  });

  it("marks a card known, counting a word learned rather than a review", async () => {
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();

    button(m.container, "Je connais").click();
    await settle();

    expect(calls.markKnown).toBe(1);
    expect(m.area.raw[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "{}" });
    expect((await loadDailyStats(m.area))[utcDay(NOW_MS)]).toEqual({
      en: { exposures: 0, unknownSeen: 0, wordsLearned: 1, reviews: 0, native: "fr" },
    });
  });

  it("counts the review under the engine's native language, a Spanish one included", async () => {
    // French is also the default: a day labelled by habit would pass every French-native test.
    const { port } = makeFakePort(DECK);
    port.nativeLanguage = async () => "es";
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();

    button(m.container, "Su").click();
    await settle();

    expect((await loadDailyStats(m.area))[utcDay(NOW_MS)]).toEqual({
      en: { exposures: 0, unknownSeen: 0, wordsLearned: 0, reviews: 1, native: "es" },
    });
  });

  it("says there is nothing to review when nothing is due", async () => {
    const m = mount(makeFakePort([]).port);
    await m.page.refresh();

    button(m.container, "Réviser").click();
    await settle();

    expect(m.container.textContent).toContain("Rien à réviser pour l'instant.");
    expect(m.page.reviewing()).toBe(false);
  });

  it("tells its host whether a session is under way, all the way to the empty queue", async () => {
    // The drawer asks this before restoring a file: a restore would end the session.
    const m = mount(makeFakePort([DECK[0]]).port);
    await m.page.refresh();
    expect(m.page.reviewing()).toBe(false);

    button(m.container, "Réviser").click();
    await settle();
    expect(m.page.reviewing()).toBe(true);

    button(m.container, "Afficher la réponse").click();
    await settle();
    button(m.container, "Su").click();
    await settle();

    expect(m.page.reviewing()).toBe(false);
  });

  it("scenario: the review page holds the count and the card, not backup, restore or sources", async () => {
    const m = mount(makeFakePort(DECK).port);
    await m.page.refresh();
    await settle();

    expect(m.container.querySelector('input[type="file"]')).toBeNull();
    expect(m.container.querySelector("details")).toBeNull();
    expect(m.container.textContent).not.toMatch(/Sauvegarder|Restaurer|Sources/);
  });

  it("starts a session of ten with the reader's daily allowance of new words", async () => {
    const { port, calls } = makeFakePort(DECK);
    const prefs = fakeArea();
    await prefs.set({ [NEW_WORDS_PER_DAY_KEY]: 20 });
    const m = mount(port, fakeArea(), { prefs });
    await m.page.refresh();

    button(m.container, "Réviser").click();
    await settle();

    expect(calls.reviewOptions[0]).toEqual({ limit: 10, newPerDay: 20, dayStart: localDayStart(NOW_SECONDS) });
  });

  it("puts the focus in the card when a session starts, so the keys answer it", async () => {
    const m = mount(makeFakePort(DECK).port);
    await m.page.refresh();

    button(m.container, "Réviser").click();
    await settle();

    expect(document.activeElement).toBe(m.container.querySelector(".review"));
  });

  it("hides a word, saves the state and moves on, counting nothing", async () => {
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    button(m.container, "Ne plus me le montrer").click();
    await settle();

    expect(calls.ignored).toBe(1);
    expect(m.area.raw[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "{}" });
    expect(await loadDailyStats(m.area)).toEqual({});
    expect(text(m.container, ".review-word")).toBe("dwells");
  });

  it("ends a session on what it did, and offers ten more while cards remain", async () => {
    const twelve: FakeCard[] = Array.from({ length: 12 }, (_, i) => ({
      headword: `w${i}`,
      surface: `w${i}`,
      sentence: "",
      gloss: null,
    }));
    const m = mount(makeFakePort(twelve).port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    for (let i = 0; i < 10; i++) {
      button(m.container, "Afficher la réponse").click();
      await settle();
      button(m.container, "Su").click();
      await settle();
    }

    expect(text(m.container, ".review-done-title")).toBe("Séance terminée");
    expect(text(m.container, ".review-done-stats")).toContain("10 mots revus");
    button(m.container, "Encore 10").click();
    await settle();
    expect(text(m.container, ".remaining")).toBe("2 carte(s) à revoir");
  });

  it("ends a session in progress on « Terminer »: the end, saved, ten more offered (add-lingua-review-stop-session)", async () => {
    const twelve: FakeCard[] = Array.from({ length: 12 }, (_, i) => ({
      headword: `w${i}`,
      surface: `w${i}`,
      sentence: "",
      gloss: null,
    }));
    const { port, calls } = makeFakePort(twelve);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();
    button(m.container, "Su").click();
    await settle();
    delete m.area.raw[ROOT_KEY];
    button(m.container, "Afficher la réponse").click(); // revealed, never answered
    await settle();

    const end = button(m.container, "Terminer");
    expect(end.getAttribute("aria-label")).toBe("Terminer la séance");
    end.click();
    await settle();

    expect(m.page.reviewing()).toBe(false);
    expect(calls.grades).toEqual(["good"]);
    expect(text(m.container, ".review-done-title")).toBe("Séance terminée");
    expect(text(m.container, ".review-done-stats")).toContain("1 mot revu");
    expect(m.container.querySelector(".review-end")).toBeNull();
    // The end every session has: the state saved with no session under way.
    expect(m.area.raw[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "{}" });
    // « Encore 10 »: the revealed card and the others not answered come up again.
    button(m.container, "Encore 10").click();
    await settle();
    expect(text(m.container, ".remaining")).toBe("10 carte(s) à revoir");
    expect(m.container.querySelector(".review-end")).not.toBeNull();
  });

  it("ends a practice session the same way, practice offered again (add-lingua-review-stop-session)", async () => {
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    for (let i = 0; i < 2; i++) {
      button(m.container, "Afficher la réponse").click();
      await settle();
      button(m.container, "Su").click();
      await settle();
    }
    button(m.container, "S'entraîner quand même").click();
    await settle();
    expect(text(m.container, ".remaining")).toBe("Entraînement · 2 carte(s)");

    button(m.container, "Terminer").click();
    await settle();

    expect(m.page.reviewing()).toBe(false);
    expect(calls.grades).toEqual(["good", "good"]);
    // Ended before any practice answer: it ended, nothing counted, never « nothing due » alone.
    expect(text(m.container, ".review-done-title")).toBe("Séance terminée");
    expect(m.container.textContent).not.toContain("Rien à réviser");
    expect(m.container.querySelector(".review-more")).toBeNull();
    expect(button(m.container, "S'entraîner quand même")).toBeTruthy();
  });

  it("ends its session when a restore beside it dropped the engine's session", async () => {
    // Réglages › Données restores through the same engine: the session the page was walking is gone.
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    await port.reset();
    await m.area.set({ [ROOT_KEY]: { v: STORAGE_VERSION, backup: '{"restored":true}' } });
    announce([ROOT_KEY]);
    await settle();

    expect(m.page.reviewing()).toBe(false);
    expect(button(m.container, "Réviser")).toBeTruthy();
    expect(calls.restored).toEqual([]); // the engine already holds the restored state
  });

  it("takes on a state another surface wrote", async () => {
    // A reading gesture or a reset in Réglages goes through the store: the page reloads the
    // engine from it and repaints, so the two hosts never disagree.
    let summaries = 0;
    const { port, calls } = makeFakePort(DECK);
    const m = mount({
      ...port,
      deckCount: async () => {
        summaries += 1;
        return 4;
      },
    });
    await m.page.refresh();
    const before = summaries;

    await m.area.set({ [ROOT_KEY]: { v: STORAGE_VERSION, backup: '{"from":"the drawer"}' } });
    announce([ROOT_KEY]);
    await settle();

    expect(calls.restored).toEqual(['{"from":"the drawer"}']);
    expect(summaries).toBeGreaterThan(before);
  });

  it("ignores the echo of the save it made itself", async () => {
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();
    button(m.container, "Su").click();
    await settle(); // this grade is what wrote the state the store is about to announce

    announce([ROOT_KEY]);
    await settle();

    expect(calls.restored).toEqual([]); // re-restoring our own write would reset the session
  });

  it("leaves a review under way alone", async () => {
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    await m.area.set({ [ROOT_KEY]: { v: STORAGE_VERSION, backup: '{"from":"elsewhere"}' } });
    announce([ROOT_KEY]);
    await settle();

    expect(calls.restored).toEqual([]);
    expect(text(m.container, ".review-word")).toBe("seldom"); // the card is still there
  });
});

describe("Révision — a reader of several languages", () => {
  const MIXED: FakeCard[] = [
    { headword: "seldom", surface: "seldom", sentence: "They seldom ship.", gloss: "rarement", language: "en" },
    { headword: "faro", surface: "faro", sentence: "El faro brilla.", gloss: "phare", language: "es" },
  ];

  /** A reader of English then Spanish, with both pairs shipped. */
  async function bilingual(deck: FakeCard[] = MIXED) {
    packs.shipped = ["en-fr", "es-fr"];
    const fake = makeFakePort(deck);
    await fake.port.setStudiedLanguages(["en", "es"]);
    return fake;
  }

  const filter = (container: HTMLElement): HTMLElement | null => container.querySelector(".review-languages");
  const choices = (container: HTMLElement): string[] =>
    [...container.querySelectorAll(".review-languages button")].map((b) => b.textContent ?? "");
  const chosen = (container: HTMLElement): string =>
    container.querySelector(".review-languages button.active")?.textContent ?? "";

  it("shows no filter, and no language on the card, to a reader of one language", async () => {
    const { port, calls } = makeFakePort([{ ...DECK[0], language: "en" }]);
    const m = mount(port, fakeArea(), { pageLanguage: async () => "en" });
    await m.page.refresh();

    expect(filter(m.container)?.hidden).toBe(true);
    button(m.container, "Réviser").click();
    await settle();

    expect(m.container.querySelector(".review-language")).toBeNull();
    expect(calls.reviewLanguages).toEqual([["en"], ["en"]]); // the count and the session: English
    // English alone: its cards are every card, so the counts are those of every language, as before.
    expect(text(m.container, ".summary")).toBe(`${await port.deckCount()} carte(s) · 0 à revoir`);
  });

  it("offers each language, never all of them at once, and opens on the first away from a page", async () => {
    const { port, calls } = await bilingual();
    const m = mount(port);
    await m.page.refresh();

    expect(filter(m.container)?.hidden).toBe(false);
    expect(choices(m.container)).toEqual(["Anglais", "Espagnol"]);
    expect(chosen(m.container)).toBe("Anglais");
    expect(calls.reviewLanguages).toEqual([["en"]]);
    expect(text(m.container, ".summary")).toBe("1 carte(s) · 0 à revoir"); // English's card, not both
  });

  it("opens beside a Spanish page on the Spanish cards and counts", async () => {
    const { port, calls } = await bilingual();
    const m = mount(port, fakeArea(), { pageLanguage: async () => "es" });
    await m.page.refresh();

    expect(chosen(m.container)).toBe("Espagnol");
    expect(text(m.container, ".summary")).toBe("1 carte(s) · 0 à revoir");
    button(m.container, "Réviser").click();
    await settle();

    expect(calls.reviewLanguages).toEqual([["es"], ["es"]]);
    expect(text(m.container, ".review-word")).toBe("faro");
    expect(text(m.container, ".remaining")).toBe("1 carte(s) à revoir");
    expect(m.prefs.raw[REVIEW_LANGUAGE_KEY]).toBeUndefined(); // the page chose, not the reader
  });

  it("Changing the native language: the review opens in Spanish, the English last chosen no longer studied", async () => {
    // A French reader of English and Spanish chose English, es-en shipping
    // (add-lingua-native-language-choice D5): their profile is English-native, studying Spanish.
    packs.shipped = ["en-fr", "es-fr", "es-en"];
    const { port, calls } = makeFakePort(MIXED);
    port.nativeLanguage = async () => "en";
    await port.setStudiedLanguages(["es"]);
    const lastEnglish = fakeArea();
    lastEnglish.raw[REVIEW_LANGUAGE_KEY] = "en";
    const m = mount(port, fakeArea(), { prefs: lastEnglish, pageLanguage: async () => null });
    await m.page.refresh();

    expect(filter(m.container)?.hidden).toBe(true); // one language studied: no filter
    expect(calls.reviewLanguages.at(-1)).toEqual(["es"]);
    button(m.container, "Réviser").click();
    await settle();
    expect(text(m.container, ".review-word")).toBe("faro");
  });

  it("opens away from a page on the language chosen last, else on the first", async () => {
    const lastSpanish = fakeArea();
    lastSpanish.raw[REVIEW_LANGUAGE_KEY] = "es";
    let m = mount((await bilingual()).port, fakeArea(), { prefs: lastSpanish, pageLanguage: async () => null });
    await m.page.refresh();
    expect(chosen(m.container)).toBe("Espagnol");

    // A page in a language the reader does not study is no page to open on.
    m = mount((await bilingual()).port, fakeArea(), { prefs: lastSpanish, pageLanguage: async () => "de" });
    await m.page.refresh();
    expect(chosen(m.container)).toBe("Espagnol");

    // Nor is a language kept that the reader no longer studies.
    const lastGerman = fakeArea();
    lastGerman.raw[REVIEW_LANGUAGE_KEY] = "de";
    m = mount((await bilingual()).port, fakeArea(), { prefs: lastGerman });
    await m.page.refresh();
    expect(chosen(m.container)).toBe("Anglais");
  });

  it("switches to the language chosen and remembers it, beside the same page and after", async () => {
    const { port, calls } = await bilingual();
    let page: string | null = "es";
    const m = mount(port, fakeArea(), { pageLanguage: async () => page });
    await m.page.refresh();

    button(m.container, "Anglais").click();
    await settle();
    expect(chosen(m.container)).toBe("Anglais");
    expect(calls.reviewLanguages.at(-1)).toEqual(["en"]);
    expect(m.prefs.raw[REVIEW_LANGUAGE_KEY]).toBe("en");

    await m.page.refresh(); // the host shows Révision again, beside the same page: the choice stands
    expect(chosen(m.container)).toBe("Anglais");

    page = "en";
    await m.page.refresh();
    page = "es"; // a page in another language than the one followed brings its own back
    await m.page.refresh();
    expect(chosen(m.container)).toBe("Espagnol");

    page = null; // away from a page, the language chosen last
    await m.page.refresh();
    expect(chosen(m.container)).toBe("Anglais");
  });

  it("opens in the page's language again when it is opened anew or the page changes", async () => {
    // A choice holds between two showings beside one page; an opening, or another page or book of
    // the same language, brings the page's language back (refine-lingua-review-language D3).
    const { port } = await bilingual();
    const m = mount(port, fakeArea(), { pageLanguage: async () => "es" });
    await m.page.refresh();
    button(m.container, "Anglais").click();
    await settle();
    await m.page.refresh();
    expect(chosen(m.container)).toBe("Anglais");

    m.page.pageChanged();
    await m.page.refresh();

    expect(chosen(m.container)).toBe("Espagnol");
    expect(m.prefs.raw[REVIEW_LANGUAGE_KEY]).toBe("en"); // still the last chosen, away from a page
  });

  it("keeps a session under way in its language, whatever the reader or the page asks", async () => {
    const { port, calls } = await bilingual();
    let page: string | null = "es";
    const m = mount(port, fakeArea(), { pageLanguage: async () => page });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    button(m.container, "Anglais").click();
    await settle();
    page = "en";
    await m.page.refresh();

    expect(chosen(m.container)).toBe("Espagnol");
    expect(text(m.container, ".review-word")).toBe("faro");
    expect(calls.reviewLanguages).toEqual([["es"], ["es"], ["es"]]); // first count, the session, the count again
    expect(m.prefs.raw[REVIEW_LANGUAGE_KEY]).toBeUndefined();
  });

  it("says each card's language, and counts its grade and its word learned in it", async () => {
    const { port } = await bilingual();
    const m = mount(port, fakeArea(), { pageLanguage: async () => "es" });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    expect(text(m.container, ".review-language")).toBe("Espagnol");
    button(m.container, "Afficher la réponse").click();
    await settle();
    button(m.container, "Su").click();
    await settle();

    // The Spanish session is over, with nothing more due in Spanish: no « Encore 10 ».
    expect(text(m.container, ".review-done-title")).toBe("Séance terminée");
    expect(m.container.textContent).not.toContain("Encore 10");
    button(m.container, "Anglais").click(); // another language offers a start of its own
    await settle();
    button(m.container, "Réviser").click();
    await settle();
    expect(text(m.container, ".review-language")).toBe("Anglais");
    button(m.container, "Afficher la réponse").click();
    await settle();
    button(m.container, "Je connais").click();
    await settle();

    expect((await loadDailyStats(m.area))[utcDay(NOW_MS)]).toEqual({
      es: { exposures: 0, unknownSeen: 0, wordsLearned: 0, reviews: 1, native: "fr" },
      en: { exposures: 0, unknownSeen: 0, wordsLearned: 1, reviews: 0, native: "fr" },
    });
  });

  it("reads a card aloud in the card's own language (refine-lingua-review-session D11)", async () => {
    const voice = (name: string, lang: string): VoiceInfo => ({
      name,
      lang,
      localService: true,
      default: false,
      voiceURI: name,
    });
    const speech = makeFakeSpeech([voice("Samantha", "en-US"), voice("Mónica", "es-ES")]);
    const { port } = await bilingual();
    const m = mount(port, fakeArea(), { pageLanguage: async () => "es", speech: speech.engine });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    button(m.container, "▶ Mot").click();
    expect(speech.spoken.map((u) => [u.text, u.voice.lang])).toEqual([["faro", "es-ES"]]);
  });

  it("goes back to a language the reader studies when the chosen one leaves their languages", async () => {
    const { port, calls } = await bilingual();
    // The state another context stores keeps English alone.
    port.restore = async (backup) => {
      calls.restored.push(backup);
      await port.setStudiedLanguages(["en"]);
    };
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Espagnol").click();
    await settle();

    await m.area.set({ [ROOT_KEY]: { v: STORAGE_VERSION, backup: '{"from":"elsewhere"}' } });
    announce([ROOT_KEY]);
    await settle();

    expect(filter(m.container)?.hidden).toBe(true);
    expect(calls.reviewLanguages.at(-1)).toEqual(["en"]); // the count is of English's cards again
  });

  it("keeps the language chosen when another surface writes the state", async () => {
    // A sync pull or a gesture elsewhere must not send the review back to the page's language.
    const { port } = await bilingual();
    const m = mount(port, fakeArea(), { pageLanguage: async () => "es" });
    await m.page.refresh();
    button(m.container, "Anglais").click();
    await settle();

    await m.area.set({ [ROOT_KEY]: { v: STORAGE_VERSION, backup: '{"from":"elsewhere"}' } });
    announce([ROOT_KEY]);
    await settle();

    expect(chosen(m.container)).toBe("Anglais");
  });
});

/**
 * The host's translator, a test double: every request it was asked and the languages warmed, its
 * answers given by the spec through `answer` — none until then, as an engine still working.
 */
function fakeTranslator(ready: (language: string) => boolean = () => true) {
  const requests: TranslationRequest[] = [];
  const warmed: string[] = [];
  const pending: ((result: TranslationResult) => void)[] = [];
  const port = {
    translate: vi.fn((request: TranslationRequest) => {
      requests.push(request);
      return new Promise<TranslationResult>((resolve) => pending.push(resolve));
    }),
    warm: vi.fn((language: string) => void warmed.push(language)),
  };
  const source: TranslatorSource = (language) => (ready(language) ? port : null);
  /** Answer the `index`th request with `sentence`, its marks as given. */
  const answer = (index: number, sentence: string, marks = [{ start: 0, end: 0 }]): void =>
    pending[index]({ kind: "translated", translation: { sentence, marks } });
  return { source, requests, warmed, answer };
}

const EXPRESSION: FakeCard = {
  headword: "such as those made",
  surface: "such as those made",
  sentence: "Rules such as those made here hold.",
  gloss: null,
};

describe("Révision — a card the pack cannot answer (add-lingua-review-translation)", () => {
  it("asks its sentence translated the moment it comes up, the expression's place marked", async () => {
    const t = fakeTranslator();
    const m = mount(makeFakePort([EXPRESSION]).port, fakeArea(), { translator: t.source });
    await m.page.refresh();

    button(m.container, "Réviser").click();
    await settle();

    expect(t.requests).toEqual([
      { sentence: "Rules such as those made here hold.", selection: { start: 6, end: 24 }, language: "en" },
    ]);
    expect(m.container.querySelector(".review-translation, .review-translating")).toBeNull(); // not before the reveal
  });

  it("asks a glossless card captured with its full stop with a span that ends before it (fix-lingua-selection-card-punctuation D4)", async () => {
    const t = fakeTranslator();
    const stood: FakeCard = {
      headword: "stood in silence.",
      surface: "stood in silence.",
      sentence: "They stood in silence. Then they left.",
      gloss: null,
    };
    const m = mount(makeFakePort([stood]).port, fakeArea(), { translator: t.source });
    await m.page.refresh();

    button(m.container, "Réviser").click();
    await settle();

    expect(t.requests).toEqual([
      {
        sentence: "They stood in silence. Then they left.",
        selection: { start: 5, end: 5 + "stood in silence".length },
        language: "en",
      },
    ]);
  });

  it("warms the route of the review's language when a session starts", async () => {
    const t = fakeTranslator();
    const m = mount(makeFakePort(DECK).port, fakeArea(), { translator: t.source });
    await m.page.refresh();

    button(m.container, "Réviser").click();
    await settle();

    expect(t.warmed).toEqual(["en"]);
  });

  it("shows a translation that came before the reveal as part of the answer", async () => {
    const t = fakeTranslator();
    const m = mount(makeFakePort([EXPRESSION]).port, fakeArea(), { translator: t.source });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    t.answer(0, "Des règles comme celles faites ici tiennent.", [{ start: 11, end: 34 }]);
    await settle();
    expect(m.container.querySelector(".review-translation")).toBeNull(); // still hidden

    button(m.container, "Afficher la réponse").click();
    await settle();

    expect(text(m.container, ".review-translation-label")).toBe("Dans votre phrase — traduction automatique");
    expect(text(m.container, ".review-translation mark")).toBe("comme celles faites ici");
    expect(m.container.querySelector(".review-no-gloss")).toBeNull();
  });

  it("says a translation is coming at the reveal, then puts it in that line's place", async () => {
    const t = fakeTranslator();
    const m = mount(makeFakePort([EXPRESSION]).port, fakeArea(), { translator: t.source });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();
    expect(text(m.container, ".review-translating")).toBe("Traduction en cours…");

    t.answer(0, "Des règles comme celles faites ici tiennent.");
    await settle();

    expect(m.container.querySelector(".review-translating")).toBeNull();
    expect(text(m.container, ".review-translation-sentence")).toBe("Des règles comme celles faites ici tiennent.");
  });

  it("keeps the no-translation note when none comes within the bound", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    const t = fakeTranslator();
    const m = mount(makeFakePort([EXPRESSION]).port, fakeArea(), { translator: t.source });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await vi.advanceTimersByTimeAsync(0);
    button(m.container, "Afficher la réponse").click();
    await vi.advanceTimersByTimeAsync(0);

    await vi.advanceTimersByTimeAsync(TRANSLATION_WAIT_MS);

    expect(m.container.querySelector(".review-translating")).toBeNull();
    expect(text(m.container, ".review-no-gloss")).toBe("Pas de traduction pour cette expression.");
  });

  it("does not ask for a card with a gloss, which shows its gloss alone", async () => {
    const t = fakeTranslator();
    const m = mount(makeFakePort(DECK).port, fakeArea(), { translator: t.source });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();

    expect(t.requests).toEqual([]);
    expect(text(m.container, ".review-gloss")).toBe("rarement");
    expect(m.container.querySelector(".review-translation, .review-translating")).toBeNull();
  });

  it("is today's review without a translator, or without a model for the card's language", async () => {
    for (const translator of [undefined, fakeTranslator(() => false).source]) {
      const m = mount(makeFakePort([EXPRESSION]).port, fakeArea(), { translator });
      await m.page.refresh();
      button(m.container, "Réviser").click();
      await settle();
      button(m.container, "Afficher la réponse").click();
      await settle();

      expect(m.container.querySelector(".review-translation, .review-translating")).toBeNull();
      expect(text(m.container, ".review-no-gloss")).toBe("Pas de traduction pour cette expression.");
    }
  });

  it("translates alone an expression its sentence does not hold, under the label that says so", async () => {
    const t = fakeTranslator();
    const card: FakeCard = { ...EXPRESSION, sentence: "Some other sentence entirely." };
    const m = mount(makeFakePort([card]).port, fakeArea(), { translator: t.source });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    t.answer(0, "comme celles faites", []);
    button(m.container, "Afficher la réponse").click();
    await settle();

    expect(t.requests).toEqual([{ sentence: "such as those made", selection: null, language: "en" }]);
    expect(text(m.container, ".review-translation-label")).toBe("Traduction automatique");
  });

  it("never translates a single word alone", async () => {
    const t = fakeTranslator();
    const card: FakeCard = { headword: "thole", surface: "thole", sentence: "Nothing of it here.", gloss: null };
    const m = mount(makeFakePort([card]).port, fakeArea(), { translator: t.source });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    expect(t.requests).toEqual([]);
  });

  it("asks in the card's own language, a Spanish card included", async () => {
    packs.shipped = ["en-fr", "es-fr"];
    const t = fakeTranslator();
    const card: FakeCard = {
      headword: "sin embargo",
      surface: "sin embargo",
      sentence: "Llegó tarde, sin embargo.",
      gloss: null,
      language: "es",
    };
    const { port } = makeFakePort([card]);
    port.studiedLanguages = async () => ["es"];
    const m = mount(port, fakeArea(), { translator: t.source });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    expect(t.warmed).toEqual(["es"]);
    expect(t.requests[0]).toEqual({
      sentence: "Llegó tarde, sin embargo.",
      selection: { start: 13, end: 24 },
      language: "es",
    });
  });

  it("shows an answer that lands after its card was answered nowhere", async () => {
    const t = fakeTranslator();
    const other: FakeCard = {
      ...EXPRESSION,
      headword: "in spite of",
      surface: "in spite of",
      sentence: "in spite of it",
    };
    const m = mount(makeFakePort([EXPRESSION, other]).port, fakeArea(), { translator: t.source });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();
    button(m.container, "Su").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();

    t.answer(0, "LATE");
    await settle();

    expect(m.container.textContent).not.toContain("LATE");
    expect(text(m.container, ".review-translating")).toBe("Traduction en cours…"); // the second card's own
  });
});

/**
 * The host's « Traduction en ligne », a test double (add-lingua-review-translation-fallback D2):
 * whether it is offered, every request it sent, its answers given by the spec through `answer`.
 */
function fakeOnline(offered = true) {
  const requests: OnlineTranslationRequest[] = [];
  const pending: ((result: OnlineTranslationResult) => void)[] = [];
  const source: ReviewOnlineSource = {
    offered: () => offered,
    translate: (request) => {
      requests.push(request);
      return new Promise<OnlineTranslationResult>((resolve) => pending.push(resolve));
    },
  };
  const answer = (index: number, result: OnlineTranslationResult): void => pending[index](result);
  return { source, requests, answer };
}

const SLACKEN: FakeCard = {
  headword: "slacken",
  surface: "slacken",
  sentence: "He felt his grip slacken.",
  gloss: null,
};

/** Mount on `deck`, start a session and reveal its first card. */
async function revealed(deck: FakeCard[], host: Host) {
  const fake = makeFakePort(deck);
  const m = mount(fake.port, fakeArea(), host);
  await m.page.refresh();
  button(m.container, "Réviser").click();
  await settle();
  button(m.container, "Afficher la réponse").click();
  await settle();
  return { ...m, calls: fake.calls };
}

const has = (container: HTMLElement, label: string): boolean =>
  [...container.querySelectorAll("button")].some((b) => b.textContent === label);

describe("Révision — online translation, « Traduire la phrase » and keeping (add-lingua-review-translation-fallback)", () => {
  it("offers the button on a card without a gloss only when the setting and the access say so", async () => {
    for (const offered of [true, false]) {
      const online = fakeOnline(offered);
      const m = await revealed([SLACKEN], { online: online.source });
      expect(has(m.container, "Traduction en ligne"), String(offered)).toBe(offered);
      expect(online.requests).toEqual([]); // nothing leaves the device before the press
    }
  });

  // (card, what is sent — or null for no button)
  const SENT: [string, FakeCard, OnlineTranslationRequest | null][] = [
    [
      "a single word, in its sentence, marked by position",
      SLACKEN,
      { sentence: "He felt his grip slacken.", selection: { start: 17, end: 24 }, source: "en" },
    ],
    [
      "an expression its sentence does not hold, alone",
      { ...EXPRESSION, sentence: "Some other sentence entirely." },
      { sentence: "such as those made", selection: { start: 0, end: 18 }, source: "en" },
    ],
    ["a single word its sentence does not hold: no button", { ...SLACKEN, sentence: "Nothing here." }, null],
  ];

  it("sends the card's sentence with the word's span, never the word alone", async () => {
    for (const [name, card, sent] of SENT) {
      const online = fakeOnline();
      const m = await revealed([card], { online: online.source });
      expect(has(m.container, "Traduction en ligne"), name).toBe(sent !== null);
      if (sent) button(m.container, "Traduction en ligne").click();
      await settle();
      expect(online.requests, name).toEqual(sent ? [sent] : []);
    }
  });

  it("sends once per card, shows the answer in place of the engine's, marked, labelled online", async () => {
    const online = fakeOnline();
    const t = fakeTranslator();
    const m = await revealed([SLACKEN], { online: online.source, translator: t.source });
    const b = button(m.container, "Traduction en ligne");
    b.click();
    b.click();
    await settle();
    expect(text(m.container, ".review-online-note")).toBe("Traduction en ligne…");
    online.answer(0, {
      kind: "translated",
      translation: { sentence: "Il sentit sa prise se relâcher.", marks: [{ start: 19, end: 30 }] },
    });
    await settle();
    t.answer(0, "LATE ENGINE"); // the engine's, later: the online answer stays
    await settle();

    expect(online.requests).toHaveLength(1);
    expect(text(m.container, ".review-translation-label")).toBe("Dans votre phrase — traduction automatique en ligne");
    expect(text(m.container, ".review-translation mark")).toBe("se relâcher");
    expect(m.container.textContent).not.toContain("LATE ENGINE");
    expect(button(m.container, "Traduction en ligne").disabled).toBe(true);
  });

  // (the service's answer, the note)
  const REFUSED: [OnlineTranslationResult, string][] = [
    [{ kind: "unavailable" }, "Traduction en ligne indisponible pour le moment."],
    [{ kind: "limit", resetsAt: Date.UTC(2026, 9, 15, 0, 0) }, "Limite quotidienne atteinte — de nouveau disponible à"],
  ];

  it("reads « unavailable » and the daily limit as the selection card does, keeping what it shows", async () => {
    for (const [result, note] of REFUSED) {
      const online = fakeOnline();
      const m = await revealed([SLACKEN], { online: online.source });
      button(m.container, "Traduction en ligne").click();
      await settle();
      online.answer(0, result);
      await settle();
      expect(text(m.container, ".review-online-note")).toContain(note);
      expect(text(m.container, ".review-no-gloss")).toBe("Pas de traduction pour ce mot.");
    }
  });

  it("drops an online answer that lands after its card was answered", async () => {
    const online = fakeOnline();
    const m = await revealed([SLACKEN, { ...SLACKEN, headword: "thole", surface: "thole", sentence: "thole it" }], {
      online: online.source,
    });
    button(m.container, "Traduction en ligne").click();
    await settle();
    button(m.container, "Su").click();
    await settle();
    online.answer(0, { kind: "translated", translation: { sentence: "LATE", marks: [] } });
    await settle();
    expect(m.container.textContent).not.toContain("LATE");
  });

  // (engine ready, online offered, the link shown)
  const LINK: [boolean, boolean, boolean][] = [
    [true, false, true],
    [false, true, true],
    [false, false, false],
  ];

  it("A word the pack glosses: the link only where the engine or online can answer, nothing asked", async () => {
    for (const [ready, offered, shown] of LINK) {
      const t = fakeTranslator(() => ready);
      const online = fakeOnline(offered);
      const m = await revealed(DECK, { translator: t.source, online: online.source });
      expect(has(m.container, "Traduire la phrase"), `${ready} ${offered}`).toBe(shown);
      expect(has(m.container, "Traduction en ligne")).toBe(false); // not before the tap
      expect(t.requests).toEqual([]);
    }
  });

  it("A useless gloss: the tap asks the engine with the word marked and shows the online button", async () => {
    const t = fakeTranslator();
    const online = fakeOnline();
    const would: FakeCard = { headword: "would", surface: "would", sentence: "I would go.", gloss: "Subjonctif passé" };
    const m = await revealed([would], { translator: t.source, online: online.source });
    button(m.container, "Traduire la phrase").click();
    await settle();
    expect(t.requests).toEqual([{ sentence: "I would go.", selection: { start: 2, end: 7 }, language: "en" }]);
    expect(has(m.container, "Traduire la phrase")).toBe(false);
    expect(has(m.container, "Traduction en ligne")).toBe(true);
    t.answer(0, "J'irais.", [{ start: 0, end: 7 }]);
    await settle();
    expect(text(m.container, ".review-gloss")).toBe("Subjonctif passé");
    expect(text(m.container, ".review-translation mark")).toBe("J'irais");
  });

  it("Replacing a useless gloss: kept through the port, shown at once, saved, the action gone", async () => {
    const t = fakeTranslator();
    const would: FakeCard = {
      headword: "would",
      surface: "would",
      sentence: "He would go.",
      gloss: "Subjonctif passé",
    };
    const m = await revealed([would], { translator: t.source });
    button(m.container, "Traduire la phrase").click();
    await settle();
    t.answer(0, "Il aurait dit.", [{ start: 3, end: 9 }]);
    await settle();
    button(m.container, "Garder « aurait »").click();
    await settle();

    expect(m.calls.kept).toEqual([{ lemma: "would", gloss: "aurait", language: "en" }]);
    expect(text(m.container, ".review-gloss")).toBe("aurait");
    expect(m.container.querySelector(".review-keep")).toBeNull();
    expect(m.container.querySelector(".review-translation")).not.toBeNull(); // the translation it came from stays
    expect(m.area.raw).toHaveProperty(ROOT_KEY); // the backup was saved
  });
});

describe("Révision — a card nothing can translate (add-lingua-review-translation-fallback D12)", () => {
  const SETTINGS = "Activez la traduction étendue dans les réglages pour traduire cette carte.";
  const WORD = "Pas de traduction pour ce mot.";

  // (case, card, engine ready, online offered, the note)
  const CASES: [string, FakeCard, boolean, boolean, string][] = [
    ["nothing can translate a word", SLACKEN, false, false, `${WORD} ${SETTINGS}`],
    [
      "nothing can translate an expression",
      EXPRESSION,
      false,
      false,
      `Pas de traduction pour cette expression. ${SETTINGS}`,
    ],
    ["online offered: the button is the way", SLACKEN, false, true, WORD],
    [
      "a single word its sentence does not hold: nothing would help",
      { ...SLACKEN, sentence: "Nothing here." },
      false,
      false,
      WORD,
    ],
  ];

  it("says how to get a translation only when nothing can translate the card, and something could", async () => {
    for (const [name, card, ready, offered, note] of CASES) {
      const asked: string[] = [];
      const t = fakeTranslator(() => ready);
      const m = await revealed([card], {
        translator: t.source,
        online: fakeOnline(offered).source,
        translationHint: (language) => (asked.push(language), inSettings()),
      });
      expect(text(m.container, ".review-no-gloss"), name).toBe(note);
      if (note !== WORD) expect(asked, name).toContain("en");
    }
  });

  it("a model ready: the translation comes, and an engine with nothing to say leaves the note alone", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    const t = fakeTranslator();
    const m = mount(makeFakePort([SLACKEN]).port, fakeArea(), { translator: t.source, translationHint: inSettings });
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await vi.advanceTimersByTimeAsync(0);
    button(m.container, "Afficher la réponse").click();
    await vi.advanceTimersByTimeAsync(0);
    expect(m.container.querySelector(".review-no-gloss")).toBeNull();
    expect(text(m.container, ".review-translating")).toBe("Traduction en cours…");

    await vi.advanceTimersByTimeAsync(TRANSLATION_WAIT_MS);

    expect(text(m.container, ".review-no-gloss")).toBe(WORD);
  });

  it("a glossed card, or a host without a hint: nothing changes", async () => {
    const glossed = await revealed(DECK, { translationHint: inSettings });
    expect(glossed.container.querySelector(".review-no-gloss")).toBeNull();
    expect(glossed.container.textContent).not.toContain(SETTINGS);
    const bare = await revealed([SLACKEN], {});
    expect(text(bare.container, ".review-no-gloss")).toBe(WORD);
  });

  it("the app's download link re-offers the download, on the reader's tap only", async () => {
    const download = vi.fn();
    const m = await revealed([SLACKEN], { translationHint: () => ({ kind: "download", download }) });
    expect(text(m.container, ".review-no-gloss")).toBe(`${WORD} Téléchargez la traduction pour l'app`);
    expect(download).not.toHaveBeenCalled();
    button(m.container, "Téléchargez la traduction pour l'app").click();
    expect(download).toHaveBeenCalledOnce();
  });
});

// add-lingua-chapter-recap D6: beside a book, the words looked up in it still to sort.
describe("Révision — the words of the book beside, to sort", () => {
  it("shows the count beside a book and opens the book's list", async () => {
    const { port } = makeFakePort();
    const openToSort = vi.fn();
    let n = 6;
    const m = mount(port, fakeArea(), { toSort: async () => n, openToSort });
    await m.page.refresh();
    const count = m.container.querySelector<HTMLButtonElement>(".review-to-sort")!;
    expect(count.hidden).toBe(false);
    expect(count.textContent).toBe("6 mots de ce livre à trier");
    count.click();
    expect(openToSort).toHaveBeenCalledTimes(1);
    n = 1;
    await m.page.refresh();
    expect(count.textContent).toBe("1 mot de ce livre à trier");
  });

  it("Nothing to sort: no count, nor away from a book, nor when the host cannot say", async () => {
    const { port } = makeFakePort();
    for (const toSort of [async () => 0, undefined, async () => Promise.reject(new Error("gone"))]) {
      const m = mount(port, fakeArea(), { toSort });
      await m.page.refresh();
      const count = m.container.querySelector<HTMLButtonElement>(".review-to-sort")!;
      expect(count.hidden).toBe(true);
      expect(count.textContent).toBe("");
      count.click(); // no list to open: nothing happens
    }
  });
});
