import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { useLinguaStore } from "@/stores/lingua";
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

function fakeLinguaClients(opts: { failUsage?: boolean } = {}) {
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
          byLanguage: [{ language: "en", activeAccounts: 4n, wordsLearned: 20n, reviews: 30n }],
        };
      },
      adminGetLinguaUsageSeries: async (req: SeriesReq) => {
        seriesCalls.push(req);
        return { points: [{ day: "2026-09-10", value: 3n }] };
      },
      adminListDataPacks: async () => ({
        packs: [
          {
            studied: "en",
            native: "fr",
            packVersion: "0.0.0-testdata",
            analyzerVersion: "1.0.0",
            builtAt: "2026-09-11",
            sizeBytes: 747n,
            notice: "AGID; wordfreq; kaikki.",
          },
        ],
      }),
    },
  } as unknown as Clients;
  return { clients, usageWindows, seriesCalls };
}

describe("lingua store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("loads the report + three series into success unions, bigint→number", async () => {
    const { clients, seriesCalls } = fakeLinguaClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load({ fromDay: "2026-09-01", toDay: "2026-09-30" });

    expect(store.report.status).toBe("success");
    if (store.report.status === "success") {
      expect(store.report.data.activeAccounts).toBe(4);
      expect(store.report.data.wordsLearned).toBe(20);
      expect(store.report.data.byLanguage).toEqual([
        { language: "en", activeAccounts: 4, wordsLearned: 20, reviews: 30 },
      ]);
    }
    expect(store.series.status).toBe("success");
    if (store.series.status === "success") {
      expect(store.series.data.wordsLearned).toEqual([{ day: "2026-09-10", value: 3 }]);
      expect(store.series.data.reviews.length).toBe(1);
      expect(store.series.data.exposures.length).toBe(1);
    }
    // The three series metrics (words=0, reviews=1, exposures=2) are all requested.
    expect(seriesCalls.map((c) => c.metric).sort()).toEqual([0, 1, 2]);
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

  it("loads the pack registry, converting the byte count", async () => {
    const { clients } = fakeLinguaClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.loadPacks();

    expect(store.packs.status).toBe("success");
    if (store.packs.status === "success") {
      expect(store.packs.data[0].sizeBytes).toBe(747);
      expect(store.packs.data[0].packVersion).toBe("0.0.0-testdata");
    }
  });
});

describe("lingua vocabulary", () => {
  // The plain-language rule: no Lingua UI string says "lemma" (say "words learned",
  // "distinct words", "dictionary form"). The screen renders every label through
  // `t("lingua.*")` / `t("nav.lingua")`, so scanning those strings is scanning the UI.
  it("no Lingua UI string contains 'lemma' in en or fr", () => {
    for (const json of [en, fr]) {
      const linguaStrings = JSON.stringify(json.lingua) + json.nav.lingua;
      expect(/lemm/i.test(linguaStrings)).toBe(false);
    }
  });
});
