import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LinguaPort, StatusOp } from "@/analyzer/port.ts";
import type { CefrLevel, LevelRow, SeedOrder } from "@/analyzer/types.ts";
import { utcDay } from "@/state/dailystats.ts";
import { type AsyncStorageArea, ROOT_KEY, STORAGE_VERSION } from "@/state/storage.ts";
import { mountStats } from "@/stats/view.ts";
import { makeFakePort } from "./helpers.ts";

// The Statistiques view as a WHOLE — the mounting the side panel and the drawer share.
// The ladder's markup, the chart and the model are pinned in stats.spec.ts; what is
// pinned here is what the view asks the port and the background, what it writes back,
// and what it rebuilds after each action.

const NOW_MS = Date.UTC(2026, 8, 17, 12, 0, 0);
const DAY = utcDay(NOW_MS);
const DAILY_KEY = "cymbra-lingua-daily";

/** A count as the view prints it (French grouping), so the assertions are locale-proof. */
const fr = (n: number): string => n.toLocaleString("fr-FR");

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

/** A full A1→C2 ladder whose first level holds `confirmed` of its 100 words. */
function ladder(confirmed: number): LevelRow[] {
  const levels: CefrLevel[] = ["A1", "A2", "B1", "B2", "C1", "C2"];
  return levels.map((level, i) => ({
    level,
    confirmed: i === 0 ? confirmed : 0,
    presumed: 0,
    toLearn: 100 - (i === 0 ? confirmed : 0),
    total: 100,
  }));
}

const op = (lemma: string, status: string, provenance: string, updated_at = 100): StatusOp => ({
  language: "en",
  lemma,
  status,
  provenance,
  updated_at,
});

/** A port with CEFR data, over which each test states only what it cares about. */
function levelledPort(over: Partial<LinguaPort> = {}): LinguaPort {
  const { port } = makeFakePort();
  return {
    ...port,
    hasLevels: async () => true,
    levelLadder: async () => ladder(80),
    vocabularyEstimate: async () => ({ estimated: 15_823, confirmed: 120, universe: 25_009, basis: "level" }),
    ...over,
  };
}

/** Stand in for the background: `account:state` and `stats:get` both come through it. */
function stubRuntime(reply: (message: unknown) => unknown): unknown[] {
  const sent: unknown[] = [];
  vi.stubGlobal("chrome", {
    runtime: {
      sendMessage: async (message: unknown) => {
        sent.push(message);
        return reply(message);
      },
    },
  });
  return sent;
}

const SIGNED_OUT = (): null => null;

let root: HTMLElement;

beforeEach(() => {
  document.body.replaceChildren();
  root = document.createElement("div");
  document.body.append(root);
  vi.spyOn(Date, "now").mockReturnValue(NOW_MS);
  stubRuntime(SIGNED_OUT);
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

function pick<T extends HTMLElement>(selector: string): T {
  const el = root.querySelector<T>(selector);
  if (!el) throw new Error(`no ${selector}`);
  return el;
}

const has = (selector: string): boolean => root.querySelector(selector) !== null;

/** The total printed on the card carrying `label`. */
function total(label: string): string {
  const card = [...root.querySelectorAll(".card")].find((c) => c.querySelector(".mlabel")?.textContent === label);
  if (!card) throw new Error(`no card "${label}"`);
  return card.querySelector(".mtotal")?.textContent ?? "";
}

function rangeButton(label: string): HTMLButtonElement {
  const found = [...root.querySelectorAll<HTMLButtonElement>(".ranges button")].find((b) => b.textContent === label);
  if (!found) throw new Error(`no range "${label}"`);
  return found;
}

/** Let the view's floating promises (a click handler is fire-and-forget) settle. */
const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

describe("Statistiques — the view", () => {
  it("renders the ladder, the vocabulary estimate and one card per metric", async () => {
    const area = fakeArea();
    await area.set({ [DAILY_KEY]: { [DAY]: { exposures: 4, wordsLearned: 2, reviews: 1 } } });

    await mountStats(root, levelledPort({ declaredLevel: async () => "A2" }), area);

    expect(pick(".vocab-n").textContent).toContain(fr(16_000)); // rounded: it is an estimate
    expect(root.querySelectorAll(".ladder-row:not(.ladder-cols)")).toHaveLength(6); // A1→C2
    expect(pick(".ladder-row--here .ladder-lvl").textContent).toBe("A2"); // the declared level
    expect(pick(".ladder-pos").textContent).toContain("A1"); // the frontier the ladder implies
    expect(root.querySelectorAll(".card")).toHaveLength(3);
    expect(total("Mots rencontrés")).toBe("4");
    expect(total("Mots appris")).toBe("2");
    expect(total("Révisions")).toBe("1");
    expect(pick(".scope").textContent).toBe("Cet appareil");
  });

  it("says so, and offers no level seeding, when the pack carries no CEFR data", async () => {
    const { port } = makeFakePort(); // hasLevels() is false by default

    await mountStats(root, port, fakeArea());

    expect(pick(".ladder-slot").textContent).toContain("Niveaux CEFR indisponibles");
    expect(has("#seed-go")).toBe(false);
    expect(root.querySelectorAll(".card")).toHaveLength(3); // the counters stand on their own
  });

  it("starts the seeding control on the level the reader declared", async () => {
    await mountStats(root, levelledPort({ declaredLevel: async () => "C1" }), fakeArea());

    expect(pick<HTMLSelectElement>("#seed-level").value).toBe("C1");
    expect(pick<HTMLInputElement>("#seed-count").max).toBe("50"); // the engine's own cap
  });

  it("seeds the chosen level, saves the deck and re-reads the ladder", async () => {
    const seeds: [CefrLevel, number, SeedOrder, number][] = [];
    let known = 80;
    const area = fakeArea();
    const port = levelledPort({
      levelLadder: async () => ladder(known),
      seedLevel: async (level, count, order, at) => {
        seeds.push([level, count, order, at]);
        known += count;
        return count;
      },
    });
    await mountStats(root, port, area);
    const firstBand = ".ladder-row:not(.ladder-cols) .ladder-frac";
    const before = pick(firstBand).textContent;

    pick<HTMLSelectElement>("#seed-level").value = "B1";
    pick<HTMLInputElement>("#seed-count").value = "5";
    pick<HTMLSelectElement>("#seed-order").value = "rare";
    pick("#seed-go").click();
    await settle();

    expect(seeds).toEqual([["B1", 5, "rare", Math.floor(NOW_MS / 1000)]]); // the engine wants seconds
    expect(area.raw[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "{}" });
    expect(pick("#seed-result").hidden).toBe(false);
    expect(pick("#seed-result").textContent).toBe("5 cartes ajoutées au deck (niveau B1).");
    expect(pick(firstBand).textContent).not.toBe(before); // the ladder followed the seed
  });

  it("keeps the count the reader types inside what the engine accepts", async () => {
    // A number field hands back anything: more than the cap, a blank (the browser wipes
    // whatever it could not parse), a decimal, or a zero. Every one of them is clamped
    // into 1…50 rather than reaching the engine.
    const counts: number[] = [];
    const port = levelledPort({
      seedLevel: async (_level, count) => {
        counts.push(count);
        return count;
      },
    });
    await mountStats(root, port, fakeArea());
    const field = pick<HTMLInputElement>("#seed-count");

    for (const typed of ["999", "", "3.7", "0"]) {
      field.value = typed;
      pick("#seed-go").click();
      await settle();
    }

    expect(counts).toEqual([50, 1, 3, 1]);
  });

  it("names one card in the singular, and says plainly when nothing was added", async () => {
    const port = levelledPort({ seedLevel: async () => 1 });
    await mountStats(root, port, fakeArea());
    pick("#seed-go").click();
    await settle();
    expect(pick("#seed-result").textContent).toBe("1 carte ajoutée au deck (niveau A1).");

    const other = document.createElement("div");
    document.body.append(other);
    await mountStats(other, levelledPort({ seedLevel: async () => 0 }), fakeArea());
    other.querySelector<HTMLButtonElement>("#seed-go")?.click();
    await settle();

    expect(other.querySelector("#seed-result")?.textContent).toContain("Aucune carte ajoutée");
  });

  it("opens the reader's own decisions and folds the automatic confirmations", async () => {
    // The decisions stay short and scannable; the reading/review confirmations grow with
    // use, so they sit behind a count and build no rows until they are opened.
    const port = levelledPort({
      exportStatusOps: async () => [
        op("city", "known", "manual", 300),
        op("seldom", "ignored", "manual", 200),
        op("cat", "known", "exposure", 400),
        op("nuance", "known", "srs", 500),
      ],
    });

    await mountStats(root, port, fakeArea());

    const groups = [...root.querySelectorAll<HTMLDetailsElement>(".marked-group")];
    expect(groups.map((g) => g.querySelector(".marked-summary-label")?.textContent)).toEqual([
      "Mes décisions",
      "Confirmés par la lecture",
      "Validés en révision",
    ]);
    expect(groups[0].open).toBe(true);
    expect([...groups[0].querySelectorAll(".marked-word")].map((w) => w.textContent)).toEqual(["city", "seldom"]);
    expect([...groups[0].querySelectorAll(".marked-badge")].map((b) => b.textContent)).toEqual(["connu", "ignoré"]);
    expect(groups[1].open).toBe(false);
    expect(groups[1].querySelector(".marked-count")?.textContent).toBe("1");
    expect(groups[1].querySelectorAll(".marked-row")).toHaveLength(0); // not built while folded
    expect(root.textContent ?? "").not.toMatch(/lemm/i); // never a word for the dictionary form
  });

  it("builds a folded section's rows the first time it is opened", async () => {
    const port = levelledPort({ exportStatusOps: async () => [op("cat", "known", "exposure")] });
    await mountStats(root, port, fakeArea());
    const reading = root.querySelectorAll<HTMLDetailsElement>(".marked-group")[0];

    reading.open = true;
    reading.dispatchEvent(new Event("toggle"));

    expect([...reading.querySelectorAll(".marked-word")].map((w) => w.textContent)).toEqual(["cat"]);
    expect(reading.querySelectorAll(".marked-badge")).toHaveLength(0); // one origin, no badge
    expect(reading.textContent).toContain("Sous ton niveau et lus plusieurs jours différents");
  });

  it("puts a marked word back to learning, saves it and re-reads list and ladder", async () => {
    let ops = [op("city", "known", "manual", 300), op("seldom", "ignored", "manual", 200)];
    const cleared: [string, string | null, number][] = [];
    let ladderReads = 0;
    const area = fakeArea();
    const port = levelledPort({
      levelLadder: async () => {
        ladderReads += 1;
        return ladder(80);
      },
      exportStatusOps: async () => ops,
      setStatusAt: async (lemma, status, atMs) => {
        cleared.push([lemma, status, atMs]);
        ops = ops.filter((o) => o.lemma !== lemma);
      },
    });
    await mountStats(root, port, area);
    const reads = ladderReads;

    pick<HTMLButtonElement>(".marked-undo").click();
    await settle();

    expect(cleared).toEqual([["city", null, NOW_MS]]); // cleared, not re-marked
    expect(area.raw[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "{}" });
    expect([...root.querySelectorAll(".marked-word")].map((w) => w.textContent)).toEqual(["seldom"]);
    expect(ladderReads).toBeGreaterThan(reads); // the word counts again, so the ladder moved
  });

  it("remembers which sections the reader folded when the list is rebuilt", async () => {
    // An undo re-renders the whole list; folding "Mes décisions" would otherwise spring
    // back open under the reader's hands on the very next click.
    let ops = [op("city", "known", "manual", 300), op("seldom", "ignored", "manual", 200)];
    const port = levelledPort({
      exportStatusOps: async () => ops,
      setStatusAt: async (lemma) => {
        ops = ops.filter((o) => o.lemma !== lemma);
      },
    });
    await mountStats(root, port, fakeArea());
    const decisions = pick<HTMLDetailsElement>(".marked-group");

    decisions.open = false;
    decisions.dispatchEvent(new Event("toggle"));
    decisions.querySelector<HTMLButtonElement>(".marked-undo")?.click();
    await settle();

    expect(pick<HTMLDetailsElement>(".marked-group").open).toBe(false);
    expect(pick(".marked-count").textContent).toBe("1"); // the count still tells the truth
  });

  it("says when nothing has been marked at all", async () => {
    await mountStats(root, levelledPort(), fakeArea()); // exportStatusOps is empty by default

    expect(pick(".marked-slot").textContent).toContain("Aucun mot marqué");
    expect(has(".marked-group")).toBe(false);
  });

  it("re-reads the counters over the window the reader picks", async () => {
    const area = fakeArea();
    await area.set({
      [DAILY_KEY]: {
        [DAY]: { exposures: 4, wordsLearned: 0, reviews: 0 },
        [DAY - 10]: { exposures: 6, wordsLearned: 0, reviews: 0 }, // inside 30 j, outside 7 j
      },
    });
    await mountStats(root, levelledPort(), area);
    expect(total("Mots rencontrés")).toBe("10");

    rangeButton("7 j").click();
    await settle();

    expect(total("Mots rencontrés")).toBe("4");
    expect(rangeButton("7 j").classList.contains("active")).toBe(true);
    expect(rangeButton("30 j").classList.contains("active")).toBe(false);
  });

  it("prefers the reader's every-device counters once signed in", async () => {
    const sent = stubRuntime((message) => {
      const type = (message as { type: string }).type;
      if (type === "account:state") return { state: { signedIn: true } };
      return { ok: true, rows: [{ day: DAY, exposures: 12, wordsLearned: 3, reviews: 4 }] };
    });
    const area = fakeArea();
    await area.set({ [DAILY_KEY]: { [DAY]: { exposures: 4, wordsLearned: 2, reviews: 1 } } });

    await mountStats(root, levelledPort(), area);

    expect(pick(".scope").textContent).toBe("Tous tes appareils");
    expect(total("Mots rencontrés")).toBe("12"); // the server's figure, not this device's
    expect(sent[1]).toEqual({ type: "stats:get", fromDay: DAY - 29, toDay: DAY });
  });

  it("falls back on this device when the consolidated counters do not come", async () => {
    const area = fakeArea();
    await area.set({ [DAILY_KEY]: { [DAY]: { exposures: 4, wordsLearned: 0, reviews: 0 } } });
    stubRuntime((message) =>
      (message as { type: string }).type === "account:state" ? { state: { signedIn: true } } : { ok: false },
    );

    await mountStats(root, levelledPort(), area);

    expect(pick(".scope").textContent).toBe("Cet appareil");
    expect(total("Mots rencontrés")).toBe("4");
  });

  it("still renders where no background answers at all (the drawer, on a dead page)", async () => {
    vi.stubGlobal("chrome", {
      runtime: {
        sendMessage: async () => {
          throw new Error("Could not establish connection");
        },
      },
    });
    const area = fakeArea();
    await area.set({ [DAILY_KEY]: { [DAY]: { exposures: 7, wordsLearned: 0, reviews: 0 } } });

    await mountStats(root, levelledPort(), area);

    expect(pick(".scope").textContent).toBe("Cet appareil");
    expect(total("Mots rencontrés")).toBe("7");
  });
});
