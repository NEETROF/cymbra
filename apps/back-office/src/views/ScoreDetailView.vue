<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import ScorePreview from "@/components/ScorePreview.vue";
import ScoreEditDrawer from "@/components/ScoreEditDrawer.vue";
import SoundFontPicker from "@/components/SoundFontPicker.vue";
import { useCatalogStore, type MetadataEdit, type ModerationStatus } from "@/stores/catalog";
import { useAuthStore } from "@/stores/auth";
import { useToastsStore } from "@/stores/toasts";
import { type Async, idle, run, success } from "@/lib/async";
import { useScoreSample } from "@/composables/useScoreSample";
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
// Drum Play guard (change: add-drums-access): `audio-wasm` renders through the
// hardcoded piano channel, so a percussion score is never fed to the player — the
// transport shows a localised "not auditionable yet" note while the notation now
// renders normally (add-drum-notation-render lifted the preview guard). The recorded
// instrument gates immediately; the render result's `percussion` flag covers a drum
// score still recorded `unknown`. `add-drum-audio-channel` lifts this guard.
const isPercussion = computed(
  () =>
    hitVm.value.hit?.instrument === "percussion" ||
    (notation.value.status === "success" && notation.value.data.percussion),
);
const playerBytes = computed(() => (isPercussion.value ? null : scoreBytes.value));
// Preview instrument sound: default piano, or a catalog font the moderator picks.
const { fonts, selectedId, sf2Bytes, loading: soundLoading, error: soundError } = useSoundFontChoice();
// Audio playback + playhead clock (Play/Pause only), reusing the app's synth/schedule.
const player = useScorePlayer(playerBytes, sf2Bytes);
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

// The moderator's rejection motive, recorded on the score and surfaced back to a
// user-proposer (change: add-score-catalog-proposal). Asked for at the moment of
// rejecting rather than parked in the action row, so accepting stays one click and a
// refusal is never sent without the moderator being given the chance to say why.
const rejecting = ref(false);
const rejectReason = ref("");
const reasonInput = ref<HTMLInputElement | null>(null);

async function askReject() {
  rejectReason.value = "";
  rejecting.value = true;
  await nextTick();
  reasonInput.value?.focus();
}

async function decide(status: ModerationStatus) {
  // The motive belongs to a rejection only: any other decision clears it server-side.
  const reason = status === "rejected" ? rejectReason.value.trim() || undefined : undefined;
  const outcome = await run(decision, () => store.setModerationStatus(props.id, status, reason));
  if (outcome.status === "success") await leave();
}

// Audio teaser (change: add-score-daily-access-rewards): the shared composable plays
// the clip the app auditions on a locked piece, or (re)generates it; here we only
// reflect a successful generation on this page's own hit.
const sample = useScoreSample();
const generating = computed(() => sample.generating(props.id));
async function generateSample() {
  const outcome = await sample.generate(props.id);
  if (outcome.status === "success" && hit.value.status === "success") {
    hit.value = success({ ...hit.value.data, hasPreview: true });
  }
}
const samplePlaying = computed(() => sample.playingId.value === props.id);
function toggleSample() {
  void sample.toggle(props.id);
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
      <button type="button" class="reject" :disabled="acting" @click="askReject()">
        {{ $t("detail.reject") }}
      </button>
      <button type="button" :disabled="acting" @click="decide('pending')">
        {{ $t("detail.requeue") }}
      </button>
      <!-- Audio teaser (change: add-score-daily-access-rewards): play it when it exists,
           else generate it (the recovery/backfill path for a piece without one). -->
      <button
        v-if="hitVm.hit?.hasPreview"
        type="button"
        class="sample"
        data-testid="play-sample"
        @click="toggleSample()"
      >
        {{ samplePlaying ? $t("detail.stopSample") : $t("detail.playSample") }}
      </button>
      <button
        v-else-if="hitVm.hit"
        type="button"
        class="sample"
        data-testid="generate-sample"
        :disabled="sample.busy.value"
        :title="$t('detail.generateSampleHint')"
        @click="generateSample()"
      >
        {{ generating ? "…" : $t("detail.generateSample") }}
      </button>
    </div>
  </div>
  <!-- Rejecting asks for the motive first: it is stored on the score and shown to the
       proposer in the app, so they know what to fix (change: add-score-catalog-proposal). -->
  <form v-if="rejecting" class="reject-row" @submit.prevent="decide('rejected')">
    <input
      ref="reasonInput"
      v-model="rejectReason"
      class="reason"
      type="text"
      :aria-label="$t('detail.rejectReason')"
      :placeholder="$t('detail.rejectReason')"
      :disabled="acting"
    />
    <button type="submit" class="reject" :disabled="acting">{{ $t("detail.rejectConfirm") }}</button>
    <button type="button" :disabled="acting" @click="rejecting = false">{{ $t("detail.cancel") }}</button>
  </form>
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
  <!-- Why it is back: the proposer's mandatory justification when they reopened a
       previously rejected score. Absent on a first proposal and on corpus rows. -->
  <p v-if="hitVm.hit?.resubmissionNote" class="resub">
    {{ $t("detail.resubmission", { note: hitVm.hit.resubmissionNote }) }}
  </p>
  <!-- The motive of the CURRENT rejection, so a moderator reopening the score's page
       sees the decision that was recorded. -->
  <p v-if="hitVm.hit?.reviewReason" class="muted">
    {{ $t("detail.reviewReason", { reason: hitVm.hit.reviewReason }) }}
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
      :percussion-guard="isPercussion"
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
.reject-row {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-bottom: 0.75rem;
}
.reject-row .reason {
  flex: 1;
  min-width: 0;
}
.resub {
  color: var(--muted);
  font-style: italic;
}
</style>
