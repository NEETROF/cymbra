<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import AppTag from "@/components/AppTag.vue";
import ScorePreview from "@/components/ScorePreview.vue";
import ScoreEditDrawer from "@/components/ScoreEditDrawer.vue";
import SoundFontPicker from "@/components/SoundFontPicker.vue";
import { useReviewSession } from "@/composables/useReviewSession";
import { useScoreRenderer } from "@/composables/useScoreRenderer";
import { useScorePlayer } from "@/composables/useScorePlayer";
import { useSoundFontChoice } from "@/composables/useSoundFontChoice";
import { type Async, failure, idle, loading, run, success } from "@/lib/async";
import { humanError } from "@/lib/errors";
import { useCatalogStore, type MetadataEdit } from "@/stores/catalog";
import { useToastsStore } from "@/stores/toasts";
import type { ModerationStatus } from "@/stores/catalog";
import type { CatalogHit } from "@/gen/score_pb";

// Burn-down review: one score at a time, auto-advancing after each decision until the
// queue is empty. Keyboard-driven for speed. Reuses the preview + playback stack.
const router = useRouter();
const session = useReviewSession();
const store = useCatalogStore();
const toasts = useToastsStore();
const { t } = useI18n();

// Bytes for the current score (prefetched by the session), fed to the renderer + player.
const bytes = ref<Async<Uint8Array>>(idle);
const bytesData = computed(() => (bytes.value.status === "success" ? bytes.value.data : null));
watch(
  () => session.current.value?.id,
  async (id) => {
    if (!id) {
      bytes.value = idle;
      return;
    }
    // Guarded by hand instead of `run()`: two quick decisions overlap two fetches, and a
    // slower earlier one must never land on a newer score — the notation and the audio
    // both read these bytes, so a stale winner would show/play the previous piece.
    bytes.value = loading;
    try {
      const data = await session.bytesFor(id);
      if (session.current.value?.id === id) bytes.value = success(data);
    } catch (e) {
      if (session.current.value?.id === id) bytes.value = failure(humanError(e));
    }
  },
  { immediate: true },
);
const bytesVm = computed(() =>
  match(bytes.value)
    .with({ status: "success" }, ({ data }) => ({
      loading: false,
      bytes: data as Uint8Array | null,
      error: null as string | null,
    }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, bytes: null, error }))
    .otherwise(() => ({ loading: true, bytes: null, error: null as string | null })),
);

const { notation } = useScoreRenderer(bytesData);
// The score's instrument family (change: add-drum-audio-channel — the old Play
// guard is lifted: `audio-wasm` now resolves the drum channel from the document
// itself, so a percussion row auditions with a kit font). The recorded instrument
// answers immediately; the render result's `percussion` flag covers a drum score
// still recorded `unknown`.
const isPercussion = computed(
  () =>
    session.current.value?.instrument === "percussion" ||
    (notation.value.status === "success" && notation.value.data.percussion),
);
const scoreFamily = computed<"keyboard" | "percussion">(() => (isPercussion.value ? "percussion" : "keyboard"));
// Preview instrument sound, filtered to the score's family: the default piano or a
// picked keyboard font, the picked kit (first accepted one by default) for drums.
const {
  fonts,
  selectedId,
  sf2Bytes,
  loading: soundLoading,
  error: soundError,
  familyEmpty: noKitFont,
} = useSoundFontChoice(scoreFamily);
// A percussion score's bytes reach the player only once the kit's bytes are in
// hand: null `sf2Bytes` means "fall back to the default piano" in useScorePlayer,
// which must never answer for a drum score. Gating the BYTES silences every path
// at once — no schedule, so canPlay stays false (disabled Play, inert spacebar)
// and auto-play waits for the kit (or stays off when the catalog has none).
const playerBytes = computed(() => (isPercussion.value && sf2Bytes.value == null ? null : bytesData.value));
const player = useScorePlayer(playerBytes, sf2Bytes);

// Hands-free review: auto-play each score once, as soon as it's playable (a decided
// score leaves and the next one starts on its own). Only once per score — pausing is
// respected and a finished score isn't restarted. Entering review mode / deciding are
// user gestures, so the browser's autoplay policy allows this.
let autoplayedFor: string | null = null;
watch(
  () => [session.current.value?.id, player.canPlay.value] as const,
  ([id, can]) => {
    if (id && can && !player.playing.value && autoplayedFor !== id) {
      autoplayedFor = id;
      player.toggle();
    }
  },
);

const acting = computed(() => session.deciding.value.status === "loading");
// Audio preparation feedback for the hoisted transport (the ~57 MB SoundFont download
// then the render): a spinner while it runs, a localized note if it fails.
const audioLoading = computed(() => player.audio.value.status === "loading");
const audioMsg = computed(() =>
  match(player.audio.value)
    .with({ status: "loading" }, () => t("preview.audioLoading"))
    .with({ status: "error" }, () => t("preview.audioError"))
    .otherwise(() => null),
);
// A failed decision surfaces as a toast (the queue load error stays inline).
watch(
  () => session.deciding.value,
  (d) => {
    if (d.status === "error") toasts.error(t("review.decideError"));
  },
);

// The moderator's rejection motive, surfaced back to a user-proposer (change:
// add-score-catalog-proposal). Only sent on a reject; cleared after each decision.
const rejectReason = ref("");

async function decide(status: ModerationStatus) {
  const reason = status === "rejected" ? rejectReason.value.trim() || undefined : undefined;
  await session.decide(status, reason);
  rejectReason.value = "";
}

// Curatorial editing without leaving the burn-down: the moderator fixes the title /
// composer / arranger / level of the score in front of them, then keeps chaining
// decisions. Its own Async state so a save never blocks accept/reject and vice-versa.
const submit = ref<Async<void>>(idle);
const submitting = computed(() => submit.value.status === "loading");
const submitError = computed(() =>
  match(submit.value)
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() => null),
);
const editing = ref(false);
function openEdit() {
  submit.value = idle;
  editing.value = true;
}
// The drawer and any save failure belong to the score they were opened on — drop both
// when the deck moves (a decision can land while the drawer is open).
watch(
  () => session.current.value?.id,
  () => {
    submit.value = idle;
    editing.value = false;
  },
);

async function saveEdit(edit: MetadataEdit) {
  const id = session.current.value?.id;
  if (!id) return;
  const outcome = await run(submit, () => store.updateCatalogScore(id, edit));
  // Keep the drawer open on failure so the moderator can fix and retry.
  if (outcome.status !== "success") return;
  editing.value = false;
  // Re-read the row (the server recomputes the derived keys) — but only if the deck
  // hasn't moved on under a slow save.
  if (session.current.value?.id === id) await session.refreshCurrent();
}

// Proposal attribution for the current score: whether it is a user proposal (vs a
// crawler-ingested corpus row), the proposer's pseudo, and any resubmission note — all
// from the privileged read the backend only returns to a moderator/admin.
const attribution = computed(() => {
  const h = session.current.value as CatalogHit | null;
  if (!h) return null;
  const isUserProposed = h.source === "user-proposal" || !!h.proposedBy;
  return {
    isUserProposed,
    proposer: h.proposerDisplayName || null,
    resubmission: h.resubmissionNote || null,
  };
});

function onKey(e: KeyboardEvent) {
  // Never steal a keystroke aimed at a form control — the reject reason and the edit
  // drawer's fields live on this screen, and "a"/"r"/"p" would otherwise decide the
  // score while the moderator is typing or picking a level.
  const el = e.target;
  if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement || el instanceof HTMLSelectElement) return;
  // While the drawer is open it owns the keyboard: Escape closes it (instead of
  // leaving review mode) and nothing else decides behind it.
  if (editing.value) {
    if (e.key === "Escape") editing.value = false;
    return;
  }
  switch (e.key.toLowerCase()) {
    case "a":
      void decide("accepted");
      break;
    case "r":
      void decide("rejected");
      break;
    case "p":
      void decide("pending");
      break;
    case "s":
    case "arrowright":
      void session.skip();
      break;
    case "e":
      openEdit();
      break;
    case " ":
      e.preventDefault();
      if (player.canPlay.value) player.toggle();
      break;
    case "escape":
      void router.push({ name: "music-queue" });
      break;
  }
}

onMounted(() => {
  session.start();
  window.addEventListener("keydown", onKey);
});
onUnmounted(() => window.removeEventListener("keydown", onKey));

const currentHit = computed(() => session.current.value as CatalogHit | null);
</script>

<template>
  <div class="page-head">
    <div>
      <h1 class="page-title">{{ $t("review.title") }}</h1>
      <p class="sub">
        {{ $t("review.progress", { reviewed: session.reviewedCount.value }) }}
        <span v-if="session.remaining.value > 0"> · {{ $t("review.remaining", { n: session.remaining.value }) }}</span>
      </p>
    </div>
    <button type="button" @click="router.push({ name: 'music-queue' })">{{ $t("review.back") }}</button>
  </div>

  <!-- Empty / done -->
  <div v-if="session.done.value" class="review-empty">
    <p>{{ $t("review.done") }}</p>
    <button type="button" class="btn-primary" @click="router.push({ name: 'music-queue' })">
      {{ $t("review.back") }}
    </button>
  </div>

  <!-- Loading the first batch -->
  <p v-else-if="session.loadState.value.status === 'loading' && !currentHit" class="muted">
    {{ $t("review.loading") }}
  </p>
  <p v-else-if="session.loadState.value.status === 'error'" class="error" role="alert">
    {{ $t("review.loadError") }}
  </p>

  <!-- The current score -->
  <template v-else-if="currentHit">
    <div class="actions">
      <!-- Listen, then decide: the transport sits in the same row as the decisions
           (hidden inside the preview) so the moderator's hands never leave it. -->
      <button type="button" class="play" :disabled="!player.canPlay.value" @click="player.toggle()">
        {{ player.playing.value ? $t("preview.pause") : $t("preview.play") }}
      </button>
      <output v-if="audioLoading" class="spinner" :aria-label="audioMsg ?? ''"></output>
      <span v-if="audioMsg" class="muted audio-msg">{{ audioMsg }}</span>
      <!-- Quiet no-kit note (change: add-drum-audio-channel): the catalog holds no
           accepted percussion-family font, so Play stays disabled — deliberately
           NOT styled as an error; the score itself is fine. -->
      <span v-if="noKitFont" class="muted audio-msg" data-testid="no-drum-kit">{{ t("preview.noDrumKit") }}</span>
      <button type="button" class="accept" :disabled="acting" @click="decide('accepted')">
        {{ $t("review.accept") }}
      </button>
      <button type="button" class="reject" :disabled="acting" @click="decide('rejected')">
        {{ $t("review.reject") }}
      </button>
      <input
        id="review-reject-reason"
        v-model="rejectReason"
        class="reason"
        type="text"
        :aria-label="$t('review.rejectReasonPlaceholder')"
        :placeholder="$t('review.rejectReasonPlaceholder')"
        :disabled="acting"
      />
      <button type="button" :disabled="acting" @click="decide('pending')">{{ $t("review.requeue") }}</button>
      <button type="button" :disabled="acting" @click="session.skip()">{{ $t("review.skip") }}</button>
      <button type="button" class="edit" @click="openEdit()">{{ $t("review.edit") }}</button>
      <span class="muted keys">{{ $t("review.keys") }}</span>
    </div>
    <!-- The instrument sound belongs to the transport: right under the Play button. -->
    <SoundFontPicker
      v-model="selectedId"
      class="sound-row"
      :fonts="fonts"
      :loading="soundLoading"
      :error="soundError"
    />
    <h2 class="score-title">
      {{ currentHit.title || $t("detail.score") }}
      <!-- Instrument badge (change: add-drums-access): a percussion proposal is
           identified before the row is opened — instrument information a
           moderator uses up front, kept now that Play auditions drums too. -->
      <AppTag v-if="currentHit.instrument === 'percussion'" variant="accent" :title="$t('table.percussionHint')">{{
        $t("table.percussion")
      }}</AppTag>
    </h2>
    <!-- Proposal attribution (moderator/admin-only privileged fields). -->
    <div v-if="attribution?.isUserProposed" class="attribution">
      <span class="badge">{{ $t("review.userProposed") }}</span>
      <span v-if="attribution.proposer" class="muted">
        · {{ $t("review.proposedBy", { name: attribution.proposer }) }}
      </span>
      <p v-if="attribution.resubmission" class="resub">
        {{ $t("review.resubmission", { note: attribution.resubmission }) }}
      </p>
    </div>
    <!-- Curatorial metadata: the read-only summary stays in the preview below, and the
         drawer edits it without a round-trip to the detail page. -->
    <ScoreEditDrawer
      :open="editing"
      :hit="currentHit"
      :submitting="submitting"
      :error="submitError"
      @submit="saveEdit"
      @close="editing = false"
    />
    <div class="preview-card">
      <ScorePreview
        :hit="currentHit"
        :bytes="bytesVm.bytes"
        :loading="bytesVm.loading"
        :bytes-error="bytesVm.error"
        :notation="notation"
        show-meta
        hide-transport
        :no-kit-font="noKitFont"
        :schedule="player.schedule.value"
        :audio="player.audio.value"
        :elapsed-ms="player.elapsedMs.value"
        :playing="player.playing.value"
        :can-play="player.canPlay.value"
        @toggle="player.toggle"
        @seek="player.playFrom"
      />
    </div>
  </template>
</template>

<style scoped>
.actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
  margin-bottom: 0.75rem;
}
.keys {
  margin-left: auto;
  font-size: 0.8rem;
}
.actions .play {
  min-width: 5rem;
}
.audio-msg {
  font-size: 0.85rem;
}
/* Small spinner while the first-play data (SoundFont) downloads. */
.spinner {
  width: 1rem;
  height: 1rem;
  border: 2px solid var(--border-2);
  border-top-color: var(--teal);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
.score-title {
  font-size: 1.4rem;
  margin: 0 0 0.25rem;
}
.reason {
  /* Sizing only: the fill/border come from the global input style, so the field
     reads as active next to the decision buttons instead of looking disabled. */
  flex: 1 1 12rem;
  min-width: 8rem;
  padding: 0.5rem 0.7rem;
}
.attribution {
  margin: 0 0 0.75rem;
  font-size: 0.9rem;
}
.attribution .badge {
  display: inline-block;
  padding: 0.1rem 0.5rem;
  border-radius: 8px;
  background: color-mix(in srgb, var(--accent, #6c8) 20%, transparent);
  font-weight: 600;
  font-size: 0.8rem;
}
.attribution .resub {
  margin: 0.35rem 0 0;
  color: var(--muted);
}
.preview-card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 1.5rem;
}
.sound-row {
  /* Sits just under the transport, aligned with the Play button. */
  margin: -0.25rem 0 0.85rem;
}
.review-empty {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 1rem;
  padding: 2rem 0;
}
.error {
  color: var(--coral);
}
.muted {
  color: var(--muted);
}
</style>
