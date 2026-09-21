import { beforeEach, describe, expect, it } from "vitest";
import {
  captureSelection,
  classifySelection,
  MAX_SELECTION_LENGTH,
  SelectionWatcher,
  sentenceAround,
} from "@/reading/selection.ts";

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

  it("leaves the page selection exactly as the reader made it", () => {
    const full = "show the parity proof now";
    document.body.innerHTML = `<p>${full}</p>`;
    const textNode = document.querySelector("p")!.firstChild!;
    const range = document.createRange();
    range.setStart(textNode, full.indexOf("the"));
    range.setEnd(textNode, full.indexOf("proof") + 3); // ends mid-word
    const sel = window.getSelection()!;
    sel.removeAllRanges();
    sel.addRange(range);
    // The word snap is internal to the capture: re-applying it would fire selectionchange
    // again and, on iOS, tear down and redraw the platform's own callout.
    expect(captureSelection()!.text).toBe("the parity proof");
    expect(String(window.getSelection())).toBe("the parity pro");
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

describe("classifySelection", () => {
  it("calls a single word a word", () => {
    expect(classifySelection("seldom")).toBe("word");
  });

  it("calls anything holding whitespace a phrase", () => {
    expect(classifySelection("ship on Friday")).toBe("phrase");
    expect(classifySelection("ship on")).toBe("phrase");
  });

  it("calls a hyphenated compound a word: one token to the analyser, so it opens the word card", () => {
    expect(classifySelection("repo-wide")).toBe("word");
  });
});

describe("SelectionWatcher", () => {
  /** Select `text` inside a fresh paragraph and return the page Selection. */
  function select(text: string, within = `They ${text} ship on Friday.`): Selection {
    document.body.innerHTML = `<p>${within}</p>`;
    const node = document.querySelector("p")!.firstChild!;
    const range = document.createRange();
    range.setStart(node, within.indexOf(text));
    range.setEnd(node, within.indexOf(text) + text.length);
    const sel = window.getSelection()!;
    sel.removeAllRanges();
    sel.addRange(range);
    return sel;
  }

  function watcher(over: Partial<ConstructorParameters<typeof SelectionWatcher>[0]> = {}) {
    const captures: Array<{ kind: string; text: string }> = [];
    const timers: Array<() => void> = [];
    const w = new SelectionWatcher({
      onCapture: (kind, cap) => captures.push({ kind, text: cap.text }),
      setTimer: (fn) => {
        timers.push(fn);
        return timers.length;
      },
      clearTimer: (h) => {
        timers[(h as number) - 1] = () => {};
      },
      ...over,
    });
    /** Run every timer armed so far (the settle delay elapsing). */
    const settle = () => {
      const pending = timers.splice(0);
      for (const fn of pending) fn();
    };
    return { w, captures, settle, timers };
  }

  beforeEach(() => {
    window.getSelection()?.removeAllRanges();
  });

  it("captures a settled multi-word selection as a phrase", () => {
    select("seldom ship");
    const { w, captures, settle } = watcher();
    w.notify();
    expect(captures).toEqual([]); // nothing until it settles
    settle();
    expect(captures).toEqual([{ kind: "phrase", text: "seldom ship" }]);
  });

  it("captures a settled single-word selection as a word", () => {
    select("seldom");
    const { w, captures, settle } = watcher();
    w.notify();
    settle();
    expect(captures).toEqual([{ kind: "word", text: "seldom" }]);
  });

  it("coalesces a burst of events into one capture", () => {
    select("seldom ship");
    const { w, captures, settle, timers } = watcher();
    w.notify();
    w.notify();
    w.notify();
    settle();
    expect(captures).toHaveLength(1);
    expect(timers).toHaveLength(0);
  });

  it("captures immediately on a pointer lift", () => {
    select("seldom ship");
    const { w, captures } = watcher();
    w.hold();
    w.notify();
    w.release();
    expect(captures).toEqual([{ kind: "phrase", text: "seldom ship" }]);
  });

  it("captures nothing on a pointer lift that selected nothing", () => {
    const { w, captures } = watcher();
    w.hold();
    w.release();
    expect(captures).toEqual([]);
  });

  it("waits longer while the pointer is down, so a slow drag does not pop the card", () => {
    select("seldom ship");
    const delays: number[] = [];
    const { w, settle } = watcher({
      setTimer: (_fn, ms) => {
        delays.push(ms);
        return 1;
      },
    });
    w.notify();
    w.hold(); // the drag re-arms with the longer settle
    expect(delays).toEqual([350, 1200]);
    settle();
  });

  it("still captures a held selection the pointer never releases", () => {
    // A native selection-handle drag may dispatch no lift at all: the delay, not the
    // lift, is what guarantees the capture.
    select("seldom ship");
    const { w, captures, settle } = watcher();
    w.hold();
    w.notify();
    settle();
    expect(captures).toEqual([{ kind: "phrase", text: "seldom ship" }]);
  });

  it("cancels the pending capture when the selection collapses", () => {
    select("seldom ship");
    const { w, captures, settle } = watcher();
    w.notify();
    window.getSelection()!.removeAllRanges();
    w.notify();
    settle();
    expect(captures).toEqual([]);
  });

  it("ignores a selection longer than a phrase", () => {
    const long = "word ".repeat(40).trim();
    select(long, long);
    const { w, captures, settle } = watcher();
    w.notify();
    settle();
    expect(captures).toEqual([]);
  });

  it("ignores a selection inside the reader's own surfaces", () => {
    document.body.innerHTML = `<div data-cymbra-lingua-skip><p>Je connais ce mot</p></div>`;
    const node = document.querySelector("p")!.firstChild!;
    const range = document.createRange();
    range.setStart(node, 0);
    range.setEnd(node, 11);
    const sel = window.getSelection()!;
    sel.removeAllRanges();
    sel.addRange(range);
    const { w, captures, settle } = watcher();
    w.notify();
    settle();
    expect(captures).toEqual([]);
  });

  it("does not re-emit the same selection twice", () => {
    select("seldom ship");
    const { w, captures, settle } = watcher();
    w.notify();
    settle();
    w.notify(); // the platform keeps firing selectionchange while the magnifier is up
    settle();
    expect(captures).toHaveLength(1);
  });

  it("re-emits the same text once the selection has been dropped in between", () => {
    select("seldom ship");
    const { w, captures, settle } = watcher();
    w.notify();
    settle();
    window.getSelection()!.removeAllRanges();
    w.notify();
    select("seldom ship");
    w.notify();
    settle();
    expect(captures).toHaveLength(2);
  });
});
