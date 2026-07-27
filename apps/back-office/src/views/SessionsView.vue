<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import { useSessionsStore, type SessionRow } from "@/stores/sessions";
import { useAuthStore } from "@/stores/auth";

const sessions = useSessionsStore();
const auth = useAuthStore();
const router = useRouter();

onMounted(() => sessions.load());

interface Vm {
  loading: boolean;
  error: string | null;
  items: SessionRow[];
}
const vm = computed<Vm>(() =>
  match(sessions.list)
    .with({ status: "idle" }, (): Vm => ({ loading: true, error: null, items: [] }))
    .with({ status: "loading" }, (): Vm => ({ loading: true, error: null, items: [] }))
    .with({ status: "error" }, ({ error }): Vm => ({ loading: false, error, items: [] }))
    .with({ status: "success" }, ({ data }): Vm => ({ loading: false, error: null, items: data }))
    .exhaustive(),
);

// The last action's error (revoke / sign-out-everywhere), if any.
const opError = computed(() =>
  match(sessions.op)
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() => null),
);
const busy = computed(() => sessions.op.status === "loading");

const isCurrent = (s: SessionRow): boolean => !!auth.sessionId && s.id === auth.sessionId;
// Only offer per-row revoke when we can tell which session is the current one; without
// a `sid` claim we can't, so we hide it (revoking your own session mid-use would sign
// you out) and leave "sign out everywhere" as the safe lever.
const currentKnown = computed(() => !!auth.sessionId);
const shortId = (id: string): string => id.slice(0, 8);

async function revoke(id: string) {
  await sessions.revoke(id);
}

async function signOutEverywhere() {
  const outcome = await sessions.revokeAll();
  if (outcome.status === "success") {
    // Clear this browser's HttpOnly cookie + in-memory token, then to sign-in.
    await auth.signOut();
    router.push({ name: "signin" });
  }
}
</script>

<template>
  <section class="sessions">
    <header class="head">
      <h1>{{ $t("sessions.title") }}</h1>
      <button type="button" class="danger" :disabled="busy || vm.items.length === 0" @click="signOutEverywhere">
        {{ $t("sessions.signOutAll") }}
      </button>
    </header>
    <p class="muted">{{ $t("sessions.intro") }}</p>

    <p v-if="opError" class="error" role="alert">{{ opError }}</p>

    <p v-if="vm.loading" class="muted">{{ $t("common.loading") }}</p>
    <p v-else-if="vm.error" class="error" role="alert">{{ vm.error }}</p>
    <p v-else-if="vm.items.length === 0" class="muted">{{ $t("sessions.empty") }}</p>

    <ul v-else class="list">
      <li v-for="s in vm.items" :key="s.id" class="row" :class="{ current: isCurrent(s) }">
        <div class="meta">
          <span class="app">{{ s.audience }}</span>
          <span class="id">{{ $t("sessions.id") }} {{ shortId(s.id) }}</span>
        </div>
        <span v-if="isCurrent(s)" class="badge">{{ $t("sessions.thisDevice") }}</span>
        <button v-else-if="currentKnown" type="button" :disabled="busy" @click="revoke(s.id)">
          {{ $t("sessions.revoke") }}
        </button>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.sessions {
  max-width: 720px;
}
.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}
h1 {
  margin: 0;
}
.muted {
  color: var(--muted);
}
.error {
  color: var(--reject);
  margin: 0.75rem 0;
}
.list {
  list-style: none;
  padding: 0;
  margin: 1.25rem 0 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.85rem 1rem;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
}
.row.current {
  border-color: color-mix(in srgb, var(--accent) 40%, var(--border));
}
.meta {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  min-width: 0;
}
.app {
  font-weight: 600;
  text-transform: capitalize;
}
.id {
  font-family: var(--mono);
  font-size: 0.78rem;
  color: var(--muted);
}
.badge {
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent) 28%, transparent);
  border-radius: 999px;
  padding: 0.15rem 0.6rem;
  font-size: 0.78rem;
}
.danger {
  color: var(--reject);
  border-color: color-mix(in srgb, var(--reject) 40%, var(--border));
}
.danger:hover:not(:disabled) {
  background: color-mix(in srgb, var(--reject) 12%, transparent);
}
</style>
