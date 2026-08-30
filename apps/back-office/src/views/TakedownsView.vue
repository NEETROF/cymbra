<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { useTakedownsStore } from "@/stores/takedowns";
import type { AdminUserScore } from "@/gen/score_pb";

// Private-score takedown (change: add-private-score-catalog). The view only
// renders and collects intent: the lookup and the removal live in the store,
// behind the injectable client seam. Removal is irreversible, so it is gated on
// an explicit confirmation that names the consequence AND a non-empty reason.

const { t } = useI18n();
const store = useTakedownsStore();

const ownerId = ref("");
const title = ref("");
const target = ref<AdminUserScore | null>(null);
const reason = ref("");

/** The reason field, focused when the dialog opens. Focus has to move INTO the
 *  dialog: it puts the cursor on the one thing the operator must fill, and it is
 *  what makes Escape reach the dialog's own handler (a keydown on the trigger
 *  button would never get there). */
const reasonInput = ref<HTMLInputElement | null>(null);

async function openConfirm(score: AdminUserScore) {
  target.value = score;
  reason.value = "";
  await nextTick();
  reasonInput.value?.focus();
}

async function confirmRemoval() {
  const score = target.value;
  if (!score || reason.value.trim() === "") return;
  await store.remove(score.id, reason.value);
  target.value = null;
}

/** A removal in flight: the dialog stays up but inert, so a second click cannot
 *  fire the same irreversible action twice. */
const acting = computed(() => store.op.status === "loading");

function formatDate(seconds: bigint): string {
  return new Date(Number(seconds) * 1000).toLocaleDateString();
}

// One union in, one view-model out — matched exhaustively, so a new Async state
// cannot silently render as "nothing".
const resultsVm = computed(() =>
  match(store.results)
    .with({ status: "idle" }, () => ({
      loading: false,
      error: null as string | null,
      rows: null as AdminUserScore[] | null,
    }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, rows: null as AdminUserScore[] | null }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, rows: null as AdminUserScore[] | null }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, rows: data }))
    .exhaustive(),
);

const opVm = computed(() =>
  match(store.op)
    .with({ status: "idle" }, () => ({ error: null as string | null, done: false }))
    .with({ status: "loading" }, () => ({ error: null, done: false }))
    .with({ status: "error" }, ({ error }) => ({ error, done: false }))
    .with({ status: "success" }, () => ({ error: null, done: true }))
    .exhaustive(),
);
</script>

<template>
  <section class="takedowns">
    <h1>{{ t("takedowns.title") }}</h1>
    <p class="intro">{{ t("takedowns.intro") }}</p>

    <form class="search" @submit.prevent="store.search({ ownerId, title })">
      <label>
        {{ t("takedowns.ownerId") }}
        <input v-model="ownerId" type="text" />
      </label>
      <label>
        {{ t("takedowns.titleFragment") }}
        <input v-model="title" type="text" />
      </label>
      <button type="submit" :disabled="!ownerId.trim() && !title.trim()">
        {{ t("takedowns.search") }}
      </button>
    </form>

    <p v-if="opVm.done" class="ok">{{ t("takedowns.removed") }}</p>
    <p v-else-if="opVm.error" class="error">{{ opVm.error }}</p>

    <template v-if="resultsVm.loading">
      <p>{{ t("common.loading") }}</p>
    </template>
    <template v-else-if="resultsVm.error">
      <p class="error">{{ resultsVm.error }}</p>
    </template>
    <template v-else-if="resultsVm.rows">
      <p v-if="resultsVm.rows.length === 0">{{ t("takedowns.noResults") }}</p>
      <table v-else>
        <thead>
          <tr>
            <th>{{ t("takedowns.colTitle") }}</th>
            <th>{{ t("takedowns.colComposer") }}</th>
            <th>{{ t("takedowns.colOwner") }}</th>
            <th>{{ t("takedowns.colCreated") }}</th>
            <th>{{ t("takedowns.colBasis") }}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="score in resultsVm.rows" :key="score.id">
            <td>{{ score.title ?? "—" }}</td>
            <td>{{ score.composer ?? "—" }}</td>
            <td class="mono">{{ score.ownerId }}</td>
            <td>{{ formatDate(score.createdAt) }}</td>
            <td>{{ score.rightsBasis }}</td>
            <td>
              <button type="button" class="danger" @click="openConfirm(score)">
                {{ t("takedowns.remove") }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </template>

    <!-- Irreversible action: the dialog states the consequence and the reason is
         mandatory (it is what the audit trail records). Asked in-app, never
         `window.confirm` — a native dialog blocks the renderer and is out of reach
         of the e2e suite. Same shape as the Plans console's reason modal. -->
    <div v-if="target" class="overlay" @click.self="target = null">
      <dialog class="modal" open aria-modal="true" @keydown.esc="target = null">
        <h2>{{ t("takedowns.confirmTitle") }}</h2>
        <p>{{ t("takedowns.confirmBody") }}</p>
        <label>
          {{ t("takedowns.reason") }}
          <input
            ref="reasonInput"
            v-model="reason"
            type="text"
            :aria-label="t('takedowns.reason')"
            :disabled="acting"
          />
        </label>
        <div class="modal-actions">
          <button type="button" class="btn-primary danger" :disabled="acting || !reason.trim()" @click="confirmRemoval">
            {{ t("takedowns.confirm") }}
          </button>
          <button type="button" :disabled="acting" @click="target = null">{{ t("common.cancel") }}</button>
        </div>
      </dialog>
    </div>
  </section>
</template>

<style scoped>
.takedowns {
  padding: 1rem;
}
.search {
  display: flex;
  gap: 0.75rem;
  align-items: end;
  margin-bottom: 1rem;
}
.mono {
  font-family: monospace;
  font-size: 0.85em;
}
.error {
  color: var(--color-error, #b3261e);
}
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 90;
  display: grid;
  place-items: center;
}
.modal {
  position: static;
  width: min(480px, 94vw);
  margin: 0;
  background: var(--panel, #1a1a24);
  color: inherit;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 1.35rem 1.4rem;
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}
.modal h2 {
  font-size: 1.05rem;
  margin: 0;
}
.modal p {
  margin: 0;
  font-size: 0.9rem;
}
.modal label {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  font-size: 0.85rem;
  color: var(--muted);
}
.modal input {
  width: 100%;
}
.modal-actions {
  display: flex;
  gap: 0.5rem;
  border-top: 1px solid var(--border);
  padding-top: 0.9rem;
}
</style>
