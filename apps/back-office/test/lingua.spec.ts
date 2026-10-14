import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { comprehension, foldOnlineUsage, formatUsd, studiedLanguageOptions, useLinguaStore } from "@/stores/lingua";
import { languageLabel } from "@/i18n/language-label";
import en from "@/i18n/locales/en.json";
import fr from "@/i18n/locales/fr.json";

interface Win {
  fromDay: string;
  toDay: string;
}
interface SeriesReq {
  window: Win;
  metric: number;
  language: string;
}

function fakeLinguaClients(opts: { failUsage?: boolean; failOnline?: boolean } = {}) {
  const usageWindows: Win[] = [];
  const seriesCalls: SeriesReq[] = [];
  const clients = {
    lingua: {
      adminGetLinguaUsage: async (req: { window: Win }) => {
        usageWindows.push(req.window);
        if (opts.failUsage) throw new Error("boom");
        return {
          activeAccounts: 4n,
          wordsLearned: 20n,
          reviews: 30n,
          wordsRead: 1000n,
          newWordsSeen: 50n,
          byLanguage: [
            {
              language: "en",
              activeAccounts: 4n,
              wordsLearned: 20n,
              reviews: 30n,
              wordsRead: 1000n,
              newWordsSeen: 50n,
            },
          ],
        };
      },
      adminGetLinguaUsageSeries: async (req: SeriesReq) => {
        seriesCalls.push(req);
        return { points: [{ day: "2026-09-10", value: 3n }] };
      },
      adminGetOnlineTranslationUsage: async () => {
        if (opts.failOnline) throw new Error("boom");
        const row = (day: string, provider: string, answered: bigint, cost: bigint) => ({
          day,
          provider,
          pair: "es-fr",
          answered,
          failed: 1n,
          limited: 0n,
          inputTokens: 270n * answered,
          outputTokens: 40n * answered,
          costMicroUsd: cost,
        });
        return {
          rows: [
            row("2026-09-10", "gemini", 10n, 1400n),
            row("2026-09-11", "gemini", 5n, 700n),
            row("2026-09-11", "anthropic", 2n, 200n),
          ],
          answered: 17n,
          failed: 3n,
          limited: 0n,
          costMicroUsd: 2300n,
        };
      },
      // The Installations section (change: add-lingua-usage-counts) — see lingua-installations.spec.ts.
      adminGetUsageFunnel: async () => ({ weeks: [] }),
      adminGetLeaveReasons: async () => ({ reasons: [], capped: false }),
    },
  } as unknown as Clients;
  return { clients, usageWindows, seriesCalls };
}

describe("lingua store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("loads the report + the four series into success unions, bigint→number", async () => {
    const { clients, seriesCalls } = fakeLinguaClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load({ fromDay: "2026-09-01", toDay: "2026-09-30" });

    expect(store.report.status).toBe("success");
    if (store.report.status === "success") {
      expect(store.report.data.activeAccounts).toBe(4);
      expect(store.report.data.wordsLearned).toBe(20);
      expect(store.report.data.wordsRead).toBe(1000);
      expect(store.report.data.newWordsSeen).toBe(50);
      expect(store.report.data.byLanguage).toEqual([
        { language: "en", activeAccounts: 4, wordsLearned: 20, reviews: 30, wordsRead: 1000, newWordsSeen: 50 },
      ]);
    }
    expect(store.series.status).toBe("success");
    if (store.series.status === "success") {
      expect(store.series.data.wordsLearned).toEqual([{ day: "2026-09-10", value: 3 }]);
      expect(store.series.data.reviews.length).toBe(1);
      expect(store.series.data.wordsRead.length).toBe(1);
      expect(store.series.data.newWordsSeen.length).toBe(1);
    }
    // Every series metric (words=0, reviews=1, words read=2, new words seen=3) is requested.
    expect(seriesCalls.map((c) => c.metric).sort()).toEqual([0, 1, 2, 3]);
  });

  it("composes the window + studied-language filter into every request", async () => {
    const { clients, usageWindows, seriesCalls } = fakeLinguaClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load({ fromDay: "2026-09-01", toDay: "2026-09-30", language: "en" });

    expect(usageWindows[0]).toEqual({ fromDay: "2026-09-01", toDay: "2026-09-30" });
    for (const c of seriesCalls) {
      expect(c.window).toEqual({ fromDay: "2026-09-01", toDay: "2026-09-30" });
      expect(c.language).toBe("en");
    }
  });

  it("a failed read lands in the error union (never throws)", async () => {
    const { clients } = fakeLinguaClients({ failUsage: true });
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load();

    expect(store.report.status).toBe("error");
    if (store.report.status === "error") expect(store.report.error).toBeTruthy();
  });
});

// The studied-language filter used to list the languages of the registered data packs; the
// registry described the test fixture and was removed (change: remove-lingua-pack-registry).
// It now lists the languages of the usage report's per-language breakdown.
describe("studiedLanguageOptions", () => {
  const usage = (language: string) => ({
    language,
    activeAccounts: 1,
    wordsLearned: 0,
    reviews: 0,
    wordsRead: 0,
    newWordsSeen: 0,
  });

  it("lists the languages the usage report breaks down, once each, sorted", () => {
    expect(studiedLanguageOptions([usage("es"), usage("en"), usage("es")], "")).toEqual(["en", "es"]);
  });

  it("keeps the selected language when the new window holds no activity in it", () => {
    // Without this a window change would leave the select on a value it no longer offers.
    expect(studiedLanguageOptions([usage("es")], "en")).toEqual(["en", "es"]);
  });

  it("offers nothing but the selected language while the report is still empty", () => {
    expect(studiedLanguageOptions([], "en")).toEqual(["en"]);
    expect(studiedLanguageOptions([], "")).toEqual([]);
  });

  it("never lists every-language itself — that is the select's own first option", () => {
    expect(studiedLanguageOptions([usage("en")], "")).not.toContain("");
  });
});

describe("comprehension", () => {
  it("is the share of words read that were not new", () => {
    expect(comprehension(1000, 50)).toBeCloseTo(0.95);
    expect(comprehension(10, 0)).toBe(1);
  });

  it("is unavailable when nothing was read, never 0 % or 100 %", () => {
    expect(comprehension(0, 0)).toBeNull();
  });

  it("stays within 0–1 whatever the counts", () => {
    expect(comprehension(10, 12)).toBe(0);
  });
});

describe("lingua vocabulary", () => {
  // The plain-language rule: no Lingua UI string says "lemma" (say "words learned",
  // "distinct words", "dictionary form"). The screen renders every label through
  // `t("lingua.*")` / `t("nav.linguaOverview")` / `t("nav.sections.lingua")`, so scanning those strings is scanning the UI.
  it("no Lingua UI string contains 'lemma' in en or fr", () => {
    for (const json of [en, fr]) {
      const linguaStrings = JSON.stringify(json.lingua) + json.nav.linguaOverview + json.nav.sections.lingua;
      expect(/lemm/i.test(linguaStrings)).toBe(false);
    }
  });

  it("labels the reading figures as words read, never exposures", () => {
    for (const json of [en, fr]) expect(/expos/i.test(JSON.stringify(json.lingua))).toBe(false);
  });
});

describe("languageLabel (change: add-admin-lingua-language-labels)", () => {
  /** vue-i18n's `t` and `te` over one locale's messages. */
  const lookup = (messages: Record<string, unknown>) => {
    const get = (key: string): unknown =>
      key
        .split(".")
        .reduce<unknown>(
          (v, k) => (v && typeof v === "object" ? (v as Record<string, unknown>)[k] : undefined),
          messages,
        );
    return { t: (key: string) => String(get(key)), te: (key: string) => typeof get(key) === "string" };
  };

  it("names the studied languages in each of the console's languages", () => {
    const f = lookup(fr);
    const e = lookup(en);
    expect(languageLabel("en", f.t, f.te)).toBe("Anglais");
    expect(languageLabel("es", f.t, f.te)).toBe("Espagnol");
    // French, studied from change 52 (change: add-lingua-french-listings, D13).
    expect(languageLabel("fr", f.t, f.te)).toBe("Français");
    expect(languageLabel("en", e.t, e.te)).toBe("English");
    expect(languageLabel("es", e.t, e.te)).toBe("Spanish");
    expect(languageLabel("fr", e.t, e.te)).toBe("French");
  });

  it("keeps the code of a language the console has no name for", () => {
    const f = lookup(fr);
    expect(languageLabel("de", f.t, f.te)).toBe("de");
  });

  it("names the same languages in both locales", () => {
    expect(Object.keys(fr.lingua.languages).sort()).toEqual(Object.keys(en.lingua.languages).sort());
  });
});

describe("online translation figures (change: add-lingua-online-translation)", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("folds the days into one line per provider and pair, busiest first", async () => {
    const { clients } = fakeLinguaClients();
    setClientsForTest(clients);
    const store = useLinguaStore();
    await store.load();
    expect(store.online.status).toBe("success");
    if (store.online.status !== "success") return;
    expect(store.online.data.answered).toBe(17);
    expect(store.online.data.costMicroUsd).toBe(2300);
    expect(store.online.data.lines.map((l) => [l.provider, l.answered, l.costMicroUsd])).toEqual([
      ["gemini", 15, 2100],
      ["anthropic", 2, 200],
    ]);
  });

  it("a failed online read leaves the rest of the console standing", async () => {
    const { clients } = fakeLinguaClients({ failOnline: true });
    setClientsForTest(clients);
    const store = useLinguaStore();
    await store.load();
    expect(store.online.status).toBe("error");
    expect(store.report.status).toBe("success");
  });

  it("formats an estimated cost in dollars, with more digits for a tiny one", () => {
    expect(formatUsd(2_300_000, "en")).toBe("$2.30");
    expect(formatUsd(2300, "en")).toBe("$0.0023");
    expect(formatUsd(0, "en")).toBe("$0.00");
    expect(foldOnlineUsage([])).toEqual([]);
  });
});
