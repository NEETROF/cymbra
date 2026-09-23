import { beforeEach, describe, expect, it } from "vitest";
import { type Block, byteToCharOffset, collectBlocks, mergeBlocks, rangeForToken } from "@/reading/blocks.ts";

beforeEach(() => {
  document.body.innerHTML = "";
});

describe("collectBlocks", () => {
  it("makes one block per block-level container and skips excluded subtrees", () => {
    document.body.innerHTML = `
      <p>The runner runs.</p>
      <script>var secret = 1;</script>
      <pre>code block</pre>
      <div contenteditable="true">editable text</div>
      <ul><li>first item</li><li>second item</li></ul>`;
    const blocks = collectBlocks();
    const texts = blocks.map((b) => b.text.trim());
    expect(texts).toContain("The runner runs.");
    expect(texts).toContain("first item");
    expect(texts).toContain("second item");
    expect(texts.join(" ")).not.toContain("secret");
    expect(texts.join(" ")).not.toContain("code block");
    expect(texts.join(" ")).not.toContain("editable text");
  });

  it("preserves inter-word whitespace across inline elements", () => {
    document.body.innerHTML = `<p>hello <b>brave</b> world</p>`;
    const [block] = collectBlocks();
    expect(block.text).toBe("hello brave world");
    // Three eligible text nodes: "hello ", "brave", " world".
    expect(block.segments.length).toBe(3);
  });

  it("never reads YouTube's player captions", () => {
    document.body.innerHTML = `
      <div id="movie_player"><div class="ytp-caption-window-container">
        <div class="caption-window"><span class="captions-text"><span class="caption-visual-line">
          <span class="ytp-caption-segment">I was a government major,</span></span></span></div>
      </div></div>
      <div id="comments"><p>What a brilliant talk.</p></div>`;
    const texts = collectBlocks().map((b) => b.text.trim());
    expect(texts).toEqual(["What a brilliant talk."]);
  });

  it("drops blocks with no letters", () => {
    document.body.innerHTML = `<p>1234 5678</p><p>real words here</p>`;
    const texts = collectBlocks().map((b) => b.text.trim());
    expect(texts).toEqual(["real words here"]);
  });
});

describe("byteToCharOffset", () => {
  it("is the identity for ASCII", () => {
    expect(byteToCharOffset("hello world", 6)).toBe(6);
    expect(byteToCharOffset("hello", 0)).toBe(0);
    expect(byteToCharOffset("hello", 5)).toBe(5);
  });

  it("maps past multi-byte code points", () => {
    // "café " — é is 2 UTF-8 bytes. The word after it starts at byte 6, char 5.
    const text = "café mot";
    expect(byteToCharOffset(text, 6)).toBe(5);
    // The 'm' of "mot": char index 5.
    expect(text.slice(byteToCharOffset(text, 6))).toBe("mot");
  });
});

describe("rangeForToken", () => {
  it("maps a byte range to a DOM Range over the right text", () => {
    document.body.innerHTML = `<p>The runner runs.</p>`;
    const [block] = collectBlocks();
    // "runner" is bytes [4, 10) in "The runner runs." (all ASCII).
    const range = rangeForToken(block, 4, 10)!;
    expect(range).not.toBeNull();
    expect(range.toString()).toBe("runner");
  });

  it("maps a token that spans inline elements", () => {
    document.body.innerHTML = `<p>hello <b>brave</b> world</p>`;
    const [block] = collectBlocks(); // "hello brave world"
    // "brave" is bytes [6, 11).
    const range = rangeForToken(block, 6, 11)!;
    expect(range.toString()).toBe("brave");
  });

  it("maps a multi-byte token correctly", () => {
    document.body.innerHTML = `<p>café time</p>`;
    const [block] = collectBlocks(); // "café time"
    // "café" is bytes [0, 5) (é = 2 bytes), i.e. chars [0, 4).
    const range = rangeForToken(block, 0, 5)!;
    expect(range.toString()).toBe("café");
  });
});

describe("mergeBlocks", () => {
  /** The page's block map after a first walk, as content.ts keeps it. */
  function painted(): Map<Element, Block> {
    const map = new Map<Element, Block>();
    mergeBlocks(map, [document.body]);
    return map;
  }

  function byId(id: string): Element {
    return document.getElementById(id)!;
  }

  it("reports the first walk of a page as a change", () => {
    document.body.innerHTML = `<p id="a">alpha words</p>`;
    const map = new Map<Element, Block>();

    expect(mergeBlocks(map, [document.body])).toBe(true);
    expect([...map.keys()]).toEqual([byId("a")]);
  });

  it("reports no change when a clock of digits ticks", () => {
    document.body.innerHTML = `<p id="a">alpha words</p><div id="clock"><span>0:14</span></div>`;
    const map = painted();

    (byId("clock").firstElementChild as HTMLElement).textContent = "0:15";

    expect(mergeBlocks(map, [byId("clock")])).toBe(false);
    expect([...map.keys()]).toEqual([byId("a")]);
  });

  it("reports no change when a walked block reads the same over the same nodes", () => {
    document.body.innerHTML = `<p id="a">alpha words</p>`;
    const map = painted();

    expect(mergeBlocks(map, [byId("a")])).toBe(false);
  });

  it("reports a change when a painted block's text changes", () => {
    document.body.innerHTML = `<p id="a">alpha words</p>`;
    const map = painted();

    byId("a").append(" and more words");

    expect(mergeBlocks(map, [byId("a")])).toBe(true);
    expect(map.get(byId("a"))!.text).toBe("alpha words and more words");
  });

  it("reports a change when the same text now sits on a new node", () => {
    document.body.innerHTML = `<p id="a">alpha words</p>`;
    const map = painted();

    byId("a").replaceChildren(document.createTextNode("alpha words"));

    expect(mergeBlocks(map, [byId("a")])).toBe(true);
  });

  it("reports a change when a block is added under a walked root", () => {
    document.body.innerHTML = `<article id="art"><p id="a">alpha words</p></article>`;
    const map = painted();

    const p = document.createElement("p");
    p.id = "b";
    p.textContent = "beta words";
    byId("art").append(p);

    expect(mergeBlocks(map, [byId("art")])).toBe(true);
    expect(map.has(byId("b"))).toBe(true);
  });

  it("reports a block added under a parent that is re-walked along with it", () => {
    // The observer hands over both the mutated parent and the node added to it.
    document.body.innerHTML = `<article id="art"><p id="a">alpha words</p></article>`;
    const map = painted();

    const p = document.createElement("p");
    p.id = "b";
    p.textContent = "beta words";
    byId("art").append(p);

    expect(mergeBlocks(map, [byId("art"), byId("b")])).toBe(true);
    expect(map.has(byId("b"))).toBe(true);
  });

  it("reports a section loaded into an empty container, re-walked with its wrapper", () => {
    document.body.innerHTML = `<p id="a">alpha words</p><div id="feed"></div>`;
    const map = painted();

    const w = document.createElement("div");
    w.id = "w";
    const p = document.createElement("p");
    p.textContent = "comments loaded on scroll";
    w.append(p);
    byId("feed").append(w);

    expect(mergeBlocks(map, [byId("feed"), byId("w")])).toBe(true);
    expect([...map.values()].map((b) => b.text)).toContain("comments loaded on scroll");
  });

  it("reports a change when a painted block leaves the page", () => {
    document.body.innerHTML = `<p id="a">alpha words</p><p id="b">beta words</p>`;
    const map = painted();

    byId("b").remove();

    expect(mergeBlocks(map, [byId("a")])).toBe(true);
    expect([...map.keys()]).toEqual([byId("a")]);
  });

  it("keeps the blocks outside every walked root as they were", () => {
    document.body.innerHTML = `<p id="a">alpha words</p><div id="w"><p id="b">beta words</p></div>`;
    const map = painted();
    const kept = map.get(byId("a"));

    expect(mergeBlocks(map, [byId("w")])).toBe(false);
    expect(map.get(byId("a"))).toBe(kept);
  });
});
