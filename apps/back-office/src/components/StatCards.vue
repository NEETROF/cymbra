<script setup lang="ts">
// Generic KPI cards (label + big number + accent icon). Presentational: the caller
// passes the items. Mirrors the score dashboard's StatBar look so the back office
// reads as one system.
export type StatItem = {
  id: string;
  label: string;
  value: string;
  accent: "accent" | "green" | "amber" | "red";
  icon: string; // SVG path data (24x24 viewBox, stroke)
};

defineProps<{ items: StatItem[] }>();
</script>

<template>
  <div class="stats">
    <div v-for="c in items" :key="c.id" class="stat" :class="c.accent" :data-testid="`stat-${c.id}`">
      <div class="stat-body">
        <span class="stat-label">{{ c.label }}</span>
        <span class="stat-value" data-testid="stat-value">{{ c.value }}</span>
      </div>
      <span class="stat-ic">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path :d="c.icon" />
        </svg>
      </span>
    </div>
  </div>
</template>

<style scoped>
.stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 1rem;
  margin-bottom: 1.5rem;
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
.stat-body {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}
.stat-label {
  font-family: var(--mono);
  font-size: 0.68rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
.stat-value {
  font-size: 1.7rem;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
}
.stat-ic {
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  border-radius: 12px;
}
.stat-ic svg {
  width: 20px;
  height: 20px;
}
.stat.accent .stat-label {
  color: var(--accent);
}
.stat.accent .stat-ic {
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 14%, transparent);
}
.stat.green .stat-label {
  color: var(--green);
}
.stat.green .stat-ic {
  color: var(--green);
  background: color-mix(in srgb, var(--green) 14%, transparent);
}
.stat.amber .stat-label {
  color: var(--amber);
}
.stat.amber .stat-ic {
  color: var(--amber);
  background: color-mix(in srgb, var(--amber) 14%, transparent);
}
.stat.red .stat-label {
  color: var(--danger, #c0392b);
}
.stat.red .stat-ic {
  color: var(--danger, #c0392b);
  background: color-mix(in srgb, var(--danger, #c0392b) 14%, transparent);
}
</style>
