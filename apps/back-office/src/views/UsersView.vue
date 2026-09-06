<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import { PAGE_SIZE, useRolesStore } from "@/stores/roles";
import { type PlanFilter, usePlansStore } from "@/stores/plans";
import { useAuthStore } from "@/stores/auth";
import type { Scope } from "@/lib/jwt";
import type { AccountRow } from "@/gen/user_pb";
import AppTag from "@/components/AppTag.vue";

// Admin-only (route- + server-guarded). A paginated directory of accounts with their
// roles, plan and betas. It is a surface for FINDING an account, not for acting on one
// (change: restructure-back-office-users-console): every per-account gesture lives on
// `/users/{user_id}`, one click away. A `music`-only admin only ever sees `music`; a
// `global/admin` can switch across global/music/live (change: scope-aware-role-admin).
// All API work lives in the store; this view only matches on the Async unions.
const store = useRolesStore();
const plans = usePlansStore();
const auth = useAuthStore();
const router = useRouter();
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
const error = computed(() => vm.value.error);

const offset = computed(() => store.params.offset);
const from = computed(() => (vm.value.total === 0 ? 0 : offset.value + 1));
const to = computed(() => Math.min(offset.value + PAGE_SIZE, vm.value.total));
const canPrev = computed(() => offset.value > 0);
const canNext = computed(() => offset.value + PAGE_SIZE < vm.value.total);
const colCount = computed(() => (showPlans.value ? 5 : 3));

function search() {
  store.list(filter.value.trim(), 0, planFilter.value, betaFilter.value);
}
function prev() {
  if (canPrev.value) store.list(store.params.query, Math.max(0, offset.value - PAGE_SIZE));
}
function next() {
  if (canNext.value) store.list(store.params.query, offset.value + PAGE_SIZE);
}

/** Opening a row is a convenience on top of the handle link, which is the real
 *  navigation: an admin directory is used at the keyboard, and a `<tr>` with a click
 *  handler is unreachable there. Clicks that land on a control of their own (the link
 *  itself, a future button) are left to that control. */
function openRow(event: MouseEvent, userId: string) {
  if ((event.target as HTMLElement | null)?.closest("a, button")) return;
  void router.push({ name: "user-detail", params: { userId } });
}

onMounted(() => {
  store.list("", 0, "any", "");
  if (showPlans.value) plans.loadCampaigns();
});
</script>

<template>
  <h1 class="page-title">{{ $t("users.title") }}</h1>
  <p class="muted">{{ $t("users.intro") }}</p>

  <div class="filter">
    <label v-if="authorizedScopes.length > 1" class="scope-picker">
      {{ $t("users.scope") }}
      <select v-model="selectedScope" :aria-label="$t('users.scope')">
        <option v-for="s in authorizedScopes" :key="s" :value="s">{{ $t(`scope.${s}`) }}</option>
      </select>
    </label>
    <input
      v-model="filter"
      type="search"
      :placeholder="$t('users.searchPlaceholder')"
      :aria-label="$t('users.searchPlaceholder')"
      @keyup.enter="search"
    />
    <template v-if="showPlans">
      <label class="scope-picker">
        {{ $t("users.planFilter") }}
        <select v-model="planFilter" :aria-label="$t('users.planFilter')" @change="search">
          <option value="any">{{ $t("users.planAny") }}</option>
          <option value="premium">{{ $t("plans.premium") }}</option>
          <option value="trial">{{ $t("plans.premiumTrial") }}</option>
        </select>
      </label>
      <label class="scope-picker">
        {{ $t("users.betaFilter") }}
        <select v-model="betaFilter" :aria-label="$t('users.betaFilter')" @change="search">
          <option value="">{{ $t("users.betaAny") }}</option>
          <option v-for="c in openCampaigns" :key="c.key" :value="c.key">
            {{ c.name }} ({{ $t(`plans.kind.${c.kind}`) }})
          </option>
        </select>
      </label>
    </template>
    <button type="button" @click="search">{{ $t("users.search") }}</button>
  </div>

  <p v-if="error" class="error" role="alert">{{ error }}</p>

  <div class="table-card">
    <table>
      <thead>
        <tr>
          <th>{{ $t("users.colHandle") }}</th>
          <th>{{ $t("users.colName") }}</th>
          <th>{{ $t("users.colRoles") }}</th>
          <th v-if="showPlans">{{ $t("users.colPlan") }}</th>
          <th v-if="showPlans">{{ $t("users.colBeta") }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="a in vm.accounts" :key="a.userId" class="row-link" @click="openRow($event, a.userId)">
          <td class="handle">
            <RouterLink :to="{ name: 'user-detail', params: { userId: a.userId } }">
              {{ a.handle || $t("users.noHandle") }}
            </RouterLink>
          </td>
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
        </tr>
        <tr v-if="vm.accounts.length === 0">
          <td :colspan="colCount" class="empty">{{ vm.loading ? $t("common.loading") : $t("users.noAccounts") }}</td>
        </tr>
      </tbody>
    </table>
  </div>

  <div class="pager">
    <span class="muted">{{ $t("users.showing", { from, to, total: vm.total }) }}</span>
    <div class="pager-btns">
      <button type="button" :disabled="!canPrev" @click="prev">{{ $t("users.prev") }}</button>
      <button type="button" :disabled="!canNext" @click="next">{{ $t("users.next") }}</button>
    </div>
  </div>
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
.handle a {
  color: inherit;
  text-decoration: none;
}
.handle a:hover,
.handle a:focus-visible {
  color: var(--accent);
  text-decoration: underline;
}
.row-link {
  cursor: pointer;
}
.row-link:hover {
  background: var(--panel-2);
}
.rolechips {
  display: flex;
  gap: 0.3rem;
  flex-wrap: wrap;
  align-items: center;
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
.empty {
  color: var(--muted);
  text-align: center;
  padding: 2rem;
}
</style>
