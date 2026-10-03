import { describe, expect, it } from "vitest";
import { bookStyles, pageColoursOf, scaleFontSizes, TEXT_SCALE_VAR } from "@/reader/book-style.ts";

// The styles laid over a book (add-lingua-reader D10): the text size scales the book's text —
// its absolute sizes included, which a root size alone would not reach — and the dark page
// replaces the book's colours, from the token sheet.

const NIGHT = { ink: "rgb(1, 2, 3)", link: "rgb(4, 5, 6)", rule: "rgb(7, 8, 9)", paperInk: "" };
const scaled = (size: string): string => `calc(${size} * var(${TEXT_SCALE_VAR}, 1))`;

describe("scaleFontSizes", () => {
  it("makes absolute sizes follow the text size, wherever they are (Let's Go sets px)", () => {
    const css = `body { font-family: sans-serif; font-size: 20px; line-height: 1.5; }
.text h1, .text h2 { font-weight: 700; font-size: 45px }
@media (min-width: 40em) { code { font-size: 9pt !important; } }`;
    expect(scaleFontSizes(css)).toBe(`body { font-family: sans-serif; font-size: ${scaled("20px")}; line-height: 1.5; }
.text h1, .text h2 { font-weight: 700; font-size: ${scaled("45px")} }
@media (min-width: 40em) { code { font-size: ${scaled("9pt")} !important; } }`);
  });

  it("leaves sizes relative to their parent alone: they follow already", () => {
    const css =
      "p { font-size: 0.75em; } .small { font-size: 90%; } h1 { font-size: 1.2rem; } s { font-size: smaller; }";
    expect(scaleFontSizes(css)).toBe(css);
  });

  it("scales a size set on the root in any unit: it is the base the rest follows", () => {
    expect(scaleFontSizes("html { font-size: 62.5%; }")).toBe(`html { font-size: ${scaled("62.5%")}; }`);
    expect(scaleFontSizes(":root, body { font-size: 1.1em }")).toBe(`:root, body { font-size: ${scaled("1.1em")} }`);
    expect(scaleFontSizes("/* base */ html.book { font-size: 10px }")).toBe(
      `/* base */ html.book { font-size: ${scaled("10px")} }`,
    );
    expect(scaleFontSizes("html body { font-size: 90% }")).toBe("html body { font-size: 90% }");
  });

  it("returns a sheet with no font size untouched", () => {
    const css = "@font-face { font-family: X; src: url(x.woff); } p { margin: 0 }";
    expect(scaleFontSizes(css)).toBe(css);
  });
});

describe("bookStyles", () => {
  it("sets the size before the book's rules, as the root's size and the factor", () => {
    const [before] = bookStyles({ textScale: 130, theme: "paper", turn: "instant" }, NIGHT);
    expect(before).toContain(`${TEXT_SCALE_VAR}: 1.3`);
    expect(before).toContain("font-size: 130%");
  });

  it("keeps the book's colours on paper", () => {
    const [, after] = bookStyles({ textScale: 100, theme: "paper", turn: "instant" }, NIGHT);
    expect(after).toContain("color-scheme: light");
    expect(after).not.toContain(NIGHT.ink);
    expect(after).toContain("hyphens: auto");
  });

  it("replaces the book's text colour on paper when the reader chose one, keeping its backgrounds", () => {
    const [, after] = bookStyles(
      { textScale: 100, theme: "paper", turn: "instant" },
      { ...NIGHT, paperInk: "#000000" },
    );
    expect(after).toContain("color-scheme: light");
    expect(after).toContain("color: #000000 !important");
    expect(after).toContain("color: inherit !important");
    expect(after).not.toContain("background");
  });

  it("replaces them on the dark page, with the token sheet's colours", () => {
    const [, after] = bookStyles({ textScale: 100, theme: "dark", turn: "instant" }, NIGHT);
    expect(after).toContain("color-scheme: dark");
    expect(after).toContain(`color: ${NIGHT.ink} !important`);
    expect(after).toContain(`color: ${NIGHT.link} !important`);
    expect(after).toContain(`border-color: ${NIGHT.rule} !important`);
    expect(after).toMatch(/:not\(img\)/);
  });
});

describe("pageColoursOf", () => {
  it("reads the page's colours off the token sheet, as the reader's colour sheet sets them", () => {
    const root = document.documentElement;
    root.style.setProperty("--cymbra-lingua-page-night-ink", " #cfd6ea");
    root.style.setProperty("--cymbra-lingua-night-link", "#d2bbff");
    root.style.setProperty("--cymbra-lingua-night-rule", "#2d3449");
    expect(pageColoursOf(document)).toEqual({ ink: "#cfd6ea", link: "#d2bbff", rule: "#2d3449", paperInk: "" });
    root.style.setProperty("--cymbra-lingua-page-paper-ink", "#111111");
    expect(pageColoursOf(document).paperInk).toBe("#111111");
    root.removeAttribute("style");
  });
});
