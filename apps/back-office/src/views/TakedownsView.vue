<script setup lang="ts">
import { computed, ref } from "vue";
import { match } from "ts-pattern";
import { useTakedownsStore } from "@/stores/takedowns";
import { t } from "@/i18n";
import type { AdminUserScore } from "@/gen/score_pb";

// Private-score takedown (change: add-private-score-catalog). The view only
// renders and collects intent: the lookup and the removal live in the store,
// behind the injectable client seam. Removal is irreversible, so it is gated on
// an explicit confirmation that names the consequence AND a non-empty reason.

const store = useTakedownsStore();

const ownerId = ref("");
const title = ref("");
const target = ref<AdminUserScore | null>(null);
const reason = ref("");

function openConfirm(score: AdminUserScore) {
  target.value = score;
  reason.value = "";
}

async function confirmRemoval() {
  const score = target.value;
  if (!score || reason.value.trim() === "") return;
  await store.remove(score.id, reason.value);
  target.value = null;
}

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
         mandatory (it is what the audit trail records). -->
    <dialog v-if="target" open class="confirm">
      <h2>{{ t("takedowns.confirmTitle") }}</h2>
      <p>{{ t("takedowns.confirmBody") }}</p>
      <label>
        {{ t("takedowns.reason") }}
        <input v-model="reason" type="text" />
      </label>
      <div class="actions">
        <button type="button" @click="target = null">{{ t("common.cancel") }}</button>
        <button type="button" class="danger" :disabled="!reason.trim()" @click="confirmRemoval">
          {{ t("takedowns.confirm") }}
        </button>
      </div>
    </dialog>
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
.confirm {
  border: 1px solid var(--color-border, #ccc);
  padding: 1rem;
}
.actions {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
  margin-top: 0.75rem;
}
</style>
