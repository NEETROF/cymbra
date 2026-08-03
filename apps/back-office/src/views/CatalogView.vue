<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import FiltersBar from "@/components/FiltersBar.vue";
import CatalogTable from "@/components/CatalogTable.vue";
import StatBar from "@/components/StatBar.vue";
import TablePager from "@/components/TablePager.vue";
import { PAGE_SIZE, useCatalogStore, type Filters } from "@/stores/catalog";
import { useAuthStore } from "@/stores/auth";
import { musicxmlFileName, saveBytesAsFile } from "@/lib/download";
import type { CatalogHit } from "@/gen/score_pb";

// Free browse of the catalog by status + hub filters, with click-to-sort columns.
const store = useCatalogStore();
const auth = useAuthStore();
const router = useRouter();
// The browse state (filters/sort/page) lives in the store so it survives opening a
// score's detail page and returning — this view remounts on return, so local refs
// would snap back to the defaults. The BO catalog defaults status to "" (Tous):
// every moderation status (change: add-score-catalog-proposal).
const view = store.catalogView;
const status = computed(() => view.filters.moderationStatus);

// One exhaustive match folds the Async state into a flat, template-safe view model
// — `.exhaustive()` makes a forgotten state a compile error.
const vm = computed(() =>
  match(store.result)
    .with({ status: "idle" }, () => ({
      loading: false,
      error: null as string | null,
      hits: [] as CatalogHit[],
      total: 0,
    }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, hits: [] as CatalogHit[], total: 0 }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, hits: [] as CatalogHit[], total: 0 }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, hits: data.hits, total: data.total }))
    .exhaustive(),
);

function run() {
  // "" = Tous → request every status (privileged); a specific status filters to it.
  // The ternary narrows `status.value` to a concrete ModerationStatus in the else arm.
  const specific = status.value === "" ? undefined : status.value;
  store.search({
    query: view.filters.query || undefined,
    author: view.filters.author || undefined,
    level: view.filters.level || undefined,
    isPiano: view.filters.isPiano,
    moderationStatus: specific,
    allStatuses: specific === undefined || undefined,
    source: view.filters.source || undefined,
    sort: view.sort,
    offset: view.offset,
  });
}

// A new filter/sort resets to the first page; only the pager advances the offset.
function runFromFirstPage() {
  view.offset = 0;
  run();
}

function onFilters(f: Filters) {
  view.filters = f;
  runFromFirstPage();
}

// Clicking a column rebuilds the single-key sort and re-queries from page 1 — all
// sorting is server-side (correct across the whole set), never client-side.
function onSort(field: string) {
  const cur = view.sort[0];
  const descending = cur?.field === field ? !cur.descending : true;
  view.sort = [{ field, descending }];
  runFromFirstPage();
}

function onPage(newOffset: number) {
  view.offset = newOffset;
  run();
}

// Download a row's linked MusicXML to the operator's machine. The store owns the RPC
// (per the architecture rule) and folds it into per-row Async state; the actual save
// (Blob + `<a download>`) is a DOM concern done here. On failure the store's per-row
// error state renders in the table — nothing to do here.
async function onDownload(hit: CatalogHit) {
  const bytes = await store.downloadBytes(hit.id);
  if (bytes) saveBytesAsFile(bytes, musicxmlFileName(hit.title, hit.id));
}

onMounted(run);
</script>

<template>
  <div class="page-head">
    <div>
      <h1 class="page-title">{{ $t("catalog.title") }}</h1>
      <p class="sub">{{ vm.loading ? $t("common.loading") : $t("catalog.count", vm.total) }}</p>
    </div>
    <FiltersBar :initial="view.filters" @change="onFilters" />
  </div>
  <StatBar />
  <p v-if="vm.error" class="error" role="alert">{{ vm.error }}</p>
  <div class="table-card">
    <CatalogTable
      :hits="vm.hits"
      :status="status"
      :sort="view.sort"
      :can-download="auth.isModerator"
      :downloads="store.downloads"
      @sort="onSort"
      @select="(id) => router.push({ name: 'music-score', params: { id } })"
      @download="onDownload"
    />
    <TablePager :offset="view.offset" :limit="PAGE_SIZE" :total="vm.total" @page="onPage" />
  </div>
</template>
