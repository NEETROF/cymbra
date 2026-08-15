// The catalog audio teaser controls shared by the catalog table and the score
// detail (change: add-score-daily-access-rewards): audition the SAME server-rendered
// clip the app plays on a locked piece (no MusicXML, no wasm synth), and (re)generate
// it through the admin route. One clip sounds at a time; the generate state is the
// store's `preview` Async union, matched exhaustively.

import { computed, onBeforeUnmount, ref } from "vue";
import { match } from "ts-pattern";
import { useI18n } from "vue-i18n";
import { useCatalogStore } from "@/stores/catalog";
import { useToastsStore } from "@/stores/toasts";

export function useScoreSample() {
  const store = useCatalogStore();
  const toasts = useToastsStore();
  const { t } = useI18n();

  // --- audition -----------------------------------------------------------
  const playingId = ref<string | null>(null);
  let audio: HTMLAudioElement | null = null;
  let url: string | null = null;

  function stop() {
    audio?.pause();
    if (url) {
      URL.revokeObjectURL(url);
      url = null;
    }
    audio = null;
    playingId.value = null;
  }

  /** Play [id]'s teaser (stopping any other), or stop it if it is the one sounding. */
  async function toggle(id: string) {
    if (playingId.value === id) {
      stop();
      return;
    }
    stop();
    let clip: Uint8Array;
    try {
      clip = await store.scorePreviewClip(id);
    } catch {
      // The clip vanished (race with a regenerate) — nothing to play.
      return;
    }
    url = URL.createObjectURL(new Blob([clip as BlobPart], { type: "audio/wav" }));
    audio = new Audio(url);
    audio.addEventListener("ended", stop);
    playingId.value = id;
    void audio.play();
  }
  onBeforeUnmount(stop);

  // --- generate -----------------------------------------------------------
  const previewVm = computed(() =>
    match(store.preview)
      .with({ status: "idle" }, () => ({ busy: false, error: null as string | null, done: false }))
      .with({ status: "loading" }, () => ({ busy: true, error: null, done: false }))
      .with({ status: "error" }, ({ error }) => ({ busy: false, error, done: false }))
      .with({ status: "success" }, () => ({ busy: false, error: null, done: true }))
      .exhaustive(),
  );
  const busy = computed(() => previewVm.value.busy);
  /** Whether [id] is the piece currently (re)generating its sample. */
  function generating(id: string): boolean {
    return store.previewTarget === id && previewVm.value.busy;
  }
  /** (Re)generate [id]'s sample; toasts the outcome. Returns the store outcome so a
   * caller can reflect the flipped `hasPreview` on its own state. */
  async function generate(id: string) {
    const outcome = await store.regenerateScorePreview(id);
    if (outcome.status === "error") toasts.error(outcome.error);
    else toasts.success(t("detail.sampleGenerated"));
    return outcome;
  }

  return { playingId, toggle, stop, busy, generating, generate };
}
