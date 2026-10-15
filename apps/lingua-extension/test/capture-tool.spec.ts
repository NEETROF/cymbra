import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { openArchive } from "@/reader/archive.ts";
import { isEpubArchive, readEpubMeta } from "@/reader/epub-meta.ts";
import { countsOf, DAILY_KEY as STORE_DAILY_KEY, loadDailyStats, utcDay } from "@/state/dailystats.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import {
  buildDifferences,
  captureExtensionId,
  extensionId,
  manifestDifferences,
  STATIC_READER,
} from "../tool/capture/build.mjs";
import { DAILY_KEY, DAYS, recentDays } from "../tool/capture/days.mjs";
import { attributesOf, type CdpNode, findIn, HOSTS, nodeText, quadCentre } from "../tool/capture/drive.mjs";
import { assembleEpub, epubEntries, parseBook, xmlEscape } from "../tool/capture/epub.mjs";
import {
  assertStorePng,
  columnCrop,
  cropAround,
  imageArgs,
  padded,
  pngHeader,
  scaled,
  union,
  videoArgs,
} from "../tool/capture/images.mjs";
import {
  BOOK,
  glossProblems,
  INTERFACE_LANGUAGES,
  OVERRIDES,
  READER_LEVELS,
  SEED_REVIEW,
  SITE_SHOTS,
  STORE_SHOTS,
  STUDIED,
  TIME_ZONE,
  WORDS,
  wordsFor,
} from "../tool/capture/plan.mjs";
import { recordedDay, seedForRun, shiftBackup } from "../tool/capture/seed.mjs";
import { distDirProblem } from "../tool/dist-dir.mjs";
import { useNodeBlob } from "./epub-fixtures.ts";

// The capture tool's pure helpers (add-site-lingua-showcase, task 1.7): the statistics days, the
// EPUB assembly, the crops' geometry, the PNG header reader, ffmpeg's arguments, the build
// comparison, the closed-shadow finder and the shot plan against its fixtures. The tool itself runs
// a browser and is never run in CI (tool/capture/capture.mjs).

const fixtures = join(__dirname, "..", "tool", "capture", "fixtures");
const DAY = 86_400_000;

function memoryArea(items: Record<string, unknown> = {}): AsyncStorageArea {
  return {
    async get(keys) {
      const wanted = keys == null ? Object.keys(items) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(wanted.filter((k) => k in items).map((k) => [k, items[k]]));
    },
    async set(more) {
      Object.assign(items, more);
    },
  };
}

describe("the thirty days of statistics (D4)", () => {
  const nowMs = Date.UTC(2026, 9, 14, 15, 30);

  it("is written under the key the extension reads", () => {
    expect(DAILY_KEY).toBe(STORE_DAILY_KEY);
  });

  it("covers the statistics' thirty-day range, the run's day the last, read back by dailystats.ts's own reader", async () => {
    expect(DAYS).toBe(30);
    const records = recentDays({ nowMs, languages: ["en", "es"], native: "fr" });
    const stats = await loadDailyStats(memoryArea({ [DAILY_KEY]: records }));
    const today = utcDay(nowMs);
    const english = countsOf(stats, "en");
    const days = Object.keys(english).map(Number);
    expect(Math.min(...days)).toBe(today - 29);
    expect(Math.max(...days)).toBe(today);
    expect(days.length).toBeGreaterThan(20);
    expect(Object.values(english).every((c) => c.exposures > 0 && c.reviews > 0)).toBe(true);
    expect(Object.keys(countsOf(stats, "es")).length).toBeGreaterThan(8);
    expect(new Set(Object.values(stats).flatMap((d) => Object.values(d).map((r) => r.native)))).toEqual(
      new Set(["fr"]),
    );
    // Every one of the thirty days holds a reading: the charts are full.
    expect(Object.keys(stats).length).toBe(30);
  });

  it("follows the day it runs: the same month a week later is a week later", () => {
    const now = recentDays({ nowMs, languages: ["es", "fr"], native: "en" });
    const later = recentDays({ nowMs: nowMs + 7 * DAY, languages: ["es", "fr"], native: "en" });
    const shifted = Object.fromEntries(Object.entries(now).map(([day, value]) => [Number(day) + 7, value]));
    expect(later).toEqual(shifted);
  });
});

describe("the seed's dates, moved to the run's day (D4)", () => {
  const seed = (language: string) =>
    JSON.parse(readFileSync(join(fixtures, "state", `${language}.json`), "utf8")) as {
      knowledge: { declared_level_at: Record<string, number>; updated: Record<string, Record<string, number>> };
      exposure: { counters: Record<string, Record<string, { last_seen: number; last_day: number }>> };
      deck: {
        cards: Record<
          string,
          Record<
            string,
            {
              updated_at: number;
              provenance: { captured_at: number };
              review: { due: number | null; last_review: number | null };
            }
          >
        >;
      };
    };

  it("dates each committed seed by its levels' declaration", () => {
    for (const language of INTERFACE_LANGUAGES) {
      const backup = seed(language);
      const declared = Math.min(...Object.values(backup.knowledge.declared_level_at));
      expect(recordedDay(backup)).toBe(Math.floor(declared / DAY));
    }
    expect(() => recordedDay({ knowledge: {} })).toThrow(/re-record/);
  });

  it("moves every moment by whole days — milliseconds, seconds and days alike — and leaves the rest", () => {
    const backup = seed("fr");
    const moved = shiftBackup(backup, 3);
    expect(moved).not.toBe(backup);
    const [language, declared] = Object.entries(backup.knowledge.declared_level_at)[0];
    expect(moved.knowledge.declared_level_at[language]).toBe(declared + 3 * DAY);
    const [studied, words] = Object.entries(backup.knowledge.updated)[0];
    const [word, at] = Object.entries(words)[0];
    expect(moved.knowledge.updated[studied][word]).toBe(at + 3 * DAY);
    const [counted, counters] = Object.entries(backup.exposure.counters)[0];
    const [form, counter] = Object.entries(counters)[0];
    expect(moved.exposure.counters[counted][form]).toMatchObject({
      last_seen: counter.last_seen + 3 * 86_400,
      last_day: counter.last_day + 3,
    });
    const cards = Object.entries(backup.deck.cards).flatMap(([l, byLemma]) =>
      Object.entries(byLemma).map(([lemma, card]) => ({ l, lemma, card })),
    );
    const answered = cards.find(({ card }) => card.review.due !== null)!;
    const fresh = cards.find(({ card }) => card.review.due === null)!;
    const movedCard = (c: { l: string; lemma: string }) => moved.deck.cards[c.l][c.lemma];
    expect(movedCard(answered).review.due).toBe(answered.card.review.due! + 3 * 86_400);
    expect(movedCard(answered).review.last_review).toBe(answered.card.review.last_review! + 3 * 86_400);
    expect(movedCard(fresh).review.due).toBeNull();
    expect(movedCard(fresh).provenance.captured_at).toBe(fresh.card.provenance.captured_at + 3 * 86_400);
    expect(movedCard(fresh).updated_at).toBe(fresh.card.updated_at + 3 * 86_400);
    // Nothing else moves: statuses, levels, glosses, the review's parameters.
    expect({ ...moved, knowledge: {}, exposure: {}, deck: {} }).toEqual({
      ...backup,
      knowledge: {},
      exposure: {},
      deck: {},
    });
    expect(shiftBackup(backup, 0)).toEqual(backup);
  });

  it("stands the restored reader at the run as at the recording, whatever day it runs", () => {
    const backup = seed("en");
    const recorded = recordedDay(backup);
    const runs = [recorded, recorded + 1, recorded + 40].map((day) => seedForRun(backup, day * DAY + 3_600_000));
    expect(runs.map((run) => recordedDay(run) - recorded)).toEqual([0, 1, 40]);
    expect(runs[0]).toEqual(backup);
    // The answered cards stay due after the run's day, as they were after the recording's.
    for (const run of runs) {
      const today = recordedDay(run);
      for (const byLemma of Object.values(run.deck.cards)) {
        for (const card of Object.values(byLemma)) {
          if (card.review.due !== null) expect(card.review.due).toBeGreaterThan(today * 86_400);
        }
      }
    }
  });
});

describe("the books (D3)", () => {
  beforeEach(useNodeBlob);
  afterEach(() => vi.unstubAllGlobals());

  it("each committed chapter has its header and its text, and no distributor's name", () => {
    for (const language of ["en", "es", "fr"]) {
      const text = readFileSync(join(fixtures, "books", `${language}.txt`), "utf8");
      const book = parseBook(text);
      expect(book.language).toBe(language);
      expect(book.paragraphs.length).toBeGreaterThan(20);
      expect(text).not.toMatch(/gutenberg|\[Illustration|_\w/i);
    }
  });

  it("parses a header, then paragraphs a blank line apart", () => {
    const book = parseBook("Title: T\nAuthor: A\nLanguage: en\nChapter: I\n\nOne\nline.\n\nTwo.\n");
    expect(book).toEqual({ title: "T", author: "A", language: "en", chapter: "I", paragraphs: ["One line.", "Two."] });
    expect(() => parseBook("Title: T\n\nText")).toThrow(/no author/);
    expect(() => parseBook("no header at all")).toThrow(/header/);
  });

  it("escapes what XML would read as markup", () => {
    expect(xmlEscape(`a < b & "c" > d`)).toBe("a &lt; b &amp; &quot;c&quot; &gt; d");
  });

  it("assembles an EPUB the reader recognises, with its title, author and language", async () => {
    const book = parseBook(readFileSync(join(fixtures, "books", "es.txt"), "utf8"));
    const bytes = await assembleEpub(book, "urn:test:es");
    const archive = await openArchive(new Blob([bytes]));
    expect(await isEpubArchive(archive)).toBe(true);
    expect(archive.names[0]).toBe("mimetype");
    const meta = await readEpubMeta(archive, "fallback");
    expect(meta).toMatchObject({ title: "Marianela", authors: ["Benito Pérez Galdós"], language: "es" });
    const chapter = await archive.loadText("OEBPS/chapter-1.xhtml");
    expect(chapter).toContain("<p>Se puso el sol.");
    expect(chapter?.match(/<p>/g)?.length).toBe(book.paragraphs.length);
  });

  it("gives the same book the same bytes", async () => {
    const book = parseBook("Title: T\nAuthor: A\nLanguage: en\nChapter: I\n\nOne.\n");
    expect(await assembleEpub(book, "id")).toEqual(await assembleEpub(book, "id"));
    expect(Object.keys(epubEntries(book, "id"))[0]).toBe("mimetype");
  });
});

describe("images (D7)", () => {
  /** The first 29 bytes of a PNG: signature, IHDR length and type, width, height, depth, colour type. */
  function pngStart(width: number, height: number, colourType: number): Uint8Array {
    const bytes = new Uint8Array(33);
    bytes.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13, 0x49, 0x48, 0x44, 0x52]);
    const view = new DataView(bytes.buffer);
    view.setUint32(16, width);
    view.setUint32(20, height);
    bytes[24] = 8;
    bytes[25] = colourType;
    return bytes;
  }

  it("reads a PNG's size and colour type", () => {
    expect(pngHeader(pngStart(1280, 800, 2))).toEqual({ width: 1280, height: 800, colourType: 2 });
    expect(() => pngHeader(new Uint8Array(40))).toThrow(/not a PNG/);
  });

  it("refuses a store capture with alpha, or of another size, naming the file", () => {
    expect(() => assertStorePng("1-card.png", pngStart(1280, 800, 2), { width: 1280, height: 800 })).not.toThrow();
    expect(() => assertStorePng("1-card.png", pngStart(1280, 800, 6), { width: 1280, height: 800 })).toThrow(
      /1-card\.png: colour type 6/,
    );
    expect(() => assertStorePng("tile.png", pngStart(440, 281, 2), { width: 440, height: 280 })).toThrow(
      /tile\.png: 440×281/,
    );
  });

  it("crops around a box at an aspect, inside the window", () => {
    const bounds = { width: 1280, height: 800 };
    const card = { x: 900, y: 300, width: 320, height: 360 };
    const share = cropAround({ box: card, aspect: 1200 / 630, padding: 40, minWidth: 900, bounds });
    expect(share.width / share.height).toBeCloseTo(1200 / 630, 1);
    expect(share.x).toBeGreaterThanOrEqual(0);
    expect(share.x + share.width).toBeLessThanOrEqual(1280);
    expect(share.y + share.height).toBeLessThanOrEqual(800);
    expect(share.x).toBeLessThanOrEqual(card.x - 40);
    // A tall box: its height decides, the width follows the aspect.
    const closeUp = cropAround({ box: { x: 100, y: 0, width: 200, height: 700 }, aspect: 4 / 5, padding: 20, bounds });
    expect(closeUp).toEqual({ x: 0, y: 0, width: 592, height: 740 });
    // Larger than the window: shrunk to it.
    expect(cropAround({ box: { x: 0, y: 0, width: 2000, height: 100 }, aspect: 2, bounds })).toEqual({
      x: 0,
      y: 0,
      width: 1280,
      height: 640,
    });
  });

  it("starts a crop at the window's top rather than through a title just above, when its box still fits", () => {
    const bounds = { width: 1280, height: 800 };
    const card = { x: 530, y: 195, width: 280, height: 275 };
    const plain = cropAround({ box: card, aspect: 1200 / 630, padding: 40, minWidth: 960, bounds });
    expect(plain.y).toBeGreaterThan(50);
    expect(cropAround({ box: card, aspect: 1200 / 630, padding: 40, minWidth: 960, snapTop: 120, bounds }).y).toBe(0);
    // A box that would not fit from the top keeps its centred crop.
    const low = { ...card, y: 400 };
    expect(cropAround({ box: low, aspect: 1200 / 630, padding: 40, minWidth: 960, snapTop: 120, bounds })).toEqual(
      cropAround({ box: low, aspect: 1200 / 630, padding: 40, minWidth: 960, bounds }),
    );
  });

  it("crops a figure from the start of its text column, moved right only as far as the card needs", () => {
    const bounds = { width: 1280, height: 800 };
    const column = { x: 320, y: 0, width: 640, height: 2000 };
    // A card within reach: the crop starts 28 px before the column, its lines whole.
    const near = columnCrop({
      column,
      focus: { x: 500, y: 300, width: 300, height: 200 },
      width: 600,
      aspect: 4 / 5,
      padding: 28,
      bounds,
    });
    expect(near).toEqual({ x: 292, y: 25, width: 600, height: 750 });
    // A card further right: the crop moves just enough to hold it and its padding.
    const far = columnCrop({
      column,
      focus: { x: 800, y: 300, width: 300, height: 200 },
      width: 600,
      aspect: 4 / 5,
      padding: 28,
      bounds,
    });
    expect(far.x + far.width).toBe(1128);
    // Never out of the window.
    const edge = columnCrop({
      column,
      focus: { x: 1100, y: 700, width: 170, height: 90 },
      width: 600,
      aspect: 4 / 5,
      bounds,
    });
    expect(edge).toEqual({ x: 670, y: 50, width: 600, height: 750 });
  });

  it("pads a box inside the window, less below it when asked", () => {
    const bounds = { width: 1280, height: 800 };
    expect(padded({ x: 380, y: 10, width: 520, height: 480 }, 28, bounds)).toEqual({
      x: 352,
      y: 0,
      width: 576,
      height: 518,
    });
    expect(padded({ x: 380, y: 40, width: 520, height: 480 }, 28, bounds, { bottom: 12 })).toEqual({
      x: 352,
      y: 12,
      width: 576,
      height: 520,
    });
    expect(padded({ x: 1200, y: 700, width: 100, height: 200 }, 28, bounds)).toEqual({
      x: 1172,
      y: 672,
      width: 108,
      height: 128,
    });
  });

  it("joins boxes and scales them to device pixels", () => {
    expect(union({ x: 10, y: 20, width: 5, height: 5 }, { x: 0, y: 30, width: 50, height: 10 })).toEqual({
      x: 0,
      y: 20,
      width: 50,
      height: 20,
    });
    expect(scaled({ x: 1.2, y: 2, width: 3, height: 4.4 }, 2)).toEqual({ x: 2, y: 4, width: 6, height: 9 });
  });

  it("gives ffmpeg a crop, a scale and each format's encoder", () => {
    const png = imageArgs({
      input: "in.png",
      output: "out.png",
      format: "png",
      crop: { x: 2, y: 4, width: 100, height: 50 },
      width: 1280,
      height: 800,
    });
    expect(png).toContain("crop=100:50:2:4,scale=1280:800:flags=lanczos");
    expect(png.join(" ")).toContain("-pix_fmt rgb24");
    expect(png.at(-1)).toBe("out.png");
    const webp = imageArgs({ input: "in.png", output: "out.webp", format: "webp", quality: 80 });
    expect(webp.join(" ")).toContain("-c:v libwebp -quality 80");
    expect(webp).not.toContain("-vf");
    expect(imageArgs({ input: "a", output: "b.jpg", format: "jpeg", width: 1200 })).toContain(
      "scale=1200:-2:flags=lanczos",
    );
  });

  it("encodes the video without sound, its index first, in 4:2:0 H.264", () => {
    const args = videoArgs({ input: "in.webm", output: "demo.mp4", start: 1.5, duration: 30 }).join(" ");
    expect(args).toContain("-ss 1.5 -t 30 -i in.webm -an");
    expect(args).toContain("-c:v libx264");
    expect(args).toContain("-pix_fmt yuv420p");
    expect(args).toContain("-movflags +faststart");
  });
});

describe("the capture build against a plain one (D1)", () => {
  let root: string;
  beforeEach(() => {
    root = mkdtempSync(join(tmpdir(), "capture-build-"));
  });
  afterEach(() => rmSync(root, { recursive: true, force: true }));

  const plainManifest = { manifest_version: 3, name: "Cymbra Lingua", version: "1.9.0" };
  function tree(name: string, files: Record<string, string>): string {
    const dir = join(root, name);
    for (const [path, content] of Object.entries(files)) {
      mkdirSync(join(dir, path, ".."), { recursive: true });
      writeFileSync(join(dir, path), content);
    }
    return dir;
  }
  const manifest = (m: object) => JSON.stringify(m);

  it("accepts a capture build that adds the static reader and nothing else", () => {
    const plain = tree("plain", { "manifest.json": manifest(plainManifest), "content.js": "x", "wasm/a.wasm": "w" });
    const capture = tree("capture", {
      "manifest.json": manifest({ ...plainManifest, content_scripts: [STATIC_READER] }),
      "content.js": "x",
      "wasm/a.wasm": "w",
    });
    expect(buildDifferences(capture, plain)).toEqual([]);
  });

  it("names a differing file, a missing one and an extra one", () => {
    const plain = tree("plain", { "manifest.json": manifest(plainManifest), "content.js": "x", "popup.js": "p" });
    const capture = tree("capture", {
      "manifest.json": manifest({ ...plainManifest, content_scripts: [STATIC_READER] }),
      "content.js": "y",
      "extra.js": "e",
    });
    expect(buildDifferences(capture, plain)).toEqual([
      "extra.js is only in the capture build",
      "popup.js is missing from the capture build",
      "content.js differs",
    ]);
  });

  it("knows the extension's id from its key, and stops without one", () => {
    // The committed development key (build.mjs) pins this id.
    const key = readFileSync(join(__dirname, "..", "build.mjs"), "utf8").match(/"(MIIBIjAN[^"]+)"/)?.[1];
    expect(key).toBeTruthy();
    expect(extensionId(key!)).toBe("figfjglfdiffocldficbimecjnhnkhkh");
    expect(captureExtensionId({ key: key! })).toBe("figfjglfdiffocldficbimecjnhnkhkh");
    expect(() => captureExtensionId({})).toThrow(/LINGUA_EXT_KEY is set to an empty value.*unset it/);
  });

  it("names any other manifest difference", () => {
    expect(manifestDifferences({ ...plainManifest, content_scripts: [STATIC_READER] }, plainManifest)).toEqual([]);
    expect(manifestDifferences(plainManifest, plainManifest)[0]).toMatch(/not the static reader alone/);
    expect(
      manifestDifferences({ ...plainManifest, permissions: ["tabs"], content_scripts: [STATIC_READER] }, plainManifest),
    ).toEqual(["the manifests also differ in permissions"]);
  });
});

describe("finding a control inside a closed shadow root (D2)", () => {
  const text = (value: string): CdpNode => ({ nodeType: 3, nodeValue: value });
  const element = (localName: string, attributes: string[], children: CdpNode[] = [], shadowRoots?: CdpNode[]) => ({
    nodeType: 1,
    localName,
    attributes,
    children,
    shadowRoots,
  });
  const document: CdpNode = {
    nodeType: 9,
    children: [
      element(
        "html",
        [],
        [
          element(
            "body",
            [],
            [
              element("button", ["class", "primary"], [text("+ Deck")]),
              element(
                "div",
                ["id", "cymbra-lingua-host"],
                [],
                [
                  {
                    nodeType: 11,
                    children: [
                      element(
                        "div",
                        ["class", "card"],
                        [
                          element(
                            "div",
                            ["class", "actions"],
                            [
                              element("button", [], [text("Je connais")]),
                              element("button", ["class", "primary"], [text(" + Deck ")]),
                            ],
                          ),
                        ],
                      ),
                    ],
                  },
                ],
              ),
            ],
          ),
        ],
      ),
    ],
  };

  it("looks only under the named host, through its shadow root", () => {
    const deck = findIn(document, "cymbra-lingua-host", { tag: "button", text: "+ Deck" });
    expect(deck && attributesOf(deck)).toEqual({ class: "primary" });
    expect(findIn(document, "cymbra-lingua-host", { className: "card" })?.localName).toBe("div");
    expect(findIn(document, "cymbra-lingua-drawer-host", { tag: "button" })).toBeNull();
    expect(findIn(document, "cymbra-lingua-host", { tag: "button", text: "Ignorer" })).toBeNull();
  });

  it("reads a node's text and a quad's centre", () => {
    expect(nodeText(element("p", [], [text("a "), element("b", [], [text("b")])]))).toBe("a b");
    expect(quadCentre([10, 20, 30, 20, 30, 60, 10, 60])).toEqual({ x: 20, y: 40 });
  });

  it("leaves out the text under a class when asked: a gloss without its part-of-speech heading", () => {
    const gloss = element(
      "div",
      ["class", "gloss"],
      [
        element(
          "div",
          ["class", "sense-group"],
          [element("span", ["class", "pos"], [text("nom féminin")]), text("Empreinte")],
        ),
      ],
    );
    expect(nodeText(gloss)).toBe("nom fémininEmpreinte");
    expect(nodeText(gloss, "pos")).toBe("Empreinte");
  });
});

describe("what the tool looks for exists in the extension (D2)", () => {
  /** Every source file of the extension, read once. */
  const sources = (() => {
    const out: string[] = [];
    const visit = (dir: string) => {
      for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const path = join(dir, entry.name);
        if (entry.isDirectory()) visit(path);
        else if (/\.(ts|html|css)$/.test(entry.name)) out.push(readFileSync(path, "utf8"));
      }
    };
    visit(join(__dirname, "..", "src"));
    return out.join("\n");
  })();
  const tool = readFileSync(join(__dirname, "..", "tool", "capture", "capture.mjs"), "utf8");

  it("each surface's shadow host is an id the extension gives", () => {
    for (const id of Object.values(HOSTS)) expect(sources, id).toContain(`"${id}"`);
  });

  it("each class the tool matches inside a surface, and each class and id its selectors name, is the extension's", () => {
    const classNames = [...tool.matchAll(/className: "([\w-]+)"/g)].map((m) => m[1]);
    const selectors = [...tool.matchAll(/(?:locator|querySelector|waitFor)\(\s*[`'"]([^`'"]+)[`'"]/g)].map((m) => m[1]);
    const fromSelectors = selectors.flatMap((s) =>
      [...s.replace(/\$\{[^}]*\}/g, "").matchAll(/([.#])([a-zA-Z][\w-]*)/g)].map((m) => m[2]),
    );
    const names = [...new Set([...classNames, ...fromSelectors])];
    expect(classNames.length).toBeGreaterThan(8);
    expect(fromSelectors.length).toBeGreaterThan(10);
    const missing = names.filter((name) => !new RegExp(`["'\\s.#]${name}["'\\s:{,.\\[]`).test(sources));
    expect(missing).toEqual([]);
  });
});

describe("the build's output folder (D1)", () => {
  const app = "/work/repo/apps/lingua-extension";

  it("takes a folder named dist-…, inside the app or elsewhere", () => {
    expect(distDirProblem(app, "dist-capture")).toBeNull();
    expect(distDirProblem(app, "/tmp/dist-plain-a1b2c3")).toBeNull();
    expect(distDirProblem(app, "build/dist-chromium")).toBeNull();
  });

  it("refuses the app's folder, a folder holding it, and any other name: the build empties it first", () => {
    for (const value of [".", "./", "..", "../..", "/", app, "/work/repo"]) {
      expect(distDirProblem(app, value), value).toMatch(/app's folder or one holding it/);
    }
    for (const value of ["src", "dist", "out", "capture-dist", "/tmp/lingua-plain-x"]) {
      expect(distDirProblem(app, value), value).toMatch(/dist-…/);
    }
  });

  it("is what build.mjs checks before writing", () => {
    const run = spawnSync(process.execPath, ["build.mjs", "chromium"], {
      cwd: join(__dirname, ".."),
      env: { ...process.env, LINGUA_DIST_DIR: "src" },
      encoding: "utf8",
    });
    expect(run.status).not.toBe(0);
    expect(run.stderr).toMatch(/LINGUA_DIST_DIR \(src\) must name a folder called dist-/);
  });
});

describe("the shot plan (D5)", () => {
  /** The fixture page's text, its markup stripped. */
  const pageText = (language: string) =>
    readFileSync(join(fixtures, "pages", `${language}.html`), "utf8")
      .replace(/<[^>]+>/g, " ")
      .replace(/\s+/g, " ");
  const hasWord = (text: string, word: string) =>
    new RegExp(`(^|[^\\p{L}\\p{N}])${word}(?![\\p{L}\\p{N}])`, "iu").test(text);

  it("sends five captures to the stores and six to the site, the reader last", () => {
    expect(STORE_SHOTS).toEqual(["card", "second", "review", "stats", "reader"]);
    expect(SITE_SHOTS).toEqual([...STORE_SHOTS, "reader-eink"]);
  });

  it("gives each interface language two studied languages other than its own, with a page and a book", () => {
    for (const language of INTERFACE_LANGUAGES) {
      const studied = STUDIED[language as keyof typeof STUDIED];
      expect(studied).toHaveLength(2);
      expect(studied).not.toContain(language);
      for (const s of studied) expect(BOOK[s as keyof typeof BOOK]).toBe(s);
    }
  });

  it("names words its fixture page holds, and never seeds a word a shot or the video clicks", () => {
    for (const interfaceLanguage of INTERFACE_LANGUAGES) {
      for (const language of STUDIED[interfaceLanguage]) {
        const words = wordsFor(interfaceLanguage, language);
        const text = pageText(language);
        const { names, ...onThePage } = words;
        for (const word of Object.values(onThePage).flat()) {
          expect(hasWord(text, word), `${interfaceLanguage}/${language}: ${word}`).toBe(true);
        }
        const seeded = new Set([...words.seedKnown, ...words.seedDeck]);
        for (const word of [...words.card, ...words.second, ...words.known]) {
          expect(seeded.has(word), `${interfaceLanguage}/${language}: ${word} is seeded`).toBe(false);
        }
        // The video marks words known after « + Deck » on the hero's verb: never that verb.
        expect(words.known).not.toContain(words.card[0]);
        expect(names.length).toBeGreaterThan(0);
      }
    }
  });

  it("answers the deck's first cards in the seed's review, so the review capture reveals the next one", () => {
    // `seedDeck`'s fifth word is the first new card: as many answers as cards before it, one missed.
    expect(SEED_REVIEW).toHaveLength(4);
    expect(SEED_REVIEW.filter((a) => a === "missed")).toHaveLength(1);
    for (const words of Object.values(WORDS)) expect(words.seedDeck.length).toBeGreaterThan(SEED_REVIEW.length);
  });

  it("names, for the book the seed opens, words the book holds", () => {
    for (const language of Object.keys(BOOK) as (keyof typeof BOOK)[]) {
      const book = readFileSync(join(fixtures, "books", `${BOOK[language]}.txt`), "utf8");
      for (const name of WORDS[language].names) expect(hasWord(book, name), `${language}: ${name}`).toBe(true);
    }
  });

  it("overrides words only for an interface language that studies that language", () => {
    for (const [interfaceLanguage, byStudied] of Object.entries(OVERRIDES)) {
      for (const studied of Object.keys(byStudied ?? {})) {
        expect(STUDIED[interfaceLanguage as keyof typeof STUDIED]).toContain(studied);
      }
    }
    expect(wordsFor("es", "en").card).toEqual(OVERRIDES.es!.en!.card);
    expect(wordsFor("fr", "en").card).toEqual(WORDS.en.card);
    expect(wordsFor("es", "en").seedDeck).toEqual(WORDS.en.seedDeck);
  });

  it("declares a reader's level among the scale's, and reads one clock", () => {
    for (const level of Object.values(READER_LEVELS)) expect(["A1", "A2", "B1", "B2", "C1", "C2"]).toContain(level);
    expect(TIME_ZONE).toBe("Europe/Paris");
  });
});

describe("a gloss a capture shows (D5)", () => {
  it("reads as a translation: short, without the dictionary's parentheses or a first name", () => {
    for (const gloss of ["Émission", "airline", "Emisión", "Impresión, huella, indicio", "sleeping berth"]) {
      expect(glossProblems(gloss), gloss).toEqual([]);
    }
  });

  it("is refused empty, long, with a parenthesis or a first name's sense", () => {
    expect(glossProblems("")).toEqual(["no translation"]);
    expect(glossProblems(null)).toEqual(["no translation"]);
    expect(glossProblems("imprint (any visible mark caused by pressure)")).toEqual(["a parenthesis"]);
    expect(glossProblems("Aube, aurore; Lever de soleil (sunrise); Prénom féminin anglophone")).toEqual([
      "66 characters, more than 60",
      "a parenthesis",
      "a first name",
    ]);
    expect(glossProblems("Amanecer; given name")).toEqual(["a first name"]);
  });
});
