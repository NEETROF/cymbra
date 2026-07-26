<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import ScorePreview from "@/components/ScorePreview.vue";
import { useCatalogStore, type ModerationStatus } from "@/stores/catalog";
import { useAuthStore } from "@/stores/auth";

const props = defineProps<{ id: string }>();
const store = useCatalogStore();
const auth = useAuthStore();
const router = useRouter();

const bytes = ref<Uint8Array | null>(null);
const loading = ref(true);
const acting = ref(false);
const error = ref<string | null>(null);

// Metadata comes from the list the moderator navigated from (avoids a second
// round-trip); bytes are always fetched fresh (moderators may fetch non-accepted).
const hit = computed(() => store.hits.find((h) => h.id === props.id) ?? null);

onMounted(async () => {
  try {
    bytes.value = await store.fetchBytes(props.id);
  } catch (e) {
    error.value = e instanceof Error ? e.message : "Could not load score bytes";
  } finally {
    loading.value = false;
  }
});

async function decide(status: ModerationStatus) {
  acting.value = true;
  error.value = null;
  try {
    await store.setModerationStatus(props.id, status);
    await router.push({ name: "queue" });
  } catch (e) {
    error.value = e instanceof Error ? e.message : "Action failed";
    acting.value = false;
  }
}
</script>

<template>
  <div class="head">
    <button @click="router.back()">← Back</button>
    <div class="actions" v-if="auth.isModerator">
      <button class="accept" :disabled="acting" @click="decide('accepted')">Accept</button>
      <button class="reject" :disabled="acting" @click="decide('rejected')">Reject</button>
      <button :disabled="acting" @click="decide('pending')" title="Send back to the queue">
        Re-queue
      </button>
    </div>
  </div>
  <h1>{{ hit?.title || "Score" }}</h1>
  <p v-if="error" class="error" role="alert">{{ error }}</p>
  <ScorePreview :hit="hit" :bytes="bytes" :loading="loading" />
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
