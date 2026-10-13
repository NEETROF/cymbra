import { describe, expect, it, vi } from "vitest";
import { COPY, readerCopy } from "@/reader/copy.ts";
import type { LookupRecord } from "@/reader/library.ts";
import {
  groupByChapter,
  markedSentence,
  type RecapActions,
  type RecapRow,
  type RecapView,
  renderRecap,
} from "@/reader/recap.ts";

// The recap sheet (add-lingua-chapter-recap D5): one row per word — the form met, its gloss, its
// sentence with the word marked — and « + Deck », « Je connais », « Laisser »; text nodes only.

function row(lemma: string, chapter = "III", over: Partial<LookupRecord> = {}): RecapRow {
  return {
    lookup: {
      hash: "h",
      chapter,
      lemma,
      surface: lemma,
      sentence: `His ${lemma} fell.`,
      language: "en",
      expression: false,
      gloss: null,
      at: 0,
      ...over,
    },
    gloss: `gloss of ${lemma}`,
  };
}

function sheet(groups = [{ chapter: "III", rows: [row("countenance"), row("moor")] }], over: Partial<RecapView> = {}) {
  const container = document.createElement("div");
  const actions = {
    add: vi.fn(async () => {}),
    know: vi.fn(async () => {}),
    close: vi.fn(),
  } satisfies RecapActions;
  renderRecap(
    container,
    { title: "Fin de « III »", intro: COPY.recapIntro, groups, headed: false, closeLabel: COPY.recapLater, ...over },
    COPY,
    actions,
  );
  const forms = () => [...container.querySelectorAll(".recap-form")].map((b) => b.textContent);
  const press = (index: number, action: string) =>
    container.querySelectorAll<HTMLButtonElement>(`.recap-row .recap-${action}`)[index].click();
  return { container, actions, forms, press };
}

const settle = () => new Promise((r) => setTimeout(r, 0));

describe("the recap sheet", () => {
  it("shows each word as met, its gloss and its sentence, with the three actions in the catalogue's words", () => {
    const { container, forms } = sheet();
    expect(container.getAttribute("role")).toBe("dialog");
    expect(container.querySelector(".recap-title")?.textContent).toBe("Fin de « III »");
    expect(forms()).toEqual(["countenance", "moor"]);
    expect(container.querySelector(".recap-gloss")?.textContent).toBe("gloss of countenance");
    const first = container.querySelector(".recap-row")!;
    expect([...first.querySelectorAll("button")].map((b) => b.textContent)).toEqual([
      "+ Deck",
      "Je connais",
      "Laisser",
    ]);
    expect(container.querySelector(".recap-close")?.textContent).toBe("Plus tard");
    expect(container.querySelector(".recap-chapter")).toBeNull();
  });

  it("speaks the interface language", () => {
    const en = readerCopy("en");
    const container = document.createElement("div");
    renderRecap(
      container,
      {
        title: en.recapAll,
        intro: en.recapAllIntro,
        groups: [{ chapter: "", rows: [row("moor", "")] }],
        headed: true,
        closeLabel: en.recapClose,
      },
      en,
      { add: async () => {}, know: async () => {}, close: () => {} },
    );
    expect([...container.querySelectorAll(".recap-row button")].map((b) => b.textContent)).toEqual([
      "+ Deck",
      "I know it",
      "Skip",
    ]);
    expect(container.querySelector(".recap-chapter")?.textContent).toBe(en.recapNoChapter);
  });

  it("writes the book's text as text, never as markup", () => {
    const { container } = sheet([
      { chapter: "III", rows: [row("moor", "III", { sentence: "<img src=x onerror=alert(1)> the moor <b>!</b>" })] },
    ]);
    const sentence = container.querySelector(".recap-sentence")!;
    expect(sentence.querySelector("img, b")).toBeNull();
    expect(sentence.querySelector("mark")?.textContent).toBe("moor");
    expect(sentence.textContent).toBe("<img src=x onerror=alert(1)> the moor <b>!</b>");
  });

  it("A card added: the row's word is handed over, and its row leaves", async () => {
    const { actions, forms, press } = sheet();
    press(0, "add");
    await settle();
    expect(actions.add).toHaveBeenCalledWith(
      expect.objectContaining({ lookup: expect.objectContaining({ lemma: "countenance" }) }),
    );
    expect(forms()).toEqual(["moor"]);
  });

  it("A status set: « Je connais » hands the word over, and the last row closes the sheet", async () => {
    const { actions, forms, press } = sheet([{ chapter: "III", rows: [row("moor")] }]);
    press(0, "known");
    await settle();
    expect(actions.know).toHaveBeenCalledTimes(1);
    expect(forms()).toEqual([]);
    expect(actions.close).toHaveBeenCalledTimes(1);
  });

  it("answers a row once, and keeps it when the engine fails", async () => {
    const { actions, forms, press } = sheet();
    actions.add.mockRejectedValueOnce(new Error("engine"));
    press(0, "add");
    press(0, "add");
    await settle();
    expect(actions.add).toHaveBeenCalledTimes(1);
    expect(forms()).toEqual(["countenance", "moor"]);
    press(0, "add");
    await settle();
    expect(forms()).toEqual(["moor"]);
  });

  it("« Laisser » decides nothing; the chapter's heading goes with its last row", () => {
    const { container, actions, forms, press } = sheet(
      [
        { chapter: "I", rows: [row("moor", "I")] },
        { chapter: "II", rows: [row("fog", "II")] },
      ],
      { headed: true },
    );
    expect([...container.querySelectorAll(".recap-chapter")].map((h) => h.textContent)).toEqual(["I", "II"]);
    press(0, "leave");
    expect(actions.add).not.toHaveBeenCalled();
    expect(actions.know).not.toHaveBeenCalled();
    expect(forms()).toEqual(["fog"]);
    expect([...container.querySelectorAll(".recap-chapter")].map((h) => h.textContent)).toEqual(["II"]);
    container.querySelector<HTMLButtonElement>(".recap-close")!.click();
    expect(actions.close).toHaveBeenCalledTimes(1);
  });
});

describe("the rows of the recap", () => {
  it("are gathered under their chapter, in the order the chapters were met", () => {
    const groups = groupByChapter([row("a", "II"), row("b", "I"), row("c", "II")]);
    expect(groups.map((g) => [g.chapter, g.rows.map((r) => r.lookup.lemma)])).toEqual([
      ["II", ["a", "c"]],
      ["I", ["b"]],
    ]);
  });

  it("mark the word met in its sentence, whatever its case; the sentence alone when it is not there", () => {
    expect(markedSentence("The Moor was wide.", "moor")).toEqual(["The ", "Moor", " was wide."]);
    expect(markedSentence("Nothing here.", "moor")).toBeNull();
    expect(markedSentence("Anything.", "")).toBeNull();
    const { container } = sheet([{ chapter: "", rows: [row("moor", "", { sentence: "Nothing here." })] }]);
    const sentence = container.querySelector(".recap-sentence")!;
    expect(sentence.querySelector("mark")).toBeNull();
    expect(sentence.textContent).toBe("Nothing here.");
  });
});
