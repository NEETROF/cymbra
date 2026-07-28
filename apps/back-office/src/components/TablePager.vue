<script setup lang="ts">
import { computed } from "vue";

// Presentational server-side pager: it only knows the current window (offset/limit)
// and the total, and emits the target offset — the view re-queries. No API here.
const props = defineProps<{ offset: number; limit: number; total: number }>();
const emit = defineEmits<{ (e: "page", offset: number): void }>();

const from = computed(() => (props.total === 0 ? 0 : props.offset + 1));
const to = computed(() => Math.min(props.offset + props.limit, props.total));
const hasPrev = computed(() => props.offset > 0);
const hasNext = computed(() => props.offset + props.limit < props.total);

function prev() {
  if (hasPrev.value) emit("page", Math.max(0, props.offset - props.limit));
}
function next() {
  if (hasNext.value) emit("page", props.offset + props.limit);
}
</script>

<template>
  <nav v-if="total > limit" class="pager" :aria-label="$t('pager.label')">
    <button type="button" :disabled="!hasPrev" @click="prev">{{ $t("pager.prev") }}</button>
    <span class="pager-range">{{ $t("pager.range", { from, to, total }) }}</span>
    <button type="button" :disabled="!hasNext" @click="next">{{ $t("pager.next") }}</button>
  </nav>
</template>

<style scoped>
.pager {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 0.75rem 0.25rem 0;
}
.pager-range {
  font-variant-numeric: tabular-nums;
  opacity: 0.75;
}
.pager button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
