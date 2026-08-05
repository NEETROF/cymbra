<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { CuratorReliability } from "@/gen/score_pb";

// Read-only curator-reliability indicator, shown in a right-side drawer (matching
// the SoundFont/Flag/GlobalAudit drawer convention) rather than a below-the-list
// panel (change: add-curation-rewards). Purely presentational: the parent view
// owns the API call + the Async fold and passes the resolved data in. It never
// triggers a role change — informational only, to inform manual promotion.
const props = defineProps<{
  open: boolean;
  handle: string;
  reliability: CuratorReliability | null;
  loading?: boolean;
}>();

defineEmits<{ (e: "close"): void }>();

const { t } = useI18n();

/** Format a bigint count for display. */
function count(v: bigint): string {
  return Number(v).toLocaleString();
}
/** Alignment rate is a 0–1 ratio over settled ratings; show it as a whole percent. */
function pct(rate: number): string {
  return `${Math.round(rate * 100)}%`;
}
</script>

<template>
  <div v-if="props.open" class="overlay" @click.self="$emit('close')">
    <dialog class="drawer" open aria-modal="true">
      <header>
        <div>
          <h3>{{ t("roles.reliabilityTitle") }}</h3>
          <p class="who">{{ props.handle }}</p>
        </div>
        <button type="button" class="x" :aria-label="t('roles.close')" @click="$emit('close')">✕</button>
      </header>

      <div v-if="props.reliability" class="rel-grid">
        <div class="rel-stat">
          <span class="rel-label">{{ t("roles.reliabilityRatings") }}</span>
          <span class="rel-value">{{ count(props.reliability.totalRatings) }}</span>
        </div>
        <div class="rel-stat">
          <span class="rel-label">{{ t("roles.reliabilityCoverage") }}</span>
          <span class="rel-value">{{ count(props.reliability.coverageContribution) }}</span>
        </div>
        <div class="rel-stat">
          <span class="rel-label">{{ t("roles.reliabilityAlignment") }}</span>
          <span class="rel-value">{{ pct(props.reliability.alignmentRate) }}</span>
          <span class="rel-note">{{
            t("roles.reliabilityAlignmentNote", {
              aligned: count(props.reliability.alignedCount),
              settled: count(props.reliability.settledCount),
            })
          }}</span>
        </div>
      </div>
      <p v-else class="muted">{{ props.loading ? t("common.loading") : t("roles.reliabilityEmpty") }}</p>
    </dialog>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 90;
}
.drawer {
  position: fixed;
  top: 0;
  right: 0;
  /* Cancel the browser's UA `dialog { left: 0 }`, which otherwise wins over `right`. */
  left: auto;
  height: 100vh;
  max-height: 100vh;
  width: min(480px, 94vw);
  max-width: none;
  margin: 0;
  overflow-y: auto;
  background: var(--panel, #1a1a24);
  color: inherit;
  border: none;
  border-left: 1px solid var(--border);
  padding: 1.35rem 1.4rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  animation: slideIn 0.22s ease;
}
@keyframes slideIn {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}
header {
  display: flex;
  align-items: flex-start;
  gap: 0.6rem;
}
header h3 {
  margin: 0;
  font-size: 1.05rem;
}
.who {
  margin: 0.15rem 0 0;
  font-weight: 600;
}
header > div {
  flex: 1;
}
.x {
  background: transparent;
  color: var(--muted);
}
.rel-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.85rem;
}
.rel-stat {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  padding: 1rem 1.1rem;
  background: var(--bg, #12121a);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
}
.rel-label {
  font-family: var(--mono);
  font-size: 0.68rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--muted);
}
.rel-value {
  font-size: 1.5rem;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
}
.rel-note {
  font-size: 0.8rem;
  color: var(--muted);
}
</style>
