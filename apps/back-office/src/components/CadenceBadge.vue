<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import AppTag from "@/components/AppTag.vue";
import { kindCadence, kindCadenceLabel, type ScheduleInfo } from "@/lib/cadence";

// Whether a job kind runs on its own, and how often (change: add-jobs-console-history).
// Props only: the schedules come from the kinds the store loaded. `null` = the kinds are
// not known (yet), so nothing is claimed.
// `wrap` lets the label break inside a table cell rather than widen its column.
const props = defineProps<{ schedules: readonly ScheduleInfo[] | null; wrap?: boolean }>();
const { t } = useI18n();

const cadence = computed(() => (props.schedules === null ? null : kindCadence(props.schedules)));
const label = computed(() => (cadence.value === null ? "" : kindCadenceLabel(cadence.value, t)));
const variant = computed(() =>
  cadence.value === null
    ? "neutral"
    : match(cadence.value)
        .with({ type: "scheduled" }, () => "accent" as const)
        .with({ type: "paused" }, () => "pending" as const)
        .with({ type: "onDemand" }, () => "neutral" as const)
        .exhaustive(),
);
</script>

<template>
  <AppTag
    v-if="cadence"
    :variant="variant"
    :data-cadence="cadence.type"
    :style="wrap ? { whiteSpace: 'normal' } : undefined"
    data-testid="cadence"
    >{{ label }}</AppTag
  >
</template>
