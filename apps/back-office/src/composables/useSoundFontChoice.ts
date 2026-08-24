// Lets a moderator pick which instrument SoundFont the preview plays with, on the
// review and detail screens (change: add-soundfont-back-office-management). It lists the
// public catalog filtered to the SCORE's family (change: add-drum-audio-channel): a
// percussion row offers only kits, a keyboard row only keyboard fonts — neither family
// is ever auditioned through the other's font. The keyboard default keeps using the
// lazy `loadSoundFont` path (no eager download) — its `sf2Bytes` stays null; every
// other pick fetches bytes, INCLUDING a percussion default: null bytes mean "fall back
// to the default piano" in `useScorePlayer`, which must never answer for a drum score.
// The `sf2Bytes` ref is handed to `useScorePlayer` (null → default piano fallback).

import { computed, onMounted, ref, shallowRef, watch, type Ref } from "vue";
import { useSoundFontsStore } from "@/stores/soundfonts";
import { DEFAULT_SOUNDFONT_ID } from "@/lib/audio/soundfont";
import { familyOf, type SoundFontFamily } from "@/lib/audio/family";

export interface SoundFontOption {
  id: string;
  label: string;
  family: SoundFontFamily;
}

export function useSoundFontChoice(scoreFamily: Ref<SoundFontFamily>) {
  const store = useSoundFontsStore();
  // The whole accepted catalog, family carried from the wire listing.
  const catalog = ref<SoundFontOption[]>([]);
  // Whether the one-shot catalog load has settled (success or failure) — the
  // "no kit available" state must not flash while the listing is in flight.
  const settled = ref(false);
  // Pre-select the default piano; picking another catalog font swaps the sound.
  const selectedId = ref(DEFAULT_SOUNDFONT_ID);
  const sf2Bytes = shallowRef<Uint8Array | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  // Per-family remembered pick, so review mode alternating drum and keyboard
  // rows restores each family's last choice instead of resetting it.
  const picked: Record<SoundFontFamily, string | null> = {
    keyboard: DEFAULT_SOUNDFONT_ID,
    percussion: null,
  };

  // What the picker offers: the score's family only.
  const fonts = computed(() => catalog.value.filter((f) => f.family === scoreFamily.value));

  // The localised "no drum kit available" state (distinct from an error — the
  // score is fine): the catalog holds no accepted font of the score's family.
  // Only percussion can be empty in practice — the keyboard default is served
  // lazily even when the listing fails — but the check is family-symmetric.
  const familyEmpty = computed(() => settled.value && fonts.value.length === 0 && scoreFamily.value === "percussion");

  /** The family's default pick: the lazy default piano for keyboard, the first
   *  accepted kit for percussion (the seeded kit once it exists — the catalog
   *  has no stable kit id to prefer yet), or null while the family is empty. */
  function defaultFor(family: SoundFontFamily): string | null {
    if (family === "keyboard") return DEFAULT_SOUNDFONT_ID;
    return catalog.value.find((f) => f.family === "percussion")?.id ?? null;
  }

  /** Re-point the selection at the current family: the remembered pick when it
   *  is still offered, else the family default ("" when the family is empty —
   *  no bytes are fetched and playback stays gated). */
  function applyFamily() {
    const family = scoreFamily.value;
    const remembered = picked[family];
    const target = remembered && fonts.value.some((f) => f.id === remembered) ? remembered : defaultFor(family);
    selectedId.value = target ?? "";
  }

  // Load the catalog once. A failure just leaves the picker empty (it hides itself).
  onMounted(async () => {
    try {
      catalog.value = (await store.publicList()).map((f) => ({
        id: f.id,
        label: f.label,
        family: familyOf(f.instrument),
      }));
    } catch {
      catalog.value = [];
    }
    settled.value = true;
    applyFamily();
  });

  // The score's family changed (review mode advanced to the other instrument):
  // the selection follows it, so a kit is never left active for a keyboard row.
  watch(scoreFamily, applyFamily);

  // Fetch the chosen font's bytes lazily; only the default piano keeps the lazy
  // loader (null bytes → useScorePlayer fetches the app default), so no eager
  // download for it. A kit pick always fetches — see the header comment.
  watch(selectedId, async (id) => {
    if (id) picked[scoreFamily.value] = id;
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

  return { fonts, selectedId, sf2Bytes, loading, error, familyEmpty };
}
