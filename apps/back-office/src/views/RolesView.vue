<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import { match } from "ts-pattern";
import { PAGE_SIZE, useRolesStore } from "@/stores/roles";
import { type PlanFilter, usePlansStore } from "@/stores/plans";
import { useSessionsStore } from "@/stores/sessions";
import { useAuthStore } from "@/stores/auth";
import { useToastsStore } from "@/stores/toasts";
import type { Scope } from "@/lib/jwt";
import type { AccountRow, RoleGrant } from "@/gen/user_pb";
import AppTag from "@/components/AppTag.vue";
import CuratorReliabilityDrawer from "@/components/CuratorReliabilityDrawer.vue";

// Admin-only (route- + server-guarded). A paginated directory of accounts with
// their roles grouped by scope; an admin picks a scope they may administer, filters
// by handle/email and grants/revokes per row within that scope. A `music`-only admin
// only ever sees `music`; a `global/admin` can switch across global/music/live
// (change: scope-aware-role-admin). All API work lives in the store; this view only
// matches on the Async unions.
const store = useRolesStore();
const plans = usePlansStore();
const sessions = useSessionsStore();
const auth = useAuthStore();
const toasts = useToastsStore();
const { t } = useI18n();
const filter = ref("");
// Plan / beta criteria (change: add-premium-subscription). Only a music-scope admin
// sees the plan columns and filters — the store also skips the badge batch otherwise.
const planFilter = ref<PlanFilter>("any");
const betaFilter = ref("");
const showPlans = computed(() => auth.adminScopes.includes("music"));
const openCampaigns = computed(() => plans.openCampaigns);
const badges = computed(() =>
  match(plans.badges)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => ({})),
);
const badgeFor = (userId: string) => badges.value[userId];
const selected = ref<string | null>(null);
// Which expandable panel is open for the selected row: the audit history or the
// read-only curator-reliability indicator (they share the `selected` row).
const panel = ref<"history" | "reliability" | null>(null);

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
// Action results (a role change, an admin session-revoke) and a failed reliability
// read surface as toasts; only the list LOAD error stays inline on the page.
watch(
  () => store.op,
  (op) => {
    if (op.status === "error") toasts.error(op.error);
  },
);
watch(
  () => sessions.op,
  (op) => {
    if (op.status === "error") toasts.error(op.error);
  },
);
watch(
  () => store.reliability,
  (r) => {
    if (r.status === "error") toasts.error(r.error);
  },
);
const error = computed(() => vm.value.error);

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

// Read-only curator reliability for the selected user (StatBar-style fold: the data
// on success, otherwise null). It is purely informational — it never triggers a
// role change (no automation). Rendered only for moderators/admins.
const reliability = computed(() =>
  match(store.reliability)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => null),
);
const reliabilityLoading = computed(() => store.reliability.status === "loading");
// Whether the reliability drawer is open (a selected row + moderator/admin).
const reliabilityOpen = computed(() => !!selected.value && panel.value === "reliability" && auth.isModerator);
// The handle of the selected account, for the drawer header.
const selectedHandle = computed(() => vm.value.accounts.find((a) => a.userId === selected.value)?.handle ?? "");

function search() {
  store.list(filter.value.trim(), 0, planFilter.value, betaFilter.value);
}
function prev() {
  if (canPrev.value) store.list(store.params.query, Math.max(0, offset.value - PAGE_SIZE));
}
function next() {
  if (canNext.value) store.list(store.params.query, offset.value + PAGE_SIZE);
}
const colCount = computed(() => (showPlans.value ? 6 : 4));
function toggle(account: AccountRow, role: string) {
  if (rolesInScope(account).includes(role)) store.revoke(account.userId, role, selectedScope.value);
  else store.grant(account.userId, role, selectedScope.value);
}
function history(userId: string) {
  selected.value = userId;
  panel.value = "history";
  store.listGrants(userId);
}
// Open the read-only reliability panel for a row. Moderator/admin-gated in the UI
// (button hidden otherwise) and enforced server-side; note /roles is currently an
// admin-only route (meta.admin), so only admins reach this today — that already
// satisfies "moderator/admin can view". Displaying only — never mutates a role.
function reliabilityFor(userId: string) {
  selected.value = userId;
  panel.value = "reliability";
  store.loadReliability(userId);
}
/** Close the reliability drawer (leaves the history panel state untouched). */
function closeReliability() {
  if (panel.value === "reliability") panel.value = null;
}
/** Admin: cut off every session of a compromised account (server-gated by require_admin).
 * Confirm first — it's destructive — and let the outcome surface via `error`. */
const pendingSessionRevoke = ref<string | null>(null);
function revokeSessions(userId: string) {
  pendingSessionRevoke.value = userId;
}
async function confirmSessionRevoke() {
  const userId = pendingSessionRevoke.value;
  pendingSessionRevoke.value = null;
  if (!userId) return;
  await sessions.revokeAccount(userId);
}
function when(atSeconds: bigint | number): string {
  const ms = Number(atSeconds) * 1000;
  return ms > 0 ? new Date(ms).toISOString().replace("T", " ").slice(0, 19) : "—";
}

onMounted(() => {
  store.list("", 0, "any", "");
  if (showPlans.value) plans.loadCampaigns();
});
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
    <template v-if="showPlans">
      <label class="scope-picker">
        {{ $t("roles.planFilter") }}
        <select v-model="planFilter" :aria-label="$t('roles.planFilter')" @change="search">
          <option value="any">{{ $t("roles.planAny") }}</option>
          <option value="premium">{{ $t("plans.premium") }}</option>
          <option value="trial">{{ $t("plans.premiumTrial") }}</option>
        </select>
      </label>
      <label class="scope-picker">
        {{ $t("roles.betaFilter") }}
        <select v-model="betaFilter" :aria-label="$t('roles.betaFilter')" @change="search">
          <option value="">{{ $t("roles.betaAny") }}</option>
          <option v-for="c in openCampaigns" :key="c.key" :value="c.key">
            {{ c.name }} ({{ $t(`plans.kind.${c.kind}`) }})
          </option>
        </select>
      </label>
    </template>
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
          <th v-if="showPlans">{{ $t("roles.colPlan") }}</th>
          <th v-if="showPlans">{{ $t("roles.colBeta") }}</th>
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
          <td v-if="showPlans" class="plan-cell">
            <template v-if="badgeFor(a.userId)?.plan === 'premium'">
              <AppTag variant="accepted" :title="badgeFor(a.userId)?.endsAt">
                {{ $t("plans.premium")
                }}<template v-if="badgeFor(a.userId)?.trial"> · {{ $t("plans.trialTag") }}</template>
              </AppTag>
            </template>
            <span v-else class="muted">{{ $t("plans.free") }}</span>
          </td>
          <td v-if="showPlans">
            <div class="rolechips">
              <AppTag v-for="k in badgeFor(a.userId)?.betaKeys ?? []" :key="k" variant="neutral" mono>{{ k }}</AppTag>
              <span v-if="(badgeFor(a.userId)?.betaKeys ?? []).length === 0" class="muted">—</span>
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
            <button v-if="auth.isModerator" type="button" :disabled="acting" @click="reliabilityFor(a.userId)">
              {{ $t("roles.reliability") }}
            </button>
            <button type="button" :disabled="acting" @click="revokeSessions(a.userId)">
              {{ $t("sessions.revokeAccount") }}
            </button>
          </td>
        </tr>
        <tr v-if="vm.accounts.length === 0">
          <td :colspan="colCount" class="empty">{{ vm.loading ? $t("common.loading") : $t("roles.noAccounts") }}</td>
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

  <CuratorReliabilityDrawer
    :open="reliabilityOpen"
    :handle="selectedHandle"
    :reliability="reliability"
    :loading="reliabilityLoading"
    @close="closeReliability"
  />

  <section v-if="selected && panel === 'history' && grants.length" class="history">
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
  <ConfirmDialog
    :message="pendingSessionRevoke ? t('sessions.revokeAccountConfirm') : null"
    @confirm="confirmSessionRevoke"
    @cancel="pendingSessionRevoke = null"
  />
</template>

<style scoped>
.filter {
  display: flex;
  gap: 0.5rem;
  margin: 1.25rem 0;
  flex-wrap: wrap;
  align-items: center;
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
