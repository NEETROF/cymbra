<script setup lang="ts">
import { computed } from "vue";
import { useRolesStore } from "@/stores/roles";
import { useAuthStore } from "@/stores/auth";
import AppTag from "@/components/AppTag.vue";
import type { AccountRow } from "@/gen/user_pb";
import type { Scope } from "@/lib/jwt";

// One account's roles, one block per scope the caller may administer (change:
// restructure-back-office-users-console). The directory needs a scope selector because
// a table has one column for the roles; a detail page does not — "which roles does this
// account hold?" is answered without touching a selector, and a single-scope admin sees
// exactly their scope. Authorization stays the server's: `require_admin_in_scope`.
const props = defineProps<{ account: AccountRow }>();

const store = useRolesStore();
const auth = useAuthStore();

const MANAGED_ROLES = ["moderator", "admin"] as const;
const scopes = computed<Scope[]>(() => auth.adminScopes);
const acting = computed(() => store.op.status === "loading");

function rolesIn(scope: string): string[] {
  return props.account.rolesByScope.find((sr) => sr.scope === scope)?.roles ?? [];
}
function toggle(scope: string, role: string) {
  // "account": the change re-reads THIS account, not a directory page that is not on screen.
  if (rolesIn(scope).includes(role)) store.revoke(props.account.userId, role, scope, "account");
  else store.grant(props.account.userId, role, scope, "account");
}
</script>

<template>
  <section class="block">
    <h2>{{ $t("users.roles") }}</h2>
    <div class="scopes">
      <div v-for="s in scopes" :key="s" class="scope-row">
        <span class="scope-name">{{ $t(`scope.${s}`) }}</span>
        <div class="rolechips">
          <AppTag v-for="r in rolesIn(s)" :key="r" variant="accent" cap>{{ $t(`role.${r}`) }}</AppTag>
          <span v-if="rolesIn(s).length === 0" class="muted">—</span>
        </div>
        <div class="actions">
          <button
            v-for="r in MANAGED_ROLES"
            :key="r"
            type="button"
            class="toggle"
            :class="{ held: rolesIn(s).includes(r) }"
            :disabled="acting"
            :aria-label="
              rolesIn(s).includes(r)
                ? $t('users.revokeRoleInScope', { role: $t(`role.${r}`), scope: $t(`scope.${s}`) })
                : $t('users.grantRoleInScope', { role: $t(`role.${r}`), scope: $t(`scope.${s}`) })
            "
            @click="toggle(s, r)"
          >
            {{ rolesIn(s).includes(r) ? "−" : "+" }} {{ $t(`role.${r}`) }}
          </button>
        </div>
      </div>
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
.scopes {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  padding: 1rem 1.25rem;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
}
.scope-row {
  display: flex;
  align-items: center;
  gap: 0.9rem;
  flex-wrap: wrap;
}
.scope-name {
  min-width: 5rem;
  font-family: var(--mono);
  font-size: 0.72rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
}
.rolechips {
  display: flex;
  gap: 0.3rem;
  flex-wrap: wrap;
  align-items: center;
  flex: 1;
}
.actions {
  display: flex;
  gap: 0.4rem;
  flex-wrap: wrap;
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
</style>
