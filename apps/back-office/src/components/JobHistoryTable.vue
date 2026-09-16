<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import AppTag from "@/components/AppTag.vue";
import CadenceBadge from "@/components/CadenceBadge.vue";
import TablePager from "@/components/TablePager.vue";
import { AttemptOutcome } from "@/gen/jobs_admin_pb";
import { currentLocale } from "@/i18n";
import { formatAbsolute, formatCount, formatDuration, formatRelative } from "@/lib/jobFormat";
import type { FinishedAttemptRow, JobKindInfo } from "@/stores/jobs";

// The finished attempts (change: add-jobs-console-history). Props and events only: the
// view hands in its view-model and relays the outcome filter and the page to the store.
const props = defineProps<{
  attempts: readonly FinishedAttemptRow[];
  total: number;
  loading: boolean;
  error: string | null;
  offset: number;
  limit: number;
  outcome: AttemptOutcome;
  kinds: readonly JobKindInfo[] | null;
}>();
const emit = defineEmits<{
  (e: "page", offset: number): void;
  (e: "outcome", outcome: AttemptOutcome): void;
}>();
const { t } = useI18n();

type TagVariant = "accepted" | "pending" | "rejected";
// Keyed by the enum so a new outcome in the contract is a compile error here.
const OUTCOMES = {
  [AttemptOutcome.SUCCEEDED]: { key: "succeeded", tag: "accepted" },
  [AttemptOutcome.FAILED]: { key: "failed", tag: "pending" },
  [AttemptOutcome.ABANDONED]: { key: "abandoned", tag: "rejected" },
} as const satisfies Record<Exclude<AttemptOutcome, AttemptOutcome.UNSPECIFIED>, { key: string; tag: TagVariant }>;
const FILTERS = [AttemptOutcome.SUCCEEDED, AttemptOutcome.FAILED, AttemptOutcome.ABANDONED] as const;
const outcomeInfo = (o: AttemptOutcome) => (o === AttemptOutcome.UNSPECIFIED ? undefined : OUTCOMES[o]);

const selected = ref<AttemptOutcome>(props.outcome);
watch(
  () => props.outcome,
  (o) => {
    selected.value = o;
  },
);

const schedulesOf = (kind: string) =>
  props.kinds === null ? null : (props.kinds.find((k) => k.name === kind)?.schedules ?? []);
const matching = computed(() => t("jobs.history.matching", { n: formatCount(props.total, currentLocale()) }));
const duration = (ms: number | null) => formatDuration(ms, t, currentLocale());
const relative = (ms: number) => formatRelative(ms, Date.now(), currentLocale());
const absolute = (ms: number) => formatAbsolute(ms, currentLocale());
</script>

<template>
  <div class="filters">
    <label>
      <span>{{ t("jobs.history.outcome") }}</span>
      <select v-model="selected" data-testid="history-outcome" @change="emit('outcome', selected)">
        <option :value="AttemptOutcome.UNSPECIFIED">{{ t("jobs.filters.any") }}</option>
        <option v-for="o in FILTERS" :key="o" :value="o">{{ t(`jobs.outcomes.${OUTCOMES[o].key}`) }}</option>
      </select>
    </label>
    <span v-if="!loading && !error" class="muted matching" data-testid="history-total">{{ matching }}</span>
  </div>

  <p v-if="error" class="error" role="alert" data-testid="history-error">{{ error }}</p>

  <div class="table-card">
    <table data-testid="history-table">
      <thead>
        <tr>
          <th>{{ t("jobs.history.columns.kind") }}</th>
          <th>{{ t("jobs.history.columns.outcome") }}</th>
          <th class="n">{{ t("jobs.history.columns.attempt") }}</th>
          <th>{{ t("jobs.history.columns.finished") }}</th>
          <th class="n">{{ t("jobs.history.columns.duration") }}</th>
          <th>{{ t("jobs.history.columns.id") }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="a in attempts" :key="`${a.jobId}-${a.attempt}`" data-testid="history-row">
          <td>
            <div class="kind">
              {{ a.kind }}
              <CadenceBadge :schedules="schedulesOf(a.kind)" wrap />
            </div>
            <div class="muted channel">{{ a.channel }}</div>
          </td>
          <td>
            <AppTag :variant="outcomeInfo(a.outcome)?.tag ?? 'neutral'" data-testid="history-outcome-tag">
              {{ outcomeInfo(a.outcome) ? t(`jobs.outcomes.${outcomeInfo(a.outcome)?.key}`) : "—" }}
            </AppTag>
          </td>
          <td class="n">{{ a.attempt }}</td>
          <td :title="absolute(a.finishedAt)">{{ relative(a.finishedAt) }}</td>
          <td class="n" data-testid="history-duration">{{ duration(a.durationMs) }}</td>
          <td>
            <code class="id" :title="a.jobId">{{ a.jobId.slice(0, 8) }}</code>
          </td>
        </tr>
        <tr v-if="loading">
          <td colspan="6" class="empty" data-testid="history-loading">{{ t("jobs.history.loading") }}</td>
        </tr>
        <tr v-else-if="attempts.length === 0 && !error">
          <td colspan="6" class="empty" data-testid="history-empty">{{ t("jobs.history.empty") }}</td>
        </tr>
      </tbody>
    </table>
  </div>

  <TablePager :offset="offset" :limit="limit" :total="total" @page="(o: number) => emit('page', o)" />
</template>

<style scoped>
.filters {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: end;
  margin-bottom: 1rem;
}
.filters label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.8rem;
  color: var(--muted);
}
.matching {
  font-size: 0.85rem;
  padding-bottom: 0.55rem;
}
.kind {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.4rem;
  font-weight: 600;
}
.channel {
  font-family: var(--mono);
  font-size: 0.72rem;
}
.id {
  font-family: var(--mono);
  font-size: 0.8rem;
  color: var(--muted);
}
th.n,
td.n {
  text-align: right;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.empty {
  color: var(--muted);
  text-align: center;
  padding: 2rem;
}
</style>
