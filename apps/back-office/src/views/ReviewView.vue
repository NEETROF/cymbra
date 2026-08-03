<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import ScorePreview from "@/components/ScorePreview.vue";
import SoundFontPicker from "@/components/SoundFontPicker.vue";
import { useReviewSession } from "@/composables/useReviewSession";
import { useScoreRenderer } from "@/composables/useScoreRenderer";
import { useScorePlayer } from "@/composables/useScorePlayer";
import { useSoundFontChoice } from "@/composables/useSoundFontChoice";
import { type Async, idle, run } from "@/lib/async";
import type { ModerationStatus } from "@/stores/catalog";
import type { CatalogHit } from "@/gen/score_pb";

// Burn-down review: one score at a time, auto-advancing after each decision until the
// queue is empty. Keyboard-driven for speed. Reuses the preview + playback stack.
const router = useRouter();
const session = useReviewSession();

// Bytes for the current score (prefetched by the session), fed to the renderer + player.
const bytes = ref<Async<Uint8Array>>(idle);
const bytesData = computed(() => (bytes.value.status === "success" ? bytes.value.data : null));
watch(
  () => session.current.value?.id,
  (id) => {
    if (!id) {
      bytes.value = idle;
      return;
    }
    run(bytes, () => session.bytesFor(id));
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
// Preview instrument sound: default piano, or a catalog font the moderator picks.
const { fonts, selectedId, sf2Bytes, loading: soundLoading, error: soundError } = useSoundFontChoice();
const player = useScorePlayer(bytesData, sf2Bytes);

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
const decideError = computed(() => session.deciding.value.status === "error");

// The moderator's rejection motive, surfaced back to a user-proposer (change:
// add-score-catalog-proposal). Only sent on a reject; cleared after each decision.
const rejectReason = ref("");

async function decide(status: ModerationStatus) {
  const reason = status === "rejected" ? rejectReason.value.trim() || undefined : undefined;
  await session.decide(status, reason);
  rejectReason.value = "";
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
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
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

  <p v-if="decideError" class="error" role="alert">{{ $t("review.decideError") }}</p>

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
      <button type="button" class="accept" :disabled="acting" @click="decide('accepted')">
        {{ $t("review.accept") }}
      </button>
      <button type="button" class="reject" :disabled="acting" @click="decide('rejected')">
        {{ $t("review.reject") }}
      </button>
      <input
        v-model="rejectReason"
        class="reason"
        type="text"
        :placeholder="$t('review.rejectReasonPlaceholder')"
        :disabled="acting"
      />
      <button type="button" :disabled="acting" @click="decide('pending')">{{ $t("review.requeue") }}</button>
      <button type="button" :disabled="acting" @click="session.skip()">{{ $t("review.skip") }}</button>
      <span class="muted keys">{{ $t("review.keys") }}</span>
    </div>
    <h2 class="score-title">{{ currentHit.title || $t("detail.score") }}</h2>
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
    <div class="preview-card">
      <SoundFontPicker
        v-model="selectedId"
        class="sound-row"
        :fonts="fonts"
        :loading="soundLoading"
        :error="soundError"
      />
      <ScorePreview
        :hit="currentHit"
        :bytes="bytesVm.bytes"
        :loading="bytesVm.loading"
        :bytes-error="bytesVm.error"
        :notation="notation"
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
.score-title {
  font-size: 1.4rem;
  margin: 0 0 0.25rem;
}
.reason {
  flex: 1 1 12rem;
  min-width: 8rem;
  padding: 0.4rem 0.6rem;
  border: 1px solid var(--border);
  border-radius: var(--radius-md, 6px);
  background: var(--panel);
  color: inherit;
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
  margin-bottom: 1rem;
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
