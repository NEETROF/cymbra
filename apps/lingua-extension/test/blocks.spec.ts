import { beforeEach, describe, expect, it } from "vitest";
import { byteToCharOffset, collectBlocks, rangeForToken } from "@/reading/blocks.ts";

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
