import type {
  ColourPreference,
  ColourPresetId,
  Colours,
  FillIntensity,
  StatusColours,
  UnderlineStyle,
  UnderlineThickness,
} from "../state/storage.ts";
import { HL_LEARNING, HL_UNKNOWN } from "./highlight.ts";

// The reader's colours (add-lingua-colour-settings): how unknown and learning words are marked,
// and the reader's paper and dark pages. A preset names token sheet colours — no colour literal
// lives here (the identity rule) — and a custom set carries the reader's own `#rrggbb`. Both
// become one sheet, `colourCss`, adopted after the token sheet in every document read (D3).

/** The two statuses a highlight paints. */
export type MarkedStatus = "unknown" | "learning";

/** A colour as a preset names it: a token of the sheet, read where the preset is painted. */
type Token = `--cymbra-lingua-${string}`;

interface StatusPreset {
  fill: { colour: Token; intensity: FillIntensity };
  underline: { colour: Token; style: UnderlineStyle; thickness: UnderlineThickness };
  text: Token | null;
}

interface PresetColours {
  unknown: StatusPreset;
  learning: StatusPreset;
  paper: { background: Token; text: Token | null };
  dark: { background: Token; text: Token };
}

const NIGHT = { background: "--cymbra-lingua-night", text: "--cymbra-lingua-night-ink" } as const;

/** The presets, in the token sheet's colours (design D7). */
export const PRESETS: Record<ColourPresetId, PresetColours> = {
  cymbra: {
    unknown: {
      fill: { colour: "--cymbra-lingua-coral", intensity: "light" },
      underline: { colour: "--cymbra-lingua-unknown-underline", style: "solid", thickness: "thin" },
      text: null,
    },
    learning: {
      fill: { colour: "--cymbra-lingua-amber", intensity: "light" },
      underline: { colour: "--cymbra-lingua-learning-underline", style: "dotted", thickness: "thin" },
      text: null,
    },
    paper: { background: "--cymbra-lingua-paper", text: null },
    dark: NIGHT,
  },
  "eink-mono": {
    unknown: {
      fill: { colour: "--cymbra-lingua-eink-grey", intensity: "strong" },
      underline: { colour: "--cymbra-lingua-eink-black", style: "solid", thickness: "thick" },
      text: null,
    },
    learning: {
      fill: { colour: "--cymbra-lingua-eink-grey", intensity: "none" },
      underline: { colour: "--cymbra-lingua-eink-black", style: "dotted", thickness: "thick" },
      text: null,
    },
    paper: { background: "--cymbra-lingua-eink-white", text: "--cymbra-lingua-eink-black" },
    dark: NIGHT,
  },
  "eink-colour": {
    unknown: {
      fill: { colour: "--cymbra-lingua-eink-red", intensity: "light" },
      underline: { colour: "--cymbra-lingua-eink-red", style: "solid", thickness: "thick" },
      text: "--cymbra-lingua-eink-red",
    },
    learning: {
      fill: { colour: "--cymbra-lingua-eink-blue", intensity: "none" },
      underline: { colour: "--cymbra-lingua-eink-blue", style: "dotted", thickness: "thick" },
      text: "--cymbra-lingua-eink-blue",
    },
    paper: { background: "--cymbra-lingua-eink-white", text: null },
    dark: NIGHT,
  },
};

/** How much of the fill colour shows behind a word. */
const FILL_SHARE: Record<FillIntensity, number> = { none: 0, light: 30, strong: 65 };

/** A thick underline: past the default hairline, which a 300 ppi e-ink panel nearly loses. */
const THICK = "3px";

/**
 * The page tokens `colourCss` overrides, each defaulted in the token sheet to the identity's
 * colours; plus `--cymbra-lingua-page-paper-ink`, unset there (the book's own text on paper).
 */
export const PAGE_TOKENS = [
  "--cymbra-lingua-page-paper",
  "--cymbra-lingua-page-night",
  "--cymbra-lingua-page-night-ink",
] as const;
export const PAPER_INK_TOKEN = "--cymbra-lingua-page-paper-ink";

/** One status, in CSS values: what the sheet and the preview both paint. */
interface StatusPaint {
  background: string;
  decoration: string;
  thickness: string | null;
  colour: string | null;
}

/** A colour of the paint: a token's reference, or the reader's own hex. */
type Ref = (colour: string) => string;
const tokenRef: Ref = (token) => `var(${token})`;
const hexRef: Ref = (hex) => hex;

function paint(status: StatusPreset | StatusColours, ref: Ref): StatusPaint {
  const share = FILL_SHARE[status.fill.intensity];
  const { underline } = status;
  return {
    background: share === 0 ? "transparent" : `color-mix(in srgb, ${ref(status.fill.colour)} ${share}%, transparent)`,
    decoration: underline.style === "none" ? "none" : `underline ${underline.style} ${ref(underline.colour)}`,
    thickness: underline.style !== "none" && underline.thickness === "thick" ? THICK : null,
    colour: status.text === null ? null : ref(status.text),
  };
}

function highlightRule(name: string, p: StatusPaint): string {
  const lines = [`background-color: ${p.background};`, `text-decoration: ${p.decoration};`];
  if (p.thickness) lines.push(`text-decoration-thickness: ${p.thickness};`);
  if (p.colour) lines.push(`color: ${p.colour};`);
  return `::highlight(${name}) {\n  ${lines.join("\n  ")}\n}`;
}

function sheet(colours: PresetColours | Colours, ref: Ref): string {
  const [paper, night, nightInk] = PAGE_TOKENS;
  const page = [
    `${paper}: ${ref(colours.paper.background)};`,
    `${night}: ${ref(colours.dark.background)};`,
    `${nightInk}: ${ref(colours.dark.text)};`,
  ];
  if (colours.paper.text !== null) page.push(`${PAPER_INK_TOKEN}: ${ref(colours.paper.text)};`);
  return [
    `:root,\n:host {\n  ${page.join("\n  ")}\n}`,
    highlightRule(HL_LEARNING, paint(colours.learning, ref)),
    highlightRule(HL_UNKNOWN, paint(colours.unknown, ref)),
  ].join("\n\n");
}

/**
 * The sheet the reader's choice paints with, adopted after the token sheet (design D3). The
 * Cymbra preset is the token sheet itself: it yields no sheet at all, so a reader who never
 * opens the block paints exactly as before.
 */
export function colourCss(preference: ColourPreference): string {
  if (preference.preset === "cymbra") return "";
  if (preference.preset === "custom") return sheet(preference.colours, hexRef);
  return sheet(PRESETS[preference.preset], tokenRef);
}

/** One status as inline style declarations, for the settings' preview (painted without highlights). */
export function statusStyle(status: StatusColours): Partial<CSSStyleDeclaration> {
  const p = paint(status, hexRef);
  return {
    backgroundColor: p.background,
    textDecoration: p.decoration,
    textDecorationThickness: p.thickness ?? "",
    color: p.colour ?? "",
  };
}

/** Whether the two statuses would look alike on a monochrome screen: the same underline style. */
export function indistinct(colours: Colours): boolean {
  return colours.unknown.underline.style === colours.learning.underline.style;
}

/** A token's value where it is painted (`getComputedStyle(…).getPropertyValue`). */
export type TokenReader = (token: string) => string;

/** A token reader for the colours an element inherits: a document's root, or a shadow host's. */
export function tokensOf(el: Element): TokenReader {
  const style = (el.ownerDocument.defaultView ?? window).getComputedStyle(el);
  return (token) => style.getPropertyValue(token);
}

/** A CSS colour as `#rrggbb`: hex (3 or 6 digits) or `rgb()`/`rgba()`, alpha dropped. */
export function hexColour(value: string): string | null {
  const v = value.trim().toLowerCase();
  const long = /^#([0-9a-f]{6})$/.exec(v);
  if (long) return `#${long[1]}`;
  const short = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/.exec(v);
  if (short) return `#${short[1]}${short[1]}${short[2]}${short[2]}${short[3]}${short[3]}`;
  const rgb = /^rgba?\(\s*(\d{1,3})[\s,]+(\d{1,3})[\s,]+(\d{1,3})/.exec(v);
  if (rgb) {
    return `#${rgb
      .slice(1, 4)
      .map((n) => Math.min(255, Number(n)).toString(16).padStart(2, "0"))
      .join("")}`;
  }
  return null;
}

/**
 * A preset as a full custom set: its tokens read where the settings are drawn (design D2). The
 * reader edits from there. A token the reader cannot resolve (a missing sheet) reads as black,
 * so the set stays valid — the preview shows it.
 */
export function resolvePreset(id: ColourPresetId, read: TokenReader): Colours {
  const preset = PRESETS[id];
  const hex = (token: Token): string => hexColour(read(token)) ?? "#000000";
  const status = (s: StatusPreset): StatusColours => ({
    fill: { colour: hex(s.fill.colour), intensity: s.fill.intensity },
    underline: { colour: hex(s.underline.colour), style: s.underline.style, thickness: s.underline.thickness },
    text: s.text === null ? null : hex(s.text),
  });
  return {
    unknown: status(preset.unknown),
    learning: status(preset.learning),
    paper: {
      background: hex(preset.paper.background),
      text: preset.paper.text === null ? null : hex(preset.paper.text),
    },
    dark: { background: hex(preset.dark.background), text: hex(preset.dark.text) },
  };
}

/** The colours a preference shows: a custom set as stored, a preset as its tokens resolve. */
export function coloursFor(preference: ColourPreference, read: TokenReader): Colours {
  return preference.preset === "custom" ? preference.colours : resolvePreset(preference.preset, read);
}
