<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import ScorePreview from "@/components/ScorePreview.vue";
import { useCatalogStore, type ModerationStatus } from "@/stores/catalog";
import { useAuthStore } from "@/stores/auth";
import { type Async, idle, run } from "@/lib/async";
import { useScoreRenderer } from "@/composables/useScoreRenderer";
import { useScorePlayer } from "@/composables/useScorePlayer";
import type { CatalogHit } from "@/gen/score_pb";

const props = defineProps<{ id: string }>();
const store = useCatalogStore();
const auth = useAuthStore();
const router = useRouter();

// The detail view is self-sufficient: it fetches the score's metadata AND bytes by
// id on mount, so it works on refresh / deep-link (not dependent on a prior list).
const hit = ref<Async<CatalogHit>>(idle);
const bytes = ref<Async<Uint8Array>>(idle);
const decision = ref<Async<void>>(idle);

const hitVm = computed(() =>
  match(hit.value)
    .with({ status: "success" }, ({ data }) => ({
      loading: false,
      hit: data as CatalogHit | null,
      error: null as string | null,
    }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, hit: null, error }))
    .otherwise(() => ({ loading: true, hit: null, error: null as string | null })),
);
// Bytes are separate from metadata: a fetch failure (e.g. the corpus bytes aren't
// synced to the serving store yet) must NOT block metadata or accept/reject — it's
// shown as an informational note inside the preview, not a page-level error.
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
// Render the notation from the fetched bytes via the isolated wasm seam. Reacts to
// the bytes Async; failures degrade to a placeholder inside the preview.
const scoreBytes = computed(() => bytesVm.value.bytes);
const { notation } = useScoreRenderer(scoreBytes);
// Audio playback + playhead clock (Play/Pause only), reusing the app's synth/schedule.
const player = useScorePlayer(scoreBytes);
const acting = computed(() => decision.value.status === "loading");
const decisionError = computed(() =>
  match(decision.value)
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() => null),
);

onMounted(() => {
  run(hit, () => store.fetchHit(props.id));
  run(bytes, () => store.fetchBytes(props.id));
});

async function decide(status: ModerationStatus) {
  const outcome = await run(decision, () => store.setModerationStatus(props.id, status));
  if (outcome.status === "success") await router.push({ name: "music-queue" });
}
</script>

<template>
  <div class="head">
    <button type="button" @click="router.back()">{{ $t("common.back") }}</button>
    <div v-if="auth.isModerator" class="actions">
      <button type="button" class="accept" :disabled="acting" @click="decide('accepted')">
        {{ $t("detail.accept") }}
      </button>
      <button type="button" class="reject" :disabled="acting" @click="decide('rejected')">
        {{ $t("detail.reject") }}
      </button>
      <button type="button" :disabled="acting" @click="decide('pending')">
        {{ $t("detail.requeue") }}
      </button>
    </div>
  </div>
  <h1 class="page-title detail-title">{{ hitVm.hit?.title || $t("detail.score") }}</h1>
  <p v-if="hitVm.error" class="error" role="alert">{{ hitVm.error }}</p>
  <p v-if="decisionError" class="error" role="alert">{{ decisionError }}</p>
  <div class="preview-card">
    <ScorePreview
      :hit="hitVm.hit"
      :bytes="bytesVm.bytes"
      :loading="bytesVm.loading || hitVm.loading"
      :bytes-error="bytesVm.error"
      :notation="notation"
      :schedule="player.schedule.value"
      :audio="player.audio.value"
      :elapsed-ms="player.elapsedMs.value"
      :playing="player.playing.value"
      :can-play="player.canPlay.value"
      @toggle="player.toggle"
    />
  </div>
</template>

<style scoped>
.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}
.actions button {
  margin-left: 0.4rem;
}
.detail-title {
  font-size: 1.6rem;
  margin-bottom: 1rem;
}
.preview-card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 1.5rem;
}
.error {
  color: var(--reject);
}
</style>
