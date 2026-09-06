<script setup lang="ts">
import type { RoleGrant } from "@/gen/user_pb";

// One account's role audit history. Purely presentational: the detail view owns the
// call and the `Async` fold and passes the resolved rows in.
const props = defineProps<{ grants: RoleGrant[]; loading?: boolean }>();

function when(atSeconds: bigint | number): string {
  const ms = Number(atSeconds) * 1000;
  return ms > 0 ? new Date(ms).toISOString().replace("T", " ").slice(0, 19) : "—";
}
</script>

<template>
  <section class="block">
    <h2>{{ $t("users.history") }}</h2>
    <div class="table-card">
      <table data-testid="role-history">
        <thead>
          <tr>
            <th>{{ $t("users.when") }}</th>
            <th>{{ $t("users.action") }}</th>
            <th>{{ $t("users.scope") }}</th>
            <th>{{ $t("users.role") }}</th>
            <th>{{ $t("users.byAdmin") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(g, i) in props.grants" :key="i">
            <td>{{ when(g.at) }}</td>
            <td>{{ g.action }}</td>
            <td>{{ g.scope }}</td>
            <td>{{ g.role }}</td>
            <td :class="{ mono: !g.actingAdminHandle }">{{ g.actingAdminHandle || g.actingAdmin }}</td>
          </tr>
          <tr v-if="props.grants.length === 0">
            <td colspan="5" class="empty">
              {{ props.loading ? $t("common.loading") : $t("users.noHistory") }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<style scoped>
.block {
  margin-top: 1.75rem;
}
.block h2 {
  font-size: 1.15rem;
  margin: 0 0 0.75rem;
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
