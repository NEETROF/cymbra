import { describe, expect, it } from "vitest";
import { escapeText, markSelection, readMarked, selectedText } from "@/translate/markup.ts";

describe("escapeText", () => {
  it("leaves ordinary text alone", () => {
    expect(escapeText("They seldom ship on Friday.")).toBe("They seldom ship on Friday.");
  });

  it("turns every markup character into text", () => {
    expect(escapeText("a < b && c > d")).toBe("a &lt; b &amp;&amp; c &gt; d");
  });
});

describe("selectedText", () => {
  const sentence = "She gave up after the third attempt.";

  it("is the text markSelection tags, clamped the same way", () => {
    expect(selectedText(sentence, { start: 4, end: 11 })).toBe("gave up");
    expect(selectedText(sentence, { start: 22, end: 999 })).toBe("third attempt.");
  });

  it("is nothing when the selection covers nothing", () => {
    expect(selectedText(sentence, null)).toBeNull();
    expect(selectedText(sentence, { start: 4, end: 4 })).toBeNull();
    expect(selectedText(sentence, { start: 50, end: 60 })).toBeNull();
  });
});

describe("markSelection", () => {
  const sentence = "She gave up after the third attempt.";
  const at = (fragment: string) => ({
    start: sentence.indexOf(fragment),
    end: sentence.indexOf(fragment) + fragment.length,
  });

  it("tags the selection inside its sentence", () => {
    expect(markSelection(sentence, at("gave up"))).toBe("She <b>gave up</b> after the third attempt.");
  });

  it("escapes page text on BOTH sides of the tag and inside it", () => {
    // Page content that happens to hold markup characters must not change the request's
    // structure: a `<b>` in the page would otherwise read as a second mark.
    const s = "Use <b> & <i> for x < y.";
    const sel = { start: s.indexOf("&"), end: s.indexOf("&") + 1 };
    expect(markSelection(s, sel)).toBe("Use &lt;b&gt; <b>&amp;</b> &lt;i&gt; for x &lt; y.");
  });

  it("gives an untagged sentence when there is no selection to mark", () => {
    expect(markSelection(sentence, null)).toBe(sentence);
  });

  it("gives no tag for an empty selection rather than an empty mark", () => {
    expect(markSelection(sentence, { start: 4, end: 4 })).toBe(sentence);
  });

  it("clamps a selection that runs past the sentence", () => {
    expect(markSelection("Hello world", { start: 6, end: 99 })).toBe("Hello <b>world</b>");
    expect(markSelection("Hello world", { start: -3, end: 5 })).toBe("<b>Hello</b> world");
  });
});

describe("readMarked", () => {
  // The four answers the engine actually gave in the spike, in its own markup.
  it("finds a fragment conjugated by its context", () => {
    const r = readMarked("Elle <b>a abandonné</b> après la troisième tentative.");
    expect(r.sentence).toBe("Elle a abandonné après la troisième tentative.");
    expect(r.marks.map((m) => r.sentence.slice(m.start, m.end))).toEqual(["a abandonné"]);
  });

  it("finds three source words collapsed onto one target word", () => {
    const r = readMarked("Il a dû <b>supporter</b> le bruit pendant une semaine.");
    expect(r.marks.map((m) => r.sentence.slice(m.start, m.end))).toEqual(["supporter"]);
  });

  it("finds the mark in a sentence the engine reordered", () => {
    const r = readMarked("Ils sont <b>rarement</b> livrés vendredi.");
    expect(r.marks.map((m) => r.sentence.slice(m.start, m.end))).toEqual(["rarement"]);
  });

  it("finds a mark whose text holds an apostrophe", () => {
    const r = readMarked("qui a étudié <b>les effets de l'inflation</b> sur l'épargne");
    expect(r.marks.map((m) => r.sentence.slice(m.start, m.end))).toEqual(["les effets de l'inflation"]);
  });

  it("decodes what the page's escaping put in, so the reader sees their own characters", () => {
    const r = readMarked("Utilisez <b>&amp;</b> pour x &lt; y &#8217;ok&#x21;");
    expect(r.sentence).toBe("Utilisez & pour x < y ’ok!");
    expect(r.marks.map((m) => r.sentence.slice(m.start, m.end))).toEqual(["&"]);
  });

  it("reports no mark when the engine dropped the tag, and still gives the sentence", () => {
    expect(readMarked("Il a dû supporter le bruit.")).toEqual({ sentence: "Il a dû supporter le bruit.", marks: [] });
  });

  it("keeps every span when the engine split the tag across a reordering", () => {
    const r = readMarked("<b>Il</b> a dû <b>supporter</b> le bruit.");
    expect(r.marks.map((m) => r.sentence.slice(m.start, m.end))).toEqual(["Il", "supporter"]);
  });

  it("keeps a highlight on text, never on the blanks around it", () => {
    const r = readMarked("Il a dû<b> supporter </b>le bruit.");
    expect(r.marks.map((m) => r.sentence.slice(m.start, m.end))).toEqual(["supporter"]);
  });

  it("ignores an empty mark and a close with no open", () => {
    expect(readMarked("Il <b></b>a dû</b> partir.").marks).toEqual([]);
  });

  it("treats a tag that is not ours as zero-width", () => {
    const r = readMarked("Il a <i>dû</i> <b>partir</b>.");
    expect(r.sentence).toBe("Il a dû partir.");
    expect(r.marks.map((m) => r.sentence.slice(m.start, m.end))).toEqual(["partir"]);
  });

  it("leaves an entity it does not know exactly as written", () => {
    expect(readMarked("a &bogus; b").sentence).toBe("a &bogus; b");
  });

  it("round-trips a marked sentence it did not translate", () => {
    // What goes in comes back out: the selection's text, at the selection's place.
    const s = "x < y & <b>z</b>";
    const sel = { start: 4, end: 5 };
    const r = readMarked(markSelection(s, sel));
    expect(r.sentence).toBe(s);
    expect(r.marks).toEqual([sel]);
  });
});
