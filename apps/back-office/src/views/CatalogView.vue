<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import FiltersBar from "@/components/FiltersBar.vue";
import CatalogTable from "@/components/CatalogTable.vue";
import StatBar from "@/components/StatBar.vue";
import {
  useCatalogStore,
  type Filters,
  type ModerationStatus,
  type SortKeyInit,
} from "@/stores/catalog";
import type { CatalogHit } from "@/gen/score_pb";

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

// One exhaustive match folds the Async state into a flat, template-safe view model
// — `.exhaustive()` makes a forgotten state a compile error.
const vm = computed(() =>
  match(store.result)
    .with({ status: "idle" }, () => ({ loading: false, error: null as string | null, hits: [] as CatalogHit[], total: 0 }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, hits: [] as CatalogHit[], total: 0 }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, hits: [] as CatalogHit[], total: 0 }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, hits: data.hits, total: data.total }))
    .exhaustive(),
);

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
  <div class="page-head">
    <div>
      <h1 class="page-title">{{ $t("catalog.title") }}</h1>
      <p class="sub">{{ vm.loading ? $t("common.loading") : $t("catalog.count", vm.total) }}</p>
    </div>
    <FiltersBar :status="status" @change="onFilters" />
  </div>
  <p v-if="vm.error" class="error" role="alert">{{ vm.error }}</p>
  <div class="table-card">
    <CatalogTable
      :hits="vm.hits"
      :status="status"
      :sort="sort"
      @sort="onSort"
      @select="(id) => router.push({ name: 'score', params: { id } })"
    />
  </div>
  <StatBar />
</template>
