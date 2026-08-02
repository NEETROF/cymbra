// Lets a moderator pick which instrument SoundFont the preview plays with, on the
// review and detail screens (change: add-soundfont-back-office-management). It lists the
// public catalog with the app's default piano pre-selected. The default row keeps using
// the lazy `loadSoundFont` path (no eager download) — its `sf2Bytes` stays null; only a
// non-default pick fetches bytes. The `sf2Bytes` ref is handed to `useScorePlayer` (null
// → default piano fallback).

import { onMounted, ref, shallowRef, watch } from "vue";
import { useSoundFontsStore } from "@/stores/soundfonts";
import { DEFAULT_SOUNDFONT_ID } from "@/lib/audio/soundfont";

export interface SoundFontOption {
  id: string;
  label: string;
}

export function useSoundFontChoice() {
  const store = useSoundFontsStore();
  const fonts = ref<SoundFontOption[]>([]);
  // Pre-select the default piano; picking another catalog font swaps the sound.
  const selectedId = ref(DEFAULT_SOUNDFONT_ID);
  const sf2Bytes = shallowRef<Uint8Array | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Load the catalog once. A failure just leaves the picker empty (it hides itself).
  onMounted(async () => {
    try {
      fonts.value = (await store.publicList()).map((f) => ({ id: f.id, label: f.label }));
    } catch {
      fonts.value = [];
    }
  });

  // Fetch the chosen font's bytes lazily; the default piano keeps the lazy loader (null
  // bytes → useScorePlayer fetches the app default), so no eager download for it.
  watch(selectedId, async (id) => {
    error.value = null;
    if (!id || id === DEFAULT_SOUNDFONT_ID) {
      sf2Bytes.value = null;
      return;
    }
    loading.value = true;
    const started = id;
    try {
      const bytes = await store.fontBytes(id);
      if (selectedId.value === started) sf2Bytes.value = bytes;
    } catch {
      if (selectedId.value === started) {
        sf2Bytes.value = null;
        error.value = "load_failed";
      }
    } finally {
      if (selectedId.value === started) loading.value = false;
    }
  });

  return { fonts, selectedId, sf2Bytes, loading, error };
}
