import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Drawer } from "@/reading/drawer.ts";
import { LinguaHud } from "@/reading/hud.ts";
import {
  applySurfaceLook,
  followSurfaceLook,
  SURFACE_THEME_ATTR,
  surfaceTheme,
  UI_SCALE_VAR,
} from "@/reading/surface-look.ts";
import { WordPopup } from "@/reading/wordpopup.ts";
import {
  type AsyncStorageArea,
  COLOURS_KEY,
  type ColourPreference,
  DEFAULT_READER_DISPLAY,
  READER_DISPLAY_KEY,
  type ReaderDisplay,
} from "@/state/storage.ts";
import { makeFakePort } from "./helpers.ts";

// How the extension's own surfaces look (add-lingua-colour-settings D8, D9): the palette from
// the reader's colours and page theme, the scale from their text size.

const paper: ReaderDisplay = { textScale: 100, theme: "paper", turn: "instant" };
const dark: ReaderDisplay = { textScale: 100, theme: "dark", turn: "instant" };
const cymbra: ColourPreference = { preset: "cymbra" };

type Listener = (changes: Record<string, { newValue?: unknown }>, areaName: string) => void;
let listeners: Listener[];

function areaWith(items: Record<string, unknown>): AsyncStorageArea {
  return {
    async get(keys) {
      const list = keys == null ? Object.keys(items) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in items).map((k) => [k, items[k]]));
    },
    async set(more) {
      Object.assign(items, more);
    },
  };
}

function change(key: string, newValue: unknown, areaName = "local"): void {
  for (const listener of listeners) listener({ [key]: { newValue } }, areaName);
}

const settle = async (): Promise<void> => {
  for (let i = 0; i < 3; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

beforeEach(() => {
  listeners = [];
  const local: Record<string, unknown> = {};
  vi.stubGlobal("chrome", {
    runtime: { sendMessage: vi.fn(async () => undefined), onMessage: { addListener: () => {} } },
    storage: {
      local: { get: async (keys: string | string[]) => areaWith(local).get(keys), set: async () => {} },
      onChanged: { addListener: (fn: Listener) => void listeners.push(fn) },
    },
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
  document.body.replaceChildren();
  document.documentElement.querySelectorAll("[data-cymbra-lingua-skip]").forEach((el) => el.remove());
});

describe("the surfaces' palette", () => {
  it("keeps every surface black on white with an e-ink preset, whatever the page", () => {
    for (const preset of ["eink-mono", "eink-colour"] as const) {
      expect(surfaceTheme({ preset }, paper)).toBe("eink");
      expect(surfaceTheme({ preset }, dark)).toBe("eink");
    }
  });

  it("follows the page theme with the Cymbra and custom colours", () => {
    const custom = { preset: "custom" } as ColourPreference;
    expect(surfaceTheme(cymbra, paper)).toBe("light");
    expect(surfaceTheme(cymbra, dark)).toBe("dark");
    expect(surfaceTheme(custom, paper)).toBe("light");
    expect(surfaceTheme(custom, dark)).toBe("dark");
  });
});

describe("a surface's root", () => {
  it("carries the palette and the text size as its scale", () => {
    const root = document.createElement("div");
    applySurfaceLook(root, { preset: "eink-mono" }, { textScale: 180, theme: "paper", turn: "instant" });
    expect(root.getAttribute(SURFACE_THEME_ATTR)).toBe("eink");
    expect(root.style.getPropertyValue(UI_SCALE_VAR)).toBe("1.8");
  });

  it("follows the stored choice, then every change made anywhere", async () => {
    const root = document.createElement("div");
    followSurfaceLook(
      root,
      areaWith({ [COLOURS_KEY]: cymbra, [READER_DISPLAY_KEY]: { textScale: 150, theme: "dark", turn: "instant" } }),
    );
    expect(root.getAttribute(SURFACE_THEME_ATTR)).toBe("light"); // the defaults, before the read
    await settle();
    expect(root.getAttribute(SURFACE_THEME_ATTR)).toBe("dark");
    expect(root.style.getPropertyValue(UI_SCALE_VAR)).toBe("1.5");

    change(COLOURS_KEY, { preset: "eink-colour" });
    expect(root.getAttribute(SURFACE_THEME_ATTR)).toBe("eink");
    change(READER_DISPLAY_KEY, { textScale: 200, theme: "paper", turn: "instant" });
    expect(root.style.getPropertyValue(UI_SCALE_VAR)).toBe("2");
    change(COLOURS_KEY, cymbra);
    expect(root.getAttribute(SURFACE_THEME_ATTR)).toBe("light");

    change(READER_DISPLAY_KEY, { textScale: 80, theme: "dark", turn: "instant" }, "sync"); // not the reader's area
    change("cymbra-lingua-enabled", false);
    expect(root.style.getPropertyValue(UI_SCALE_VAR)).toBe("2");
    expect(root.getAttribute(SURFACE_THEME_ATTR)).toBe("light");
  });

  it("keeps the defaults when the stored choice cannot be read", async () => {
    const root = document.createElement("div");
    const failing: AsyncStorageArea = { get: async () => Promise.reject(new Error("gone")), set: async () => {} };
    followSurfaceLook(root, failing);
    await settle();
    expect(root.getAttribute(SURFACE_THEME_ATTR)).toBe("light");
    expect(root.style.getPropertyValue(UI_SCALE_VAR)).toBe(String(DEFAULT_READER_DISPLAY.textScale / 100));
  });
});

describe("the surfaces a page carries", () => {
  const area = areaWith({ [COLOURS_KEY]: { preset: "eink-mono" }, [READER_DISPLAY_KEY]: paper });

  it("theme the word card, the pill and the drawer when asked to", async () => {
    const popup = new WordPopup({ css: "", onGesture: () => {}, followLook: true });
    const hud = new LinguaHud({ css: "", actions: { onReview() {}, onStats() {}, onSettings() {} }, followLook: true });
    const drawer = new Drawer({
      css: "",
      port: makeFakePort().port,
      area,
      store: areaWith({}),
      now: () => 0,
      onChange: async () => {},
      followLook: true,
    });
    await settle();
    expect(popup.host.getAttribute(SURFACE_THEME_ATTR)).toBe("light"); // chrome.storage.local holds nothing
    hud.mount();
    const hudHost = document.getElementById("cymbra-lingua-hud-host");
    expect(hudHost?.getAttribute(SURFACE_THEME_ATTR)).toBe("light");
    await drawer.openOn("review");
    const drawerHost = document.getElementById("cymbra-lingua-drawer-host");
    // The drawer reads the area it was given: the e-ink preset stored there.
    expect(drawerHost?.getAttribute(SURFACE_THEME_ATTR)).toBe("eink");

    change(COLOURS_KEY, { preset: "eink-colour" });
    expect(popup.host.getAttribute(SURFACE_THEME_ATTR)).toBe("eink");
    expect(hudHost?.getAttribute(SURFACE_THEME_ATTR)).toBe("eink");
    change(READER_DISPLAY_KEY, { textScale: 170, theme: "dark", turn: "instant" });
    change(COLOURS_KEY, cymbra);
    expect(drawerHost?.getAttribute(SURFACE_THEME_ATTR)).toBe("dark");
    expect(drawerHost?.style.getPropertyValue(UI_SCALE_VAR)).toBe("1.7");
  });

  it("leave them as designed otherwise", () => {
    const popup = new WordPopup({ css: "", onGesture: () => {} });
    expect(popup.host.hasAttribute(SURFACE_THEME_ATTR)).toBe(false);
    expect(listeners).toEqual([]);
  });
});

// Every surface stylesheet scales with the text size, and says notices, successes and dangers
// through the tokens the themes re-point — never through the colours the presets are made of.
const SRC = join(dirname(fileURLToPath(import.meta.url)), "../src");

function cssFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) return name === "pkg" ? [] : cssFiles(p);
    return p.endsWith(".css") && !p.endsWith("tokens.css") ? [p] : [];
  });
}

describe("the surface stylesheets", () => {
  const sheets = cssFiles(SRC).map((p) => ({ rel: p.slice(SRC.length + 1), css: readFileSync(p, "utf8") }));

  it("are all found", () => {
    expect(sheets.length).toBeGreaterThanOrEqual(10);
  });

  for (const { rel, css } of sheets) {
    it(`scale every font size with the text size: ${rel}`, () => {
      const fixed = [...css.matchAll(/font-size:\s*([^;]+);/g)]
        .map((m) => m[1].trim())
        .filter((value) => /\dpx/.test(value) && !value.includes(`var(${UI_SCALE_VAR}`));
      expect(fixed).toEqual([]);
    });

    it(`colour messages with the themed tokens: ${rel}`, () => {
      const preset = [
        ...css.matchAll(/(?:^|[\s;{])(color|border-color|border):[^;]*var\(--cymbra-lingua-(amber|green|coral)\)/g),
      ];
      expect(preset.map((m) => m[0].trim())).toEqual([]);
    });
  }
});

describe("the token sheet's surface themes", () => {
  const tokens = readFileSync(join(SRC, "styles/tokens.css"), "utf8");
  const block = (theme: string): string => {
    const start = tokens.indexOf(`:root[data-cymbra-lingua-ui="${theme}"] {`);
    expect(start).toBeGreaterThan(0);
    return tokens.slice(start, tokens.indexOf("}", start));
  };
  const SURFACE_TOKENS = [
    "bg",
    "bg-deep",
    "panel",
    "panel-2",
    "panel-3",
    "border",
    "border-2",
    "text",
    "muted",
    "faint",
    "accent",
    "accent-strong",
    "accent-ink",
    "teal",
    "warn",
    "ok",
    "danger",
    "shadow",
  ];

  for (const theme of ["light", "eink"]) {
    it(`re-points every surface token in the ${theme} theme, and no preset colour`, () => {
      const body = block(theme);
      for (const token of SURFACE_TOKENS) expect(body).toContain(`--cymbra-lingua-${token}:`);
      for (const presetColour of ["amber", "coral", "green", "learning-fill", "unknown-fill", "eink-black"]) {
        expect(body).not.toContain(`--cymbra-lingua-${presetColour}:`);
      }
    });
  }
});
