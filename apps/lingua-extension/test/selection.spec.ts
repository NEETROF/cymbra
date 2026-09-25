import { beforeEach, describe, expect, it } from "vitest";
import {
  captureSelection,
  classifySelection,
  MAX_SELECTION_LENGTH,
  SelectionWatcher,
  sentenceAndSelection,
  sentenceForRange,
} from "@/reading/selection.ts";

beforeEach(() => {
  document.body.innerHTML = "";
});

describe("sentenceForRange", () => {
  /** A range over the first occurrence of `text` in the page's only paragraph. */
  function rangeOver(text: string, html: string): Range {
    document.body.innerHTML = html;
    const p = document.querySelector("p")!;
    const node = p.firstChild!;
    const i = (node.textContent ?? "").indexOf(text);
    const r = document.createRange();
    r.setStart(node, i);
    r.setEnd(node, i + text.length);
    return r;
  }

  it("returns the sentence the range sits in", () => {
    const r = rangeOver("seldom", `<p>First sentence here. They seldom ship on Friday. A third one.</p>`);
    expect(sentenceForRange(r)).toBe("They seldom ship on Friday.");
  });

  it("takes the sentence by POSITION, not by looking the word up in the text", () => {
    // `put` occurs inside `input` one sentence earlier: the search this replaces returned
    // "Check the input first." and the card kept that sentence.
    const r = rangeOver("put it", `<p>Check the input first. Then put it away.</p>`);
    expect(sentenceForRange(r)).toBe("Then put it away.");
  });

  it("holds both sentences when the range crosses from one into the next", () => {
    const r = rangeOver("first. Then", `<p>Check the input first. Then put it away.</p>`);
    expect(sentenceForRange(r)).toBe("Check the input first. Then put it away.");
  });

  it("collapses whitespace and falls back to the whole block when it holds no sentence end", () => {
    const r = rangeOver("two", `<p>one   two\n  three</p>`);
    expect(sentenceForRange(r)).toBe("one two three");
  });

  it("finds the sentence across inline markup inside the block", () => {
    document.body.innerHTML = `<p>First one here. They <em>seldom</em> ship on <b>Friday</b>. A third.</p>`;
    const em = document.querySelector("em")!.firstChild!;
    const r = document.createRange();
    r.setStart(em, 0);
    r.setEnd(em, 6);
    expect(sentenceForRange(r)).toBe("They seldom ship on Friday.");
  });

  it("stays in the block the range starts in when the selection leaves it", () => {
    document.body.innerHTML = `<p>They seldom ship.</p><p>Another block here.</p>`;
    const [first, second] = [...document.querySelectorAll("p")];
    const r = document.createRange();
    r.setStart(first!.firstChild!, 5);
    r.setEnd(second!.firstChild!, 7);
    expect(sentenceForRange(r)).toBe("They seldom ship.");
  });

  it("returns empty for a range in an empty block", () => {
    document.body.innerHTML = `<p></p>`;
    const r = document.createRange();
    r.selectNodeContents(document.querySelector("p")!);
    expect(sentenceForRange(r)).toBe("");
  });
});

describe("sentenceAndSelection", () => {
  /** A range over the first occurrence of `text` in the page's only paragraph. */
  function rangeOver(text: string, html: string): Range {
    document.body.innerHTML = html;
    const node = document.querySelector("p")!.firstChild!;
    const i = (node.textContent ?? "").indexOf(text);
    const r = document.createRange();
    r.setStart(node, i);
    r.setEnd(node, i + text.length);
    return r;
  }

  /** The text the offsets point at — the only thing a caller does with them. */
  const marked = ({ sentence, selection }: ReturnType<typeof sentenceAndSelection>) =>
    selection ? sentence.slice(selection.start, selection.end) : null;

  it("gives the selection's place in its sentence", () => {
    const r = rangeOver("gave up", "<p>First one. She gave up after the third attempt. Last.</p>");
    const got = sentenceAndSelection(r);
    expect(got.sentence).toBe("She gave up after the third attempt.");
    expect(marked(got)).toBe("gave up");
  });

  it("places the mark by POSITION, not by finding the words", () => {
    // `put` occurs inside `input` earlier in the same sentence; a search would mark `input`.
    const r = rangeOver("put it", "<p>Check the input and then put it away.</p>");
    const got = sentenceAndSelection(r);
    expect(got.selection).toEqual({
      start: "Check the input and then ".length,
      end: "Check the input and then put it".length,
    });
  });

  it("keeps the offsets right across collapsed whitespace and a leading blank", () => {
    const r = rangeOver("gave", "<p>   Earlier.   She   gave   up.</p>");
    const got = sentenceAndSelection(r);
    expect(got.sentence).toBe("She gave up.");
    expect(marked(got)).toBe("gave");
  });

  it("keeps the offsets right across inline markup", () => {
    document.body.innerHTML = "<p>First one. They <em>seldom</em> ship on <b>Friday</b>. A third.</p>";
    const em = document.querySelector("em")!.firstChild!;
    const r = document.createRange();
    r.setStart(em, 0);
    r.setEnd(em, 6);
    expect(marked(sentenceAndSelection(r))).toBe("seldom");
  });

  it("spans both sentences when the selection crosses into the next", () => {
    const r = rangeOver("first. Then", "<p>Check the input first. Then put it away.</p>");
    const got = sentenceAndSelection(r);
    expect(got.sentence).toBe("Check the input first. Then put it away.");
    expect(marked(got)).toBe("first. Then");
  });

  it("tightens a selection that took the blanks around a word", () => {
    const r = rangeOver(" seldom ", "<p>They seldom ship.</p>");
    expect(marked(sentenceAndSelection(r))).toBe("seldom");
  });

  it("gives no selection for an empty block", () => {
    document.body.innerHTML = "<p></p>";
    const r = document.createRange();
    r.selectNodeContents(document.querySelector("p")!);
    expect(sentenceAndSelection(r)).toEqual({ sentence: "", selection: null });
  });

  it("agrees with sentenceForRange on the sentence", () => {
    const r = rangeOver("seldom", "<p>First sentence here. They seldom ship on Friday. A third one.</p>");
    expect(sentenceAndSelection(r).sentence).toBe(sentenceForRange(r));
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

  // Safari only (dropPhraseOnLift): a finger lifting from several words drops the selection,
  // and the callout drawn over the expression card with it; a single word keeps its handles.
  describe("dropping a phrase on a finger lift", () => {
    it("drops a phrase's selection once the finger lifts, after capturing it", () => {
      select("seldom ship");
      const { w, captures } = watcher({ dropPhraseOnLift: () => true });
      w.hold();
      w.notify();
      w.release("finger");
      expect(captures).toEqual([{ kind: "phrase", text: "seldom ship" }]);
      expect(window.getSelection()!.isCollapsed).toBe(true);
    });

    it("keeps a single word selected, so its handles can still extend it", () => {
      select("seldom");
      const { w, captures } = watcher({ dropPhraseOnLift: () => true });
      w.hold();
      w.notify();
      w.release("finger");
      expect(captures).toEqual([{ kind: "word", text: "seldom" }]);
      expect(String(window.getSelection())).toBe("seldom");
    });

    it("drops a phrase the held timer already captured mid-gesture", () => {
      select("seldom ship");
      const { w, captures, settle } = watcher({ dropPhraseOnLift: () => true });
      w.hold();
      w.notify();
      settle();
      w.release("finger");
      expect(captures).toHaveLength(1);
      expect(window.getSelection()!.isCollapsed).toBe(true);
    });

    it("keeps the selection after any other lift, and wherever the option is off", () => {
      select("seldom ship");
      watcher({ dropPhraseOnLift: () => true }).w.release("other");
      expect(String(window.getSelection())).toBe("seldom ship");
      watcher().w.release("finger");
      expect(String(window.getSelection())).toBe("seldom ship");
      watcher({ dropPhraseOnLift: () => false }).w.release("finger");
      expect(String(window.getSelection())).toBe("seldom ship");
    });

    it("keeps a selection too long to be a phrase: no card was opened for it", () => {
      const long = "word ".repeat(40).trim();
      select(long, long);
      const { w, captures } = watcher({ dropPhraseOnLift: () => true });
      w.release("finger");
      expect(captures).toEqual([]);
      expect(window.getSelection()!.isCollapsed).toBe(false);
    });
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
