import { ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import type { RoleGrant } from "@/gen/user_pb";

// Admin-only role administration (the server enforces `require_admin`; the UI just
// hides these actions for non-admins). Scope defaults to `music`. Both the audit
// listing and the last grant/revoke outcome are modelled as `Async` so views match
// on them — a denied grant lands in `op` as `{ status: "error" }`, never a throw.
export const useRolesStore = defineStore("roles", () => {
  const grants = ref<Async<RoleGrant[]>>(idle);
  const op = ref<Async<void>>(idle);

  async function listGrants(userId: string) {
    await run(grants, async () => (await api().user.listRoleGrants({ userId })).grants);
  }

  async function grant(userId: string, role: string, scope = "music") {
    const outcome = await run(op, async () => {
      await api().user.grantRole({ userId, scope, role });
    });
    if (outcome.status === "success") await listGrants(userId);
    return outcome;
  }

  async function revoke(userId: string, role: string, scope = "music") {
    const outcome = await run(op, async () => {
      await api().user.revokeRole({ userId, scope, role });
    });
    if (outcome.status === "success") await listGrants(userId);
    return outcome;
  }

  return { grants, op, listGrants, grant, revoke };
});
