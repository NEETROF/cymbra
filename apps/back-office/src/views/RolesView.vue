<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { PAGE_SIZE, useRolesStore } from "@/stores/roles";
import { useSessionsStore } from "@/stores/sessions";
import { useAuthStore } from "@/stores/auth";
import type { Scope } from "@/lib/jwt";
import type { AccountRow, RoleGrant } from "@/gen/user_pb";
import AppTag from "@/components/AppTag.vue";

// Admin-only (route- + server-guarded). A paginated directory of accounts with
// their roles grouped by scope; an admin picks a scope they may administer, filters
// by handle/email and grants/revokes per row within that scope. A `music`-only admin
// only ever sees `music`; a `global/admin` can switch across global/music/live
// (change: scope-aware-role-admin). All API work lives in the store; this view only
// matches on the Async unions.
const store = useRolesStore();
const sessions = useSessionsStore();
const auth = useAuthStore();
const { t } = useI18n();
const filter = ref("");
const selected = ref<string | null>(null);

const MANAGED_ROLES = ["moderator", "admin"] as const;

// The scopes this admin may administer, and the one currently selected. Default to
// the first authorized scope; if the set changes (e.g. after refresh) and the
// current pick is no longer valid, fall back to the first.
const authorizedScopes = computed<Scope[]>(() => auth.adminScopes);
const selectedScope = ref<Scope>(authorizedScopes.value[0] ?? ("music" as Scope));
watch(
  authorizedScopes,
  (scopes) => {
    if (!scopes.includes(selectedScope.value) && scopes.length > 0) selectedScope.value = scopes[0];
  },
  { immediate: true },
);

/** The roles an account holds in the currently selected scope. */
function rolesInScope(account: AccountRow): string[] {
  return account.rolesByScope.find((sr) => sr.scope === selectedScope.value)?.roles ?? [];
}

const vm = computed(() =>
  match(store.directory)
    .with({ status: "success" }, ({ data }) => ({
      loading: false,
      error: null as string | null,
      accounts: data.accounts,
      total: data.total,
    }))
    .with({ status: "error" }, ({ error }) => ({
      loading: false,
      error,
      accounts: [] as AccountRow[],
      total: 0,
    }))
    .otherwise(() => ({ loading: true, error: null as string | null, accounts: [] as AccountRow[], total: 0 })),
);
// Both the role op and the session-revoke op count as "acting" (disable while either
// runs), and either op's error is surfaced — a failed admin session-revoke must not be
// silent.
const acting = computed(() => store.op.status === "loading" || sessions.op.status === "loading");
const opError = computed(() =>
  match(store.op)
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() => null),
);
const sessionOpError = computed(() =>
  match(sessions.op)
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() => null),
);
const error = computed(() => vm.value.error ?? opError.value ?? sessionOpError.value);

const offset = computed(() => store.params.offset);
const from = computed(() => (vm.value.total === 0 ? 0 : offset.value + 1));
const to = computed(() => Math.min(offset.value + PAGE_SIZE, vm.value.total));
const canPrev = computed(() => offset.value > 0);
const canNext = computed(() => offset.value + PAGE_SIZE < vm.value.total);

const grants = computed(() =>
  match(store.grants)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => [] as RoleGrant[]),
);

function search() {
  store.list(filter.value.trim(), 0);
}
function prev() {
  if (canPrev.value) store.list(store.params.query, Math.max(0, offset.value - PAGE_SIZE));
}
function next() {
  if (canNext.value) store.list(store.params.query, offset.value + PAGE_SIZE);
}
function toggle(account: AccountRow, role: string) {
  if (rolesInScope(account).includes(role)) store.revoke(account.userId, role, selectedScope.value);
  else store.grant(account.userId, role, selectedScope.value);
}
function history(userId: string) {
  selected.value = userId;
  store.listGrants(userId);
}
/** Admin: cut off every session of a compromised account (server-gated by require_admin).
 * Confirm first — it's destructive — and let the outcome surface via `error`. */
async function revokeSessions(userId: string) {
  if (globalThis.confirm && !globalThis.confirm(t("sessions.revokeAccountConfirm"))) return;
  await sessions.revokeAccount(userId);
}
function when(atSeconds: bigint | number): string {
  const ms = Number(atSeconds) * 1000;
  return ms > 0 ? new Date(ms).toISOString().replace("T", " ").slice(0, 19) : "—";
}

onMounted(() => store.list("", 0));
</script>

<template>
  <h1 class="page-title">{{ $t("roles.title") }}</h1>
  <p class="muted">{{ $t("roles.intro") }}</p>

  <div class="filter">
    <label v-if="authorizedScopes.length > 1" class="scope-picker">
      {{ $t("roles.scope") }}
      <select v-model="selectedScope" :aria-label="$t('roles.scope')">
        <option v-for="s in authorizedScopes" :key="s" :value="s">{{ $t(`scope.${s}`) }}</option>
      </select>
    </label>
    <input
      v-model="filter"
      type="search"
      :placeholder="$t('roles.searchPlaceholder')"
      :aria-label="$t('roles.searchPlaceholder')"
      @keyup.enter="search"
    />
    <button type="button" @click="search">{{ $t("roles.search") }}</button>
  </div>

  <p v-if="error" class="error" role="alert">{{ error }}</p>

  <div class="table-card">
    <table>
      <thead>
        <tr>
          <th>{{ $t("roles.colHandle") }}</th>
          <th>{{ $t("roles.colName") }}</th>
          <th>{{ $t("roles.colRoles") }}</th>
          <th>{{ $t("roles.colActions") }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="a in vm.accounts" :key="a.userId">
          <td class="handle">{{ a.handle || $t("roles.noHandle") }}</td>
          <td>{{ a.displayName || "—" }}</td>
          <td>
            <div class="rolechips">
              <AppTag v-for="r in rolesInScope(a)" :key="r" variant="accent" cap>{{ $t(`role.${r}`) }}</AppTag>
              <span v-if="rolesInScope(a).length === 0" class="muted">—</span>
            </div>
          </td>
          <td class="actions">
            <button
              v-for="r in MANAGED_ROLES"
              :key="r"
              type="button"
              class="toggle"
              :class="{ held: rolesInScope(a).includes(r) }"
              :disabled="acting"
              :aria-label="
                rolesInScope(a).includes(r)
                  ? $t('roles.revokeRole', { role: $t(`role.${r}`) })
                  : $t('roles.grantRole', { role: $t(`role.${r}`) })
              "
              @click="toggle(a, r)"
            >
              {{ rolesInScope(a).includes(r) ? "−" : "+" }} {{ $t(`role.${r}`) }}
            </button>
            <button type="button" :disabled="acting" @click="history(a.userId)">{{ $t("roles.history") }}</button>
            <button type="button" :disabled="acting" @click="revokeSessions(a.userId)">
              {{ $t("sessions.revokeAccount") }}
            </button>
          </td>
        </tr>
        <tr v-if="vm.accounts.length === 0">
          <td colspan="4" class="empty">{{ vm.loading ? $t("common.loading") : $t("roles.noAccounts") }}</td>
        </tr>
      </tbody>
    </table>
  </div>

  <div class="pager">
    <span class="muted">{{ $t("roles.showing", { from, to, total: vm.total }) }}</span>
    <div class="pager-btns">
      <button type="button" :disabled="!canPrev" @click="prev">{{ $t("roles.prev") }}</button>
      <button type="button" :disabled="!canNext" @click="next">{{ $t("roles.next") }}</button>
    </div>
  </div>

  <section v-if="selected && grants.length" class="history">
    <h2>{{ $t("roles.history") }}</h2>
    <div class="table-card">
      <table>
        <thead>
          <tr>
            <th>{{ $t("roles.when") }}</th>
            <th>{{ $t("roles.action") }}</th>
            <th>{{ $t("roles.scope") }}</th>
            <th>{{ $t("roles.role") }}</th>
            <th>{{ $t("roles.byAdmin") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(g, i) in grants" :key="i">
            <td>{{ when(g.at) }}</td>
            <td>{{ g.action }}</td>
            <td>{{ g.scope }}</td>
            <td>{{ g.role }}</td>
            <td :class="{ mono: !g.actingAdminHandle }">{{ g.actingAdminHandle || g.actingAdmin }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<style scoped>
.filter {
  display: flex;
  gap: 0.5rem;
  margin: 1.25rem 0;
}
.filter input[type="search"] {
  min-width: 16rem;
}
.scope-picker {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.9rem;
  color: var(--muted);
}
.scope-picker select {
  text-transform: capitalize;
}
.handle {
  font-weight: 600;
}
.actions {
  display: flex;
  gap: 0.4rem;
  flex-wrap: wrap;
}
.rolechips {
  display: flex;
  gap: 0.3rem;
  flex-wrap: wrap;
  align-items: center;
}
.toggle {
  font-size: 0.8rem;
  padding: 0.3rem 0.6rem;
  text-transform: capitalize;
}
.toggle.held {
  border-color: color-mix(in srgb, var(--accent) 45%, transparent);
  color: var(--accent);
}
.pager {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 1rem;
}
.pager-btns {
  display: flex;
  gap: 0.4rem;
}
.history {
  margin-top: 2rem;
}
.history h2 {
  font-size: 1.05rem;
}
.empty {
  color: var(--muted);
  text-align: center;
  padding: 2rem;
}
.mono {
  font-family: var(--mono);
  font-size: 0.85rem;
}
</style>
