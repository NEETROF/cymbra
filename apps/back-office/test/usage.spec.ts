import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { useUsageStore } from "@/stores/usage";

interface Query {
  fromDay: string;
  toDay: string;
  platform?: string;
  deviceClass?: string;
  action?: string;
}

function fakeUsageClients(opts: { failSummary?: boolean } = {}) {
  const calls: Query[] = [];
  const clients = {
    usage: {
      getUsersSummary: async (req: { query: Query }) => {
        calls.push(req.query);
        if (opts.failSummary) throw new Error("boom");
        return {
          totalUsers: 5n,
          byPlatform: [{ platform: "ios", users: 3n }],
          byDeviceClass: [{ deviceClass: "phone", users: 4n }],
        };
      },
      getActionBreakdown: async (req: { query: Query }) => {
        calls.push(req.query);
        return { rows: [{ action: "play_start", variant: "", events: 9n }] };
      },
      listActions: async () => ({ actions: ["auth_sign_in", "play_start"] }),
      getUsageSeries: async () => ({
        points: [{ day: "2026-06-15", series: "ios", value: 3n }],
      }),
    },
  } as unknown as Clients;
  return { clients, calls };
}

describe("usage store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("loads the report into a success union, converting bigint counts to numbers", async () => {
    const { clients } = fakeUsageClients();
    setClientsForTest(clients);
    const store = useUsageStore();

    await store.load({ fromDay: "2026-06-01", toDay: "2026-06-30" });

    expect(store.report.status).toBe("success");
    if (store.report.status === "success") {
      expect(store.report.data.summary.totalUsers).toBe(5);
      expect(store.report.data.summary.byPlatform).toEqual([{ platform: "ios", users: 3 }]);
      expect(store.report.data.summary.byDeviceClass).toEqual([{ deviceClass: "phone", users: 4 }]);
      expect(store.report.data.rows).toEqual([{ action: "play_start", variant: "", events: 9 }]);
    }
  });

  it("composes the filters into the query (empty filters omitted)", async () => {
    const { clients, calls } = fakeUsageClients();
    setClientsForTest(clients);
    const store = useUsageStore();

    await store.load({
      fromDay: "2026-06-01",
      toDay: "2026-06-30",
      platform: "android",
      deviceClass: "tablet",
      action: "",
    });

    // Both RPCs receive the same composed query; empty filters become undefined.
    expect(calls.length).toBe(2);
    for (const q of calls) {
      expect(q.fromDay).toBe("2026-06-01");
      expect(q.toDay).toBe("2026-06-30");
      expect(q.platform).toBe("android");
      expect(q.deviceClass).toBe("tablet");
      expect(q.action).toBeUndefined();
    }
  });

  it("a failed read lands in the error union (never throws)", async () => {
    const { clients } = fakeUsageClients({ failSummary: true });
    setClientsForTest(clients);
    const store = useUsageStore();

    await store.load();

    expect(store.report.status).toBe("error");
    if (store.report.status === "error") expect(store.report.error).toBeTruthy();
  });

  it("populates the data-driven action filter list", async () => {
    const { clients } = fakeUsageClients();
    setClientsForTest(clients);
    const store = useUsageStore();

    await store.loadActions();

    expect(store.actions.status).toBe("success");
    if (store.actions.status === "success") {
      expect(store.actions.data).toEqual(["auth_sign_in", "play_start"]);
    }
  });
});
