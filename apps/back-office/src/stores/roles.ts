import { reactive, ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import type { AccountRow, RoleGrant } from "@/gen/user_pb";

/** Directory page size (mirrors the server's default window). */
export const PAGE_SIZE = 25;

export interface AccountDirectory {
  accounts: AccountRow[];
  total: number;
}

// Admin-only role administration (the server enforces `require_admin`; the UI just
// hides these actions for non-admins). Scope defaults to `music`. The account
// directory, the per-account audit listing, and the last grant/revoke outcome are
// each an `Async` union so views match on them — a denied action lands in `op` as
// `{ status: "error" }`, never a throw.
export const useRolesStore = defineStore("roles", () => {
  const directory = ref<Async<AccountDirectory>>(idle);
  const grants = ref<Async<RoleGrant[]>>(idle);
  const op = ref<Async<void>>(idle);
  // Current directory query/offset, so a grant/revoke can re-list the same page.
  const params = reactive<{ query: string; offset: number }>({ query: "", offset: 0 });

  /** Load a page of the account directory (empty query lists all). */
  async function list(query = params.query, offset = params.offset) {
    params.query = query;
    params.offset = offset;
    await run(directory, async () => {
      const resp = await api().user.listAccounts({ query, limit: PAGE_SIZE, offset });
      return { accounts: resp.accounts, total: resp.total };
    });
  }

  /** Per-account audit history, most recent first. */
  async function listGrants(userId: string) {
    await run(grants, async () => (await api().user.listRoleGrants({ userId })).grants);
  }

  async function grant(userId: string, role: string, scope = "music") {
    const outcome = await run(op, async () => {
      await api().user.grantRole({ userId, scope, role });
    });
    // Re-list the current page so the row's role badges reflect the change.
    if (outcome.status === "success") await list();
    return outcome;
  }

  async function revoke(userId: string, role: string, scope = "music") {
    const outcome = await run(op, async () => {
      await api().user.revokeRole({ userId, scope, role });
    });
    if (outcome.status === "success") await list();
    return outcome;
  }

  return { directory, grants, op, params, list, listGrants, grant, revoke };
});
