<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { useCatalogStore } from "@/stores/catalog";
import { currentLocale } from "@/i18n";

// Header KPI cards (mockup: Total / Approved / Awaiting review). Data comes from
// the store's `stats` Async; a failure degrades gracefully to "—".
const store = useCatalogStore();
const { t } = useI18n();

const vm = computed(() =>
  match(store.stats)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => null),
);

function num(v: number | undefined): string {
  return v == null ? "—" : v.toLocaleString(currentLocale());
}

const cards = computed(() => [
  { id: "total", key: "stats.total", value: num(vm.value?.total), accent: "accent", icon: "M4 7h16M4 12h16M4 17h10" },
  { id: "approved", key: "stats.approved", value: num(vm.value?.accepted), accent: "green", icon: "M20 6 9 17l-5-5" },
  { id: "pending", key: "stats.pending", value: num(vm.value?.pending), accent: "amber", icon: "M12 7v5l3 2M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Z" },
]);

onMounted(() => store.loadStats());
</script>

<template>
  <div class="stats">
    <div v-for="c in cards" :key="c.key" class="stat" :class="c.accent" :data-testid="`stat-${c.id}`">
      <div class="stat-body">
        <span class="stat-label">{{ t(c.key) }}</span>
        <span class="stat-value" data-testid="stat-value">{{ c.value }}</span>
      </div>
      <span class="stat-ic">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
             stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path :d="c.icon" />
        </svg>
      </span>
    </div>
  </div>
</template>

<style scoped>
.stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
  margin-top: 1.5rem;
}
.stat {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1.1rem 1.25rem;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
}
.stat-body { display: flex; flex-direction: column; gap: 0.35rem; }
.stat-label {
  font-family: var(--mono);
  font-size: 0.68rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
.stat-value { font-size: 1.7rem; font-weight: 800; font-variant-numeric: tabular-nums; }
.stat-ic {
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  border-radius: 12px;
}
.stat-ic svg { width: 20px; height: 20px; }

.stat.accent .stat-label { color: var(--accent); }
.stat.accent .stat-ic { color: var(--accent); background: color-mix(in srgb, var(--accent) 14%, transparent); }
.stat.green .stat-label { color: var(--green); }
.stat.green .stat-ic { color: var(--green); background: color-mix(in srgb, var(--green) 14%, transparent); }
.stat.amber .stat-label { color: var(--amber); }
.stat.amber .stat-ic { color: var(--amber); background: color-mix(in srgb, var(--amber) 14%, transparent); }

@media (max-width: 720px) {
  .stats { grid-template-columns: 1fr; }
}
</style>
