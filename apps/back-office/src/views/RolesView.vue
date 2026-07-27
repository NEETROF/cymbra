<script setup lang="ts">
import { computed, ref } from "vue";
import { match } from "ts-pattern";
import { useRolesStore } from "@/stores/roles";
import type { RoleGrant } from "@/gen/user_pb";

// Admin-only (route-gated + server-guarded by require_admin). Grant/revoke a role
// for an account by its user id, and view that account's audit history.
const store = useRolesStore();
const userId = ref("");
const role = ref("moderator");

const grants = computed(() =>
  match(store.grants)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => [] as RoleGrant[]),
);
const busy = computed(() => store.op.status === "loading");
const error = computed(() =>
  match(store.op)
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() =>
      match(store.grants)
        .with({ status: "error" }, ({ error }) => error)
        .otherwise(() => null),
    ),
);

async function load() {
  if (userId.value) await store.listGrants(userId.value);
}
async function grant() {
  if (userId.value) await store.grant(userId.value, role.value);
}
async function revoke() {
  if (userId.value) await store.revoke(userId.value, role.value);
}

function when(atSeconds: bigint | number): string {
  const ms = Number(atSeconds) * 1000;
  return ms > 0 ? new Date(ms).toISOString().replace("T", " ").slice(0, 19) : "—";
}
</script>

<template>
  <h1 class="page-title">{{ $t("roles.title") }}</h1>
  <p class="muted">{{ $t("roles.intro") }}</p>

  <div class="form">
    <input
      v-model="userId"
      type="text"
      :placeholder="$t('roles.userIdPlaceholder')"
      :aria-label="$t('roles.userIdLabel')"
    />
    <select v-model="role" :aria-label="$t('roles.roleLabel')">
      <option value="moderator">{{ $t("role.moderator") }}</option>
      <option value="admin">{{ $t("role.admin") }}</option>
    </select>
    <button :disabled="busy || !userId" @click="grant">{{ $t("roles.grant") }}</button>
    <button :disabled="busy || !userId" @click="revoke">{{ $t("roles.revoke") }}</button>
    <button :disabled="!userId" @click="load">{{ $t("roles.loadHistory") }}</button>
  </div>

  <p v-if="error" class="error" role="alert">{{ error }}</p>

  <div v-if="grants.length" class="table-card">
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
          <td class="mono">{{ g.actingAdmin }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.form {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  margin: 1.25rem 0;
  padding: 0.9rem;
  flex-wrap: wrap;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 14px;
}
.muted {
  color: var(--muted);
}
.error {
  color: var(--reject);
}
.mono {
  font-family: ui-monospace, monospace;
  font-size: 0.85rem;
}
</style>
