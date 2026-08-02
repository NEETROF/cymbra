<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { SoundFontOption } from "@/composables/useSoundFontChoice";

// Presentational picker for the preview's instrument sound (change:
// add-soundfont-back-office-management). It only renders the catalog options and emits
// the selection — the fetch + player wiring live in useSoundFontChoice/useScorePlayer.
// Hidden when the catalog is empty (nothing to choose but the default).
defineProps<{
  fonts: SoundFontOption[];
  modelValue: string;
  loading?: boolean;
  error?: string | null;
}>();
const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();
const { t } = useI18n();
</script>

<template>
  <div v-if="fonts.length" class="sf-picker">
    <label>
      <span>{{ t("preview.sound") }}</span>
      <select
        :value="modelValue"
        :aria-label="t('preview.sound')"
        :disabled="loading"
        @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
      >
        <option v-for="f in fonts" :key="f.id" :value="f.id">{{ f.label }}</option>
      </select>
    </label>
    <output v-if="loading" class="spinner" :aria-label="t('preview.soundLoading')"></output>
    <span v-if="error" class="err">{{ t("preview.soundError") }}</span>
  </div>
</template>

<style scoped>
.sf-picker {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}
.sf-picker label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
  color: var(--muted);
}
.sf-picker select {
  padding: 0.3rem 0.4rem;
}
.err {
  color: var(--coral, #e06c75);
  font-size: 0.85rem;
}
.spinner {
  width: 1rem;
  height: 1rem;
  border: 2px solid var(--border-2, #444);
  border-top-color: var(--teal, #3ba);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
