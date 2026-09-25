import type { ReaderDisplay } from "../state/storage.ts";

// The styles the reader lays over a book (add-lingua-reader D10): its base style, the text size
// the reader chose, and the page — paper, in the book's own colours, or dark. foliate-js sets two
// sheets on each section: one before the book's own, which a book's rules outrank, and one after,
// which outranks them.
//
// The size scales the book's text, not the page: `zoom` would scale everything and pagination
// with it, and WebKit reports the boxes of a zoomed page at their unzoomed size, so the word
// popup would open beside its word. So the root size is raised in the sheet before the book's —
// what every size given in em, rem or % follows — and the sizes a book gives in absolute units
// (Let's Go sets its text in px) are rewritten, as its style sheets load, to follow the same
// factor (`scaleFontSizes`).

/** The factor a rewritten absolute font size is multiplied by, set on the section's root. */
export const TEXT_SCALE_VAR = "--cymbra-lingua-text-scale";

/** The dark page's colours, read off the token sheet by the page that draws the book. */
export interface NightColours {
  ink: string;
  link: string;
  rule: string;
}

/**
 * The book's base style, after foliate-js's own reader (reader.js `getCSS`, MIT): readable lines,
 * and code, images and tables held to their column where the book allows it — what overflows a
 * column is drawn over the next page. Its notes stay in the text: foliate's reader hides them for
 * a popup this reader does not have.
 */
const BASE_CSS = `
  p, li, blockquote, dd {
    line-height: 1.45;
    -webkit-hyphens: auto;
    hyphens: auto;
    widows: 2;
  }
  pre { white-space: pre-wrap !important; overflow-wrap: anywhere; }
  img, svg, video, table { max-width: 100%; }
`;

/** Paper: the book's own colours, on a light page like the one it was typeset for. */
const PAPER_CSS = `
  html { color-scheme: light; }
`;

/**
 * Dark: light text on the night page, whatever the book set — a book typeset for paper sets
 * dark text, which would vanish there. Its backgrounds go (a code block's, a table header's);
 * pictures keep their colours.
 */
function darkCss(night: NightColours): string {
  return `
  html { color-scheme: dark; }
  html, body { color: ${night.ink} !important; background: none !important; }
  body *:not(img):not(svg):not(video):not(picture) {
    color: inherit !important;
    background-color: transparent !important;
    border-color: ${night.rule} !important;
  }
  a:link, a:visited { color: ${night.link} !important; }
  a:link *, a:visited * { color: inherit !important; }
`;
}

/** The two sheets for a section: before the book's own, and after it. */
export function bookStyles(display: ReaderDisplay, night: NightColours): [string, string] {
  const before = `html { ${TEXT_SCALE_VAR}: ${display.textScale / 100}; font-size: ${display.textScale}%; }`;
  const page = display.theme === "dark" ? darkCss(night) : PAPER_CSS;
  return [before, `${BASE_CSS}${page}`];
}

/** Units a font size does not follow the root in: rewritten wherever they are. */
const ABSOLUTE = "px|pt|pc|cm|mm|in|q";
/** On the root itself every length or percentage sets the base the rest follows. */
const ANY = `${ABSOLUTE}|%|em|rem|ex|ch`;

const isRoot = (selector: string): boolean => /^(html|:root)([.#[:][^\s>+~]*)?$/i.test(selector.trim());

/**
 * A book's style sheet, its font sizes made to follow the reader's text size: every size in an
 * absolute unit, and every size set on the root, becomes `calc(size * var(--…-text-scale, 1))`.
 * Sizes relative to their parent (em, %) are left alone — they follow already. The rest of the
 * sheet is returned untouched.
 */
export function scaleFontSizes(css: string): string {
  return css.replace(/([^{}]+)\{([^{}]*)\}/g, (rule, selectors: string, body: string) => {
    const root = selectors
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .split(",")
      .some(isRoot);
    const value = new RegExp(`^-?(\\d+\\.?\\d*|\\.\\d+)(${root ? ANY : ABSOLUTE})$`, "i");
    const scaled = body.replace(
      /(font-size\s*:\s*)([^;!}]+?)(\s*!important)?(\s*)(?=;|$)/gi,
      (decl, prop: string, size: string, important = "", space: string) =>
        value.test(size.trim()) ? `${prop}calc(${size.trim()} * var(${TEXT_SCALE_VAR}, 1))${important}${space}` : decl,
    );
    return scaled === body ? rule : `${selectors}{${scaled}}`;
  });
}

/** The dark page's colours as the token sheet defines them for `doc`. */
export function nightColoursOf(doc: Document): NightColours {
  const style = (doc.defaultView ?? window).getComputedStyle(doc.documentElement);
  const token = (name: string): string => style.getPropertyValue(`--cymbra-lingua-night-${name}`).trim();
  return { ink: token("ink"), link: token("link"), rule: token("rule") };
}
