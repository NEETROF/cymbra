import { afterEach, describe, expect, it, vi } from "vitest";
import {
  colourCss,
  coloursFor,
  hexColour,
  indistinct,
  PAGE_TOKENS,
  PAPER_INK_TOKEN,
  PRESETS,
  resolvePreset,
  statusStyle,
  tokensOf,
} from "@/reading/colours.ts";
import { applyColourSheet, HL_LEARNING, HL_UNKNOWN } from "@/reading/highlight.ts";
import { customColours, TOKENS, tokenValues } from "./colour-fixtures.ts";
import {
  COLOURS_KEY,
  colourPreferenceOf,
  coloursOf,
  DEFAULT_COLOUR_PREFERENCE,
  loadColourPreference,
  saveColourPreference,
} from "@/state/storage.ts";

// The reader's colours (add-lingua-colour-settings): the stored preference made safe, the sheet
// it paints with, the presets read off the token sheet, and the sheet applied to a document.

describe("colourPreferenceOf", () => {
  it("keeps a preset and a well-formed custom set", () => {
    expect(colourPreferenceOf({ preset: "eink-mono" })).toEqual({ preset: "eink-mono" });
    const colours = customColours();
    expect(colourPreferenceOf({ preset: "custom", colours })).toEqual({ preset: "custom", colours });
  });

  it("falls back to Cymbra for anything else", () => {
    expect(colourPreferenceOf(undefined)).toEqual(DEFAULT_COLOUR_PREFERENCE);
    expect(colourPreferenceOf({ preset: "neon" })).toEqual(DEFAULT_COLOUR_PREFERENCE);
    expect(colourPreferenceOf({ preset: "custom" })).toEqual(DEFAULT_COLOUR_PREFERENCE);
    const bad = customColours() as unknown as { unknown: { fill: { colour: string } } };
    bad.unknown.fill.colour = "red";
    expect(colourPreferenceOf({ preset: "custom", colours: bad })).toEqual(DEFAULT_COLOUR_PREFERENCE);
  });

  it("refuses a set with an unknown style, intensity, thickness or a missing colour", () => {
    type Loose = Record<
      string,
      Record<string, unknown> & { fill: Record<string, unknown>; underline: Record<string, unknown> }
    >;
    const mutate = (fn: (c: Loose) => void): unknown => {
      const c = customColours() as unknown as Loose;
      fn(c);
      return c;
    };
    expect(coloursOf(mutate((c) => (c.unknown.underline.style = "zigzag")))).toBeUndefined();
    expect(coloursOf(mutate((c) => (c.learning.fill.intensity = "max")))).toBeUndefined();
    expect(coloursOf(mutate((c) => (c.learning.underline.thickness = "huge")))).toBeUndefined();
    expect(coloursOf(mutate((c) => (c.unknown.text = 3)))).toBeUndefined();
    expect(coloursOf(mutate((c) => (c.paper.text = "#zzzzzz")))).toBeUndefined();
    expect(coloursOf(mutate((c) => delete c.dark.text))).toBeUndefined();
  });

  it("lower-cases colours", () => {
    const c = customColours();
    c.dark.text = "#ABCDEF";
    expect(coloursOf(c)?.dark.text).toBe("#abcdef");
  });

  it("is saved and loaded under its key, made safe on the way", async () => {
    const store: Record<string, unknown> = {};
    const area = {
      get: async (k: string | string[] | null) => ({ [k as string]: store[k as string] }),
      set: async (items: Record<string, unknown>) => void Object.assign(store, items),
    };
    expect(await loadColourPreference(area)).toEqual(DEFAULT_COLOUR_PREFERENCE);
    await saveColourPreference(area, { preset: "eink-colour" });
    expect(store[COLOURS_KEY]).toEqual({ preset: "eink-colour" });
    expect(await loadColourPreference(area)).toEqual({ preset: "eink-colour" });
  });
});

describe("colourCss", () => {
  it("is empty for Cymbra: the token sheet paints alone", () => {
    expect(colourCss({ preset: "cymbra" })).toBe("");
  });

  it("paints a preset with the token sheet's colours, never a literal", () => {
    const css = colourCss({ preset: "eink-mono" });
    expect(css).not.toMatch(/#[0-9a-f]{3,8}\b/i);
    expect(css).toContain(`::highlight(${HL_UNKNOWN})`);
    expect(css).toContain(`::highlight(${HL_LEARNING})`);
    expect(css).toContain("color-mix(in srgb, var(--cymbra-lingua-eink-grey) 65%, transparent)");
    expect(css).toContain("text-decoration: underline solid var(--cymbra-lingua-eink-black)");
    expect(css).toContain("text-decoration: underline dotted var(--cymbra-lingua-eink-black)");
    expect(css).toContain("text-decoration-thickness: 3px");
    expect(css).toContain(`${PAPER_INK_TOKEN}: var(--cymbra-lingua-eink-black)`);
  });

  it("paints a custom set with the reader's colours", () => {
    const css = colourCss({ preset: "custom", colours: customColours() });
    const unknownRule = css.slice(css.indexOf(`::highlight(${HL_UNKNOWN})`));
    expect(unknownRule).toContain("color-mix(in srgb, #ff0000 65%, transparent)");
    expect(unknownRule).toContain("text-decoration: underline solid #00ff00;");
    expect(unknownRule).toContain("text-decoration-thickness: 3px");
    expect(unknownRule).toContain("color: #0000ff");
    const learningRule = css.slice(
      css.indexOf(`::highlight(${HL_LEARNING})`),
      css.indexOf(`::highlight(${HL_UNKNOWN})`),
    );
    expect(learningRule).toContain("background-color: transparent");
    expect(learningRule).not.toContain("thickness");
    expect(learningRule).not.toMatch(/^\s*color:/m);
    expect(css).toContain("--cymbra-lingua-page-paper: #fefefe");
    expect(css).toContain("--cymbra-lingua-page-night: #010101");
    expect(css).toContain("--cymbra-lingua-page-night-ink: #eeeeee");
    expect(css).not.toContain(PAPER_INK_TOKEN);
  });

  it("draws no line, and no thickness, for an underline set to none", () => {
    const colours = customColours();
    colours.unknown.underline.style = "none";
    const css = colourCss({ preset: "custom", colours });
    expect(css).toContain("text-decoration: none;");
    expect(css).not.toContain("thickness");
  });

  it("names only tokens the token sheet defines", () => {
    const named = new Set<string>(PAGE_TOKENS);
    for (const preset of Object.values(PRESETS)) {
      const json = JSON.stringify(preset);
      for (const m of json.matchAll(/--cymbra-lingua-[\w-]+/g)) named.add(m[0]);
    }
    for (const token of named) expect(TOKENS, token).toContain(`${token}:`);
    // The paper page's text is the one deliberately unset: the book keeps its own colours.
    expect(TOKENS).not.toContain(`${PAPER_INK_TOKEN}:`);
  });
});

describe("the presets as colours", () => {
  it("resolves each preset's tokens to hex, as the settings show them", () => {
    const cymbra = resolvePreset("cymbra", tokenValues());
    expect(cymbra.unknown.fill).toEqual({ colour: "#ffb4ab", intensity: "light" });
    expect(cymbra.unknown.underline).toEqual({ colour: "#d8502e", style: "solid", thickness: "thin" });
    expect(cymbra.learning.underline.style).toBe("dotted");
    expect(cymbra.paper).toEqual({ background: "#fbf9f4", text: null });
    expect(cymbra.dark).toEqual({ background: "#060e20", text: "#cfd6ea" });
    expect(resolvePreset("eink-mono", tokenValues()).paper).toEqual({ background: "#ffffff", text: "#000000" });
    expect(resolvePreset("eink-colour", tokenValues()).unknown.text).toBe("#b0001e");
    expect(colourPreferenceOf({ preset: "custom", colours: resolvePreset("eink-colour", tokenValues()) }).preset).toBe(
      "custom",
    );
  });

  it("reads black for a token it cannot resolve, so the set stays valid", () => {
    expect(resolvePreset("cymbra", () => "").unknown.fill.colour).toBe("#000000");
  });

  it("shows a custom set as stored, a preset as its tokens resolve", () => {
    const colours = customColours();
    expect(coloursFor({ preset: "custom", colours }, tokenValues())).toBe(colours);
    expect(coloursFor({ preset: "eink-mono" }, tokenValues()).unknown.underline.thickness).toBe("thick");
  });

  it("keeps the two statuses apart without colour in every preset", () => {
    for (const id of ["cymbra", "eink-mono", "eink-colour"] as const) {
      expect(indistinct(resolvePreset(id, tokenValues())), id).toBe(false);
    }
    const same = customColours();
    same.learning.underline.style = "solid";
    expect(indistinct(same)).toBe(true);
  });

  it("reads tokens off an element's computed style", () => {
    document.documentElement.style.setProperty("--cymbra-lingua-coral", "#123456");
    expect(tokensOf(document.documentElement)("--cymbra-lingua-coral").trim()).toBe("#123456");
    document.documentElement.removeAttribute("style");
  });
});

describe("hexColour", () => {
  it("normalises the colours a token sheet holds", () => {
    expect(hexColour(" #ABCDEF")).toBe("#abcdef");
    expect(hexColour("#abc")).toBe("#aabbcc");
    expect(hexColour("rgb(255, 180, 171)")).toBe("#ffb4ab");
    expect(hexColour("rgba(255,180,84,0.28)")).toBe("#ffb454");
    expect(hexColour("color-mix(in srgb, red, blue)")).toBeNull();
  });
});

describe("statusStyle", () => {
  it("is the sheet's paint, as inline declarations for the preview", () => {
    const { unknown, learning } = customColours();
    expect(statusStyle(unknown)).toEqual({
      backgroundColor: "color-mix(in srgb, #ff0000 65%, transparent)",
      textDecoration: "underline solid #00ff00",
      textDecorationThickness: "3px",
      color: "#0000ff",
    });
    expect(statusStyle(learning)).toMatchObject({
      backgroundColor: "transparent",
      textDecorationThickness: "",
      color: "",
    });
  });
});

describe("applyColourSheet", () => {
  afterEach(() => {
    document.getElementById("cymbra-lingua-colours")?.remove();
  });

  it("adds no sheet for an empty choice, and replaces one in place", () => {
    applyColourSheet(document, "");
    expect(document.getElementById("cymbra-lingua-colours")).toBeNull();
    applyColourSheet(document, "a {}");
    applyColourSheet(document, "b {}");
    const nodes = document.querySelectorAll("#cymbra-lingua-colours");
    expect(nodes).toHaveLength(1);
    expect(nodes[0].textContent).toBe("b {}");
    applyColourSheet(document, "");
    expect(document.getElementById("cymbra-lingua-colours")?.textContent).toBe("");
  });

  it("puts its node after the token sheet's, so it wins at equal specificity", () => {
    const tokens = document.createElement("style");
    tokens.id = "cymbra-lingua-style";
    document.documentElement.append(tokens);
    applyColourSheet(document, "a {}");
    tokens.remove();
    document.documentElement.append(tokens);
    applyColourSheet(document, "a {}");
    const ids = [...document.documentElement.querySelectorAll("style")].map((s) => s.id);
    expect(ids.indexOf("cymbra-lingua-colours")).toBeGreaterThan(ids.indexOf("cymbra-lingua-style"));
    tokens.remove();
  });

  it("adopts one constructable sheet, last, where the document can", () => {
    class FakeSheet {
      text = "";
      replaceSync(t: string): void {
        this.text = t;
      }
    }
    const other = new FakeSheet();
    const doc = {
      adoptedStyleSheets: [other] as unknown[],
      defaultView: { CSSStyleSheet: FakeSheet },
    } as unknown as Document;
    applyColourSheet(doc, "");
    expect(doc.adoptedStyleSheets).toHaveLength(1);
    applyColourSheet(doc, "a {}");
    applyColourSheet(doc, "b {}");
    const sheets = doc.adoptedStyleSheets as unknown as FakeSheet[];
    expect(sheets).toHaveLength(2);
    expect(sheets[1].text).toBe("b {}");
    // Another sheet adopted later (the token sheet re-asserted): the colours move back to last.
    const late = new FakeSheet();
    (doc as unknown as { adoptedStyleSheets: unknown[] }).adoptedStyleSheets = [...sheets, late];
    applyColourSheet(doc, "c {}");
    expect((doc.adoptedStyleSheets as unknown as FakeSheet[]).at(-1)?.text).toBe("c {}");
  });

  it("falls back to a node when the constructable sheet throws", () => {
    const Throwing = vi.fn(function (this: unknown) {
      throw new Error("no");
    });
    const real = document as unknown as { adoptedStyleSheets?: unknown };
    Object.defineProperty(real, "adoptedStyleSheets", { value: [], configurable: true, writable: true });
    const view = document.defaultView as unknown as { CSSStyleSheet: unknown };
    const saved = view.CSSStyleSheet;
    view.CSSStyleSheet = Throwing;
    applyColourSheet(document, "a {}");
    expect(document.getElementById("cymbra-lingua-colours")?.textContent).toBe("a {}");
    view.CSSStyleSheet = saved;
    delete real.adoptedStyleSheets;
  });
});
