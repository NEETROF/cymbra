import { describe, expect, it, vi } from "vitest";
import {
  BADGE_COUNT_COLOUR,
  BADGE_EINK_COLOUR,
  BADGE_NEUTRAL_COLOUR,
  BADGE_PERCENT_COLOUR,
  countSupported,
  einkColours,
  formatBadgeCount,
  globalBadge,
  listenTabLoads,
  tabBadge,
} from "@/state/badge.ts";

// The toolbar icon (add-lingua-comeback-hints D4–D6): the global badge counts what a review would
// hold today across the studied languages; a tab's own badge is its page's percentage, or nothing,
// so it shows the count. Every state, pure.

const on = { shown: true, eink: false, interfaceLanguage: "fr" as const };

describe("the count as the badge writes it (D5)", () => {
  it("is nothing at 0, the number up to 99, « 99+ » above", () => {
    expect(formatBadgeCount(0)).toBe("");
    expect(formatBadgeCount(1)).toBe("1");
    expect(formatBadgeCount(12)).toBe("12");
    expect(formatBadgeCount(99)).toBe("99");
    expect(formatBadgeCount(100)).toBe("99+");
    expect(formatBadgeCount(140)).toBe("99+");
  });
});

describe("the global badge", () => {
  it("Coming back to the browser: 12 English cards, and a title that says it", () => {
    expect(globalBadge([{ language: "en", count: 12 }], on)).toEqual({
      text: "12",
      colour: BADGE_COUNT_COLOUR,
      title: "Cymbra Lingua — 12 carte(s) à revoir",
    });
  });

  it("Two languages: the sum, and the title names English 8 and Spanish 4, in the interface language", () => {
    const counts = [
      { language: "en", count: 8 },
      { language: "es", count: 4 },
    ];
    expect(globalBadge(counts, on)).toMatchObject({
      text: "12",
      title: "Cymbra Lingua — à revoir : anglais 8, espagnol 4",
    });
    expect(globalBadge(counts, { ...on, interfaceLanguage: "en" }).title).toBe(
      "Cymbra Lingua — to review: English 8, Spanish 4",
    );
    expect(globalBadge(counts, { ...on, interfaceLanguage: "es" }).title).toBe(
      "Cymbra Lingua — por repasar: inglés 8, español 4",
    );
  });

  it("names only the languages holding cards: one of two is a plain count", () => {
    const counts = [
      { language: "en", count: 0 },
      { language: "es", count: 1 },
    ];
    expect(globalBadge(counts, { ...on, interfaceLanguage: "en" })).toMatchObject({
      text: "1",
      title: "Cymbra Lingua — 1 card to review",
    });
    expect(globalBadge([{ language: "es", count: 3 }], { ...on, interfaceLanguage: "es" }).title).toBe(
      "Cymbra Lingua — 3 tarjetas por repasar",
    );
  });

  it("Many cards: « 99+ »", () => {
    expect(globalBadge([{ language: "en", count: 140 }], on).text).toBe("99+");
  });

  it("Nothing waiting: empty, the title the extension's name", () => {
    expect(globalBadge([{ language: "en", count: 0 }], on)).toEqual({
      text: "",
      colour: BADGE_COUNT_COLOUR,
      title: "Cymbra Lingua",
    });
    expect(globalBadge([], on).text).toBe("");
  });

  it("Turned off: no count, the title the extension's name", () => {
    expect(globalBadge([{ language: "en", count: 12 }], { ...on, shown: false })).toEqual({
      text: "",
      colour: BADGE_COUNT_COLOUR,
      title: "Cymbra Lingua",
    });
  });

  it("E-ink colours: the count drawn black", () => {
    expect(globalBadge([{ language: "en", count: 12 }], { ...on, eink: true }).colour).toBe(BADGE_EINK_COLOUR);
  });
});

describe("a tab's own badge", () => {
  const counting = { eink: false, counting: true };

  it("Badge on an analysed page: its percentage, in violet", () => {
    expect(tabBadge({ pct: 94 }, counting)).toEqual({ text: "94%", colour: BADGE_PERCENT_COLOUR });
  });

  it("A page with nothing to count: no dash, the tab shows the global count in its colour", () => {
    expect(tabBadge({ pct: null }, counting)).toEqual({ text: null, colour: BADGE_COUNT_COLOUR });
  });

  it("A tab whose reader is switched off shows the count too", () => {
    expect(tabBadge({ pct: 94, disabled: true }, counting)).toEqual({ text: null, colour: BADGE_COUNT_COLOUR });
  });

  it("E-ink colours: the percentage and the count both black, the « % » telling them apart", () => {
    const eink = { eink: true, counting: true };
    expect(tabBadge({ pct: 94 }, eink)).toEqual({ text: "94%", colour: BADGE_EINK_COLOUR });
    expect(tabBadge({ pct: null }, eink)).toEqual({ text: null, colour: BADGE_EINK_COLOUR });
  });

  it("Safari 17: the badge as before — the percentage, the dash, nothing when off", () => {
    const safari17 = { eink: false, counting: false };
    expect(tabBadge({ pct: 94 }, safari17)).toEqual({ text: "94%", colour: BADGE_PERCENT_COLOUR });
    expect(tabBadge({ pct: null }, safari17)).toEqual({ text: "—", colour: BADGE_NEUTRAL_COLOUR });
    expect(tabBadge({ pct: null, disabled: true }, safari17)).toEqual({ text: "", colour: BADGE_NEUTRAL_COLOUR });
  });
});

describe("the e-ink presets", () => {
  it("are the two e-ink presets, not Cymbra nor the reader's own colours", () => {
    expect(einkColours({ preset: "eink-mono" })).toBe(true);
    expect(einkColours({ preset: "eink-colour" })).toBe(true);
    expect(einkColours({ preset: "cymbra" })).toBe(false);
  });
});

describe("where the count can show (D4)", () => {
  const MAC_17 =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.6 Safari/605.1.15";
  const MAC_18 =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15";
  const IPHONE_17 =
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1";
  const IPHONE_18 =
    "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1";
  const IPAD_17 =
    "Mozilla/5.0 (iPad; CPU OS 17_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1";
  // An iPad on iPadOS 18 asking for its mobile site (Réglages › Safari › « Demander le site pour
  // ordinateur » off): it says iPad and its Safari version.
  const IPAD_18 =
    "Mozilla/5.0 (iPad; CPU OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1";
  // An iPad's Safari asks for desktop sites by default: it presents itself as a Mac.
  const IPAD_26_AS_MAC =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15";
  const CHROME =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0 Safari/537.36";

  it("Safari 17: no count, on the Mac, the iPhone and the iPad", () => {
    for (const ua of [MAC_17, IPHONE_17, IPAD_17]) expect(countSupported(ua, "safari"), ua).toBe(false);
  });

  it("Safari 18 and later: the count, on the Mac, the iPhone and the iPad, as an iPad or as a Mac", () => {
    for (const ua of [MAC_18, IPHONE_18, IPAD_18, IPAD_26_AS_MAC]) expect(countSupported(ua, "safari"), ua).toBe(true);
  });

  it("a Safari that names no version is taken for an old one", () => {
    expect(countSupported("Mozilla/5.0 AppleWebKit/605.1.15", "safari")).toBe(false);
  });

  it("Chrome and Firefox always, whatever their user agent says", () => {
    expect(countSupported(CHROME, "chromium")).toBe(true);
    expect(countSupported(MAC_17, "firefox")).toBe(true);
    expect(countSupported(CHROME)).toBe(true); // the build under test is Chromium's
  });
});

describe("listenTabLoads", () => {
  const listener = (): void => {};

  it("asks for `status` alone where the event takes a filter (Firefox for the desktop)", () => {
    const addListener = vi.fn();
    listenTabLoads({ addListener }, listener, true);
    expect(addListener.mock.calls).toEqual([[listener, { properties: ["status"] }]]);
  });

  it("falls back to the plain listener when the filter is refused (Firefox for Android)", () => {
    const addListener = vi.fn((_l: unknown, filter?: unknown) => {
      if (filter) throw new Error("Incorrect argument types for tabs.onUpdated.");
    });
    expect(() => listenTabLoads({ addListener }, listener, true)).not.toThrow();
    expect(addListener).toHaveBeenLastCalledWith(listener);
    expect(addListener).toHaveBeenCalledTimes(2);
  });

  it("never offers a filter where none is wanted (Chrome, Safari)", () => {
    const addListener = vi.fn();
    listenTabLoads({ addListener }, listener, false);
    expect(addListener.mock.calls).toEqual([[listener]]);
  });
});
