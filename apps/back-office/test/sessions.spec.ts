import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { useSessionsStore } from "@/stores/sessions";

// A tiny fake of just the admin revoke RPC (cast to the generated client type).
function fake(opts: { fail?: boolean } = {}): { clients: Clients; calls: string[] } {
  const calls: string[] = [];
  const clients = {
    auth: {
      revokeAccountSessions: async (req: { userId: string }) => {
        if (opts.fail) throw new ConnectError("denied", Code.PermissionDenied);
        calls.push(req.userId);
        return {};
      },
    },
  } as unknown as Clients;
  return { clients, calls };
}

describe("sessions store (admin revoke)", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("revokes a target account's sessions and records success in the op union", async () => {
    const { clients, calls } = fake();
    setClientsForTest(clients);
    const store = useSessionsStore();

    const outcome = await store.revokeAccount("u-42");

    expect(outcome.status).toBe("success");
    expect(calls).toEqual(["u-42"]);
  });

  it("a denied revoke lands in the op union as an error, never a throw", async () => {
    const { clients } = fake({ fail: true });
    setClientsForTest(clients);
    const store = useSessionsStore();

    await store.revokeAccount("u-42");

    expect(store.op.status).toBe("error");
  });
});
