<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { useRolesStore } from "@/stores/roles";
import { useSessionsStore } from "@/stores/sessions";
import { useAuthStore } from "@/stores/auth";
import { useToastsStore } from "@/stores/toasts";
import AccountPlanPanel from "@/components/AccountPlanPanel.vue";
import AccountRolesPanel from "@/components/AccountRolesPanel.vue";
import AccountHistoryPanel from "@/components/AccountHistoryPanel.vue";
import CuratorReliabilityDrawer from "@/components/CuratorReliabilityDrawer.vue";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import IdBadge from "@/components/IdBadge.vue";
import type { RoleGrant } from "@/gen/user_pb";

// One account, one address (change: restructure-back-office-users-console). Everything
// the console knows and can do about this person: subscription, roles in every scope the
// caller administers, role audit history, curator reliability, session revocation.
//
// The page stands on its own: it loads the account BY ID, so a bookmark, a reload or a
// link from a campaign's member list works without the directory ever being visited.
const props = defineProps<{ userId: string }>();

const store = useRolesStore();
const sessions = useSessionsStore();
const auth = useAuthStore();
const toasts = useToastsStore();
const { t } = useI18n();

/** Plan data is music-admin only: another scope's admin sees the account without any
 *  subscription block, and the plan RPCs are never issued for them. */
const showPlans = computed(() => auth.adminScopes.includes("music"));

const vm = computed(() =>
  match(store.account)
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null as string | null, account: data }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, account: null }))
    .otherwise(() => ({ loading: true, error: null as string | null, account: null })),
);
const grants = computed(() =>
  match(store.grants)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => [] as RoleGrant[]),
);
const grantsLoading = computed(() => store.grants.status === "loading" || store.grants.status === "idle");
const acting = computed(() => store.op.status === "loading" || sessions.op.status === "loading");
const title = computed(() => vm.value.account?.handle || vm.value.account?.displayName || "");

// Read-only curator reliability, loaded on demand (it is an extra call, and an operator
// asks for it rather than always reading it). Informational only — it never triggers a
// role change.
const reliabilityOpen = ref(false);
const reliability = computed(() =>
  match(store.reliability)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => null),
);
const reliabilityLoading = computed(() => store.reliability.status === "loading");
function openReliability() {
  reliabilityOpen.value = true;
  void store.loadReliability(props.userId);
}

/** Admin: cut off every session of a compromised account (server-gated by require_admin).
 *  Confirm first — it's destructive — and let the outcome surface as a toast. */
const pendingSessionRevoke = ref(false);
async function confirmSessionRevoke() {
  pendingSessionRevoke.value = false;
  await sessions.revokeAccount(props.userId);
}

// Action results and a failed reliability read surface as toasts (localized, never a raw
// gRPC status); only the account LOAD error stays inline on the page.
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

// Load on arrival AND on every account switch. The per-account slots are shared store
// state, so they are dropped first: one frame of account A's roles under account B's
// name is unacceptable on a page where rights are revoked.
watch(
  () => props.userId,
  (userId) => {
    reliabilityOpen.value = false;
    store.resetAccount();
    void store.loadAccount(userId);
    void store.listGrants(userId);
  },
  { immediate: true },
);
</script>

<template>
  <div class="head">
    <RouterLink class="back" :to="{ name: 'users' }">← {{ $t("users.backToDirectory") }}</RouterLink>
  </div>

  <p v-if="vm.loading" class="muted">{{ $t("common.loading") }}</p>
  <p v-if="vm.error" class="error" role="alert">{{ vm.error }}</p>

  <template v-if="!vm.loading && !vm.error && !vm.account">
    <h1 class="page-title">{{ $t("users.notFoundTitle") }}</h1>
    <p class="muted">{{ $t("users.notFound") }}</p>
  </template>

  <template v-if="vm.account">
    <div class="identity">
      <h1 class="page-title">{{ title || $t("users.noHandle") }}</h1>
      <span v-if="vm.account.displayName && vm.account.handle" class="muted">{{ vm.account.displayName }}</span>
      <IdBadge :id="vm.account.userId" />
    </div>
    <div class="head-actions">
      <button type="button" :disabled="acting" @click="openReliability">{{ $t("users.reliability") }}</button>
      <button type="button" :disabled="acting" @click="pendingSessionRevoke = true">
        {{ $t("sessions.revokeAccount") }}
      </button>
    </div>

    <!-- Music-admin only: the block is absent (not empty) for any other admin. -->
    <AccountPlanPanel v-if="showPlans" :key="vm.account.userId" :user-id="vm.account.userId" :handle="title" />

    <AccountRolesPanel :account="vm.account" />

    <AccountHistoryPanel :grants="grants" :loading="grantsLoading" />
  </template>

  <CuratorReliabilityDrawer
    :open="reliabilityOpen"
    :handle="title"
    :reliability="reliability"
    :loading="reliabilityLoading"
    @close="reliabilityOpen = false"
  />
  <ConfirmDialog
    :message="pendingSessionRevoke ? t('sessions.revokeAccountConfirm') : null"
    :busy="acting"
    @confirm="confirmSessionRevoke"
    @cancel="pendingSessionRevoke = false"
  />
</template>

<style scoped>
.head {
  margin-bottom: 0.75rem;
}
.back {
  color: var(--muted);
  text-decoration: none;
  font-size: 0.85rem;
}
.back:hover,
.back:focus-visible {
  color: var(--accent);
}
.identity {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}
.identity .page-title {
  margin: 0;
}
.head-actions {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  margin-top: 0.75rem;
}
</style>
