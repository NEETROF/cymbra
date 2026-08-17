import { reactive, ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import { type PlanFilter, usePlansStore } from "@/stores/plans";
import { useAuthStore } from "@/stores/auth";
import type { AccountRow, RoleGrant } from "@/gen/user_pb";
import type { CuratorReliability } from "@/gen/score_pb";

/** Directory page size (mirrors the server's default window). */
export const PAGE_SIZE = 25;

export interface AccountDirectory {
  accounts: AccountRow[];
  total: number;
}

/** The directory's search criteria: free text plus the plan/beta filters (change:
 *  add-premium-subscription) — resolved by the plan service into ids, never by the
 *  identity service, which knows nothing of plans. */
export interface DirectoryParams {
  query: string;
  offset: number;
  plan: PlanFilter;
  /** An open campaign key, or "" for any. */
  beta: string;
}

// Admin-only role administration. The server enforces scope-matched authorization
// (`require_admin_in_scope`) and returns only the scopes the caller may administer;
// the UI mirrors that by asking for an explicit `scope` on every grant/revoke
// (change: scope-aware-role-admin). The account directory, the per-account audit
// listing, and the last grant/revoke outcome are each an `Async` union so views
// match on them — a denied action lands in `op` as `{ status: "error" }`, never a throw.
export const useRolesStore = defineStore("roles", () => {
  const directory = ref<Async<AccountDirectory>>(idle);
  const grants = ref<Async<RoleGrant[]>>(idle);
  // Read-only per-user curator reliability (change: add-curation-rewards, task 5.2).
  // MODERATOR/ADMIN gated server-side; it only informs manual promotion — it never
  // triggers a role change here.
  const reliability = ref<Async<CuratorReliability>>(idle);
  const op = ref<Async<void>>(idle);
  // Current directory criteria, so a grant/revoke can re-list the same page.
  const params = reactive<DirectoryParams>({ query: "", offset: 0, plan: "any", beta: "" });

  /** Whether the caller may see plan data at all: a music-scope admin only. A
   *  moderator or another scope's admin gets neither badges nor filters, and the batch
   *  lookup is never issued. */
  const plansVisible = () => useAuthStore().adminScopes.includes("music");

  /** Load a page of the account directory (empty query lists all). A plan/beta filter
   *  is pre-resolved into ids by the plan service; an empty resolved set is an empty
   *  page (total 0) without calling the directory. The page's plan badges are then
   *  fetched in one batch call. */
  async function list(query = params.query, offset = params.offset, plan = params.plan, beta = params.beta) {
    Object.assign(params, { query, offset, plan, beta });
    const plans = usePlansStore();
    const outcome = await run(directory, async () => {
      let ids: string[] = [];
      if (plan !== "any" || beta !== "") {
        ids = await plans.accountIdsByPlan(plan, beta);
        if (ids.length === 0) return { accounts: [], total: 0 };
      }
      const resp = await api().user.listAccounts({ query, limit: PAGE_SIZE, offset, ids });
      return { accounts: resp.accounts, total: resp.total };
    });
    if (outcome.status === "success" && plansVisible()) {
      await plans.plansForAccounts(outcome.data.accounts.map((a) => a.userId));
    }
  }

  /** Per-account audit history, most recent first. */
  async function listGrants(userId: string) {
    await run(grants, async () => (await api().user.listRoleGrants({ userId })).grants);
  }

  /** Load a user's curator reliability (read-only; server enforces moderator/admin). */
  async function loadReliability(userId: string) {
    await run(reliability, () => api().score.getCuratorReliability({ userId }));
  }

  async function grant(userId: string, role: string, scope: string) {
    const outcome = await run(op, async () => {
      await api().user.grantRole({ userId, scope, role });
    });
    // Re-list the current page so the row's role badges reflect the change.
    if (outcome.status === "success") await list();
    return outcome;
  }

  async function revoke(userId: string, role: string, scope: string) {
    const outcome = await run(op, async () => {
      await api().user.revokeRole({ userId, scope, role });
    });
    if (outcome.status === "success") await list();
    return outcome;
  }

  return { directory, grants, reliability, op, params, list, listGrants, loadReliability, grant, revoke };
});
