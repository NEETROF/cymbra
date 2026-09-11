import { beforeEach, describe, expect, it } from "vitest";
import { captureSelection, MAX_SELECTION_LENGTH, sentenceAround } from "@/reading/selection.ts";

beforeEach(() => {
  document.body.innerHTML = "";
});

describe("sentenceAround", () => {
  it("returns the sentence containing the needle within the block", () => {
    document.body.innerHTML = `<p>First sentence here. They seldom ship on Friday. A third one.</p>`;
    const node = document.querySelector("p")!.firstChild!;
    expect(sentenceAround(node, "seldom")).toBe("They seldom ship on Friday.");
  });

  it("collapses whitespace and falls back to the whole block when no sentence matches", () => {
    document.body.innerHTML = `<p>one   two\n  three</p>`;
    const node = document.querySelector("p")!.firstChild!;
    expect(sentenceAround(node, "absent")).toBe("one two three");
  });

  it("returns empty for a null node", () => {
    expect(sentenceAround(null, "x")).toBe("");
  });
});

describe("captureSelection", () => {
  it("returns null when there is no selection", () => {
    window.getSelection()?.removeAllRanges();
    expect(captureSelection()).toBeNull();
  });

  it("captures a selected word with its source sentence", () => {
    document.body.innerHTML = `<p>They seldom ship on Friday.</p>`;
    const textNode = document.querySelector("p")!.firstChild!;
    const range = document.createRange();
    // Select "seldom" (chars 5–11 of "They seldom ship on Friday.").
    range.setStart(textNode, 5);
    range.setEnd(textNode, 11);
    const sel = window.getSelection()!;
    sel.removeAllRanges();
    sel.addRange(range);
    const cap = captureSelection();
    expect(cap).not.toBeNull();
    expect(cap!.text).toBe("seldom");
    expect(cap!.sentence).toBe("They seldom ship on Friday.");
  });

  it("snaps a partial selection out to whole words", () => {
    const full = "show the parity proof now";
    document.body.innerHTML = `<p>${full}</p>`;
    const textNode = document.querySelector("p")!.firstChild!;
    const range = document.createRange();
    range.setStart(textNode, full.indexOf("the")); // starts at a word boundary
    range.setEnd(textNode, full.indexOf("proof") + 3); // ends mid-word inside "proof"
    const sel = window.getSelection()!;
    sel.removeAllRanges();
    sel.addRange(range);
    expect(captureSelection()!.text).toBe("the parity proof");
  });

  it("rejects a selection longer than a phrase", () => {
    document.body.innerHTML = `<p>${"word ".repeat(60)}</p>`;
    const textNode = document.querySelector("p")!.firstChild!;
    const range = document.createRange();
    range.setStart(textNode, 0);
    range.setEnd(textNode, MAX_SELECTION_LENGTH + 10);
    const sel = window.getSelection()!;
    sel.removeAllRanges();
    sel.addRange(range);
    expect(captureSelection()).toBeNull();
  });
});
