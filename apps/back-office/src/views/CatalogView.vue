<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import FiltersBar from "@/components/FiltersBar.vue";
import CatalogTable from "@/components/CatalogTable.vue";
import {
  useCatalogStore,
  type Filters,
  type ModerationStatus,
  type SortKeyInit,
} from "@/stores/catalog";

// Free browse of the catalog by status + hub filters, with click-to-sort columns.
const store = useCatalogStore();
const router = useRouter();
const status = ref<ModerationStatus>("pending");
const sort = ref<SortKeyInit[]>([]);
let filters: Filters = {
  query: "",
  author: "",
  level: "",
  isPiano: undefined,
  moderationStatus: "pending",
};

function run() {
  store.search({
    query: filters.query || undefined,
    author: filters.author || undefined,
    level: filters.level || undefined,
    isPiano: filters.isPiano,
    moderationStatus: status.value,
    sort: sort.value,
  });
}

function onFilters(f: Filters) {
  filters = f;
  status.value = f.moderationStatus;
  run();
}

// Clicking a column rebuilds the single-key sort and re-queries from page 1 — all
// sorting is server-side (correct across the whole set), never client-side.
function onSort(field: string) {
  const cur = sort.value[0];
  const descending = cur?.field === field ? !cur.descending : true;
  sort.value = [{ field, descending }];
  run();
}

onMounted(run);
</script>

<template>
  <h1>Catalog</h1>
  <FiltersBar :status="status" @change="onFilters" />
  <p v-if="store.error" class="error" role="alert">{{ store.error }}</p>
  <p class="muted">{{ store.total }} score(s)</p>
  <CatalogTable
    :hits="store.hits"
    :status="status"
    :sort="sort"
    @sort="onSort"
    @select="(id) => router.push({ name: 'score', params: { id } })"
  />
</template>

<style scoped>
.muted {
  color: var(--muted);
}
.error {
  color: var(--reject);
}
</style>
