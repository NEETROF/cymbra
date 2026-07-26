<script setup lang="ts">
import { reactive, watch } from "vue";
import type { Filters, ModerationStatus } from "@/stores/catalog";

// Reuses the app-hub filters (text/author/level/piano) plus the back-office-only
// moderation-status selector. Emits the current filter set whenever it changes.
const emit = defineEmits<{ change: [Filters] }>();

const props = withDefaults(defineProps<{ status?: ModerationStatus }>(), { status: "pending" });

const f = reactive<Filters>({
  query: "",
  author: "",
  level: "",
  isPiano: undefined,
  moderationStatus: props.status,
});

watch(
  f,
  () => emit("change", { ...f }),
  { deep: true },
);
</script>

<template>
  <div class="filters">
    <input v-model="f.query" type="search" placeholder="title or composer" aria-label="search" />
    <input v-model="f.author" type="text" placeholder="composer" aria-label="composer" />
    <select v-model="f.level" aria-label="level">
      <option value="">any level</option>
      <option value="beginner">beginner</option>
      <option value="intermediate">intermediate</option>
      <option value="advanced">advanced</option>
    </select>
    <label class="piano">
      <input type="checkbox" :checked="f.isPiano === true" @change="f.isPiano = f.isPiano ? undefined : true" />
      piano only
    </label>
    <select v-model="f.moderationStatus" aria-label="moderation status" class="status">
      <option value="pending">pending</option>
      <option value="accepted">accepted</option>
      <option value="rejected">rejected</option>
    </select>
  </div>
</template>

<style scoped>
.filters {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
  margin-bottom: 1rem;
}
.piano {
  display: flex;
  gap: 0.3rem;
  align-items: center;
  color: var(--muted);
}
.status {
  margin-left: auto;
}
</style>
