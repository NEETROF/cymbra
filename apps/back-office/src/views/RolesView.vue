<script setup lang="ts">
import { ref } from "vue";
import { useRolesStore } from "@/stores/roles";

// Admin-only (route-gated + server-guarded by require_admin). Grant/revoke a role
// for an account by its user id, and view that account's audit history.
const store = useRolesStore();
const userId = ref("");
const role = ref("moderator");

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
  <h1>Roles</h1>
  <p class="muted">
    Grant or revoke a role for an account (by its user id) within the
    <code>music</code> scope. Every change is recorded in the audit trail below.
  </p>

  <div class="form">
    <input v-model="userId" type="text" placeholder="target user id (UUID)" aria-label="user id" />
    <select v-model="role" aria-label="role">
      <option value="moderator">moderator</option>
      <option value="admin">admin</option>
    </select>
    <button :disabled="store.busy || !userId" @click="grant">Grant</button>
    <button :disabled="store.busy || !userId" @click="revoke">Revoke</button>
    <button :disabled="!userId" @click="load">Load history</button>
  </div>

  <p v-if="store.error" class="error" role="alert">{{ store.error }}</p>

  <table v-if="store.grants.length">
    <thead>
      <tr>
        <th>When</th>
        <th>Action</th>
        <th>Scope</th>
        <th>Role</th>
        <th>By admin</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="(g, i) in store.grants" :key="i">
        <td>{{ when(g.at) }}</td>
        <td>{{ g.action }}</td>
        <td>{{ g.scope }}</td>
        <td>{{ g.role }}</td>
        <td class="mono">{{ g.actingAdmin }}</td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
.form {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  margin: 1rem 0;
  flex-wrap: wrap;
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
