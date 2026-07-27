<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import CatalogTable from "@/components/CatalogTable.vue";
import StatBar from "@/components/StatBar.vue";
import { useCatalogStore, QUEUE_SORT, type SortKeyInit } from "@/stores/catalog";
import type { CatalogHit } from "@/gen/score_pb";

// The review queue: pending scores ordered by the default review-priority sort
// (re-review flag → status → substance), sent on every request; a column click
// overrides it with a single key and re-queries from page 1.
const store = useCatalogStore();
const router = useRouter();
const sort = ref<SortKeyInit[]>([...QUEUE_SORT]);

const vm = computed(() =>
  match(store.result)
    .with({ status: "idle" }, () => ({ loading: false, error: null as string | null, hits: [] as CatalogHit[], total: 0 }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, hits: [] as CatalogHit[], total: 0 }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, hits: [] as CatalogHit[], total: 0 }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, hits: data.hits, total: data.total }))
    .exhaustive(),
);

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
  <div class="page-head">
    <div>
      <h1 class="page-title">{{ $t("queue.title") }}</h1>
      <p class="sub">
        {{ vm.loading ? $t("common.loading") : $t("queue.pending", vm.total) }} — {{ $t("queue.hint") }}
      </p>
    </div>
    <button @click="resetToPriority">{{ $t("queue.priorityOrder") }}</button>
  </div>
  <p v-if="vm.error" class="error" role="alert">{{ vm.error }}</p>
  <div class="table-card">
    <CatalogTable
      :hits="vm.hits"
      status="pending"
      :sort="sort"
      @sort="onSort"
      @select="(id) => router.push({ name: 'score', params: { id } })"
    />
  </div>
  <StatBar />
</template>
