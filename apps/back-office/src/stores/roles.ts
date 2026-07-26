import { ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import type { RoleGrant } from "@/gen/user_pb";

// Admin-only role administration (the server enforces `require_admin`; the UI just
// hides these actions for non-admins). Scope defaults to `music`.
export const useRolesStore = defineStore("roles", () => {
  const grants = ref<RoleGrant[]>([]);
  const busy = ref(false);
  const error = ref<string | null>(null);

  async function grant(userId: string, role: string, scope = "music") {
    await run(() => api().user.grantRole({ userId, scope, role }));
    await listGrants(userId);
  }

  async function revoke(userId: string, role: string, scope = "music") {
    await run(() => api().user.revokeRole({ userId, scope, role }));
    await listGrants(userId);
  }

  async function listGrants(userId: string) {
    const resp = await api().user.listRoleGrants({ userId });
    grants.value = resp.grants;
  }

  async function run(fn: () => Promise<unknown>) {
    busy.value = true;
    error.value = null;
    try {
      await fn();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      busy.value = false;
    }
  }

  return { grants, busy, error, grant, revoke, listGrants };
});
