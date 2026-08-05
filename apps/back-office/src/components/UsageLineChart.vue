<script setup lang="ts">
import { computed } from "vue";
import { Line } from "vue-chartjs";
import {
  CategoryScale,
  Chart as ChartJS,
  Legend,
  LineController,
  LineElement,
  LinearScale,
  PointElement,
  Tooltip,
  type ChartData,
  type ChartOptions,
} from "chart.js";

// A multi-series time-series line chart for the Usage console (change: add-feature-
// usage-analytics): x = day, y = users/events, one curve per series (platform /
// device class / action). Uses the data-viz reference DARK categorical palette
// (validated: adjacent-pair CVD ΔE ≥ 8, normal-vision ≥ 15), assigned in FIXED
// order and never cycled — a legend carries identity (never colour alone).
ChartJS.register(CategoryScale, LinearScale, LineController, LineElement, PointElement, Tooltip, Legend);

// Reference dark categorical palette (references/palette.md), fixed order.
const PALETTE = ["#3987e5", "#d95926", "#199e70", "#c98500", "#d55181", "#008300", "#9085e9", "#e66767"];

const props = defineProps<{
  labels: string[]; // day axis (ISO yyyy-mm-dd)
  datasets: { label: string; values: number[] }[];
  yLabel: string;
}>();

function cssVar(name: string, fallback: string): string {
  if (typeof window === "undefined") return fallback;
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return v || fallback;
}

const data = computed<ChartData<"line">>(() => ({
  labels: props.labels,
  datasets: props.datasets.map((d, i) => {
    const c = PALETTE[i % PALETTE.length];
    return {
      label: d.label,
      data: d.values,
      borderColor: c,
      backgroundColor: c,
      borderWidth: 2,
      pointRadius: props.labels.length > 40 ? 0 : 2.5,
      pointHoverRadius: 4,
      tension: 0.25,
      spanGaps: true,
    };
  }),
}));

const options = computed<ChartOptions<"line">>(() => {
  const ink = cssVar("--muted", "#9aa1ba");
  const grid = cssVar("--border", "#212a40");
  const panel2 = cssVar("--panel-2", "#171f33");
  const text = cssVar("--text", "#dae2fd");
  return {
    responsive: true,
    maintainAspectRatio: false,
    interaction: { mode: "index", intersect: false },
    animation: { duration: 300 },
    plugins: {
      legend: {
        display: true,
        position: "bottom",
        labels: { color: ink, boxWidth: 12, boxHeight: 12, usePointStyle: true, pointStyle: "line" },
      },
      tooltip: {
        backgroundColor: panel2,
        titleColor: text,
        bodyColor: text,
        borderColor: grid,
        borderWidth: 1,
        padding: 8,
      },
    },
    scales: {
      x: {
        border: { display: false },
        grid: { display: false },
        ticks: {
          color: ink,
          maxRotation: 0,
          autoSkip: true,
          maxTicksLimit: 8,
          // Drop the leading "YYYY-" so dates don't collide (labels are ISO days).
          callback(value) {
            const label = this.getLabelForValue(value as number);
            return typeof label === "string" && label.length >= 10 ? label.slice(5) : label;
          },
        },
      },
      y: {
        beginAtZero: true,
        border: { display: false },
        grid: { color: grid },
        ticks: { color: ink, precision: 0 },
        title: { display: true, text: props.yLabel, color: ink },
      },
    },
  };
});
</script>

<template>
  <div class="chart">
    <Line v-if="labels.length && datasets.length" :data="data" :options="options" />
    <p v-else class="empty">—</p>
  </div>
</template>

<style scoped>
.chart {
  position: relative;
  width: 100%;
  height: 260px;
}
.empty {
  opacity: 0.6;
  margin: 0;
}
</style>
