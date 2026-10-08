import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { StatusOp } from "@/analyzer/port.ts";
import type { CefrLevel, LevelRow } from "@/analyzer/types.ts";
import { drawer as enDrawer } from "@/i18n/en/drawer.ts";
import { stats as enStats } from "@/i18n/en/stats.ts";
import { stats as esStats } from "@/i18n/es/stats.ts";
import {
  COPY_PENDING_ATTR,
  fillPageInLanguage,
  formatNumber,
  INTERFACE_LANGUAGE_KEY,
  type InterfaceLanguage,
  plural,
} from "@/i18n/index.ts";
import { Drawer } from "@/reading/drawer.ts";
import { mountReview } from "@/review/review-page.ts";
import type { ReviewView } from "@/review/session.ts";
import { renderReview } from "@/review/view.ts";
import { utcDay } from "@/state/dailystats.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import { start as startPanel } from "@/sidepanel/panel.ts";
import { ladderView, vocabularyView } from "@/stats/ladder.ts";
import { start as startStatsPage } from "@/stats/page.ts";
import { mountStats, statsCopy } from "@/stats/view.ts";
import {
  type FakeCard,
  makeFakePort,
  type FakePort,
  PENDING_RULE,
  pageArea,
  refusingArea,
  REVEAL_KEYFRAMES,
} from "./helpers.ts";

// Review and statistics speak the interface language (localise-lingua-review-stats): the review's
// view and page, the statistics' view, ladder and page, handed the language by their hosts — what
// `view`, `review-page`, `stats` and `stats-view` assert in French, unchanged, asserted here in
// English and Spanish — and the statistics' own page, filled from the catalogue as change 14's are.

// The bundle's pairs are a build-time constant: the pairs the surfaces consult are set per test.
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

/** A narrow no-break space, the RAE's (and fr-FR's) separator between groups of three digits. */
const NNBSP = " ";
const NBSP = " ";
const NOW_MS = Date.UTC(2026, 8, 17, 12, 0, 0);
const NOW_SECONDS = Math.floor(NOW_MS / 1000);
const DAY = utcDay(NOW_MS);
const DAILY_KEY = "cymbra-lingua-daily-v3";

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

/** Let the surfaces' floating promises (every handler is fire-and-forget) settle. */
const settle = async (): Promise<void> => {
  for (let i = 0; i < 3; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

const buttons = (root: ParentNode): string[] => [...root.querySelectorAll("button")].map((b) => b.textContent ?? "");

function button(root: ParentNode, label: string): HTMLButtonElement {
  const found = [...root.querySelectorAll<HTMLButtonElement>("button")].find((b) => b.textContent === label);
  if (!found) throw new Error(`no button "${label}"`);
  return found;
}

const text = (root: ParentNode, selector: string): string => root.querySelector(selector)?.textContent ?? "";

beforeEach(() => {
  packs.shipped = ["en-fr"];
  document.body.replaceChildren();
  vi.spyOn(Date, "now").mockReturnValue(NOW_MS);
  vi.stubGlobal("chrome", {
    runtime: { sendMessage: vi.fn(async () => null), onMessage: { addListener: () => {} } },
    storage: { local: { get: async () => ({}), set: async () => {} }, onChanged: { addListener: () => {} } },
  });
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
  document.documentElement.querySelectorAll("[data-cymbra-lingua-skip]").forEach((el) => el.remove());
});

describe("An English-native reader's review", () => {
  const actions = { start: vi.fn(), reveal: vi.fn(), grade: vi.fn(), markKnown: vi.fn() };
  const card = (over: Partial<NonNullable<ReviewView["card"]>> = {}): NonNullable<ReviewView["card"]> => ({
    headword: "faro",
    surface: "faro",
    sentence: "El faro brilla.",
    gloss: "lighthouse",
    revealed: false,
    remaining: 3,
    language: "es",
    ...over,
  });
  const english = { interfaceLanguage: "en" as const };

  it("three cards read « 3 cards to review », one card « 1 card to review »", () => {
    const root = document.createElement("div");
    renderReview(root, { phase: "reviewing", card: card() }, actions, english);
    expect(text(root, ".remaining")).toBe("3 cards to review");
    renderReview(root, { phase: "reviewing", card: card({ remaining: 1 }) }, actions, english);
    expect(text(root, ".remaining")).toBe("1 card to review");
    // A count the language groups: English through en-US, where the French writes the bare figure.
    renderReview(root, { phase: "reviewing", card: card({ remaining: 1234 }) }, actions, english);
    expect(text(root, ".remaining")).toBe("1,234 cards to review");
  });

  it("names the start, the answer, the grades and the mark in English, the card's words in their language", () => {
    const root = document.createElement("div");
    renderReview(root, { phase: "idle", card: null }, actions, english);
    expect(buttons(root)).toEqual(["Review"]);
    renderReview(root, { phase: "done", card: null }, actions, english);
    expect(root.textContent).toBe("Nothing to review for now.");
    renderReview(root, { phase: "reviewing", card: card() }, actions, english);
    expect(buttons(root)).toEqual(["Show the answer"]);

    renderReview(root, { phase: "reviewing", card: card({ revealed: true }) }, actions, english);
    expect(buttons(root)).toEqual(["Again", "Hard", "Good", "Easy", "I know it ✓"]);
    button(root, "Good").click();
    expect(actions.grade).toHaveBeenCalledWith("good");
    expect(text(root, ".review-sentence")).toBe("“El faro brilla.”");
    // The document's words keep the studied language's lang (localise-lingua-reading-surfaces D3).
    expect(root.querySelector(".review-headword")?.getAttribute("lang")).toBe("es");
    expect(root.querySelector(".review-sentence span")?.getAttribute("lang")).toBe("es");
    expect(root.querySelector(".review-sentence span")?.textContent).toBe("El faro brilla.");
  });

  it("a Spanish-native reader counts in Spanish", () => {
    const root = document.createElement("div");
    const spanish = { interfaceLanguage: "es" as const };
    renderReview(root, { phase: "reviewing", card: card({ remaining: 1 }) }, actions, spanish);
    expect(text(root, ".remaining")).toBe("1 tarjeta por repasar");
    renderReview(root, { phase: "reviewing", card: card({ remaining: 12_345 }) }, actions, spanish);
    expect(text(root, ".remaining")).toBe(`12${NNBSP}345 tarjetas por repasar`);
  });

  const DECK: FakeCard[] = [
    { headword: "seldom", surface: "seldom", sentence: "They seldom ship.", gloss: "rarely" },
    { headword: "dwell", surface: "dwells", sentence: "It dwells in the north.", gloss: "to live" },
    { headword: "thrive", surface: "thrives", sentence: "It thrives.", gloss: "to flourish" },
  ];

  function mount(port: FakePort): { container: HTMLElement; refresh: () => Promise<void> } {
    const container = document.createElement("div");
    document.body.append(container);
    const page = mountReview(container, port, fakeArea(), {
      now: () => NOW_SECONDS,
      prefs: fakeArea(),
      interfaceLanguage: "en",
    });
    return { container, refresh: page.refresh };
  }

  it("the page: the summary, the counts as the session runs, the tools and the sources in English", async () => {
    const { port } = makeFakePort(DECK);
    const m = mount(port);
    await m.refresh();
    expect(text(m.container, ".summary")).toBe("3 cards · 0 to review");
    expect(buttons(m.container.querySelector(".review")!)).toEqual(["Review"]);
    expect(buttons(m.container.querySelector(".tools")!)).toEqual(["Back up", "Restore"]);
    expect(text(m.container, "details summary")).toBe("Sources & privacy");
    expect(text(m.container, ".privacy")).toBe("Nothing leaves your device: analysis and translations are local.");
    await settle();
    expect(m.container.querySelector("details")?.textContent).toContain("Sources: L1");

    button(m.container, "Review").click();
    await settle();
    expect(text(m.container, ".remaining")).toBe("3 cards to review");
    for (const left of ["2 cards to review", "1 card to review"]) {
      button(m.container, "Show the answer").click();
      await settle();
      button(m.container, "Good").click();
      await settle();
      expect(text(m.container, ".remaining")).toBe(left);
    }
    expect(text(m.container, ".summary")).toBe("3 cards · 1 to review");
  });

  it("the page: each count of the summary in bold, grouped as English groups it, in its own form", async () => {
    const { port } = makeFakePort();
    const m = mount({ ...port, deckCount: async () => 1, dueCount: async () => 12_345 });
    await m.refresh();
    expect(text(m.container, ".summary")).toBe("1 card · 12,345 to review");
    expect([...m.container.querySelectorAll(".summary b")].map((b) => b.textContent)).toEqual(["1", "12,345"]);
  });

  it("the page: the language filter and the languages are named in English (add-lingua-native-language-labels)", async () => {
    packs.shipped = ["en-fr", "es-fr"];
    const { port } = makeFakePort([...DECK, { ...DECK[0], headword: "faro", language: "es" }]);
    await port.setStudiedLanguages(["en", "es"]);
    const m = mount(port);
    await m.refresh();
    const filter = m.container.querySelector(".review-languages");
    expect(filter?.getAttribute("aria-label")).toBe("Language");
    expect([...(filter?.querySelectorAll("button") ?? [])].map((b) => b.textContent)).toEqual(["English", "Spanish"]);
  });

  it("the page: a restored backup and a file that is not one, said in English", async () => {
    const { port } = makeFakePort(DECK);
    const choose = async (container: HTMLElement, contents: string): Promise<void> => {
      const input = container.querySelector<HTMLInputElement>('input[type="file"]')!;
      const file = new File([contents], "backup.json", { type: "application/json" });
      // jsdom's Blob predates `Blob.text()`: the file answers it directly.
      Object.defineProperty(file, "text", { value: async () => contents });
      Object.defineProperty(input, "files", { value: [file], configurable: true });
      input.dispatchEvent(new Event("change"));
      await settle();
    };
    const good = mount(port);
    await good.refresh();
    await choose(good.container, "{}");
    expect(text(good.container, ".msg")).toBe("Backup restored.");

    const bad = mount({
      ...port,
      restore: async () => {
        throw new Error("unknown version");
      },
    });
    await bad.refresh();
    await choose(bad.container, "not json");
    expect(text(bad.container, ".msg")).toBe("Backup file not recognized.");
  });
});

/** A full A1→C2 ladder: bands growing past ten thousand, so the RAE's grouping shows. */
function ladder(): LevelRow[] {
  const levels: CefrLevel[] = ["A1", "A2", "B1", "B2", "C1", "C2"];
  return levels.map((level, i) => ({
    level,
    confirmed: i === 0 ? 900 : 0,
    presumed: 0,
    toLearn: 0,
    total: i === 0 ? 1_000 : 2_500 * i,
    typicalVocabulary: [1_285, 3_312, 7_904, 15_823, 19_950, 21_004][i],
  }));
}

const op = (lemma: string, status: string, provenance: string, updated_at = 100): StatusOp => ({
  language: "en",
  lemma,
  status,
  provenance,
  updated_at,
});

function levelledPort(over: Partial<FakePort> = {}): FakePort {
  const { port } = makeFakePort();
  return {
    ...port,
    hasLevels: async () => true,
    declaredLevel: async () => "A2",
    levelLadder: async () => ladder(),
    vocabularyEstimate: async () => ({ estimated: 15_823, confirmed: 12_345, universe: 25_009, basis: "level" }),
    exportStatusOps: async () => [op("city", "known", "manual", 300), op("seldom", "ignored", "manual", 200)],
    ...over,
  };
}

describe("A Spanish-native reader's statistics", () => {
  let root: HTMLElement;

  beforeEach(() => {
    root = document.createElement("div");
    document.body.append(root);
  });

  /** The total on the card labelled `label`. */
  function total(label: string): string {
    const card = [...root.querySelectorAll(".card")].find((c) => c.querySelector(".mlabel")?.textContent === label);
    if (!card) throw new Error(`no card "${label}"`);
    return card.querySelector(".mtotal")?.textContent ?? "";
  }

  async function today(): Promise<Area> {
    const area = fakeArea();
    await area.set({ [DAILY_KEY]: { [DAY]: { en: { exposures: 12_345, wordsLearned: 5_000, reviews: 3 } } } });
    return area;
  }

  it("the counters are the Spanish catalogue's, their totals grouped as the RAE writes them", async () => {
    await mountStats(root, levelledPort(), await today(), undefined, "es");

    expect([...root.querySelectorAll(".card .mlabel")].map((l) => l.textContent)).toEqual([
      "Palabras leídas",
      "Palabras aprendidas",
      "Repasos",
    ]);
    expect(total("Palabras leídas")).toBe(`12${NNBSP}345`);
    expect(total("Palabras aprendidas")).toBe("5000"); // no grouping below ten thousand
    expect(total("Repasos")).toBe("3");
    expect(root.querySelector(".card svg")?.getAttribute("aria-label")).toBe("Palabras leídas");
    expect(text(root, ".scope")).toBe("Este dispositivo");
    expect([...root.querySelectorAll(".ranges button")].map((b) => b.textContent)).toEqual(["7 d", "30 d", "90 d"]);
    expect(text(root, ".topline + .cards + .note")).toBe(esStats.agentNote);
  });

  it("the ladder's notes are the Spanish catalogue's, its numbers grouped as the RAE writes them", async () => {
    await mountStats(root, levelledPort(), fakeArea(), undefined, "es");

    expect(text(root, ".vocab .mlabel")).toBe("Vocabulario estimado");
    expect(text(root, ".vocab-n")).toBe(`≈${NBSP}16${NNBSP}000 palabras`);
    expect(text(root, ".vocab .note")).toBe(
      `Según tu nivel declarado y tus palabras marcadas, extrapolado tramo de frecuencia por tramo, ` +
        `de las 25${NNBSP}009 palabras del diccionario (12${NNBSP}345 de ellas confirmadas).`,
    );
    expect(text(root, ".ladder-pos")).toBe("nivel estimado A2"); // A1 is cleared at 90 %
    expect(text(root, ".ladder-pos b")).toBe("A2");
    expect([...root.querySelectorAll(".ladder-cols span")].map((s) => s.textContent)).toEqual([
      "",
      "",
      "este nivel",
      "enseñadas",
      "estimadas",
    ]);
    const rows = [...root.querySelectorAll(".ladder-row:not(.ladder-cols)")];
    expect(rows.map((r) => r.querySelector(".ladder-frac")?.textContent)).toEqual([
      "900 / 1000",
      "0 / 2500",
      "0 / 5000",
      "0 / 7500",
      `0 / 10${NNBSP}000`,
      `0 / 12${NNBSP}500`,
    ]);
    expect(rows.map((r) => r.querySelector(".ladder-cum")?.textContent)).toEqual([
      "1000",
      "3500",
      "8500",
      `16${NNBSP}000`,
      `26${NNBSP}000`,
      `38${NNBSP}500`,
    ]);
    expect(rows.map((r) => r.querySelector(".ladder-est")?.textContent)).toEqual([
      `≈${NBSP}1300`,
      `≈${NBSP}3300`,
      `≈${NBSP}7900`,
      `≈${NBSP}16${NNBSP}000`,
      `≈${NBSP}20${NNBSP}000`,
      `≈${NBSP}21${NNBSP}000`,
    ]);
    expect(text(root, ".ladder-legend")).toBe(esStats.legend);
    expect(text(root, ".ladder-scope")).toBe(esStats.scopeTaught + esStats.scopeTypical(esStats.extrapolated));
    // The ladder's title names the language in Spanish (add-lingua-native-language-labels D2).
    expect(text(root, ".ladder-head .mlabel")).toBe("Mi nivel de inglés");
  });

  it("the seeding control, its result and the marked words speak Spanish", async () => {
    const added: number[] = [5, 1, 0];
    await mountStats(root, levelledPort({ seedLevel: async () => added.shift() ?? 0 }), fakeArea(), undefined, "es");

    expect(text(root, ".seed .mlabel")).toBe("Reforzar un nivel");
    expect(root.querySelector("#seed-level")?.getAttribute("aria-label")).toBe("Nivel");
    expect(root.querySelector("#seed-count")?.getAttribute("aria-label")).toBe("Número de palabras");
    expect(root.querySelector("#seed-order")?.getAttribute("aria-label")).toBe("Orden");
    expect([...root.querySelectorAll("#seed-order option")].map((o) => o.textContent)).toEqual([
      "corrientes primero",
      "raras primero",
    ]);
    const results: string[] = [];
    for (let i = 0; i < 3; i++) {
      button(root, "Añadir al mazo").click();
      await settle();
      results.push(text(root, "#seed-result"));
    }
    expect(results).toEqual([
      "5 tarjetas añadidas al mazo (nivel A2).",
      "1 tarjeta añadida al mazo (nivel A2).",
      esStats.noCardsAdded,
    ]);

    expect(text(root, ".marked .mlabel")).toBe("Palabras marcadas");
    expect(text(root, ".marked-summary-label")).toBe("Mis decisiones");
    expect([...root.querySelectorAll(".marked-badge")].map((b) => b.textContent)).toEqual(["conocida", "ignorada"]);
    expect([...root.querySelectorAll(".marked-undo")].map((b) => b.textContent)).toEqual([
      "Volver a aprender",
      "Volver a aprender",
    ]);
  });

  it("a pack without levels says so in Spanish, the scale named as Spanish names it", async () => {
    await mountStats(root, makeFakePort().port, fakeArea(), undefined, "es");
    expect(text(root, ".ladder-slot")).toBe(
      "Niveles MCER no disponibles para este idioma (el paquete no tiene datos MCER).",
    );
    expect(text(root, ".marked .note")).toBe(esStats.noMarked);
  });

  it("keeps Spanish when the reader picks another studied language", async () => {
    packs.shipped = ["en-fr", "es-fr"];
    const port = levelledPort();
    await port.setStudiedLanguages(["en", "es"]);
    await mountStats(root, port, fakeArea(), undefined, "es");
    const picker = root.querySelector(".stats-languages");
    expect(picker?.getAttribute("aria-label")).toBe("Idioma");

    button(root, "Español").click();
    // The remount's last write is the scope: once it reads Spanish, the whole view is the new one.
    await vi.waitFor(() => {
      expect(text(root, ".stats-languages .active")).toBe("Español");
      expect(text(root, ".scope")).toBe("Este dispositivo");
    });
    expect(text(root, ".ladder-cols .ladder-frac")).toBe("este nivel");
  });

  it("every device's counters once signed in, said in Spanish", async () => {
    vi.stubGlobal("chrome", {
      runtime: {
        sendMessage: async (message: { type: string }) =>
          message.type === "account:state"
            ? { state: { signedIn: true } }
            : { ok: true, rows: [{ day: DAY, exposures: 20_000, wordsLearned: 0, reviews: 0 }] },
      },
    });
    await mountStats(root, levelledPort(), fakeArea(), undefined, "es");
    expect(text(root, ".scope")).toBe("Todos tus dispositivos");
    expect(total("Palabras leídas")).toBe(`20${NNBSP}000`);
  });
});

describe("the ladder's functions, handed the interface language", () => {
  const english = { interfaceLanguage: "en" as const };

  it("writes English's figures through en-US, and the French as before when handed nothing", () => {
    const view = ladderView(ladder(), "A2", "en", false, english);
    expect(view.querySelector(".ladder-row:not(.ladder-cols) .ladder-frac")?.textContent).toBe("900 / 1,000");
    expect([...view.querySelectorAll(".ladder-est")].at(-1)?.textContent).toBe(`≈${NBSP}21,000`);
    expect(view.querySelector(".ladder-scope")?.textContent).toBe(
      enStats.scopeTaught + enStats.scopeTypical(enStats.extrapolated),
    );
    // Handed nothing: the French module and fr-FR, the bytes the ladder always wrote.
    const french = ladderView(ladder(), "A2", "en");
    expect([...french.querySelectorAll(".ladder-cum")].at(-1)?.textContent).toBe(`38${NNBSP}500`);
    // Pinned to the bytes 6acecef0's ladder wrote, not to the French catalogue.
    expect(french.querySelector(".ladder-scope")?.textContent).toBe(
      `«${NNBSP}enseignés${NNBSP}»${NBSP}: les mots de base introduits jusqu'à ce niveau par les listes d'enseignement. ` +
        `«${NNBSP}estimés${NNBSP}»${NBSP}: le vocabulaire qu'a en général un lecteur de ce niveau, ` +
        "extrapolé des mots des niveaux inférieurs sur tout le dictionnaire.",
    );
  });

  it("counts an exact vocabulary in plural forms: one word, many words", () => {
    const exact = (n: number, language: InterfaceLanguage) =>
      vocabularyView({ estimated: n, confirmed: n, universe: 25_009, basis: "marked" }, true, {
        interfaceLanguage: language,
      })?.querySelector(".vocab-n")?.textContent;
    expect(exact(1, "en")).toBe("1 word");
    expect(exact(1_249, "en")).toBe("1,249 words");
    expect(exact(1, "es")).toBe("1 palabra");
    expect(exact(12_490, "es")).toBe(`12${NNBSP}490 palabras`);
    expect(exact(1, "fr")).toBe("1 mots"); // the French as it always was
  });

  // An estimate's figure and its confirmed words, counted in each language's plural forms; the
  // French keeps the plural it always wrote, « ≈ 1 mots », « (dont 1 confirmés) ».
  const COUNTS: Record<InterfaceLanguage, Array<[number, string, string]>> = {
    fr: [
      [0, `≈${NBSP}0 mots`, " (dont 0 confirmés)"],
      [1, `≈${NBSP}1 mots`, " (dont 1 confirmés)"],
      [2, `≈${NBSP}2 mots`, " (dont 2 confirmés)"],
      [1234, `≈${NBSP}1${NNBSP}234 mots`, ` (dont 1${NNBSP}234 confirmés)`],
    ],
    en: [
      [0, `≈${NBSP}0 words`, " (0 of them confirmed)"],
      [1, `≈${NBSP}1 word`, " (1 of them confirmed)"],
      [2, `≈${NBSP}2 words`, " (2 of them confirmed)"],
      [1234, `≈${NBSP}1,234 words`, " (1,234 of them confirmed)"],
    ],
    es: [
      [0, `≈${NBSP}0 palabras`, " (0 de ellas confirmadas)"],
      [1, `≈${NBSP}1 palabra`, " (1 de ellas confirmada)"],
      [2, `≈${NBSP}2 palabras`, " (2 de ellas confirmadas)"],
      [1234, `≈${NBSP}1234 palabras`, " (1234 de ellas confirmadas)"],
    ],
  };

  for (const language of ["fr", "en", "es"] as const) {
    it(`counts an estimate and its confirmed words in ${language}'s plural forms: 0, 1, 2, 1234`, () => {
      const copy = statsCopy(language);
      for (const [n, approx, confirmed] of COUNTS[language]) {
        const figure = formatNumber(language, n);
        expect(plural(language, n, copy.approxWords, figure), `${n}`).toBe(approx);
        expect(plural(language, n, copy.confirmedCount, figure), `${n}`).toBe(confirmed);
      }
      // Through the view: a figure the rounding leaves as it is, its confirmed words alike.
      for (const [n, approx, confirmed] of COUNTS[language].slice(1)) {
        const view = vocabularyView({ estimated: n, confirmed: n, universe: 25_009, basis: "level" }, true, {
          interfaceLanguage: language,
        });
        expect(view?.querySelector(".vocab-n")?.textContent, `${n}`).toBe(approx);
        expect(view?.querySelector(".note")?.textContent, `${n}`).toContain(`${confirmed}.`);
      }
    });
  }

  it("estimated levels with borrowed figures: the whole scope in English, the languages named in it", () => {
    // The scope's wrapper is the interface language's, and so is the clause naming the borrowed
    // figures (borrowedTypicalNote, add-lingua-native-language-labels D2): one English sentence.
    const rows = ladder().map((r) => ({ ...r, typicalFrom: "en" as const }));
    const levelsEstimated = true;
    const view = ladderView(rows, null, "es", levelsEstimated, english);
    expect(view.querySelector(".ladder-cols .ladder-cum")?.textContent).toBe("common");
    expect(view.querySelector(".ladder-scope")?.textContent).toBe(
      "“common”: the most frequent words up to this level. " +
        "“estimated”: the vocabulary a reader at this level usually has, " +
        "taken from English, whose level sizes Spanish borrows.",
    );
    expect(view.querySelector(".ladder-head .mlabel")?.textContent).toBe("My estimated Spanish level");
  });

  it("asks for what is missing in the language it is handed", () => {
    const missing = vocabularyView({ estimated: 0, confirmed: 0, universe: 9, basis: "marked" }, false, english);
    expect(missing?.textContent).toBe(enStats.vocabularyKnown + enStats.noEstimateYet(enStats.setCommonWords));
  });
});

describe("the drawer hands review and statistics the language the session handed it", () => {
  /** A drawer whose closed shadow root is opened, so the test can look inside; open on Révision. */
  function drawer(language: InterfaceLanguage | undefined): { drawer: Drawer; shadow: ShadowRoot } {
    const attach = HTMLElement.prototype.attachShadow;
    let shadow: ShadowRoot | null = null;
    const spy = vi.spyOn(HTMLElement.prototype, "attachShadow").mockImplementation(function (this: HTMLElement) {
      shadow = attach.call(this, { mode: "open" });
      return shadow;
    });
    const d = new Drawer({
      css: "",
      port: levelledPort(),
      area: fakeArea(),
      store: fakeArea(),
      now: () => NOW_SECONDS,
      onChange: async () => {},
      ...(language ? { language, copy: enDrawer } : {}),
    });
    spy.mockRestore();
    return { drawer: d, shadow: shadow! };
  }

  it("An English-native reader: Révision and Statistiques in English", async () => {
    const { drawer: d, shadow } = drawer("en");
    await d.openOn("review");
    expect(buttons(shadow.querySelector(".review")!)).toEqual(["Review"]);
    expect(text(shadow, ".summary")).toBe("0 cards · 0 to review");
    await d.openOn("stats");
    expect(text(shadow, ".scope")).toBe("This device");
    expect(text(shadow, ".seed .mlabel")).toBe("Strengthen a level");
  });

  it("built without a language, the French it was", async () => {
    const { drawer: d, shadow } = drawer(undefined);
    await d.openOn("review");
    expect(buttons(shadow.querySelector(".review")!)).toEqual(["Réviser"]);
    expect(text(shadow, ".summary")).toBe("0 carte(s) · 0 à revoir");
    await d.openOn("stats");
    expect(text(shadow, ".scope")).toBe("Cet appareil");
  });
});

describe("the statistics' page, filled from the catalogue", () => {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const HTML = readFileSync(join(root, "src/stats/stats.html"), "utf8");
  const CSS = readFileSync(join(root, "src/stats/stats-page.css"), "utf8");
  /** The sheet the side panel and the in-page surfaces share: no pending rule of the tab's. */
  const SHARED_CSS = readFileSync(join(root, "src/stats/stats.css"), "utf8");
  /** Every `data-copy` node of the page, by selector, with the text the page held before. */
  const TEXTS: Array<[string, string]> = [
    ["title", "Statistiques — Cymbra Lingua"],
    ["h1", "Statistiques d'apprentissage"],
  ];
  const page = (): Document => new DOMParser().parseFromString(HTML, "text/html");

  it("A page before its script: holds no text, and hides its body until filled", () => {
    const doc = page();
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(true);
    expect(doc.documentElement.hasAttribute("lang")).toBe(false);
    for (const [selector] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe("");
    // The tab's own sheet hides it; the sheet the side panel and the in-page surfaces share does not.
    expect(doc.querySelector('link[href="stats-page.css"]')).not.toBeNull();
    expect(CSS).toMatch(PENDING_RULE);
    expect(CSS).toMatch(REVEAL_KEYFRAMES);
    expect(SHARED_CSS).not.toContain("data-copy-pending");
    expect(SHARED_CSS).not.toContain("lingua-copy-reveal");
    const surfaceCss = readFileSync(join(root, "src/reading/surface-css.ts"), "utf8");
    const panelHtml = readFileSync(join(root, "src/sidepanel/sidepanel.html"), "utf8");
    expect(surfaceCss).not.toContain("stats-page.css");
    expect(panelHtml).not.toContain("stats-page.css");
  });

  it("Every reader today: filled as page.ts fills it, every node holds the text the page held", async () => {
    const doc = page();
    const { language } = await fillPageInLanguage(doc, pageArea(), statsCopy);
    expect(language).toBe("fr");
    for (const [selector, value] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe(value);
    expect(doc.querySelectorAll("[data-copy]").length).toBe(TEXTS.length);
    expect(doc.documentElement.lang).toBe("fr");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    // The skeleton the script drives: the root the view mounts into, inside the tab's shell.
    expect(doc.querySelector("body.stats-page main #stats-root")).not.toBeNull();
  });

  it("A storage that cannot be read: the page still shows, in French, and says so", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const doc = page();
    await fillPageInLanguage(doc, refusingArea(), statsCopy);
    for (const [selector, value] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe(value);
    expect(doc.documentElement.lang).toBe("fr");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("[Cymbra Lingua]"), expect.any(Error));
  });

  it("A Spanish-native reader: the title and the heading are the Spanish catalogue's, and the page says so", async () => {
    const doc = page();
    await fillPageInLanguage(doc, pageArea({ [INTERFACE_LANGUAGE_KEY]: "es" }), statsCopy);
    expect(doc.title).toBe("Estadísticas — Cymbra Lingua");
    expect(doc.querySelector("h1")?.textContent).toBe("Estadísticas de aprendizaje");
    expect(doc.documentElement.lang).toBe("es");
  });
});

/** A preferences area whose interface language answers only when the test says so. */
function heldPrefs(language: InterfaceLanguage): AsyncStorageArea & { answer: () => void } {
  const area = fakeArea();
  let answer = (): void => {};
  const held = new Promise<void>((resolve) => (answer = resolve));
  return {
    answer,
    async get(keys) {
      if (keys === INTERFACE_LANGUAGE_KEY) {
        await held;
        return { [INTERFACE_LANGUAGE_KEY]: language };
      }
      return area.get(keys);
    },
    set: area.set,
  };
}

/** The page `html` describes, as the global document, so the views the hosts mount land in it. */
function load(html: string): Document {
  const parsed = new DOMParser().parseFromString(html, "text/html");
  document.documentElement.replaceChildren(
    ...[...parsed.documentElement.childNodes].map((n) => document.importNode(n, true)),
  );
  for (const { name, value } of [...parsed.documentElement.attributes])
    document.documentElement.setAttribute(name, value);
  return document;
}

describe("the side panel hands review and statistics the language it read (panel.ts)", () => {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const HTML = readFileSync(join(root, "src/sidepanel/sidepanel.html"), "utf8");
  const deps = (prefs: AsyncStorageArea) => ({
    prefs,
    store: fakeArea(),
    port: levelledPort(),
    now: () => NOW_SECONDS,
  });

  afterEach(() => {
    document.documentElement.removeAttribute("lang");
    document.documentElement.removeAttribute(COPY_PENDING_ATTR);
  });

  it("An English-native reader: the page, Révision and Statistiques mount in English", async () => {
    const doc = load(HTML);
    const panel = startPanel(doc, deps(pageArea({ [INTERFACE_LANGUAGE_KEY]: "en" }) as AsyncStorageArea));
    expect(await panel.interfaceLanguage).toBe("en");
    expect(doc.documentElement.lang).toBe("en");
    const review = await panel.review(doc.getElementById("view-review")!);
    await review.refresh();
    expect(buttons(doc.querySelector("#view-review .review")!)).toEqual(["Review"]);
    expect(text(doc, "#view-review .summary")).toBe("0 cards · 0 to review");
    await panel.stats(doc.getElementById("view-stats")!);
    expect(text(doc, "#view-stats .scope")).toBe("This device");
    expect(text(doc, "#view-stats .seed .mlabel")).toBe("Strengthen a level");
  });

  it("Révision asked for before the language has answered waits for it, and is mounted once, in English", async () => {
    const doc = load(HTML);
    const prefs = heldPrefs("en");
    const panel = startPanel(doc, deps(prefs));
    const container = doc.getElementById("view-review")!;
    // A view requested first (the popup's flag, a tab clicked): the read has not answered yet.
    const first = panel.review(container);
    const second = panel.review(container);
    await settle();
    expect(container.childElementCount).toBe(0); // nothing mounted in French meanwhile
    prefs.answer();
    const page = await first;
    expect(await second).toBe(page); // one page for the panel's lifetime
    await page.refresh();
    expect(buttons(container.querySelector(".review")!)).toEqual(["Review"]);
  });

  it("Statistiques asked for before the language has answered mounts in English too", async () => {
    const doc = load(HTML);
    const prefs = heldPrefs("en");
    const panel = startPanel(doc, deps(prefs));
    const shown = panel.stats(doc.getElementById("view-stats")!);
    await settle();
    prefs.answer();
    await shown;
    expect(text(doc, "#view-stats .scope")).toBe("This device");
  });
});

describe("the statistics tab starts in the language it read (page.ts)", () => {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const HTML = readFileSync(join(root, "src/stats/stats.html"), "utf8");

  afterEach(() => {
    document.documentElement.removeAttribute("lang");
    document.documentElement.removeAttribute(COPY_PENDING_ATTR);
  });

  it("An English-native reader: the page and the view in English", async () => {
    const doc = load(HTML);
    await startStatsPage(doc, {
      prefs: pageArea({ [INTERFACE_LANGUAGE_KEY]: "en" }),
      store: fakeArea(),
      port: levelledPort(),
    });
    expect(doc.title).toBe("Statistics — Cymbra Lingua");
    expect(text(doc, "h1")).toBe("Learning statistics");
    expect(doc.documentElement.lang).toBe("en");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(text(doc, "#stats-root .scope")).toBe("This device");
    expect(text(doc, "#stats-root .vocab .mlabel")).toBe("Estimated vocabulary");
  });

  it("A language read that never answers: French after the bound, with a warning, the view mounted", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const doc = load(HTML);
    await startStatsPage(doc, {
      prefs: { get: () => new Promise(() => {}) },
      store: fakeArea(),
      port: levelledPort(),
      languageReadBoundMs: 5,
    });
    expect(doc.documentElement.lang).toBe("fr");
    expect(text(doc, "h1")).toBe("Statistiques d'apprentissage");
    expect(text(doc, "#stats-root .scope")).toBe("Cet appareil");
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("[Cymbra Lingua]"), expect.any(Error));
  });

  it("A read that fails at once is French as well, and a page without its root mounts nothing", async () => {
    vi.spyOn(console, "warn").mockImplementation(() => {});
    const doc = load('<html><head><title data-copy="pageTitle"></title></head><body></body></html>');
    await startStatsPage(doc, { prefs: refusingArea(), store: fakeArea(), port: levelledPort() });
    expect(doc.title).toBe("Statistiques — Cymbra Lingua");
    expect(doc.querySelector(".stats")).toBeNull();
  });
});
