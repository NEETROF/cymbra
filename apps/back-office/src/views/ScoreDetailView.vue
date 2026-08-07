<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import ScorePreview from "@/components/ScorePreview.vue";
import ScoreEditDrawer from "@/components/ScoreEditDrawer.vue";
import SoundFontPicker from "@/components/SoundFontPicker.vue";
import { useCatalogStore, type MetadataEdit, type ModerationStatus } from "@/stores/catalog";
import { useAuthStore } from "@/stores/auth";
import { useToastsStore } from "@/stores/toasts";
import { type Async, idle, run } from "@/lib/async";
import { useScoreRenderer } from "@/composables/useScoreRenderer";
import { useScorePlayer } from "@/composables/useScorePlayer";
import { useSoundFontChoice } from "@/composables/useSoundFontChoice";
import type { CatalogHit } from "@/gen/score_pb";

const props = defineProps<{ id: string }>();
const store = useCatalogStore();
const auth = useAuthStore();
const toasts = useToastsStore();
const router = useRouter();

// The detail view is self-sufficient: it fetches the score's metadata AND bytes by
// id on mount, so it works on refresh / deep-link (not dependent on a prior list).
const hit = ref<Async<CatalogHit>>(idle);
const bytes = ref<Async<Uint8Array>>(idle);
const decision = ref<Async<void>>(idle);
// The curatorial-edit submit state (moderator-only form). Separate from `decision`
// so a save never blocks accept/reject and vice-versa.
const submit = ref<Async<void>>(idle);

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
// Preview instrument sound: default piano, or a catalog font the moderator picks.
const { fonts, selectedId, sf2Bytes, loading: soundLoading, error: soundError } = useSoundFontChoice();
// Audio playback + playhead clock (Play/Pause only), reusing the app's synth/schedule.
const player = useScorePlayer(scoreBytes, sf2Bytes);
const acting = computed(() => decision.value.status === "loading");
// A failed moderation decision (accept/reject/re-queue) surfaces as a toast; the
// score LOAD error stays inline, and the edit-form error stays in the form.
watch(
  () => decision.value,
  (d) => {
    if (d.status === "error") toasts.error(d.error);
  },
);

onMounted(() => {
  run(hit, () => store.fetchHit(props.id));
  run(bytes, () => store.fetchBytes(props.id));
});

// Return to wherever the operator opened this score from (catalog or queue) so their
// filters, sort and page survive the round-trip — falling back to the queue only on a
// direct deep-link, where there is no in-app history entry to go back to.
async function leave() {
  if (router.options.history.state.back) router.back();
  else await router.push({ name: "music-queue" });
}

async function decide(status: ModerationStatus) {
  const outcome = await run(decision, () => store.setModerationStatus(props.id, status));
  if (outcome.status === "success") await leave();
}

const submitting = computed(() => submit.value.status === "loading");
const submitError = computed(() =>
  match(submit.value)
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() => null),
);

// The edit drawer's open state. Opening resets any error from a previous attempt so
// the form doesn't reopen on a stale alert.
const editing = ref(false);
function openEdit() {
  submit.value = idle;
  editing.value = true;
}

// Save a curatorial edit, then re-fetch the hit so the summary reflects the recomputed
// metadata (the server also recomputes the derived search keys + audits). The drawer
// stays open on failure, with the error inside it.
async function saveEdit(edit: MetadataEdit) {
  const outcome = await run(submit, () => store.updateCatalogScore(props.id, edit));
  if (outcome.status !== "success") return;
  editing.value = false;
  await run(hit, () => store.fetchHit(props.id));
}
</script>

<template>
  <div class="head">
    <button type="button" @click="leave()">{{ $t("common.back") }}</button>
    <div v-if="auth.isModerator" class="actions">
      <button type="button" class="edit" @click="openEdit()">{{ $t("edit.open") }}</button>
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
  <!-- Origin: the score's source, plus the proposer's pseudo for a user upload
       (privileged read — change: add-score-catalog-proposal). -->
  <p v-if="hitVm.hit" class="origin">
    {{ $t("detail.source") }}: <strong>{{ hitVm.hit.source }}</strong>
    <span v-if="hitVm.hit.proposerDisplayName" class="muted">
      · {{ $t("detail.proposedBy", { name: hitVm.hit.proposerDisplayName }) }}
    </span>
  </p>
  <!-- The metadata surface is always the read-only summary (inside the preview); a
       moderator edits it through the drawer opened from the action row. -->
  <ScoreEditDrawer
    :open="editing"
    :hit="hitVm.hit"
    :submitting="submitting"
    :error="submitError"
    @submit="saveEdit"
    @close="editing = false"
  />
  <div class="preview-card">
    <SoundFontPicker
      v-model="selectedId"
      class="sound-row"
      :fonts="fonts"
      :loading="soundLoading"
      :error="soundError"
    />
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
      show-meta
      @toggle="player.toggle"
      @seek="player.playFrom"
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
.sound-row {
  margin-bottom: 1rem;
}
.error {
  color: var(--reject);
}
</style>
