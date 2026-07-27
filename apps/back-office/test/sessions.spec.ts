import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { useSessionsStore } from "@/stores/sessions";

interface Sess {
  id: string;
  audience: string;
}
interface Calls {
  revoked: string[];
  revokedAll: number;
  revokedAccount: string[];
}

// A tiny fake of just the auth session RPCs (cast to the generated client type). The
// list mutates in place so a revoke is visible on re-list, mirroring the server.
function fake(opts: { sessions?: Sess[]; failList?: boolean } = {}): { clients: Clients; calls: Calls } {
  const sessions = [...(opts.sessions ?? [])];
  const calls: Calls = { revoked: [], revokedAll: 0, revokedAccount: [] };
  const clients = {
    auth: {
      listSessions: async () => {
        if (opts.failList) throw new ConnectError("denied", Code.PermissionDenied);
        return { sessions };
      },
      revokeSession: async (req: { sessionId: string }) => {
        calls.revoked.push(req.sessionId);
        const i = sessions.findIndex((s) => s.id === req.sessionId);
        if (i >= 0) sessions.splice(i, 1);
        return {};
      },
      revokeAllSessions: async () => {
        calls.revokedAll += 1;
        return {};
      },
      revokeAccountSessions: async (req: { userId: string }) => {
        calls.revokedAccount.push(req.userId);
        return {};
      },
    },
  } as unknown as Clients;
  return { clients, calls };
}

describe("sessions store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("loads the caller's active sessions into a success union", async () => {
    const { clients } = fake({
      sessions: [
        { id: "a", audience: "music" },
        { id: "b", audience: "live" },
      ],
    });
    setClientsForTest(clients);
    const store = useSessionsStore();

    await store.load();

    expect(store.list.status).toBe("success");
    if (store.list.status === "success") expect(store.list.data.map((s) => s.id)).toEqual(["a", "b"]);
  });

  it("revokes one session then re-lists without it", async () => {
    const { clients, calls } = fake({
      sessions: [
        { id: "a", audience: "music" },
        { id: "b", audience: "live" },
      ],
    });
    setClientsForTest(clients);
    const store = useSessionsStore();
    await store.load();

    const outcome = await store.revoke("a");

    expect(outcome.status).toBe("success");
    expect(calls.revoked).toEqual(["a"]);
    if (store.list.status === "success") expect(store.list.data.map((s) => s.id)).toEqual(["b"]);
  });

  it("sign-out-everywhere revokes all sessions", async () => {
    const { clients, calls } = fake({ sessions: [{ id: "a", audience: "music" }] });
    setClientsForTest(clients);
    const store = useSessionsStore();

    const outcome = await store.revokeAll();

    expect(outcome.status).toBe("success");
    expect(calls.revokedAll).toBe(1);
  });

  it("admin revoke targets the given account", async () => {
    const { clients, calls } = fake();
    setClientsForTest(clients);
    const store = useSessionsStore();

    await store.revokeAccount("u-42");

    expect(calls.revokedAccount).toEqual(["u-42"]);
  });

  it("a failed list lands in the union as an error, never a throw", async () => {
    const { clients } = fake({ failList: true });
    setClientsForTest(clients);
    const store = useSessionsStore();

    await store.load();

    expect(store.list.status).toBe("error");
  });
});
