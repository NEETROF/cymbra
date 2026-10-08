/// <reference types="vite/client" />
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  COPY_PENDING_ATTR,
  DEFAULT_INTERFACE_LANGUAGE,
  fillPage,
  fillPageInLanguage,
  fillSlots,
  formatCount,
  formatDate,
  formatNumber,
  formatPercent,
  type GrammarRenderer,
  INTERFACE_LANGUAGE_KEY,
  type InterfaceLanguage,
  interfaceLanguage,
  isInterfaceLanguage,
  NODE_SLOT,
  plural,
  type PluralForms,
  regionName,
  renderAround,
  setDocumentLanguage,
  slot,
  SURFACES,
} from "@/i18n/index.ts";
import { popup as enPopup } from "@/i18n/en/popup.ts";
import { popup as esPopup } from "@/i18n/es/popup.ts";
import { popup as frPopup } from "@/i18n/fr/popup.ts";
import { review as frReview } from "@/i18n/fr/review.ts";
import { review as enReview } from "@/i18n/en/review.ts";
import { review as esReview } from "@/i18n/es/review.ts";
import { colours as enColours } from "@/i18n/en/colours.ts";
import { card as frCard } from "@/i18n/fr/card.ts";
import { colours as frColours } from "@/i18n/fr/colours.ts";
import { settings as frSettings } from "@/i18n/fr/settings.ts";
import { stats as frStats } from "@/i18n/fr/stats.ts";
import { sync as frSync } from "@/i18n/fr/sync.ts";
import { translation as frTranslation } from "@/i18n/fr/translation.ts";
import { account as frAccount } from "@/i18n/fr/account.ts";
import { lineText } from "@/reading/grammar-description.ts";

// The catalogue (add-lingua-interface-language): the key every surface reads before its copy, the
// helpers a count and a figure go through, and the parity of the English and Spanish drafts with the
// French source — what the compiler says about the key set, this says about the values.

const NNBSP = "\u202F";
const NBSP = "\u00A0";
/** The narrow no-break space `fr-FR` groups with (what the surfaces write today). */
const NARROW = (1000).toLocaleString("fr-FR").charAt(1);

/** A surface's preferences area, with no engine behind it. */
function fakeArea(seed: Record<string, unknown> = {}) {
  return {
    async get(key: string) {
      return key in seed ? { [key]: seed[key] } : {};
    },
  };
}

describe("the interface language, under its own key", () => {
  it("Every installed reader: an absent key reads French", async () => {
    expect(await interfaceLanguage(fakeArea())).toBe("fr");
    expect(DEFAULT_INTERFACE_LANGUAGE).toBe("fr");
    expect(INTERFACE_LANGUAGE_KEY).toBe("cymbra-lingua-interface-language");
  });

  it("A surface before any engine: the key is read from the area alone", async () => {
    expect(await interfaceLanguage(fakeArea({ [INTERFACE_LANGUAGE_KEY]: "es" }))).toBe("es");
    expect(await interfaceLanguage(fakeArea({ [INTERFACE_LANGUAGE_KEY]: "en" }))).toBe("en");
  });

  it("an unknown value reads French too", async () => {
    expect(await interfaceLanguage(fakeArea({ [INTERFACE_LANGUAGE_KEY]: "de" }))).toBe("fr");
    expect(await interfaceLanguage(fakeArea({ [INTERFACE_LANGUAGE_KEY]: 3 }))).toBe("fr");
    expect(isInterfaceLanguage("es")).toBe(true);
    expect(isInterfaceLanguage("pt")).toBe(false);
  });

  it("names the document's language", () => {
    setDocumentLanguage(document, "es");
    expect(document.documentElement.lang).toBe("es");
    setDocumentLanguage(document, "fr");
    expect(document.documentElement.lang).toBe("fr");
  });

  describe("a read that fails (localise-lingua-reading-surfaces D2)", () => {
    afterEach(() => {
      vi.restoreAllMocks();
    });

    it("answers French, and says why in the console", async () => {
      const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
      const failure = new Error("storage unavailable");
      expect(await interfaceLanguage({ get: () => Promise.reject(failure) })).toBe("fr");
      expect(warn).toHaveBeenCalledTimes(1);
      expect(warn.mock.calls[0]![0]).toMatch(/^\[Cymbra Lingua\] /);
      expect(warn.mock.calls[0]![1]).toBe(failure);
    });

    it("answers French when the area throws before it answers (no chrome.storage in the context)", async () => {
      vi.spyOn(console, "warn").mockImplementation(() => {});
      const area = {
        get(): Promise<Record<string, unknown>> {
          throw new TypeError("Cannot read properties of undefined (reading 'local')");
        },
      };
      expect(await interfaceLanguage(area)).toBe("fr");
    });

    it("answers French when the area answers nothing at all", async () => {
      const area = { get: async () => undefined as unknown as Record<string, unknown> };
      expect(await interfaceLanguage(area)).toBe("fr");
    });
  });
});

describe("a count in each language", () => {
  it("A count in each language: the review's remaining cards for 1 and 3", () => {
    expect(plural("fr", 1, frReview.remaining)).toBe("1 carte(s) à revoir");
    expect(plural("fr", 3, frReview.remaining)).toBe("3 carte(s) à revoir");
    expect(plural("en", 1, enReview.remaining)).toBe("1 card to review");
    expect(plural("en", 3, enReview.remaining)).toBe("3 cards to review");
    expect(plural("es", 1, esReview.remaining)).toBe("1 tarjeta por repasar");
    expect(plural("es", 3, esReview.remaining)).toBe("3 tarjetas por repasar");
  });

  it("A French format Intl would change: the raw count, the megabytes, the minutes", () => {
    expect(plural("fr", 1234, frReview.remaining)).toBe("1234 carte(s) à revoir");
    expect(plural("en", 1234, enReview.remaining)).toBe("1,234 cards to review");
    expect(plural("es", 1234, esReview.remaining)).toBe("1234 tarjetas por repasar");
    const oneDecimal = { minimumFractionDigits: 1, maximumFractionDigits: 1 };
    expect(frTranslation.megabytes(formatNumber("fr", 25.8, oneDecimal))).toBe("25,8 Mo");
    expect(frSync.syncedMinutesAgo("3")).toBe("Synchronisé il y a 3 min.");
    expect(frSync.lastSyncOn(formatDate("fr", new Date(2026, 9, 4)))).toBe("Dernière synchronisation le 04/10/2026.");
  });

  it("many, for French and Spanish at a million; English falls back to other", () => {
    const forms: PluralForms = { one: (n) => `one ${n}`, many: (n) => `many ${n}`, other: (n) => `other ${n}` };
    expect(plural("fr", 1_000_000, forms)).toBe("many 1000000");
    expect(plural("es", 1_000_000, forms)).toBe(`many 1${NNBSP}000${NNBSP}000`);
    expect(plural("en", 1_000_000, forms)).toBe("other 1,000,000");
    const twoForms: PluralForms = { one: (n) => `one ${n}`, other: (n) => `other ${n}` };
    expect(plural("fr", 1_000_000, twoForms)).toBe("other 1000000");
    expect(plural("fr", 0, twoForms)).toBe("one 0");
  });

  it("formats a figure as each language writes it, the Spanish as the RAE does", () => {
    expect(formatNumber("fr", 20_000)).toBe(`20${NARROW}000`);
    expect(formatNumber("en", 20_000)).toBe("20,000");
    expect(formatNumber("es", 5_000)).toBe("5000");
    expect(formatNumber("es", 20_000)).toBe(`20${NNBSP}000`);
    expect(formatNumber("es", 1_234_567)).toBe(`1${NNBSP}234${NNBSP}567`);
    expect(formatNumber("es", -12_345)).toBe(`-12${NNBSP}345`);
    const oneDecimal = { minimumFractionDigits: 1, maximumFractionDigits: 1 };
    expect(formatNumber("es", 25.8, oneDecimal)).toBe("25,8");
    expect(formatNumber("es", 12345.6, oneDecimal)).toBe(`12${NNBSP}345,6`);
    expect(formatNumber("en", 25.8, oneDecimal)).toBe("25.8");
  });

  it("writes a count as each language does: the French bare, as the surfaces write it today", () => {
    expect(formatCount("fr", 0)).toBe("0");
    expect(formatCount("fr", 1234)).toBe("1234");
    expect(formatCount("fr", 20_000)).toBe("20000");
    expect(formatCount("en", 1234)).toBe("1,234");
    expect(formatCount("es", 1234)).toBe("1234");
    expect(formatCount("es", 12_345)).toBe(`12${NNBSP}345`);
    // The popup's « Réviser (n) », French as the page wrote it, the others grouped.
    expect(frPopup.review(formatCount("fr", 1234))).toBe("Réviser (1234)");
    expect(enPopup.review(formatCount("en", 1234))).toBe("Review (1,234)");
    expect(esPopup.review(formatCount("es", 12_345))).toBe(`Repasar (12${NNBSP}345)`);
  });

  it("writes a percentage in each surface's French form, the Spanish with its space", () => {
    expect(formatPercent("fr", 45, "tight")).toBe("45%");
    expect(formatPercent("fr", 110, "spaced")).toBe("110 %");
    expect(formatPercent("en", 96, "spaced")).toBe("96%");
    expect(formatPercent("es", 96, "tight")).toBe(`96${NNBSP}%`);
  });

  it("writes a date through each language's locale", () => {
    const date = new Date(2026, 9, 4);
    const long = { day: "numeric", month: "long", year: "numeric" } as const;
    expect(frAccount.linkedOn(formatDate("fr", date, long))).toBe("Lié le 4 octobre 2026");
    expect(formatDate("en", date)).toBe("10/4/2026");
    expect(formatDate("en", date, long)).toBe("October 4, 2026");
    expect(formatDate("es", date)).toBe("4/10/2026");
    expect(formatDate("es", date, long)).toBe("4 de octubre de 2026");
  });

  it("names a region in each language, French as Réglages named it, the code where none is known", () => {
    expect(regionName("fr", "US")).toBe(new Intl.DisplayNames(["fr"], { type: "region" }).of("US"));
    expect(regionName("fr", "GB")).toBe("Royaume-Uni");
    expect(regionName("en", "GB")).toBe("United Kingdom");
    expect(regionName("es", "US")).toBe("Estados Unidos");
    expect(regionName("es", "LATN")).toBe("LATN"); // a script, not a region: the runtime refuses it
  });

  it("keeps the ladder's escapes as the surface wrote them", () => {
    expect(frStats.scopeCommon).toBe(
      `«${NNBSP}courants${NNBSP}»${NBSP}: les mots les plus fréquents jusqu'à ce niveau. `,
    );
    expect(plural("fr", 16_000, frStats.approxWords, "16" + NARROW + "000")).toBe(`≈${NBSP}16${NARROW}000 mots`);
    expect(plural("fr", 20_000, frStats.words, formatNumber("fr", 20_000))).toBe(`20${NARROW}000 mots`);
  });
});

describe("a slot message rendered around its parts (localise-lingua-settings D3)", () => {
  const b = (text: string): HTMLElement => Object.assign(document.createElement("b"), { textContent: text });

  it("A bold count: the French sentence around it, no empty text left at either end", () => {
    const count = b("3000");
    expect(fillSlots(frSettings.knowCommonest(slot(0)), [count])).toEqual([
      "Je connais les ",
      count,
      " mots les plus courants",
    ]);
    const keys = [b("Alt"), "+", b("S")];
    expect(fillSlots(frSettings.shortcutSidePanel(slot(0)), [keys])).toEqual([...keys, " — panneau latéral"]);
  });

  it("Two parts: each where the language put it, whatever the order", () => {
    const [unknown, learning] = [b("U"), b("L")];
    expect(fillSlots(frColours.preview(slot(0), slot(1)), [unknown, learning])).toEqual([
      "Un mot ",
      unknown,
      ", un mot ",
      learning,
      " et un mot connu.",
    ]);
    expect(fillSlots(enColours.preview(slot(0), slot(1)), [unknown, learning])).toEqual([
      "An ",
      unknown,
      " word, a word ",
      learning,
      " and a known word.",
    ]);
    // A language that names the second part first.
    expect(fillSlots(`${slot(1)}, then ${slot(0)}`, [unknown, learning])).toEqual([learning, ", then ", unknown]);
  });

  it("leaves out a part it was not given, and keeps a message without any", () => {
    expect(fillSlots(`a${slot(3)}b`, [])).toEqual(["a", "b"]);
    expect(fillSlots("plain", [])).toEqual(["plain"]);
  });

  it("A translation that dropped a slot: the part it was given is appended, never lost", () => {
    const level = b("B1");
    expect(fillSlots("Level", [level])).toEqual(["Level", level]);
    // The parts it names where it names them, the one it dropped after the message.
    const [first, second] = [b("1"), b("2")];
    expect(fillSlots(`${slot(1)} first`, [first, second])).toEqual([second, " first", first]);
  });

  it("A message that names a part twice: shown twice, a node copied the second time", () => {
    expect(fillSlots(`${slot(0)} + ${slot(0)}`, ["Alt"])).toEqual(["Alt", " + ", "Alt"]);
    const level = b("B1");
    const nodes = fillSlots(`${slot(0)} — ${slot(0)}`, [level]);
    expect(nodes[0]).toBe(level);
    expect(nodes[1]).toBe(" — ");
    expect(nodes[2]).not.toBe(level);
    expect((nodes[2] as HTMLElement).outerHTML).toBe("<b>B1</b>");
    const line = document.createElement("span");
    line.append(...nodes);
    expect(line.textContent).toBe("B1 — B1");
  });

  it("One mechanism: a node's slot and a numbered slot render through either helper", () => {
    expect(NODE_SLOT).toBe(slot(0));
    // Change 15's message through change 14's helper: the count in place, no stray "0".
    const label = document.createElement("label");
    const count = b("3000");
    renderAround(label, frSettings.knowCommonest(slot(0)), count);
    expect(label.textContent).toBe("Je connais les 3000 mots les plus courants");
    expect(label.querySelector("b")).toBe(count);
    // Change 14's message through change 15's helper: the node and the closing guillemet kept.
    const form = b("vino");
    expect(fillSlots(frCard.seenForm(NODE_SLOT), [form])).toEqual(["forme vue : « ", form, " »"]);
  });
});

describe("a page filled from the catalogue (localise-lingua-reading-surfaces D2)", () => {
  /** A page's skeleton, as the HTML pages are written: no text, keys on the nodes, `<html>` pending. */
  function page(): Document {
    return new DOMParser().parseFromString(
      [
        `<!doctype html><html ${COPY_PENDING_ATTR}><head><title data-copy="title"></title></head><body>`,
        '<h1 data-copy="heading"></h1>',
        '<button id="gear" data-copy-aria-label="settings" data-copy-title="settings">⚙</button>',
        '<p id="missing" data-copy="noSuchKey">kept</p>',
        '<p id="slot" data-copy="review">kept too</p>',
        "</body></html>",
      ].join(""),
      "text/html",
    );
  }
  const copy = { title: "Cymbra Lingua — révision", heading: "Cymbra Lingua", settings: "Réglages", review: () => "" };

  it("A page before its script: the text nodes and the attributes take their entries, then the page shows", () => {
    const doc = page();
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(true);
    fillPage(doc, copy);
    expect(doc.title).toBe("Cymbra Lingua — révision");
    expect(doc.querySelector("h1")?.textContent).toBe("Cymbra Lingua");
    const gear = doc.getElementById("gear")!;
    expect(gear.getAttribute("aria-label")).toBe("Réglages");
    expect(gear.getAttribute("title")).toBe("Réglages");
    expect(gear.textContent).toBe("⚙"); // an attribute key fills the attribute, not the text
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
  });

  it("leaves a node whose key is not a plain text of the module, and shows the page anyway", () => {
    const doc = page();
    fillPage(doc, copy);
    expect(doc.getElementById("missing")?.textContent).toBe("kept");
    expect(doc.getElementById("slot")?.textContent).toBe("kept too"); // a slot message is the script's
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
  });

  it("fills the same page in another language", () => {
    const doc = page();
    fillPage(doc, { ...copy, heading: "Cymbra Lingua", settings: "Settings", title: "Cymbra Lingua — review" });
    setDocumentLanguage(doc, "en");
    expect(doc.title).toBe("Cymbra Lingua — review");
    expect(doc.getElementById("gear")?.getAttribute("aria-label")).toBe("Settings");
    expect(doc.documentElement.lang).toBe("en");
  });

  it("writes only the attributes that carry words, and replaces a keyed node's whole text", () => {
    const doc = new DOMParser().parseFromString(
      [
        `<!doctype html><html ${COPY_PENDING_ATTR}><body>`,
        '<input id="search" data-copy-placeholder="settings" />',
        '<img id="logo" data-copy-alt="heading" />',
        '<a id="link" href="#home" data-copy-href="title" data-copy-onclick="title" data-copy-style="title"></a>',
        '<p id="mixed" data-copy="heading"><b>old</b> words</p>',
        "</body></html>",
      ].join(""),
      "text/html",
    );
    fillPage(doc, copy);
    expect(doc.getElementById("search")?.getAttribute("placeholder")).toBe("Réglages");
    expect(doc.getElementById("logo")?.getAttribute("alt")).toBe("Cymbra Lingua");
    const link = doc.getElementById("link")!;
    expect(link.getAttribute("href")).toBe("#home");
    expect(link.hasAttribute("onclick")).toBe(false);
    expect(link.hasAttribute("style")).toBe(false);
    const mixed = doc.getElementById("mixed")!;
    expect(mixed.textContent).toBe("Cymbra Lingua");
    expect(mixed.children).toHaveLength(0);
  });

  describe("opened as a page's script opens it", () => {
    const modules: Record<InterfaceLanguage, typeof copy> = {
      fr: copy,
      en: { ...copy, settings: "Settings" },
      es: { ...copy, settings: "Ajustes" },
    };
    const area = (items: Record<string, unknown>) => ({ get: async (key: string) => ({ [key]: items[key] }) });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it("reads the key, says the language in lang, fills the page and shows it, and hands both back", async () => {
      const doc = page();
      const opened = await fillPageInLanguage(doc, area({ [INTERFACE_LANGUAGE_KEY]: "es" }), (l) => modules[l]);
      expect(opened).toEqual({ language: "es", copy: modules.es });
      expect(doc.documentElement.lang).toBe("es");
      expect(doc.getElementById("gear")?.getAttribute("aria-label")).toBe("Ajustes");
      expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    });

    it("A storage that cannot be read: the page shows anyway, filled in French", async () => {
      vi.spyOn(console, "warn").mockImplementation(() => {});
      const doc = page();
      const opened = await fillPageInLanguage(doc, { get: () => Promise.reject(new Error("gone")) }, (l) => modules[l]);
      expect(opened.language).toBe("fr");
      expect(doc.documentElement.lang).toBe("fr");
      expect(doc.getElementById("gear")?.getAttribute("aria-label")).toBe("Réglages");
      expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    });
  });
});

describe("a slot message rendered around a node", () => {
  function level(text: string): HTMLElement {
    const b = document.createElement("b");
    b.textContent = text;
    return b;
  }

  /** What `element` holds, a node as `<tag>` and a text as itself, in order. */
  const parts = (element: Element): string[] =>
    [...element.childNodes].map((n) => (n.nodeType === Node.TEXT_NODE ? (n.textContent ?? "") : `<${n.nodeName}>`));

  it("puts the node where the French puts it, the label first", () => {
    const line = document.createElement("span");
    const node = level("B1");
    renderAround(line, frPopup.levelLine("Niveau d'anglais", NODE_SLOT), node);
    expect(line.textContent).toBe("Niveau d'anglais : B1");
    expect(parts(line)).toEqual(["Niveau d'anglais : ", "<B>"]);
    expect(line.lastChild).toBe(node);
  });

  it("puts it where the English puts it", () => {
    const line = document.createElement("span");
    renderAround(line, enPopup.levelLine("English level", NODE_SLOT), level("B1"));
    expect(line.textContent).toBe("English level: B1");
  });

  it("follows a message that puts the node first, and keeps the text around it", () => {
    const line = document.createElement("span");
    const levelFirst = (title: string, value: string) => `${value} — ${title} (déclaré)`;
    renderAround(line, levelFirst("niveau d'anglais", NODE_SLOT), level("B1"));
    expect(parts(line)).toEqual(["<B>", " — niveau d'anglais (déclaré)"]);
  });

  it("appends the node to a message that dropped its slot, so the level still shows", () => {
    const line = document.createElement("span");
    renderAround(line, "Level", level("B1"));
    expect(parts(line)).toEqual(["Level", "<B>"]);
    expect(line.textContent).toBe("LevelB1");
  });

  it("replaces what the element held", () => {
    const line = document.createElement("span");
    line.textContent = "stale";
    renderAround(line, `Niveau : ${NODE_SLOT}`, level("A2"));
    expect(line.textContent).toBe("Niveau : A2");
  });
});

// — Parity of the drafts with the French (D6) —

type Catalogue = Record<string, unknown>;
const modules = import.meta.glob<Catalogue>("../src/i18n/{fr,en,es}/*.ts", { eager: true });

function catalogue(language: InterfaceLanguage, surface: string): Catalogue {
  const mod = modules[`../src/i18n/${language}/${surface}.ts`];
  if (!mod) throw new Error(`no ${language} module for ${surface}`);
  const names = Object.keys(mod);
  expect(names, `${language}/${surface} exports one object`).toHaveLength(1);
  return mod[names[0]] as Catalogue;
}

/** Texts that are the same in every language: names, symbols, separators, spoken previews. */
const SAME_EVERYWHERE = new Set([
  "Cymbra Lingua",
  "Cymbra",
  "Google",
  "Apple",
  "—",
  "–",
  "…",
  "⚙",
  "⌄",
  "×",
  "‹",
  "›",
  "✕",
  "ⓘ",
  "Aa",
  "Stats",
  "A−",
  "A+",
  "Alt",
  "S",
  "D",
  "L",
  "+",
  "/",
  "; ",
  " · ",
  "This is how your pages will sound when Lingua reads them aloud.",
  "Así sonarán tus páginas cuando Lingua las lea en voz alta.",
  // The native languages, each named in its own language (add-lingua-native-language-choice D4).
  "Français",
  "English",
  "Español",
]);

/** Texts a language happens to share with the French, and no other. */
const SAME_AS_FRENCH: Record<"en" | "es", Set<string>> = {
  en: new Set(["Email", "Option", "Double", "+ Deck", "CEFR"]),
  es: new Set(),
};

/** Slot messages whose text is only their parts and a symbol — the same shape in every language. */
const SAME_SHAPE_EVERYWHERE = new Set(["@α", "▶ α", "α → β", "α / β", `≈${NBSP}α`, "α…", "α — β"]);
const SAME_SHAPE_AS_FRENCH: Record<"en" | "es", Set<string>> = {
  en: new Set(),
  es: new Set(),
};

/**
 * What a slot message is called with: one sentinel per part, a letter no draft writes, and a
 * different alphabet at each depth of nesting, so that a message returned by a message — an inner
 * form that dropped its own part — is caught even though its text carries the outer one.
 */
const SENTINELS: readonly (readonly string[])[] = [[..."αβγδ"], [..."εζηθ"], [..."ικλμ"]];

type Message = (...args: string[]) => unknown;

interface Text {
  path: string;
  text: string;
  /** The sentinels given on the way to this text: each must appear in it. */
  given: string[];
}

/** The parts each French message takes, by path — what a draft at the same path is called with. */
type Slots = ReadonlyMap<string, number>;

function slotsOf(value: unknown, path: string, into = new Map<string, number>(), depth = 0): Slots {
  if (typeof value === "function") {
    into.set(path, value.length);
    slotsOf((value as Message)(...(SENTINELS[depth] ?? [])), `${path}()`, into, depth + 1);
  } else if (value && typeof value === "object") {
    for (const [key, v] of Object.entries(value as Catalogue)) slotsOf(v, `${path}.${key}`, into, depth);
  }
  return into;
}

/**
 * Every text a value can produce. A message is called with as many sentinels as the French message
 * at its path takes — its own count would let a draft that dropped a part pass — and its texts must
 * contain each one. A `many` form the French lacks takes what the French `other` takes.
 */
function texts(value: unknown, path: string, slots: Slots, depth = 0, given: string[] = []): Text[] {
  if (typeof value === "string") return [{ path, text: value, given }];
  if (typeof value === "function") {
    const count = Math.max(1, slots.get(path) ?? slots.get(path.replace(/\.many$/, ".other")) ?? value.length);
    const alphabet = SENTINELS[depth] ?? [];
    if (count > alphabet.length) throw new Error(`${path}: more parts, or deeper, than the sentinels cover`);
    const args = alphabet.slice(0, count);
    return texts((value as Message)(...args), `${path}()`, slots, depth + 1, [...given, ...args]);
  }
  if (value && typeof value === "object") {
    return Object.entries(value as Catalogue).flatMap(([key, v]) => texts(v, `${path}.${key}`, slots, depth, given));
  }
  throw new Error(`${path}: a catalogue value is a string, a function or an object of them`);
}

/** The key set of a value, nested; a plural form's `many` is optional. */
function shape(value: unknown): unknown {
  if (typeof value === "function") return shape((value as Message)(...SENTINELS[0]));
  if (value && typeof value === "object") {
    const entries = Object.entries(value as Catalogue).filter(([key]) => key !== "many");
    return Object.fromEntries(entries.map(([key, v]) => [key, shape(v)]));
  }
  return typeof value;
}

/**
 * The surfaces whose modules are texts. `grammar` is a renderer, not texts
 * (generalise-lingua-card-wording D2): it is checked as one below.
 */
const TEXT_SURFACES = SURFACES.filter((surface) => surface !== "grammar");

describe("the drafts are whole", () => {
  it("has a module for every surface in every language, and no other", () => {
    for (const language of ["fr", "en", "es"] as const) {
      const found = Object.keys(modules)
        .filter((p) => p.includes(`/${language}/`))
        .map((p) => p.replace(/.*\//, "").replace(/\.ts$/, ""))
        .sort();
      expect(found, language).toEqual([...SURFACES].sort());
    }
  });

  for (const surface of TEXT_SURFACES) {
    describe(surface, () => {
      const fr = catalogue("fr", surface);

      it("has the French key set in English and in Spanish, plural forms and slots included", () => {
        expect(shape(catalogue("en", surface))).toEqual(shape(fr));
        expect(shape(catalogue("es", surface))).toEqual(shape(fr));
      });

      for (const language of ["en", "es"] as const) {
        it(`The drafts are whole: ${language} is full, takes every slot, and is not the French`, () => {
          const slots = slotsOf(fr, surface);
          const draft = texts(catalogue(language, surface), surface, slots);
          const source = new Map(texts(fr, surface, slots).map((t) => [t.path, t]));
          for (const { path, text, given } of draft) {
            // The `many` a translation adds answers the French `other` (the French has no `many`).
            const original = source.get(path) ?? source.get(path.replace(/\.many\(\)$/, ".other()"));
            expect(original, `${language}: ${path} has no French source`).toBeDefined();
            expect(text.trim(), `${language}: ${path} is empty`).not.toBe("");
            for (const arg of given) {
              expect(text, `${language}: ${path} drops the slot ${arg}`).toContain(arg);
            }
            if (text !== original!.text) continue;
            const allowed =
              SAME_EVERYWHERE.has(text) ||
              SAME_AS_FRENCH[language].has(text) ||
              SAME_SHAPE_EVERYWHERE.has(text) ||
              SAME_SHAPE_AS_FRENCH[language].has(text);
            expect(allowed, `${language}: ${path} is still the French « ${text} »`).toBe(true);
          }
        });
      }
    });
  }

  it("the lists of texts that are the same everywhere name only texts that are", () => {
    const equal = { en: new Set<string>(), es: new Set<string>() };
    for (const surface of TEXT_SURFACES) {
      const fr = catalogue("fr", surface);
      const slots = slotsOf(fr, surface);
      const source = new Map(texts(fr, surface, slots).map((t) => [t.path, t.text]));
      for (const language of ["en", "es"] as const) {
        for (const { path, text } of texts(catalogue(language, surface), surface, slots)) {
          if (source.get(path) === text) equal[language].add(text);
        }
      }
    }
    for (const text of [...SAME_EVERYWHERE, ...SAME_SHAPE_EVERYWHERE]) {
      expect(equal.en.has(text) && equal.es.has(text), `« ${text} » is not the same everywhere`).toBe(true);
    }
    for (const language of ["en", "es"] as const) {
      for (const text of [...SAME_AS_FRENCH[language], ...SAME_SHAPE_AS_FRENCH[language]]) {
        expect(equal[language].has(text), `${language}: « ${text} » is not the French`).toBe(true);
        const other = language === "en" ? "es" : "en";
        expect(equal[other].has(text), `« ${text} » is the same in both: list it as same everywhere`).toBe(false);
      }
    }
  });

  it("A message in three languages: the quoted « Réviser » is the French, the drafts differ", () => {
    expect(frReview.start).toBe("Réviser");
    expect(enReview.start).not.toBe("Réviser");
    expect(esReview.start).not.toBe("Réviser");
    expect(enReview.start).toBe("Review");
    expect(esReview.start).toBe("Repasar");
  });

  it("calls a draft with the French message's parts, so a parameter it dropped is seen", () => {
    // `(a) => a` compiles where the French takes two parts; called with its own count it would pass.
    const slots = slotsOf({ pair: (a: string, b: string) => `${a} et ${b}` }, "x");
    const [pair] = texts({ pair: (a: string) => a }, "x", slots);
    expect(pair.given).toEqual(["α", "β"]);
    expect(pair.text).not.toContain("β");
  });

  it("gives an inner message its own sentinels, so a part it dropped is seen behind the outer one", () => {
    const french = { nested: (a: string) => ({ inner: (b: string) => `${a} puis ${b}` }) };
    const slots = slotsOf(french, "x");
    const [inner] = texts({ nested: (a: string) => ({ inner: () => a }) }, "x", slots);
    expect(inner.path).toBe("x.nested().inner()");
    expect(inner.given).toEqual(["α", "ε"]);
    expect(inner.text).toBe("α"); // the outer sentinel alone: one shared sentinel would not see it
    expect(texts(french, "x", slots)[0].text).toBe("α puis ε");
  });

  it("A key missing in Spanish does not compile", () => {
    // What `yarn typecheck` enforces: a draft typed after the French cannot leave a key out.
    const incomplete = Object.fromEntries(Object.entries(frPopup).filter(([key]) => key !== "settings")) as Omit<
      typeof frPopup,
      "settings"
    >;
    // @ts-expect-error — `settings` is missing from the draft.
    const draft: typeof frPopup = incomplete;
    expect(draft).toBeDefined();
  });
});

describe("the grammar modules are renderers (generalise-lingua-card-wording D2)", () => {
  type Tag = { pos: string; features?: Record<string, string> };
  const PAST: Tag = { pos: "VERB", features: { Mood: "Ind", Tense: "Past", VerbForm: "Fin" } };
  const THIRD = { pos: "VERB", features: { Mood: "Ind", Number: "Sing", Person: "3", Tense: "Pres", VerbForm: "Fin" } };
  const grammarOf = (readings: Tag[], others: { lemma: string; readings: Tag[] }[] = []) => ({
    gloss: null,
    senses: [],
    readings,
    others,
    pieces: [],
  });
  const fr = catalogue("fr", "grammar") as unknown as GrammarRenderer;

  for (const language of ["en", "es"] as const) {
    it(`${language} says what the French says, in other words`, () => {
      const draft = catalogue(language, "grammar") as unknown as GrammarRenderer;
      for (const tag of [PAST, THIRD, { pos: "NOUN", features: { Number: "Plur" } }]) {
        const [french, said] = [fr.readingName(tag), draft.readingName(tag)];
        expect(said?.name.trim(), JSON.stringify(tag)).toBeTruthy();
        expect(said?.name).not.toBe(french?.name);
      }
      for (const tag of [{ pos: "NOUN", features: { Gender: "Fem" } }, { pos: "VERB" }]) {
        expect(draft.senseHeading(tag)).toBeTruthy();
        expect(draft.senseHeading(tag)).not.toBe(fr.senseHeading(tag));
      }
      const inputs: [ReturnType<typeof grammarOf>, string, string][] = [
        [grammarOf([PAST]), "go", "went"],
        [grammarOf([PAST]), "put", "put"],
        [
          grammarOf([THIRD], [{ lemma: "leaf", readings: [{ pos: "NOUN", features: { Number: "Plur" } }] }]),
          "leave",
          "leaves",
        ],
      ];
      for (const [grammar, headword, surface] of inputs) {
        const said = draft.grammarLines(grammar, headword, surface, surface).map(lineText);
        const french = fr.grammarLines(grammar, headword, surface, surface).map(lineText);
        expect(said).toHaveLength(french.length);
        said.forEach((line, i) => expect(line).not.toBe(french[i]));
      }
    });
  }
});
