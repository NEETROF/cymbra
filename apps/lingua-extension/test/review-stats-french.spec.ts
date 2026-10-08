import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CefrLevel, LevelRow } from "@/analyzer/types.ts";
import { mountReview } from "@/review/review-page.ts";
import type { ReviewView } from "@/review/session.ts";
import { renderReview } from "@/review/view.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import { ladderView, vocabularyView } from "@/stats/ladder.ts";
import { mountStats } from "@/stats/view.ts";
import { type FakeCard, makeFakePort } from "./helpers.ts";

// Every reader today (localise-lingua-review-stats): review and statistics in French, called as
// every host called them before the catalogue — no interface language — pinned to LITERAL strings,
// the exact bytes 6acecef0's code rendered (its no-break spaces written as escapes), never to the
// French catalogue, which could drift along with the code reading it. Every call here is one the
// pre-change code also took, so this spec passes against it unchanged.

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

/** fr-FR's separator between groups of three digits, and the no-break space before a colon. */
const NNBSP = " ";
const NBSP = " ";
const NOW_MS = Date.UTC(2026, 8, 17, 12, 0, 0);

function fakeArea(): AsyncStorageArea {
  const raw: Record<string, unknown> = {};
  return {
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

const settle = async (): Promise<void> => {
  for (let i = 0; i < 3; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};
const buttons = (root: ParentNode): string[] => [...root.querySelectorAll("button")].map((b) => b.textContent ?? "");
const text = (root: ParentNode, selector: string): string => root.querySelector(selector)?.textContent ?? "";
function button(root: ParentNode, label: string): HTMLButtonElement {
  const found = [...root.querySelectorAll<HTMLButtonElement>("button")].find((b) => b.textContent === label);
  if (!found) throw new Error(`no button "${label}"`);
  return found;
}

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
});

describe("Every reader today: the review's header and counts, in French as they were", () => {
  const actions = { start: vi.fn(), reveal: vi.fn(), grade: vi.fn(), markKnown: vi.fn() };
  const card = (over: Partial<NonNullable<ReviewView["card"]>> = {}): NonNullable<ReviewView["card"]> => ({
    headword: "seldom",
    surface: "seldom",
    sentence: "They seldom ship.",
    gloss: "rarement",
    revealed: false,
    remaining: 3,
    language: "en",
    ...over,
  });

  it("the view: its buttons, its count and its sentence", () => {
    const root = document.createElement("div");
    renderReview(root, { phase: "idle", card: null }, actions);
    expect(buttons(root)).toEqual(["Réviser"]);
    renderReview(root, { phase: "done", card: null }, actions);
    expect(root.textContent).toBe("Rien à réviser pour l'instant.");
    for (const [n, said] of [
      [0, "0 carte(s) à revoir"],
      [1, "1 carte(s) à revoir"],
      [3, "3 carte(s) à revoir"],
      [1234, "1234 carte(s) à revoir"],
    ] as const) {
      renderReview(root, { phase: "reviewing", card: card({ remaining: n }) }, actions);
      expect(text(root, ".remaining")).toBe(said);
    }
    renderReview(root, { phase: "reviewing", card: card() }, actions);
    expect(buttons(root)).toEqual(["Afficher la réponse"]);
    renderReview(root, { phase: "reviewing", card: card({ revealed: true }) }, actions);
    expect(buttons(root)).toEqual(["À revoir", "Difficile", "Correct", "Facile", "Je connais ✓"]);
    expect(text(root, ".review-sentence")).toBe("« They seldom ship. »");
  });

  const DECK: FakeCard[] = [
    { headword: "seldom", surface: "seldom", sentence: "They seldom ship.", gloss: "rarement" },
    { headword: "dwell", surface: "dwells", sentence: "It dwells in the north.", gloss: "habiter" },
    { headword: "thrive", surface: "thrives", sentence: "It thrives.", gloss: "prospérer" },
  ];

  it("the page: its summary, its tools, its sources, its filter, and the counts as a session runs", async () => {
    packs.shipped = ["en-fr", "es-fr"];
    const { port } = makeFakePort([...DECK, { ...DECK[0], headword: "faro", language: "es" }]);
    await port.setStudiedLanguages(["en", "es"]);
    const container = document.createElement("div");
    document.body.append(container);
    const page = mountReview(container, port, fakeArea(), { now: () => Math.floor(NOW_MS / 1000), prefs: fakeArea() });
    await page.refresh();
    await settle();
    expect(text(container, ".summary")).toBe("3 carte(s) · 0 à revoir");
    expect([...container.querySelectorAll(".summary b")].map((b) => b.textContent)).toEqual(["3", "0"]);
    expect(container.querySelector(".review-languages")?.getAttribute("aria-label")).toBe("Langue");
    expect(buttons(container.querySelector(".tools")!)).toEqual(["Sauvegarder", "Restaurer"]);
    expect(text(container, "details summary")).toBe("Sources & confidentialité");
    expect(text(container, ".privacy")).toBe(
      "Rien ne quitte votre appareil : l'analyse et les traductions sont locales.",
    );
    expect(container.querySelector("details")?.textContent).toContain("Sources : L1");

    button(container, "Réviser").click();
    await settle();
    expect(text(container, ".remaining")).toBe("3 carte(s) à revoir");
    button(container, "Afficher la réponse").click();
    await settle();
    button(container, "Correct").click();
    await settle();
    expect(text(container, ".remaining")).toBe("2 carte(s) à revoir");
  });
});

/** A full A1→C2 ladder, bands past ten thousand so fr-FR's grouping shows. */
function ladder(typicalFrom?: "en"): LevelRow[] {
  const levels: CefrLevel[] = ["A1", "A2", "B1", "B2", "C1", "C2"];
  return levels.map((level, i) => ({
    level,
    confirmed: i === 0 ? 900 : 0,
    presumed: 0,
    toLearn: 0,
    total: i === 0 ? 1_000 : 2_500 * i,
    typicalVocabulary: [1_285, 3_312, 7_904, 15_823, 19_950, 21_004][i],
    ...(typicalFrom ? { typicalFrom } : {}),
  }));
}

describe("Every reader today: the ladder's title and notes, in French as they were", () => {
  it("taught levels: the title, the position, the columns, the figures, the legend and the scope", () => {
    const view = ladderView(ladder(), "A2", "en");
    expect(text(view, ".ladder-head .mlabel")).toBe("Mon niveau d'anglais");
    expect(text(view, ".ladder-pos")).toBe("niveau estimé A2");
    expect([...view.querySelectorAll(".ladder-cols span")].map((s) => s.textContent)).toEqual([
      "",
      "",
      "ce niveau",
      "enseignés",
      "estimés",
    ]);
    const rows = [...view.querySelectorAll(".ladder-row:not(.ladder-cols)")];
    expect(rows.map((r) => r.querySelector(".ladder-frac")?.textContent)).toEqual([
      `900 / 1${NNBSP}000`,
      `0 / 2${NNBSP}500`,
      `0 / 5${NNBSP}000`,
      `0 / 7${NNBSP}500`,
      `0 / 10${NNBSP}000`,
      `0 / 12${NNBSP}500`,
    ]);
    expect(rows.map((r) => r.querySelector(".ladder-est")?.textContent).at(-1)).toBe(`≈${NBSP}21${NNBSP}000`);
    expect(text(view, ".ladder-legend")).toBe("Confirmés (lus / appris), présumés (sous ton niveau), à apprendre.");
    expect(text(view, ".ladder-scope")).toBe(
      `«${NNBSP}enseignés${NNBSP}»${NBSP}: les mots de base introduits jusqu'à ce niveau par les listes d'enseignement. ` +
        `«${NNBSP}estimés${NNBSP}»${NBSP}: le vocabulaire qu'a en général un lecteur de ce niveau, ` +
        "extrapolé des mots des niveaux inférieurs sur tout le dictionnaire.",
    );
  });

  it("estimated levels with borrowed figures: the title, the note and the scope", () => {
    const view = ladderView(ladder("en"), null, "es", true);
    expect(text(view, ".ladder-head .mlabel")).toBe("Mon niveau d'espagnol estimé");
    expect(text(view, ".ladder-estimate")).toBe(
      "Niveaux estimés d'après la fréquence des mots, faute de liste CEFR libre de droits pour l'espagnol.",
    );
    expect(text(view, ".ladder-cols .ladder-cum")).toBe("courants");
    expect(text(view, ".ladder-scope")).toBe(
      `«${NNBSP}courants${NNBSP}»${NBSP}: les mots les plus fréquents jusqu'à ce niveau. ` +
        `«${NNBSP}estimés${NNBSP}»${NBSP}: le vocabulaire qu'a en général un lecteur de ce niveau, ` +
        "repris de l'anglais, dont l'espagnol reprend les tailles de niveaux.",
    );
  });
});

describe("Every reader today: the vocabulary view, in French as it was", () => {
  it("an estimate from the declared level, its figure rounded and its confirmed words counted", () => {
    const view = vocabularyView({ estimated: 15_823, confirmed: 12_345, universe: 25_009, basis: "level" }, true);
    expect(text(view!, ".mlabel")).toBe("Vocabulaire estimé");
    expect(text(view!, ".vocab-n")).toBe(`≈${NBSP}16${NNBSP}000 mots`);
    expect(text(view!, ".note")).toBe(
      "D'après ton niveau déclaré et tes mots marqués, extrapolé tranche de fréquence par tranche, " +
        `sur les 25${NNBSP}009 mots du dictionnaire (dont 12${NNBSP}345 confirmés).`,
    );
  });

  it("the plural French always wrote, whatever the count: ≈ 1 mots, (dont 1 confirmés)", () => {
    for (const [n, figure] of [
      [1, "1"],
      [2, "2"],
      [1234, `1${NNBSP}234`],
    ] as const) {
      const view = vocabularyView({ estimated: n, confirmed: n, universe: 25_009, basis: "level" }, true);
      expect(text(view!, ".vocab-n"), String(n)).toBe(`≈${NBSP}${figure} mots`);
      expect(text(view!, ".note"), String(n)).toContain(`(dont ${figure} confirmés).`);
    }
    // No confirmed word: no clause at all.
    const none = vocabularyView({ estimated: 2_000, confirmed: 0, universe: 25_009, basis: "level" }, true);
    expect(text(none!, ".note")).toBe(
      "D'après ton niveau déclaré et tes mots marqués, extrapolé tranche de fréquence par tranche, " +
        `sur les 25${NNBSP}009 mots du dictionnaire.`,
    );
  });

  it("an exact count of marked words, and the hints before there is one", () => {
    const exact = vocabularyView({ estimated: 1_249, confirmed: 1_249, universe: 25_009, basis: "marked" }, true);
    expect(exact?.textContent).toBe(
      `Vocabulaire connu1${NNBSP}249 motsLes mots que tu as marqués connus, sur les 25${NNBSP}009 mots du dictionnaire.`,
    );
    const one = vocabularyView({ estimated: 1, confirmed: 1, universe: 25_009, basis: "marked" }, true);
    expect(text(one!, ".vocab-n")).toBe("1 mots");
    expect(vocabularyView({ estimated: 0, confirmed: 0, universe: 9, basis: "level" }, true)?.textContent).toBe(
      `Vocabulaire estiméTon niveau ne présume encore aucun mot${NBSP}: marque ceux que tu connais pour lancer l'estimation.`,
    );
    expect(vocabularyView({ estimated: 0, confirmed: 0, universe: 9, basis: "marked" }, true)?.textContent).toBe(
      `Vocabulaire connuPas encore d'estimation${NBSP}: déclare ton niveau dans les réglages, ou marque des mots que tu connais.`,
    );
    expect(vocabularyView({ estimated: 0, confirmed: 0, universe: 9, basis: "marked" }, false)?.textContent).toBe(
      `Vocabulaire connuPas encore d'estimation${NBSP}: règle les mots courants que tu connais dans les réglages, ou marque des mots que tu connais.`,
    );
  });
});

describe("Every reader today: the statistics' language picker, in French as it was", () => {
  it("is named « Langue », and the view it mounts stays French", async () => {
    packs.shipped = ["en-fr", "es-fr"];
    const { port } = makeFakePort();
    await port.setStudiedLanguages(["en", "es"]);
    const root = document.createElement("div");
    document.body.append(root);
    await mountStats(root, port, fakeArea());
    expect(root.querySelector(".stats-languages")?.getAttribute("aria-label")).toBe("Langue");
    expect(text(root, ".stats-languages .active")).toBe("Anglais");
    button(root, "Espagnol").click();
    await vi.waitFor(() => {
      expect(text(root, ".stats-languages .active")).toBe("Espagnol");
      expect(text(root, ".scope")).toBe("Cet appareil");
    });
    expect([...root.querySelectorAll(".ranges:not(.stats-languages) button")].map((b) => b.textContent)).toEqual([
      "7 j",
      "30 j",
      "90 j",
    ]);
  });
});
