<script setup lang="ts">
import { computed } from "vue";
import { Bar } from "vue-chartjs";
import {
  BarController,
  BarElement,
  CategoryScale,
  Chart as ChartJS,
  LinearScale,
  Tooltip,
  type ChartData,
  type ChartOptions,
} from "chart.js";

// A single-series horizontal bar chart for the Usage console (change: add-feature-
// usage-analytics). One accent hue (magnitude comparison, no legend needed); the
// Table toggle is the accessible tabular view. Colours are read from the theme's
// CSS custom properties so it tracks the back-office palette.
ChartJS.register(BarController, BarElement, CategoryScale, LinearScale, Tooltip);

const props = defineProps<{
  labels: string[];
  values: number[];
  /** Dataset name used in the tooltip (e.g. "Users", "Events"). */
  seriesLabel: string;
}>();

function cssVar(name: string, fallback: string): string {
  if (typeof window === "undefined") return fallback;
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return v || fallback;
}

const data = computed<ChartData<"bar">>(() => ({
  labels: props.labels,
  datasets: [
    {
      label: props.seriesLabel,
      data: props.values,
      backgroundColor: cssVar("--accent-strong", "#7c3aed"),
      hoverBackgroundColor: cssVar("--accent", "#d2bbff"),
      borderRadius: 4,
      borderSkipped: false,
      maxBarThickness: 26,
    },
  ],
}));

const options = computed<ChartOptions<"bar">>(() => {
  const ink = cssVar("--muted", "#9aa1ba");
  const grid = cssVar("--border", "#212a40");
  const panel2 = cssVar("--panel-2", "#171f33");
  const text = cssVar("--text", "#dae2fd");
  return {
    indexAxis: "y",
    responsive: true,
    maintainAspectRatio: false,
    animation: { duration: 300 },
    plugins: {
      legend: { display: false },
      tooltip: {
        backgroundColor: panel2,
        titleColor: text,
        bodyColor: text,
        borderColor: grid,
        borderWidth: 1,
        padding: 8,
        displayColors: false,
      },
    },
    scales: {
      x: {
        beginAtZero: true,
        border: { display: false },
        grid: { color: grid },
        ticks: { color: ink, precision: 0 },
      },
      y: {
        border: { display: false },
        grid: { display: false },
        ticks: { color: ink, autoSkip: false },
      },
    },
  };
});

// Horizontal bars: height scales with the number of categories.
const height = computed(() => Math.max(140, props.labels.length * 30 + 28));
</script>

<template>
  <div class="chart" :style="{ height: `${height}px` }">
    <Bar v-if="labels.length" :data="data" :options="options" />
    <p v-else class="empty">—</p>
  </div>
</template>

<style scoped>
.chart {
  position: relative;
  width: 100%;
}
.empty {
  opacity: 0.6;
  margin: 0;
}
</style>
