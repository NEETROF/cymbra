<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import CadenceBadge from "@/components/CadenceBadge.vue";
import { currentLocale } from "@/i18n";
import { formatCount, formatDuration, formatAbsolute, formatRelative } from "@/lib/jobFormat";
import { breakdownRows, breakdownUnavailable, type JobKindInfo, type JobStats } from "@/stores/jobs";

// The period figures per job kind (change: add-jobs-console-history). Props and events
// only — the store owns the data. Every registered kind is listed, zeros included, so a
// scheduled kind that did not run stands out; selecting a kind asks for its history.
const props = defineProps<{
  stats: JobStats | null;
  kinds: readonly JobKindInfo[] | null;
  kindFilter: string;
}>();
const emit = defineEmits<{ (e: "select", kind: string): void }>();
const { t } = useI18n();

const unavailable = computed(() => props.stats !== null && breakdownUnavailable(props.stats));
const rows = computed(() =>
  props.stats === null || unavailable.value
    ? []
    : breakdownRows(props.kinds ?? [], props.stats.byKind, props.kindFilter),
);
const schedulesOf = (kind: string) =>
  props.kinds === null ? null : (props.kinds.find((k) => k.name === kind)?.schedules ?? []);
const count = (n: number) => formatCount(n, currentLocale());
const duration = (ms: number | null) => formatDuration(ms, t, currentLocale());
const relative = (ms: number | null) => formatRelative(ms, Date.now(), currentLocale());
const absolute = (ms: number | null) => (ms === null ? undefined : formatAbsolute(ms, currentLocale()));
</script>

<template>
  <section v-if="stats" class="breakdown" data-testid="breakdown">
    <h3 class="section">{{ t("jobs.breakdown.title") }}</h3>
    <p v-if="unavailable" class="muted" data-testid="breakdown-unavailable">{{ t("jobs.breakdown.unavailable") }}</p>
    <div v-else class="table-card">
      <table>
        <thead>
          <tr>
            <th>{{ t("jobs.breakdown.columns.kind") }}</th>
            <th class="n">{{ t("jobs.breakdown.columns.completed") }}</th>
            <th class="n">{{ t("jobs.breakdown.columns.failed") }}</th>
            <th class="n">{{ t("jobs.breakdown.columns.dead") }}</th>
            <th class="n">{{ t("jobs.breakdown.columns.cancelled") }}</th>
            <th class="n">{{ t("jobs.breakdown.columns.avg") }}</th>
            <th>{{ t("jobs.breakdown.columns.last") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="row in rows"
            :key="row.kind"
            :class="{ idle: row.lastFinishedAt === null && row.period.completed === 0 }"
            data-testid="breakdown-row"
            :data-kind="row.kind"
          >
            <td>
              <button
                type="button"
                class="kind"
                :title="t('jobs.breakdown.open', { kind: row.kind })"
                data-testid="breakdown-open"
                @click="emit('select', row.kind)"
              >
                {{ row.kind }}
              </button>
              <CadenceBadge :schedules="schedulesOf(row.kind)" />
            </td>
            <td class="n" data-testid="breakdown-completed">{{ count(row.period.completed) }}</td>
            <td class="n">{{ count(row.period.failedAttempts) }}</td>
            <td class="n">{{ count(row.period.deadLettered) }}</td>
            <td class="n">{{ count(row.period.cancelled) }}</td>
            <td class="n">{{ duration(row.period.avgRunMs) }}</td>
            <td :title="absolute(row.lastFinishedAt)" data-testid="breakdown-last">
              {{ relative(row.lastFinishedAt) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<style scoped>
.breakdown {
  margin-bottom: 1.5rem;
}
.section {
  margin: 0 0 0.6rem;
  font-size: 0.9rem;
  color: var(--muted);
}
.kind {
  padding: 0;
  margin-right: 0.5rem;
  border: 0;
  background: none;
  color: var(--accent);
  font-weight: 600;
  cursor: pointer;
}
.kind:hover {
  text-decoration: underline;
  background: none;
}
tr.idle td {
  color: var(--muted);
}
th.n,
td.n {
  text-align: right;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
</style>
