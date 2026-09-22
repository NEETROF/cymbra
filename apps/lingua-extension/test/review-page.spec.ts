import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LinguaPort } from "@/analyzer/port.ts";
import { mountReview } from "@/review/review-page.ts";
import { loadDailyStats, utcDay } from "@/state/dailystats.ts";
import { type AsyncStorageArea, ROOT_KEY, STORAGE_VERSION } from "@/state/storage.ts";
import { STORE_CHANGED_KEY } from "@/state/store.ts";
import { type FakeCard, makeFakePort } from "./helpers.ts";

// The Révision page as a WHOLE: the summary, the FSRS widget, backup/restore, the pack's
// credits, and the reaction to a state written by another surface. The render itself is
// pinned in view.spec.ts and the session in session.spec.ts — what is pinned here is the
// mounting: what the page asks the port, what it saves, and when it rebuilds itself.

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

const blobs: Blob[] = [];
const downloads: string[] = [];
let revoked = 0;

// jsdom's Blob predates `Blob.text()`, which the page uses to read the chosen file (and
// which the download assertion reads back). FileReader IS implemented, so stand it in.
if (typeof Blob.prototype.text !== "function") {
  Blob.prototype.text = function (this: Blob): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result));
      reader.onerror = () => reject(reader.error ?? new Error("cannot read the file"));
      reader.readAsText(this);
    });
  };
}

beforeEach(() => {
  listeners.length = 0;
  blobs.length = 0;
  downloads.length = 0;
  revoked = 0;
  document.body.replaceChildren();
  vi.spyOn(Date, "now").mockReturnValue(NOW_MS);
  vi.stubGlobal("chrome", {
    storage: {
      onChanged: {
        addListener: (fn: ChangeListener) => void listeners.push(fn),
      },
    },
  });
  // jsdom implements neither blob URLs nor a navigating anchor, and the download is the
  // only thing the page does that leaves the DOM: stand in for both and record the call.
  URL.createObjectURL = (blob) => {
    blobs.push(blob as Blob);
    return "blob:lingua";
  };
  URL.revokeObjectURL = () => void (revoked += 1);
  vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (this: HTMLAnchorElement) {
    downloads.push(this.download);
  });
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

function mount(port: LinguaPort, area: Area = fakeArea()) {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const page = mountReview(container, port, area, { now: () => NOW_SECONDS });
  return { page, container, area };
}

/**
 * Let the page's floating promises settle (every handler is fire-and-forget). Several
 * rounds, because reading the chosen file goes through FileReader's own task.
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

const filePicker = (container: HTMLElement): HTMLInputElement => {
  const input = container.querySelector<HTMLInputElement>('input[type="file"]');
  if (!input) throw new Error("no file input");
  return input;
};

/** Hand the hidden picker a file, as the browser does once the reader has chosen one. */
function choose(input: HTMLInputElement, contents: string): void {
  const file = new File([contents], "cymbra-lingua-backup.json", { type: "application/json" });
  Object.defineProperty(input, "files", { value: [file], configurable: true });
  input.dispatchEvent(new Event("change"));
}

describe("Révision — the page", () => {
  it("shows the deck and what is due, asked on the injected clock", async () => {
    // The clock is a seam precisely so the page never reads Date.now() for the schedule.
    const asked: number[] = [];
    const { port } = makeFakePort();
    const m = mount({
      ...port,
      deckCount: async () => 12,
      dueCount: async (now) => {
        asked.push(now);
        return 3;
      },
    });

    await m.page.refresh();

    expect(text(m.container, ".summary")).toBe("12 carte(s) · 3 à revoir");
    expect(asked).toEqual([NOW_SECONDS]);
    expect(button(m.container, "Réviser")).toBeTruthy();
  });

  it("starts a session and hands the first card over without its answer", async () => {
    const m = mount(makeFakePort(DECK).port);
    await m.page.refresh();

    button(m.container, "Réviser").click();
    await settle();

    expect(text(m.container, ".review-headword")).toBe("seldom");
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
    expect(text(m.container, ".review-sentence")).toBe("« They seldom ship. »");
  });

  it("grades a card, saves the state and counts the review for the day", async () => {
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();

    button(m.container, "Correct").click();
    await settle();

    expect(calls.grades).toEqual(["good"]);
    // A grade changes the engine, so the whole state is written back for every surface.
    expect(m.area.raw[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "{}" });
    expect((await loadDailyStats(m.area))[utcDay(NOW_MS)]).toEqual({ exposures: 0, wordsLearned: 0, reviews: 1 });
    expect(text(m.container, ".review-headword")).toBe("dwell"); // moved on to the next card
  });

  it("marks a card known, counting a word learned rather than a review", async () => {
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();
    button(m.container, "Afficher la réponse").click();
    await settle();

    button(m.container, "Je connais ✓").click();
    await settle();

    expect(calls.markKnown).toBe(1);
    expect(m.area.raw[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "{}" });
    expect((await loadDailyStats(m.area))[utcDay(NOW_MS)]).toEqual({ exposures: 0, wordsLearned: 1, reviews: 0 });
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
    button(m.container, "Correct").click();
    await settle();

    expect(m.page.reviewing()).toBe(false);
  });

  it("hands the whole state over as a JSON file named for the app", async () => {
    const m = mount(makeFakePort(DECK).port);
    await m.page.refresh();
    vi.useFakeTimers(); // the page releases the blob URL on a timer a second later

    button(m.container, "Sauvegarder").click();
    await vi.runAllTimersAsync();
    vi.useRealTimers();

    expect(downloads).toEqual(["cymbra-lingua-backup.json"]);
    expect(blobs).toHaveLength(1);
    expect(blobs[0].type).toBe("application/json");
    expect(await blobs[0].text()).toBe("{}"); // the engine's own backup, byte for byte
    expect(revoked).toBe(1);
  });

  it("asks for a file through the hidden picker", async () => {
    const m = mount(makeFakePort(DECK).port);
    await m.page.refresh();
    const input = filePicker(m.container);
    const clicked = vi.spyOn(input, "click").mockImplementation(() => {});

    button(m.container, "Restaurer").click();

    expect(clicked).toHaveBeenCalledOnce();
    expect(input.hidden).toBe(true);
    expect(input.accept).toBe("application/json");
  });

  it("does nothing when the reader closes the picker without choosing", async () => {
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();

    filePicker(m.container).dispatchEvent(new Event("change"));
    await settle();

    expect(calls.restored).toEqual([]);
    expect(text(m.container, ".msg")).toBe("");
  });

  it("restores a chosen backup, saves it and starts the page over from it", async () => {
    // The restored engine is a different deck, so the controller is rebuilt: any session
    // under way is dropped and the page goes back to its start button.
    const { port, calls } = makeFakePort(DECK);
    const m = mount(port);
    await m.page.refresh();
    button(m.container, "Réviser").click();
    await settle();

    choose(filePicker(m.container), '{"v":2,"deck":[]}');
    await settle();

    expect(calls.restored).toEqual(['{"v":2,"deck":[]}']);
    expect(m.area.raw[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: '{"v":2,"deck":[]}' });
    expect(text(m.container, ".msg")).toBe("Sauvegarde restaurée.");
    expect(m.page.reviewing()).toBe(false);
    expect(button(m.container, "Réviser")).toBeTruthy();
  });

  it("keeps the page as it was when the file is not a backup", async () => {
    const { port } = makeFakePort(DECK);
    const m = mount({
      ...port,
      restore: async () => {
        throw new Error("unknown version");
      },
    });
    await m.page.refresh();

    choose(filePicker(m.container), "not json at all");
    await settle();

    expect(text(m.container, ".msg")).toBe("Fichier de sauvegarde non reconnu.");
    expect(m.area.raw[ROOT_KEY]).toBeUndefined(); // nothing was written over the state
  });

  it("credits the pack's sources and shows its notice", async () => {
    const m = mount(makeFakePort(DECK).port);

    await settle(); // the credits are loaded on mount, not on refresh

    expect(m.container.querySelector("details")?.textContent).toContain("Sources : L1");
    expect(text(m.container, ".notice")).toBe("NOTICE");
    expect(text(m.container, ".privacy")).toContain("Rien ne quitte votre appareil");
  });

  it("leaves the credit line out for a pack that names no source", async () => {
    const { port } = makeFakePort(DECK);
    const m = mount({ ...port, licences: async () => [] });

    await settle();

    expect(m.container.querySelector("details")?.textContent).not.toContain("Sources :");
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
    button(m.container, "Correct").click();
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
    expect(text(m.container, ".review-headword")).toBe("seldom"); // the card is still there
  });
});
