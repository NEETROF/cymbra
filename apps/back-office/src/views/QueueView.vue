<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import CatalogTable from "@/components/CatalogTable.vue";
import { useCatalogStore, QUEUE_SORT, type SortKeyInit } from "@/stores/catalog";

// The review queue: pending scores ordered by the default review-priority sort
// (re-review flag → status → substance). Sent on every request; clicking a column
// overrides it with a single key and re-queries from page 1.
const store = useCatalogStore();
const router = useRouter();
const sort = ref<SortKeyInit[]>([...QUEUE_SORT]);

function run() {
  store.search({ moderationStatus: "pending", sort: sort.value });
}

function onSort(field: string) {
  const cur = sort.value[0];
  const descending = cur?.field === field ? !cur.descending : true;
  sort.value = [{ field, descending }];
  run();
}

function resetToPriority() {
  sort.value = [...QUEUE_SORT];
  run();
}

onMounted(run);
</script>

<template>
  <div class="head">
    <h1>Review queue</h1>
    <button @click="resetToPriority">Priority order</button>
  </div>
  <p class="muted">
    {{ store.total }} pending — most substantial first. Re-review flagging arrives with app ratings (#2).
  </p>
  <p v-if="store.error" class="error" role="alert">{{ store.error }}</p>
  <CatalogTable
    :hits="store.hits"
    status="pending"
    :sort="sort"
    @sort="onSort"
    @select="(id) => router.push({ name: 'score', params: { id } })"
  />
</template>

<style scoped>
.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.muted {
  color: var(--muted);
}
.error {
  color: var(--reject);
}
</style>
