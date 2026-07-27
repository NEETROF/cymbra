<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import ScorePreview from "@/components/ScorePreview.vue";
import { useCatalogStore, type ModerationStatus } from "@/stores/catalog";
import { useAuthStore } from "@/stores/auth";
import { type Async, idle, run } from "@/lib/async";

const props = defineProps<{ id: string }>();
const store = useCatalogStore();
const auth = useAuthStore();
const router = useRouter();

// Metadata comes from the list the moderator navigated from (avoids a second
// round-trip); bytes are always fetched fresh (moderators may fetch non-accepted).
const hit = computed(() =>
  match(store.result)
    .with({ status: "success" }, ({ data }) => data.hits.find((h) => h.id === props.id) ?? null)
    .otherwise(() => null),
);

const bytes = ref<Async<Uint8Array>>(idle);
const decision = ref<Async<void>>(idle);

const bytesVm = computed(() =>
  match(bytes.value)
    .with({ status: "success" }, ({ data }) => ({ loading: false, bytes: data, error: null as string | null }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, bytes: null, error }))
    .otherwise(() => ({ loading: true, bytes: null, error: null as string | null })),
);
const acting = computed(() => decision.value.status === "loading");
const decisionError = computed(() =>
  match(decision.value)
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() => null),
);

onMounted(() => run(bytes, () => store.fetchBytes(props.id)));

async function decide(status: ModerationStatus) {
  const outcome = await run(decision, () => store.setModerationStatus(props.id, status));
  if (outcome.status === "success") await router.push({ name: "queue" });
}
</script>

<template>
  <div class="head">
    <button @click="router.back()">{{ $t("common.back") }}</button>
    <div v-if="auth.isModerator" class="actions">
      <button class="accept" :disabled="acting" @click="decide('accepted')">
        {{ $t("detail.accept") }}
      </button>
      <button class="reject" :disabled="acting" @click="decide('rejected')">
        {{ $t("detail.reject") }}
      </button>
      <button :disabled="acting" @click="decide('pending')">
        {{ $t("detail.requeue") }}
      </button>
    </div>
  </div>
  <h1>{{ hit?.title || $t("detail.score") }}</h1>
  <p v-if="bytesVm.error" class="error" role="alert">{{ bytesVm.error }}</p>
  <p v-if="decisionError" class="error" role="alert">{{ decisionError }}</p>
  <ScorePreview :hit="hit" :bytes="bytesVm.bytes" :loading="bytesVm.loading" />
</template>

<style scoped>
.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}
.actions button {
  margin-left: 0.4rem;
}
.error {
  color: var(--reject);
}
</style>
